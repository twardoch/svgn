// this_file: svgn/src/plugins/convert_ellipse_to_circle.rs

//! Plugin to convert non-eccentric `<ellipse>` elements to `<circle>` elements
//!
//! This plugin converts ellipse elements where rx and ry are equal (or one is "auto")
//! into circle elements, which is a more compact representation.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin to convert ellipses to circles
pub struct ConvertEllipseToCirclePlugin;

/// Convert an ellipse element to a circle if possible
fn convert_ellipse(element: &mut Element) {
    if element.name != "ellipse" {
        return;
    }
    
    let rx = element.attributes.get("rx").map(|s| s.as_str()).unwrap_or("0");
    let ry = element.attributes.get("ry").map(|s| s.as_str()).unwrap_or("0");
    
    // Check if the ellipse can be converted to a circle
    if rx == ry || rx == "auto" || ry == "auto" {
        // Change element name to circle
        element.name = "circle".to_string();
        
        // Determine the radius value
        let radius = if rx == "auto" {
            ry.to_string()
        } else {
            rx.to_string()
        };
        
        // Remove rx and ry attributes
        element.attributes.shift_remove("rx");
        element.attributes.shift_remove("ry");
        
        // Add r attribute
        element.attributes.insert("r".to_string(), radius);
    }
}

/// Recursively process nodes
fn process_node(node: &mut Node) {
    if let Node::Element(ref mut element) = node {
        convert_ellipse(element);
        
        // Process children
        for child in &mut element.children {
            process_node(child);
        }
    }
}

impl Plugin for ConvertEllipseToCirclePlugin {
    fn name(&self) -> &'static str {
        "convertEllipseToCircle"
    }
    
    fn description(&self) -> &'static str {
        "converts non-eccentric <ellipse>s to <circle>s"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Process root element
        convert_ellipse(&mut document.root);
        
        // Process children
        for child in &mut document.root.children {
            process_node(child);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_ellipse(cx: &str, cy: &str, rx: &str, ry: &str) -> Element {
        let mut ellipse = Element {
            name: "ellipse".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        ellipse.attributes.insert("cx".to_string(), cx.to_string());
        ellipse.attributes.insert("cy".to_string(), cy.to_string());
        ellipse.attributes.insert("rx".to_string(), rx.to_string());
        ellipse.attributes.insert("ry".to_string(), ry.to_string());
        ellipse
    }

    #[test]
    fn test_convert_equal_radii() {
        let mut doc = Document::default();
        doc.root = create_ellipse("50", "50", "25", "25");
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        assert_eq!(doc.root.name, "circle");
        assert_eq!(doc.root.attributes.get("r"), Some(&"25".to_string()));
        assert_eq!(doc.root.attributes.get("rx"), None);
        assert_eq!(doc.root.attributes.get("ry"), None);
        assert_eq!(doc.root.attributes.get("cx"), Some(&"50".to_string()));
        assert_eq!(doc.root.attributes.get("cy"), Some(&"50".to_string()));
    }

    #[test]
    fn test_keep_unequal_radii() {
        let mut doc = Document::default();
        doc.root = create_ellipse("50", "50", "30", "20");
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        assert_eq!(doc.root.name, "ellipse");
        assert_eq!(doc.root.attributes.get("rx"), Some(&"30".to_string()));
        assert_eq!(doc.root.attributes.get("ry"), Some(&"20".to_string()));
        assert_eq!(doc.root.attributes.get("r"), None);
    }

    #[test]
    fn test_convert_rx_auto() {
        let mut doc = Document::default();
        doc.root = create_ellipse("50", "50", "auto", "25");
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        assert_eq!(doc.root.name, "circle");
        assert_eq!(doc.root.attributes.get("r"), Some(&"25".to_string()));
    }

    #[test]
    fn test_convert_ry_auto() {
        let mut doc = Document::default();
        doc.root = create_ellipse("50", "50", "30", "auto");
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        assert_eq!(doc.root.name, "circle");
        assert_eq!(doc.root.attributes.get("r"), Some(&"30".to_string()));
    }

    #[test]
    fn test_default_zero_values() {
        let mut doc = Document::default();
        let mut ellipse = Element {
            name: "ellipse".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        ellipse.attributes.insert("cx".to_string(), "50".to_string());
        ellipse.attributes.insert("cy".to_string(), "50".to_string());
        // No rx/ry attributes - should default to "0"
        
        doc.root = ellipse;
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        assert_eq!(doc.root.name, "circle");
        assert_eq!(doc.root.attributes.get("r"), Some(&"0".to_string()));
    }

    #[test]
    fn test_nested_ellipses() {
        let mut doc = Document::default();
        
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        
        svg.children.push(Node::Element(create_ellipse("25", "25", "10", "10")));
        svg.children.push(Node::Element(create_ellipse("75", "75", "20", "15")));
        
        doc.root = svg;
        
        let mut plugin = ConvertEllipseToCirclePlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        // First ellipse should be converted
        if let Some(Node::Element(ref first)) = doc.root.children.get(0) {
            assert_eq!(first.name, "circle");
            assert_eq!(first.attributes.get("r"), Some(&"10".to_string()));
        }
        
        // Second ellipse should remain
        if let Some(Node::Element(ref second)) = doc.root.children.get(1) {
            assert_eq!(second.name, "ellipse");
            assert_eq!(second.attributes.get("rx"), Some(&"20".to_string()));
            assert_eq!(second.attributes.get("ry"), Some(&"15".to_string()));
        }
    }
}