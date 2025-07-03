// this_file: svgn/src/plugins/remove_comments.rs

//! Remove comments plugin
//!
//! This plugin removes all comments from the SVG document.

use crate::ast::{Document, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that removes all comments from the SVG
pub struct RemoveCommentsPlugin;

impl Plugin for RemoveCommentsPlugin {
    fn name(&self) -> &'static str {
        "removeComments"
    }
    
    fn description(&self) -> &'static str {
        "Remove comments from SVG document"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        // Check if we should preserve legal comments
        let preserve_patterns = params
            .and_then(|v| v.get("preservePatterns"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to true, preserve legal comments
        
        // Remove comments from prologue
        document.prologue.retain(|node| {
            match node {
                Node::Comment(comment) => preserve_patterns && is_legal_comment(comment),
                _ => true,
            }
        });
        
        // Remove comments from the main document tree
        remove_comments_from_node(&mut document.root, preserve_patterns);
        
        // Remove comments from epilogue
        document.epilogue.retain(|node| {
            match node {
                Node::Comment(comment) => preserve_patterns && is_legal_comment(comment),
                _ => true,
            }
        });
        
        Ok(())
    }
}

/// Check if a comment is a legal comment (starts with !)
fn is_legal_comment(comment: &str) -> bool {
    comment.trim_start().starts_with('!')
}

/// Recursively remove comments from a node and its children
fn remove_comments_from_node(node: &mut crate::ast::Element, preserve_patterns: bool) {
    // Filter out comment nodes (except legal comments if preservePatterns is true)
    node.children.retain(|child| {
        match child {
            Node::Comment(comment) => {
                preserve_patterns && is_legal_comment(comment)
            }
            _ => true,
        }
    });
    
    // Clean up whitespace-only text nodes if element now only has element children
    let has_element_children = node.children.iter().any(|c| c.is_element());
    let has_meaningful_text = node.children.iter().any(|c| {
        matches!(c, Node::Text(text) if !text.trim().is_empty())
    });
    
    if has_element_children && !has_meaningful_text {
        // Remove whitespace-only text nodes
        node.children.retain(|child| {
            !matches!(child, Node::Text(text) if text.trim().is_empty())
        });
    }
    
    // Recursively process child elements
    for child in &mut node.children {
        if let Node::Element(element) = child {
            remove_comments_from_node(element, preserve_patterns);
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::Element;
    use crate::parser::Parser;
    
    #[test]
    fn test_remove_comments() {
        let svg = r#"<svg>
            <!-- This is a comment -->
            <g>
                <!-- Another comment -->
                <rect/>
            </g>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveCommentsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check that comments are removed (we pass None for params, so legal comments would be preserved)
        assert!(!has_comments(&document.root));
    }
    
    fn has_comments(element: &Element) -> bool {
        for child in &element.children {
            match child {
                Node::Comment(_) => return true,
                Node::Element(el) => {
                    if has_comments(el) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}