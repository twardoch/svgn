// this_file: svgn/src/plugins/remove_dimensions.rs

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

pub struct RemoveDimensionsPlugin;

impl RemoveDimensionsPlugin {
    pub fn new() -> Self {
        Self
    }

    fn process_svg_element(&self, element: &mut Element) {
        if element.name != "svg" {
            return;
        }

        // If viewBox already exists, just remove width and height
        if element.attributes.contains_key("viewBox") {
            element.attributes.shift_remove("width");
            element.attributes.shift_remove("height");
        } else {
            // Try to create viewBox from width and height if both are present and numeric
            let width_str = element.attributes.get("width");
            let height_str = element.attributes.get("height");

            if let (Some(width_str), Some(height_str)) = (width_str, height_str) {
                // Try to parse width and height as numbers
                if let (Ok(width), Ok(height)) = (width_str.parse::<f64>(), height_str.parse::<f64>()) {
                    // Only proceed if both are valid numbers (not NaN)
                    if !width.is_nan() && !height.is_nan() {
                        // Create viewBox and remove width/height
                        let viewbox = format!("0 0 {} {}", width, height);
                        element.attributes.insert("viewBox".to_string(), viewbox);
                        element.attributes.shift_remove("width");
                        element.attributes.shift_remove("height");
                    }
                }
            }
        }
    }

    fn process_element(&self, element: &mut Element) {
        // Process this element if it's an SVG element
        self.process_svg_element(element);

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.process_element(child_elem);
            }
        }
    }
}

impl Plugin for RemoveDimensionsPlugin {
    fn name(&self) -> &'static str {
        "removeDimensions"
    }

    fn description(&self) -> &'static str {
        "removes width and height in presence of viewBox (opposite to removeViewBox)"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        self.process_element(&mut document.root);
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::PluginInfo;
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_test_document() -> Document {
        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            prologue: vec![],
            epilogue: vec![],
            metadata: crate::ast::DocumentMetadata {
                path: None,
                encoding: None,
                version: None,
            },
        }
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = RemoveDimensionsPlugin::new();
        assert_eq!(plugin.name(), "removeDimensions");
        assert_eq!(plugin.description(), "removes width and height in presence of viewBox (opposite to removeViewBox)");
    }

    #[test]
    fn test_remove_dimensions_with_existing_viewbox() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with width, height, and viewBox
        doc.root.attributes.insert("width".to_string(), "100".to_string());
        doc.root.attributes.insert("height".to_string(), "50".to_string());
        doc.root.attributes.insert("viewBox".to_string(), "0 0 200 100".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Width and height should be removed, viewBox should remain unchanged
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert_eq!(doc.root.attributes.get("viewBox"), Some(&"0 0 200 100".to_string()));
    }

    #[test]
    fn test_create_viewbox_from_dimensions() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with only width and height
        doc.root.attributes.insert("width".to_string(), "100".to_string());
        doc.root.attributes.insert("height".to_string(), "50".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Width and height should be removed, viewBox should be created
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert_eq!(doc.root.attributes.get("viewBox"), Some(&"0 0 100 50".to_string()));
    }

    #[test]
    fn test_decimal_dimensions() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with decimal dimensions
        doc.root.attributes.insert("width".to_string(), "100.5".to_string());
        doc.root.attributes.insert("height".to_string(), "50.25".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Width and height should be removed, viewBox should be created with decimals
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert_eq!(doc.root.attributes.get("viewBox"), Some(&"0 0 100.5 50.25".to_string()));
    }

    #[test]
    fn test_invalid_dimensions_ignored() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with invalid dimensions
        doc.root.attributes.insert("width".to_string(), "invalid".to_string());
        doc.root.attributes.insert("height".to_string(), "50".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Width and height should remain since they're not both valid numbers
        assert_eq!(doc.root.attributes.get("width"), Some(&"invalid".to_string()));
        assert_eq!(doc.root.attributes.get("height"), Some(&"50".to_string()));
        assert!(!doc.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_missing_dimension_ignored() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with only width
        doc.root.attributes.insert("width".to_string(), "100".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Width should remain since height is missing
        assert_eq!(doc.root.attributes.get("width"), Some(&"100".to_string()));
        assert!(!doc.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_only_processes_svg_elements() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Add a rect element with width and height (should not be processed)
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("width".to_string(), "100".to_string());
        rect_attrs.insert("height".to_string(), "50".to_string());
        rect_attrs.insert("x".to_string(), "10".to_string());
        rect_attrs.insert("y".to_string(), "10".to_string());
        
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Rect dimensions should remain unchanged
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.attributes.get("width"), Some(&"100".to_string()));
            assert_eq!(rect.attributes.get("height"), Some(&"50".to_string()));
            assert!(!rect.attributes.contains_key("viewBox"));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_nested_svg_elements() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up root SVG
        doc.root.attributes.insert("width".to_string(), "200".to_string());
        doc.root.attributes.insert("height".to_string(), "100".to_string());
        
        // Add nested SVG element
        let mut nested_svg_attrs = IndexMap::new();
        nested_svg_attrs.insert("width".to_string(), "100".to_string());
        nested_svg_attrs.insert("height".to_string(), "50".to_string());
        
        doc.root.children.push(Node::Element(Element {
            name: "svg".to_string(),
            attributes: nested_svg_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Root SVG should have viewBox and no dimensions
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert_eq!(doc.root.attributes.get("viewBox"), Some(&"0 0 200 100".to_string()));
        
        // Nested SVG should also be processed
        if let Node::Element(nested_svg) = &doc.root.children[0] {
            assert!(!nested_svg.attributes.contains_key("width"));
            assert!(!nested_svg.attributes.contains_key("height"));
            assert_eq!(nested_svg.attributes.get("viewBox"), Some(&"0 0 100 50".to_string()));
        } else {
            panic!("Expected nested SVG element");
        }
    }

    #[test]
    fn test_zero_dimensions() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG with zero dimensions
        doc.root.attributes.insert("width".to_string(), "0".to_string());
        doc.root.attributes.insert("height".to_string(), "0".to_string());
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Should still create viewBox even with zero dimensions
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert_eq!(doc.root.attributes.get("viewBox"), Some(&"0 0 0 0".to_string()));
    }

    #[test]
    fn test_no_dimensions_no_change() {
        let mut plugin = RemoveDimensionsPlugin::new();
        let mut doc = create_test_document();
        
        // SVG with no width, height, or viewBox
        let original_count = doc.root.attributes.len();
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Should not add any attributes
        assert_eq!(doc.root.attributes.len(), original_count);
        assert!(!doc.root.attributes.contains_key("width"));
        assert!(!doc.root.attributes.contains_key("height"));
        assert!(!doc.root.attributes.contains_key("viewBox"));
    }
}