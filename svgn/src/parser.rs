// this_file: svgn/src/parser.rs

//! SVG parser using quick-xml
//!
//! This module provides functionality to parse SVG strings into our custom AST
//! using the quick-xml crate for fast streaming XML parsing.

use crate::ast::{Document, Element, Node};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Error as XmlError, Reader};
use thiserror::Error;

/// Parser error types
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    XmlError(#[from] XmlError),
    #[error("Attribute parsing error: {0}")]
    AttrError(String),
    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Document structure error: {0}")]
    StructureError(String),
    #[error("Unexpected end of document")]
    UnexpectedEnd,
}

/// Parse result type
pub type ParseResult<T> = Result<T, ParseError>;

/// SVG parser
pub struct Parser {
    /// Whether to preserve whitespace
    preserve_whitespace: bool,
    /// Whether to preserve comments
    preserve_comments: bool,
}

impl Parser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self {
            preserve_whitespace: false,
            preserve_comments: false,
        }
    }

    /// Set whether to preserve whitespace
    pub fn preserve_whitespace(mut self, preserve: bool) -> Self {
        self.preserve_whitespace = preserve;
        self
    }

    /// Set whether to preserve comments
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }

    /// Parse an SVG string into a Document
    pub fn parse(&self, input: &str) -> ParseResult<Document> {
        let mut reader = Reader::from_str(input);
        reader.config_mut().expand_empty_elements = true;
        reader.config_mut().trim_text_start = self.preserve_whitespace;
        reader.config_mut().trim_text_end = self.preserve_whitespace;

        let mut document = Document::new();
        let mut element_stack = Vec::new();
        let mut current_element: Option<Element> = None;
        let mut buf = Vec::new();
        let mut found_root = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element = self.parse_start_element(e)?;

                    if current_element.is_none() {
                        // This is the root element
                        current_element = Some(element);
                    } else {
                        // Push current element to stack and start new one
                        if let Some(elem) = current_element.take() {
                            element_stack.push(elem);
                        }
                        current_element = Some(element);
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(finished_element) = current_element.take() {
                        if let Some(mut parent) = element_stack.pop() {
                            parent.add_child(Node::Element(finished_element));
                            current_element = Some(parent);
                        } else {
                            // This was the root element
                            document.root = finished_element;
                            found_root = true;
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let element = self.parse_start_element(e)?;

                    if let Some(ref mut parent) = current_element {
                        parent.add_child(Node::Element(element));
                    } else {
                        // Empty root element
                        document.root = element;
                        found_root = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().map_err(|e| ParseError::XmlError(e))?;
                    let text_content = text.into_owned();

                    if self.preserve_whitespace || !text_content.trim().is_empty() {
                        if let Some(ref mut element) = current_element {
                            element.add_child(Node::Text(text_content));
                        }
                    }
                }
                Ok(Event::Comment(ref e)) => {
                    if self.preserve_comments {
                        let comment = std::str::from_utf8(e.as_ref())?.to_string();
                        if let Some(ref mut element) = current_element {
                            element.add_child(Node::Comment(comment));
                        } else if !found_root {
                            // This is a prologue comment
                            document.prologue.push(Node::Comment(comment));
                        } else {
                            // This is an epilogue comment
                            document.epilogue.push(Node::Comment(comment));
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    let cdata = std::str::from_utf8(e.as_ref())?.to_string();
                    if let Some(ref mut element) = current_element {
                        element.add_child(Node::CData(cdata));
                    }
                }
                Ok(Event::PI(ref e)) => {
                    let pi_data = std::str::from_utf8(e.as_ref())?;
                    let parts: Vec<&str> = pi_data.splitn(2, ' ').collect();
                    let target = parts[0].to_string();
                    let data = parts.get(1).unwrap_or(&"").to_string();

                    let pi_node = Node::ProcessingInstruction { target, data };

                    if let Some(ref mut element) = current_element {
                        element.add_child(pi_node);
                    } else if !found_root {
                        document.prologue.push(pi_node);
                    } else {
                        document.epilogue.push(pi_node);
                    }
                }
                Ok(Event::Decl(ref e)) => {
                    // Handle XML declaration
                    if let Ok(version) = e.version() {
                        document.metadata.version =
                            Some(String::from_utf8_lossy(&version).to_string());
                    }
                    if let Some(Ok(enc)) = e.encoding() {
                        document.metadata.encoding =
                            Some(String::from_utf8_lossy(&enc).to_string());
                    }
                }
                Ok(Event::DocType(ref e)) => {
                    let doctype = e.unescape().map_err(|e| ParseError::XmlError(e))?.into_owned();
                    if !found_root {
                        // DOCTYPE should come before the root element
                        document.prologue.push(Node::DocType(doctype));
                    }
                }
                Ok(Event::Eof) => break,
                Ok(Event::GeneralRef(_)) => {
                    // Handle general references (new in quick-xml v0.38.0)
                    // For SVG parsing, we typically ignore these
                }
                Err(e) => return Err(ParseError::XmlError(e)),
                // All event types are now handled explicitly
            }
            buf.clear();
        }

        if current_element.is_none() && document.root.name.is_empty() {
            return Err(ParseError::StructureError(
                "No root element found".to_string(),
            ));
        }

        Ok(document)
    }

    /// Parse a start element into an Element
    fn parse_start_element(&self, start: &BytesStart) -> ParseResult<Element> {
        let name = std::str::from_utf8(start.name().as_ref())?.to_string();
        let mut element = Element::new(&name);

        // Parse attributes
        for attr_result in start.attributes() {
            let attr = attr_result.map_err(|e| ParseError::AttrError(e.to_string()))?;
            let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
            let value = attr.unescape_value()?.to_string();

            // Handle namespace declarations
            if key.starts_with("xmlns") {
                if key == "xmlns" {
                    element.namespaces.insert("".to_string(), value.clone());
                } else if let Some(ns_name) = key.strip_prefix("xmlns:") {
                    element
                        .namespaces
                        .insert(ns_name.to_string(), value.clone());
                }
            }

            element.set_attr(key, value);
        }

        Ok(element)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to parse an SVG string
pub fn parse_svg(input: &str) -> ParseResult<Document> {
    Parser::new().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg =
            r#"<svg width="100" height="100"><rect x="10" y="10" width="50" height="50"/></svg>"#;
        let document = parse_svg(svg).unwrap();

        assert_eq!(document.root.name, "svg");
        assert_eq!(document.root.attr("width"), Some(&"100".to_string()));
        assert_eq!(document.root.children.len(), 1);

        if let Some(Node::Element(rect)) = document.root.children.first() {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attr("x"), Some(&"10".to_string()));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_parse_with_text() {
        let svg = r#"<svg><text>Hello World</text></svg>"#;
        let document = parse_svg(svg).unwrap();

        if let Some(Node::Element(text_elem)) = document.root.children.first() {
            assert_eq!(text_elem.name, "text");
            if let Some(Node::Text(text)) = text_elem.children.first() {
                assert_eq!(text, "Hello World");
            } else {
                panic!("Expected text node");
            }
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let svg = r#"<svg><!-- This is a comment --><rect/></svg>"#;
        let parser = Parser::new().preserve_comments(true);
        let document = parser.parse(svg).unwrap();

        assert_eq!(document.root.children.len(), 2);
        if let Some(Node::Comment(comment)) = document.root.children.first() {
            assert_eq!(comment, " This is a comment ");
        } else {
            panic!("Expected comment node");
        }
    }

    #[test]
    fn test_parse_empty_element() {
        let svg = r#"<svg/>"#;
        let document = parse_svg(svg).unwrap();

        assert_eq!(document.root.name, "svg");
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_parse_with_namespaces() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><rect/></svg>"#;
        let document = parse_svg(svg).unwrap();

        assert_eq!(
            document.root.namespaces.get(""),
            Some(&"http://www.w3.org/2000/svg".to_string())
        );
        assert_eq!(
            document.root.namespaces.get("xlink"),
            Some(&"http://www.w3.org/1999/xlink".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_xml() {
        let svg = r#"<svg><rect></svg>"#; // Unclosed rect tag
        let result = parse_svg(svg);
        assert!(result.is_err());
    }
}
