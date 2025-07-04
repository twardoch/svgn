// this_file: svgn/src/stringifier.rs

//! SVG stringifier for converting AST back to SVG strings
//!
//! This module provides functionality to convert our AST back into optimized
//! SVG strings with configurable formatting options.

use crate::ast::{Document, Element, Node};
use crate::config::{QuoteAttrsStyle, LineEnding};
use std::fmt::Write;
use thiserror::Error;

/// Stringifier error types
#[derive(Error, Debug)]
pub enum StringifyError {
    #[error("Formatting error: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Invalid document structure: {0}")]
    StructureError(String),
}

/// Stringifier result type
pub type StringifyResult<T> = Result<T, StringifyError>;

/// SVG stringifier with configurable output options
pub struct Stringifier {
    /// Pretty-print the output
    pretty: bool,
    /// Indentation string (spaces or tabs)
    indent_string: String,
    /// Current indentation level
    #[allow(dead_code)]
    current_indent: usize,
    /// Use self-closing tags for empty elements
    self_closing: bool,
    /// How to quote attributes
    quote_attrs: QuoteAttrsStyle,
    /// Line ending style
    eol: LineEnding,
    /// Add final newline
    final_newline: bool,
}

impl Stringifier {
    /// Create a new stringifier with default settings
    pub fn new() -> Self {
        Self {
            pretty: false,
            indent_string: "  ".to_string(), // 2 spaces
            current_indent: 0,
            self_closing: true,
            quote_attrs: QuoteAttrsStyle::Auto,
            eol: LineEnding::default(),
            final_newline: false,
        }
    }

    /// Set pretty-printing
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Set indentation (number of spaces)
    pub fn indent(mut self, spaces: usize) -> Self {
        self.indent_string = " ".repeat(spaces);
        self
    }

    /// Set indentation string directly
    pub fn indent_string(mut self, indent: String) -> Self {
        self.indent_string = indent;
        self
    }

    /// Set self-closing tag behavior
    pub fn self_closing(mut self, self_closing: bool) -> Self {
        self.self_closing = self_closing;
        self
    }

    /// Set attribute quoting style
    pub fn quote_attrs(mut self, style: QuoteAttrsStyle) -> Self {
        self.quote_attrs = style;
        self
    }
    
    /// Set line ending style
    pub fn eol(mut self, eol: LineEnding) -> Self {
        self.eol = eol;
        self
    }
    
    /// Set final newline
    pub fn final_newline(mut self, final_newline: bool) -> Self {
        self.final_newline = final_newline;
        self
    }

    /// Convert a document to an SVG string
    pub fn stringify(&self, document: &Document) -> StringifyResult<String> {
        let mut output = String::new();

        // Add XML declaration if needed
        if let Some(version) = &document.metadata.version {
            if let Some(encoding) = &document.metadata.encoding {
                write!(
                    output,
                    r#"<?xml version="{}" encoding="{}"?>"#,
                    version, encoding
                )?;
                self.write_newline(&mut output)?;
            } else {
                write!(output, r#"<?xml version="{}"?>"#, version)?;
                self.write_newline(&mut output)?;
            }
        }

        // Add prologue nodes (comments, PIs before root element)
        for node in &document.prologue {
            self.stringify_node(node, &mut output, 0)?;
            if self.pretty {
                self.write_newline(&mut output)?;
            }
        }

        // Stringify the root element
        self.stringify_element(&document.root, &mut output, 0)?;

        // Add epilogue nodes (comments, PIs after root element)
        for node in &document.epilogue {
            if self.pretty {
                self.write_newline(&mut output)?;
            }
            self.stringify_node(node, &mut output, 0)?;
        }

        if self.pretty && !self.ends_with_newline(&output) {
            self.write_newline(&mut output)?;
        }
        
        // Add final newline if requested
        if self.final_newline && !self.ends_with_newline(&output) {
            self.write_newline(&mut output)?;
        }

        Ok(output)
    }
    
    /// Write a newline with the configured line ending
    fn write_newline(&self, output: &mut String) -> StringifyResult<()> {
        output.push_str(self.eol.as_str());
        Ok(())
    }
    
    /// Check if string ends with any kind of newline
    fn ends_with_newline(&self, s: &str) -> bool {
        s.ends_with('\n') || s.ends_with("\r\n")
    }

    /// Stringify an element
    fn stringify_element(
        &self,
        element: &Element,
        output: &mut String,
        depth: usize,
    ) -> StringifyResult<()> {
        // Add indentation if pretty-printing
        if self.pretty && depth > 0 {
            self.write_indent(output, depth);
        }

        // Write opening tag
        write!(output, "<{}", element.name)?;

        // Write attributes
        self.write_attributes(element, output)?;

        // Handle empty elements or elements with only whitespace
        let is_effectively_empty =
            element.children.is_empty() || (self.pretty && element.is_whitespace_only());

        if is_effectively_empty {
            if self.self_closing {
                write!(output, "/>")?;
            } else {
                write!(output, "></{}>", element.name)?;
            }

            if self.pretty {
                self.write_newline(output)?;
            }
            return Ok(());
        }

        // Close opening tag
        write!(output, ">")?;

        // Handle mixed content vs element-only content
        let has_element_children = element.children.iter().any(|child| child.is_element());
        let has_text_content = element.children.iter().any(|child| child.is_text());
        let has_only_text = has_text_content && !has_element_children;

        if self.pretty && (has_element_children || has_only_text) {
            writeln!(output)?;
        }

        // Write children
        for child in &element.children {
            match child {
                Node::Element(child_element) => {
                    self.stringify_element(child_element, output, depth + 1)?;
                }
                Node::Text(text) => {
                    let escaped_text = self.escape_text(text);
                    if self.pretty && (has_element_children || has_only_text) {
                        let trimmed = escaped_text.trim();
                        if !trimmed.is_empty() {
                            self.write_indent(output, depth + 1);
                            write!(output, "{}", trimmed)?;
                            self.write_newline(output)?;
                        }
                        // Skip whitespace-only text nodes when pretty-printing
                    } else if !self.pretty || !text.trim().is_empty() {
                        // Only write non-whitespace text or all text when not pretty-printing
                        write!(output, "{}", escaped_text)?;
                    }
                }
                Node::Comment(comment) => {
                    if self.pretty && has_element_children {
                        self.write_indent(output, depth + 1);
                    }
                    write!(output, "<!--{}-->", comment)?;
                    if self.pretty && has_element_children {
                        self.write_newline(output)?;
                    }
                }
                Node::CData(cdata) => {
                    if self.pretty && has_element_children {
                        self.write_indent(output, depth + 1);
                    }
                    write!(output, "<![CDATA[{}]]>", cdata)?;
                    if self.pretty && has_element_children {
                        self.write_newline(output)?;
                    }
                }
                Node::ProcessingInstruction { target, data } => {
                    if self.pretty && has_element_children {
                        self.write_indent(output, depth + 1);
                    }
                    if data.is_empty() {
                        write!(output, "<?{}?>", target)?;
                    } else {
                        write!(output, "<?{} {}?>", target, data)?;
                    }
                    if self.pretty && has_element_children {
                        self.write_newline(output)?;
                    }
                }
                Node::DocType(_) => {
                    // DOCTYPE shouldn't appear inside elements
                    // but handle it gracefully if it does
                }
            }
        }

        // Write closing tag
        if self.pretty && has_element_children && !has_text_content {
            self.write_indent(output, depth);
        }
        write!(output, "</{}>", element.name)?;

        if self.pretty {
            writeln!(output)?;
        }

        Ok(())
    }

    /// Stringify a node (for use with prologue/epilogue)
    fn stringify_node(
        &self,
        node: &Node,
        output: &mut String,
        _depth: usize,
    ) -> StringifyResult<()> {
        match node {
            Node::Comment(comment) => {
                write!(output, "<!--{}-->", comment)?;
            }
            Node::ProcessingInstruction { target, data } => {
                if data.is_empty() {
                    write!(output, "<?{}?>", target)?;
                } else {
                    write!(output, "<?{} {}?>", target, data)?;
                }
            }
            Node::DocType(doctype) => {
                write!(output, "<!DOCTYPE {}>", doctype)?;
            }
            _ => {
                // Other node types should not appear in prologue/epilogue
            }
        }
        Ok(())
    }

    /// Write element attributes
    fn write_attributes(&self, element: &Element, output: &mut String) -> StringifyResult<()> {
        // Collect attributes with their original index to preserve order when priorities are equal
        let mut attrs: Vec<_> = element.attributes.iter().enumerate().collect();
        attrs.sort_by(|(a_idx, (a_name, _)), (b_idx, (b_name, _))| {
            let a_priority = get_attribute_priority(a_name);
            let b_priority = get_attribute_priority(b_name);

            // First sort by priority, then by original index to preserve insertion order
            match a_priority.cmp(&b_priority) {
                std::cmp::Ordering::Equal => a_idx.cmp(b_idx),
                other => other,
            }
        });

        for (_idx, (name, value)) in attrs {
            write!(output, " {}", name)?;

            if !value.is_empty() {
                let quote_char = self.choose_quote_char(value);
                write!(
                    output,
                    "={}{}{}",
                    quote_char,
                    self.escape_attr_value(value, quote_char),
                    quote_char
                )?;
            }
        }

        Ok(())
    }

    /// Choose appropriate quote character for attribute value
    fn choose_quote_char(&self, value: &str) -> char {
        match self.quote_attrs {
            QuoteAttrsStyle::Always => '"',
            QuoteAttrsStyle::Never => {
                // Only use quotes if necessary
                if value.contains(' ')
                    || value.contains('\t')
                    || value.contains('\n')
                    || value.contains('\r')
                    || value.contains('"')
                    || value.contains('\'')
                    || value.contains('<')
                    || value.contains('>')
                    || value.contains('&')
                {
                    if value.contains('"') && !value.contains('\'') {
                        '\''
                    } else {
                        '"'
                    }
                } else {
                    '\0' // No quotes needed
                }
            }
            QuoteAttrsStyle::Auto => {
                if value.contains('"') && !value.contains('\'') {
                    '\''
                } else {
                    '"'
                }
            }
        }
    }

    /// Escape attribute value based on quote character
    fn escape_attr_value(&self, value: &str, quote_char: char) -> String {
        let mut result = String::with_capacity(value.len());

        for ch in value.chars() {
            match ch {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '"' if quote_char == '"' => result.push_str("&quot;"),
                '\'' if quote_char == '\'' => result.push_str("&apos;"),
                _ => result.push(ch),
            }
        }

        result
    }

    /// Escape text content
    fn escape_text(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        for ch in text.chars() {
            match ch {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                _ => result.push(ch),
            }
        }

        result
    }

    /// Write indentation
    fn write_indent(&self, output: &mut String, depth: usize) {
        for _ in 0..depth {
            output.push_str(&self.indent_string);
        }
    }
}

/// Get attribute priority for SVGO-compatible ordering
/// Lower numbers come first
fn get_attribute_priority(attr_name: &str) -> u8 {
    match attr_name {
        // xmlns attributes come first
        name if name.starts_with("xmlns") => 0,
        // id comes early
        "id" => 1,
        // positioning attributes (x, y, cx, cy)
        "x" | "y" | "cx" | "cy" => 2,
        // size attributes (width, height before radius)
        "width" | "height" => 3,
        "r" | "rx" | "ry" => 4,
        // viewBox and transform
        "viewBox" => 5,
        "transform" => 6,
        // style and presentation attributes come later
        "style" => 8,
        "fill" | "stroke" | "opacity" => 9,
        // everything else
        _ => 10,
    }
}

impl Default for Stringifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to stringify a document with default settings
pub fn stringify(document: &Document) -> StringifyResult<String> {
    Stringifier::new().stringify(document)
}

/// Convenience function to stringify a document with pretty-printing
pub fn stringify_pretty(document: &Document) -> StringifyResult<String> {
    Stringifier::new().pretty(true).stringify(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};

    fn create_test_element() -> Element {
        let mut element = Element::new("rect");
        element.set_attr("x".to_string(), "10".to_string());
        element.set_attr("y".to_string(), "20".to_string());
        element.set_attr("width".to_string(), "50".to_string());
        element.set_attr("height".to_string(), "30".to_string());
        element
    }

    #[test]
    fn test_stringify_simple_element() {
        let element = create_test_element();
        let mut document = Document::new();
        document.root = element;

        let result = stringify(&document).unwrap();
        assert!(result.contains("<rect"));
        assert!(result.contains("x=\"10\""));
        assert!(result.contains("y=\"20\""));
        assert!(result.contains("/>"));
    }

    #[test]
    fn test_stringify_pretty() {
        let mut element = Element::new("svg");
        element.add_child(Node::Element(create_test_element()));

        let mut document = Document::new();
        document.root = element;

        let result = stringify_pretty(&document).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("  <rect")); // Should have indentation
    }

    #[test]
    fn test_stringify_with_text() {
        let mut element = Element::new("text");
        element.add_child(Node::Text("Hello & World".to_string()));

        let mut document = Document::new();
        document.root = element;

        let result = stringify(&document).unwrap();
        assert!(result.contains("<text>Hello &amp; World</text>"));
    }

    #[test]
    fn test_stringify_with_comment() {
        let mut element = Element::new("svg");
        element.add_child(Node::Comment(" This is a comment ".to_string()));

        let mut document = Document::new();
        document.root = element;

        let result = stringify(&document).unwrap();
        assert!(result.contains("<!-- This is a comment -->"));
    }

    #[test]
    fn test_attribute_escaping() {
        let mut element = Element::new("test");
        element.set_attr(
            "attr".to_string(),
            "value with \"quotes\" & <tags>".to_string(),
        );

        let mut document = Document::new();
        document.root = element;

        let result = stringify(&document).unwrap();
        assert!(result.contains("attr='value with \"quotes\" &amp; &lt;tags>'"));
    }

    #[test]
    fn test_self_closing_elements() {
        let element = create_test_element();
        let mut document = Document::new();
        document.root = element;

        let stringifier = Stringifier::new().self_closing(true);
        let result = stringifier.stringify(&document).unwrap();
        assert!(result.ends_with("/>"));

        let stringifier = Stringifier::new().self_closing(false);
        let result = stringifier.stringify(&document).unwrap();
        assert!(result.contains("></rect>"));
    }

    #[test]
    fn test_quote_styles() {
        let mut element = Element::new("test");
        element.set_attr("simple".to_string(), "value".to_string());
        element.set_attr(
            "with_quotes".to_string(),
            "value with \"quotes\"".to_string(),
        );

        let mut document = Document::new();
        document.root = element;

        // Test auto quoting
        let stringifier = Stringifier::new().quote_attrs(QuoteAttrsStyle::Auto);
        let result = stringifier.stringify(&document).unwrap();
        assert!(result.contains("simple=\"value\""));
        assert!(result.contains("with_quotes='value with \"quotes\"'"));
    }
}
