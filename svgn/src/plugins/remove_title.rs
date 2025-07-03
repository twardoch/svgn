// this_file: svgn/src/plugins/remove_title.rs

//! Remove title plugin
//!
//! This plugin removes all <title> elements from the SVG document.

use crate::ast::{Document, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that removes all <title> elements from the SVG
pub struct RemoveTitlePlugin;

impl Plugin for RemoveTitlePlugin {
    fn name(&self) -> &'static str {
        "removeTitle"
    }
    
    fn description(&self) -> &'static str {
        "Remove <title> elements from SVG document"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Remove title elements from the document
        remove_title_from_node(&mut document.root);
        Ok(())
    }
}

/// Recursively remove title elements from a node and its children
fn remove_title_from_node(node: &mut crate::ast::Element) {
    // Filter out title elements
    node.children.retain(|child| {
        if let Node::Element(element) = child {
            element.name != "title"
        } else {
            true
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
            remove_title_from_node(element);
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
    fn test_remove_title() {
        let svg = r#"<svg>
            <title>My SVG Document</title>
            <g>
                <title>Group Title</title>
                <rect/>
            </g>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveTitlePlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check that title elements are removed
        assert!(!has_title(&document.root));
    }
    
    fn has_title(element: &Element) -> bool {
        if element.name == "title" {
            return true;
        }
        
        for child in &element.children {
            if let Node::Element(el) = child {
                if has_title(el) {
                    return true;
                }
            }
        }
        false
    }
}