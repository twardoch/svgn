// this_file: svgn/src/plugins/sort_attrs.rs

//! Plugin to sort element attributes for better compression
//!
//! Sorts attributes according to a customizable priority order, with special
//! handling for namespaces and grouped attributes (like fill/fill-opacity).

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use indexmap::IndexMap;
use std::cmp::Ordering;

/// Plugin to sort element attributes
pub struct SortAttrsPlugin;

/// Configuration parameters for attribute sorting
#[derive(Debug, Clone)]
pub struct SortAttrsParams {
    /// Custom order for important attributes
    pub order: Vec<String>,
    /// How to handle xmlns attributes: "front" or "alphabetical"
    pub xmlns_order: XmlnsOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XmlnsOrder {
    /// Place xmlns attributes at the front
    Front,
    /// Sort xmlns attributes alphabetically with others
    Alphabetical,
}

impl Default for SortAttrsParams {
    fn default() -> Self {
        Self {
            order: vec![
                "id".to_string(),
                "width".to_string(),
                "height".to_string(),
                "x".to_string(),
                "x1".to_string(),
                "x2".to_string(),
                "y".to_string(),
                "y1".to_string(),
                "y2".to_string(),
                "cx".to_string(),
                "cy".to_string(),
                "r".to_string(),
                "fill".to_string(),
                "stroke".to_string(),
                "marker".to_string(),
                "d".to_string(),
                "points".to_string(),
            ],
            xmlns_order: XmlnsOrder::Front,
        }
    }
}

impl SortAttrsParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut params = Self::default();
        
        if let Some(Value::Object(map)) = value {
            // Parse custom order array
            if let Some(Value::Array(order_array)) = map.get("order") {
                let mut order = Vec::new();
                for item in order_array {
                    if let Value::String(attr_name) = item {
                        order.push(attr_name.clone());
                    }
                }
                if !order.is_empty() {
                    params.order = order;
                }
            }
            
            // Parse xmlns order preference
            if let Some(Value::String(xmlns_order)) = map.get("xmlnsOrder") {
                match xmlns_order.as_str() {
                    "front" => params.xmlns_order = XmlnsOrder::Front,
                    "alphabetical" => params.xmlns_order = XmlnsOrder::Alphabetical,
                    _ => {} // Keep default
                }
            }
        }
        
        params
    }
}

impl Plugin for SortAttrsPlugin {
    fn name(&self) -> &'static str {
        "sortAttrs"
    }

    fn description(&self) -> &'static str {
        "Sort element attributes for better compression"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = SortAttrsParams::from_value(params);
        visit_elements(&mut document.root, &config);
        Ok(())
    }
}

/// Visit all elements in the AST and sort their attributes
fn visit_elements(element: &mut Element, config: &SortAttrsParams) {
    sort_element_attributes(element, config);
    
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            visit_elements(child_element, config);
        }
    }
}

/// Sort attributes in a single element
fn sort_element_attributes(element: &mut Element, config: &SortAttrsParams) {
    // Convert attributes to a vector of (name, value) pairs
    let mut attrs: Vec<(String, String)> = element.attributes.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    // Sort the attributes
    attrs.sort_by(|a, b| compare_attrs(&a.0, &b.0, config));
    
    // Rebuild the attributes map in sorted order
    let mut sorted_attributes = IndexMap::new();
    for (name, value) in attrs {
        sorted_attributes.insert(name, value);
    }
    
    element.attributes = sorted_attributes;
}

/// Compare two attribute names for sorting
fn compare_attrs(a_name: &str, b_name: &str, config: &SortAttrsParams) -> Ordering {
    // Get namespace priorities
    let a_priority = get_namespace_priority(a_name, &config.xmlns_order);
    let b_priority = get_namespace_priority(b_name, &config.xmlns_order);
    
    // Sort by namespace priority first (higher priority comes first)
    match b_priority.cmp(&a_priority) {
        Ordering::Equal => {
            // Same namespace priority, continue with other rules
        }
        other => return other,
    }
    
    // Extract the first part from attributes (e.g., "fill" from "fill-opacity")
    let a_part = a_name.split('-').next().unwrap_or(a_name);
    let b_part = b_name.split('-').next().unwrap_or(b_name);
    
    // If the first parts are different, apply order-based sorting
    if a_part != b_part {
        let a_in_order = config.order.iter().position(|x| x == a_part);
        let b_in_order = config.order.iter().position(|x| x == b_part);
        
        match (a_in_order, b_in_order) {
            (Some(a_pos), Some(b_pos)) => {
                // Both are in the custom order, sort by position
                return a_pos.cmp(&b_pos);
            }
            (Some(_), None) => {
                // Only a is in order, a comes first
                return Ordering::Less;
            }
            (None, Some(_)) => {
                // Only b is in order, b comes first
                return Ordering::Greater;
            }
            (None, None) => {
                // Neither is in order, fall through to alphabetical
            }
        }
    }
    
    // Sort alphabetically
    a_name.cmp(b_name)
}

/// Get the namespace priority for an attribute name
fn get_namespace_priority(name: &str, xmlns_order: &XmlnsOrder) -> u8 {
    if *xmlns_order == XmlnsOrder::Front {
        // Put xmlns first
        if name == "xmlns" {
            return 3;
        }
        // xmlns:* attributes second
        if name.starts_with("xmlns:") {
            return 2;
        }
    }
    
    // Other namespaces after and sort them alphabetically
    if name.contains(':') {
        return 1;
    }
    
    // Other attributes (lowest priority)
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use serde_json::json;

    #[test]
    fn test_sorts_by_default_order() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        // Add attributes in random order
        element.attributes.insert("stroke".to_string(), "black".to_string());
        element.attributes.insert("id".to_string(), "rect1".to_string());
        element.attributes.insert("height".to_string(), "100".to_string());
        element.attributes.insert("fill".to_string(), "red".to_string());
        element.attributes.insert("width".to_string(), "200".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Check that attributes are in the expected order
        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // id should come first, then width, height, fill, stroke
        let expected_positions = [("id", 0), ("width", 1), ("height", 2), ("fill", 3), ("stroke", 4)];
        
        for (attr, expected_pos) in expected_positions {
            let actual_pos = attr_names.iter().position(|x| *x == attr).unwrap();
            assert_eq!(actual_pos, expected_pos, "Attribute '{}' should be at position {}", attr, expected_pos);
        }
    }

    #[test]
    fn test_xmlns_front_ordering() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        
        element.attributes.insert("width".to_string(), "100".to_string());
        element.attributes.insert("xmlns:xlink".to_string(), "http://www.w3.org/1999/xlink".to_string());
        element.attributes.insert("id".to_string(), "svg1".to_string());
        element.attributes.insert("xmlns".to_string(), "http://www.w3.org/2000/svg".to_string());
        element.attributes.insert("height".to_string(), "100".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // xmlns should be first, xmlns:xlink second, then regular attributes
        assert_eq!(attr_names[0], "xmlns");
        assert_eq!(attr_names[1], "xmlns:xlink");
        // id should come before width and height according to default order
        assert!(attr_names.iter().position(|x| *x == "id").unwrap() < 
                attr_names.iter().position(|x| *x == "width").unwrap());
    }

    #[test]
    fn test_xmlns_alphabetical_ordering() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        
        element.attributes.insert("width".to_string(), "100".to_string());
        element.attributes.insert("xmlns:xlink".to_string(), "http://www.w3.org/1999/xlink".to_string());
        element.attributes.insert("xmlns".to_string(), "http://www.w3.org/2000/svg".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        let params = json!({"xmlnsOrder": "alphabetical"});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // With alphabetical ordering, regular attributes and xmlns should be mixed
        // width comes before xmlns alphabetically
        assert!(attr_names.iter().position(|x| *x == "width").unwrap() < 
                attr_names.iter().position(|x| *x == "xmlns").unwrap());
    }

    #[test]
    fn test_grouped_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        element.attributes.insert("fill-opacity".to_string(), "0.5".to_string());
        element.attributes.insert("stroke-width".to_string(), "2".to_string());
        element.attributes.insert("fill".to_string(), "red".to_string());
        element.attributes.insert("stroke".to_string(), "black".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // fill should come before stroke (default order)
        let fill_pos = attr_names.iter().position(|x| *x == "fill").unwrap();
        let stroke_pos = attr_names.iter().position(|x| *x == "stroke").unwrap();
        assert!(fill_pos < stroke_pos);
        
        // fill-opacity should come after fill (alphabetical within group)
        let fill_opacity_pos = attr_names.iter().position(|x| *x == "fill-opacity").unwrap();
        assert!(fill_pos < fill_opacity_pos);
    }

    #[test]
    fn test_custom_order() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        element.attributes.insert("y".to_string(), "10".to_string());
        element.attributes.insert("x".to_string(), "5".to_string());
        element.attributes.insert("id".to_string(), "rect1".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        let params = json!({"order": ["y", "x", "id"]});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // Should follow the custom order: y, x, id
        assert_eq!(attr_names[0], "y");
        assert_eq!(attr_names[1], "x");
        assert_eq!(attr_names[2], "id");
    }

    #[test]
    fn test_alphabetical_fallback() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        // Add attributes not in the default order
        element.attributes.insert("z-index".to_string(), "1".to_string());
        element.attributes.insert("class".to_string(), "rect".to_string());
        element.attributes.insert("data-test".to_string(), "value".to_string());
        
        document.root = element;

        let mut plugin = SortAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        let attr_names: Vec<&String> = document.root.attributes.keys().collect();
        
        // Should be in alphabetical order: class, data-test, z-index
        assert_eq!(attr_names[0], "class");
        assert_eq!(attr_names[1], "data-test");
        assert_eq!(attr_names[2], "z-index");
    }

    #[test]
    fn test_nested_elements() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut child = Element::new("rect");
        
        // Root element attributes
        root.attributes.insert("height".to_string(), "100".to_string());
        root.attributes.insert("id".to_string(), "svg1".to_string());
        root.attributes.insert("width".to_string(), "100".to_string());
        
        // Child element attributes
        child.attributes.insert("y".to_string(), "10".to_string());
        child.attributes.insert("fill".to_string(), "red".to_string());
        child.attributes.insert("x".to_string(), "5".to_string());
        
        root.children.push(Node::Element(child));
        document.root = root;

        let mut plugin = SortAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Check root attributes are sorted
        let root_attr_names: Vec<&String> = document.root.attributes.keys().collect();
        assert_eq!(root_attr_names[0], "id");
        assert_eq!(root_attr_names[1], "width");
        assert_eq!(root_attr_names[2], "height");
        
        // Check child attributes are sorted
        if let Node::Element(child_element) = &document.root.children[0] {
            let child_attr_names: Vec<&String> = child_element.attributes.keys().collect();
            assert_eq!(child_attr_names[0], "x");
            assert_eq!(child_attr_names[1], "y");
            assert_eq!(child_attr_names[2], "fill");
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = SortAttrsPlugin;
        assert_eq!(plugin.name(), "sortAttrs");
        assert_eq!(plugin.description(), "Sort element attributes for better compression");
    }
}