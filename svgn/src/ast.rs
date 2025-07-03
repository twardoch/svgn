// this_file: svgn/src/ast.rs

//! Abstract Syntax Tree (AST) for SVG documents
//! 
//! This module defines the core data structures for representing SVG documents
//! in memory. The AST is designed for efficient traversal and mutation during
//! optimization passes.

use std::collections::HashMap;
use indexmap::IndexMap;

/// A complete SVG document
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Nodes that come before the root element (e.g., comments, processing instructions)
    pub prologue: Vec<Node>,
    /// Root element of the document (typically <svg>)
    pub root: Element,
    /// Nodes that come after the root element
    pub epilogue: Vec<Node>,
    /// Document-level metadata
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DocumentMetadata {
    /// Original file path (if any)
    pub path: Option<String>,
    /// Document encoding
    pub encoding: Option<String>,
    /// XML version
    pub version: Option<String>,
}

/// An XML/SVG element
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Element tag name (e.g., "svg", "path", "rect")
    pub name: String,
    /// Element attributes
    pub attributes: IndexMap<String, String>,
    /// Child nodes
    pub children: Vec<Node>,
    /// Namespace declarations
    pub namespaces: HashMap<String, String>,
}

/// A node in the SVG tree
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// An XML element
    Element(Element),
    /// Text content
    Text(String),
    /// XML comment
    Comment(String),
    /// Processing instruction
    ProcessingInstruction {
        target: String,
        data: String,
    },
    /// CDATA section
    CData(String),
    /// DOCTYPE declaration
    DocType(String),
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            prologue: Vec::new(),
            root: Element::new("svg"),
            epilogue: Vec::new(),
            metadata: DocumentMetadata::default(),
        }
    }

    /// Get the root element
    pub fn root(&self) -> &Element {
        &self.root
    }

    /// Get a mutable reference to the root element
    pub fn root_mut(&mut self) -> &mut Element {
        &mut self.root
    }
}

impl Element {
    /// Create a new element with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attributes: IndexMap::new(),
            children: Vec::new(),
            namespaces: HashMap::new(),
        }
    }

    /// Get an attribute value by name
    pub fn attr(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute
    pub fn set_attr(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    /// Remove an attribute
    pub fn remove_attr(&mut self, name: &str) -> Option<String> {
        self.attributes.shift_remove(name)
    }

    /// Check if element has a specific attribute
    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Remove all children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Get iterator over child elements only
    pub fn child_elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|node| {
            if let Node::Element(element) = node {
                Some(element)
            } else {
                None
            }
        })
    }

    /// Get mutable iterator over child elements only
    pub fn child_elements_mut(&mut self) -> impl Iterator<Item = &mut Element> {
        self.children.iter_mut().filter_map(|node| {
            if let Node::Element(element) = node {
                Some(element)
            } else {
                None
            }
        })
    }

    /// Check if element is empty (no children)
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Check if element has only whitespace text content
    pub fn is_whitespace_only(&self) -> bool {
        self.children.iter().all(|child| {
            match child {
                Node::Text(text) => text.trim().is_empty(),
                Node::Comment(_) => true,
                _ => false,
            }
        })
    }
}

impl Node {
    /// Check if this node is an element
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    /// Check if this node is text
    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    /// Check if this node is a comment
    pub fn is_comment(&self) -> bool {
        matches!(self, Node::Comment(_))
    }
    
    /// Check if this node is a DOCTYPE
    pub fn is_doctype(&self) -> bool {
        matches!(self, Node::DocType(_))
    }

    /// Get element if this node is an element
    pub fn as_element(&self) -> Option<&Element> {
        if let Node::Element(element) = self {
            Some(element)
        } else {
            None
        }
    }

    /// Get mutable element if this node is an element
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        if let Node::Element(element) = self {
            Some(element)
        } else {
            None
        }
    }

    /// Get text content if this node is text
    pub fn as_text(&self) -> Option<&String> {
        if let Node::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let element = Element::new("rect");
        assert_eq!(element.name, "rect");
        assert!(element.attributes.is_empty());
        assert!(element.children.is_empty());
    }

    #[test]
    fn test_attribute_operations() {
        let mut element = Element::new("rect");
        
        element.set_attr("x".to_string(), "10".to_string());
        assert_eq!(element.attr("x"), Some(&"10".to_string()));
        assert!(element.has_attr("x"));
        
        let removed = element.remove_attr("x");
        assert_eq!(removed, Some("10".to_string()));
        assert!(!element.has_attr("x"));
    }

    #[test]
    fn test_child_operations() {
        let mut parent = Element::new("g");
        let child = Element::new("rect");
        
        parent.add_child(Node::Element(child));
        parent.add_child(Node::Text("test".to_string()));
        parent.add_child(Node::Comment("comment".to_string()));
        
        assert_eq!(parent.children.len(), 3);
        assert_eq!(parent.child_elements().count(), 1);
        assert!(!parent.is_empty());
    }

    #[test]
    fn test_whitespace_detection() {
        let mut element = Element::new("g");
        element.add_child(Node::Text("   \n  ".to_string()));
        element.add_child(Node::Comment("comment".to_string()));
        
        assert!(element.is_whitespace_only());
        
        element.add_child(Node::Text("content".to_string()));
        assert!(!element.is_whitespace_only());
    }
}