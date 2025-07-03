// this_file: svgn/src/plugins/remove_metadata.rs

//! Remove metadata plugin
//!
//! This plugin removes all <metadata> elements from the SVG document.

use crate::ast::{Document, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that removes all <metadata> elements from the SVG
pub struct RemoveMetadataPlugin;

impl Plugin for RemoveMetadataPlugin {
    fn name(&self) -> &'static str {
        "removeMetadata"
    }
    
    fn description(&self) -> &'static str {
        "Remove <metadata> elements from SVG document"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Remove metadata elements from the document
        remove_metadata_from_node(&mut document.root);
        Ok(())
    }
}

/// Recursively remove metadata elements from a node and its children
fn remove_metadata_from_node(node: &mut crate::ast::Element) {
    // Filter out metadata elements
    node.children.retain(|child| {
        if let Node::Element(element) = child {
            element.name != "metadata"
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
            remove_metadata_from_node(element);
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
    fn test_remove_metadata() {
        let svg = r#"<svg>
            <metadata>
                <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                    <rdf:Description/>
                </rdf:RDF>
            </metadata>
            <g>
                <metadata>Some metadata</metadata>
                <rect/>
            </g>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveMetadataPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check that metadata elements are removed
        assert!(!has_metadata(&document.root));
    }
    
    fn has_metadata(element: &Element) -> bool {
        if element.name == "metadata" {
            return true;
        }
        
        for child in &element.children {
            if let Node::Element(el) = child {
                if has_metadata(el) {
                    return true;
                }
            }
        }
        false
    }
}