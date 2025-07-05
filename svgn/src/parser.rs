// this_file: svgn/src/parser.rs

//! SVG parser using quick-xml
//!
//! This module provides functionality to parse SVG strings into our custom AST
//! using the quick-xml crate for fast streaming XML parsing.

use crate::ast::{Document, Element, Node};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::LazyLock;
use thiserror::Error;

/// Parse error types
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::Error),
    
    #[error("Attribute parsing error: {0}")]
    AttrError(String),
    
    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    
    #[error("Document structure error: {0}")]
    StructureError(String),
    
    #[error("Unexpected end of document")]
    UnexpectedEnd,
    
    #[error("{0}")]
    DetailedError(DetailedParseError),
}


/// Detailed parse error with context
#[derive(Debug, Clone)]
pub struct DetailedParseError {
    /// File path (if available)
    pub file_path: Option<String>,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Error message
    pub message: String,
    /// Source code context
    pub context: Option<ErrorContext>,
}

/// Error context with source code snippet
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Lines of source code around the error
    pub lines: Vec<String>,
    /// Index of the error line in the lines vector
    pub error_line_index: usize,
    /// Column position in the error line
    pub error_column: usize,
}

impl fmt::Display for DetailedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: file.svg:line:column: error message
        if let Some(ref path) = self.file_path {
            write!(f, "{}:", path)?;
        }
        write!(f, "{}:{}: {}", self.line, self.column, self.message)?;
        
        // Add source context if available
        if let Some(ref ctx) = self.context {
            writeln!(f)?;
            writeln!(f)?;
            
            // Display lines with line numbers
            let start_line = self.line.saturating_sub(ctx.error_line_index);
            for (i, line) in ctx.lines.iter().enumerate() {
                let line_num = start_line + i;
                let prefix = if i == ctx.error_line_index { ">" } else { " " };
                writeln!(f, "{} {:3} | {}", prefix, line_num, line)?;
                
                // Add error pointer on the error line
                if i == ctx.error_line_index {
                    let spaces = " ".repeat(ctx.error_column + 6);
                    writeln!(f, "{} ^", spaces)?;
                }
            }
        }
        
        Ok(())
    }
}

/// Parse result type
pub type ParseResult<T> = Result<T, ParseError>;

/// Elements where whitespace should be preserved
static TEXT_ELEMENTS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // Text content elements
        "text", "tspan", "textPath", "altGlyph", "tref", 
        "glyph", "glyphRef", "altGlyphDef", "altGlyphItem",
        // Other elements that need whitespace preservation
        "pre", "title", "script", "style"
    ])
});

/// SVG parser
pub struct Parser {
    /// Whether to preserve whitespace
    preserve_whitespace: bool,
    /// Whether to preserve comments
    preserve_comments: bool,
    /// Whether to expand XML entities
    expand_entities: bool,
    /// File path (for error reporting)
    file_path: Option<String>,
}

impl Parser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self {
            preserve_whitespace: false,
            preserve_comments: false,
            expand_entities: true,
            file_path: None,
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

    /// Set whether to expand XML entities
    pub fn expand_entities(mut self, expand: bool) -> Self {
        self.expand_entities = expand;
        self
    }
    
    /// Set the file path for error reporting
    pub fn file_path(mut self, path: Option<String>) -> Self {
        self.file_path = path;
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
        let mut entities: HashMap<String, String> = HashMap::new();
        let mut element_name_stack: Vec<String> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element = self.parse_start_element(e, &entities, input)?;
                    
                    // Track element name for whitespace preservation
                    element_name_stack.push(element.name.clone());

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
                    // Pop element name from stack
                    element_name_stack.pop();
                    
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
                    let element = self.parse_start_element(e, &entities, input)?;
                    
                    // Empty elements don't need name tracking since they have no content

                    if let Some(ref mut parent) = current_element {
                        parent.add_child(Node::Element(element));
                    } else {
                        // Empty root element
                        document.root = element;
                        found_root = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    // Convert bytes to string and unescape
                    let text_str = std::str::from_utf8(e.as_ref())?;
                    let text = quick_xml::escape::unescape(text_str)
                        .unwrap_or(std::borrow::Cow::Borrowed(text_str));
                    let mut text_content = text.into_owned();
                    
                    // Expand custom entities if enabled
                    if self.expand_entities && !entities.is_empty() {
                        text_content = self.expand_entities_in_text(&text_content, &entities);
                    }

                    // Check if we should preserve whitespace for this element
                    let should_preserve_whitespace = self.preserve_whitespace || 
                        element_name_stack.last()
                            .map(|name| TEXT_ELEMENTS.contains(name.as_str()))
                            .unwrap_or(false);

                    if should_preserve_whitespace || !text_content.trim().is_empty() {
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
                    // For DOCTYPE, just convert bytes to string without unescaping
                    let doctype = std::str::from_utf8(e.as_ref())?.to_string();
                    
                    // Parse entity declarations from DOCTYPE if entity expansion is enabled
                    if self.expand_entities {
                        self.parse_entities_from_doctype(&doctype, &mut entities);
                    }
                    
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
                Err(e) => {
                    // Try to get position information for better error reporting
                    let byte_pos = reader.buffer_position().try_into().unwrap();
                    return Err(self.create_detailed_error(
                        input,
                        byte_pos,
                        format!("XML parsing error: {}", e),
                    ));
                }
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
    fn parse_start_element(&self, start: &BytesStart, entities: &HashMap<String, String>, _input: &str) -> ParseResult<Element> {
        let name = std::str::from_utf8(start.name().as_ref())?.to_string();
        let mut element = Element::new(&name);

        // Parse attributes
        for attr_result in start.attributes() {
            let attr = match attr_result {
                Ok(attr) => attr,
                Err(e) => {
                    // Since we can't get the exact position, use a generic error
                    return Err(ParseError::AttrError(format!("Attribute parsing error: {}", e)));
                }
            };
            let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
            let mut value = attr.unescape_value()?.to_string();
            
            // Expand custom entities in attribute values if enabled
            if self.expand_entities && !entities.is_empty() {
                value = self.expand_entities_in_text(&value, entities);
            }

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
    
    /// Parse entity declarations from DOCTYPE
    fn parse_entities_from_doctype(&self, doctype: &str, entities: &mut HashMap<String, String>) {
        // Simple regex-based parsing for entity declarations
        // This handles patterns like: <!ENTITY name "value">
        let entity_pattern = regex::Regex::new(r#"<!ENTITY\s+(\w+)\s+"([^"]+)"\s*>"#).unwrap();
        
        for capture in entity_pattern.captures_iter(doctype) {
            if let (Some(name), Some(value)) = (capture.get(1), capture.get(2)) {
                entities.insert(name.as_str().to_string(), value.as_str().to_string());
            }
        }
        
        // Also handle single-quoted entities: <!ENTITY name 'value'>
        let entity_pattern_single = regex::Regex::new(r#"<!ENTITY\s+(\w+)\s+'([^']+)'\s*>"#).unwrap();
        
        for capture in entity_pattern_single.captures_iter(doctype) {
            if let (Some(name), Some(value)) = (capture.get(1), capture.get(2)) {
                entities.insert(name.as_str().to_string(), value.as_str().to_string());
            }
        }
    }
    
    /// Expand entity references in text
    fn expand_entities_in_text(&self, text: &str, entities: &HashMap<String, String>) -> String {
        let mut result = text.to_string();
        
        // Replace custom entity references (&entity;)
        for (name, value) in entities {
            let entity_ref = format!("&{};", name);
            result = result.replace(&entity_ref, value);
        }
        
        result
    }
    
    /// Calculate line and column from byte position
    fn calculate_line_and_column(&self, input: &str, byte_pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        
        for (i, ch) in input.char_indices() {
            if i >= byte_pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }
    
    /// Create a detailed parse error with context
    fn create_detailed_error(&self, input: &str, byte_pos: usize, message: String) -> ParseError {
        let (line, column) = self.calculate_line_and_column(input, byte_pos);
        
        // Extract context lines
        let lines: Vec<&str> = input.lines().collect();
        let mut context_lines = Vec::new();
        let mut error_line_index = 0;
        
        // Get 2 lines before and after the error
        let start_line = line.saturating_sub(3);
        let end_line = (line + 2).min(lines.len());
        
        for i in start_line..end_line {
            if let Some(line_content) = lines.get(i) {
                context_lines.push(line_content.to_string());
                if i + 1 == line {
                    error_line_index = context_lines.len() - 1;
                }
            }
        }
        
        let context = if !context_lines.is_empty() {
            Some(ErrorContext {
                lines: context_lines,
                error_line_index,
                error_column: column.saturating_sub(1),
            })
        } else {
            None
        };
        
        ParseError::DetailedError(DetailedParseError {
            file_path: self.file_path.clone(),
            line,
            column,
            message,
            context,
        })
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
