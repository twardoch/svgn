// this_file: svgn/src/plugins/remove_empty_attrs.rs

//! Plugin to remove attributes with empty values
//!
//! Removes attributes that have empty string values, which are generally
//! unnecessary. However, preserves conditional processing attributes where
//! empty values have semantic meaning (prevent element rendering).

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Set of conditional processing attributes that should be preserved even when empty
/// as they have semantic meaning for element rendering
static CONDITIONAL_PROCESSING_ATTRS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "requiredExtensions",
        "requiredFeatures", 
        "systemLanguage",
    ]
    .into_iter()
    .collect()
});

/// Plugin to remove attributes with empty values
pub struct RemoveEmptyAttrsPlugin;

impl Plugin for RemoveEmptyAttrsPlugin {
    fn name(&self) -> &'static str {
        "removeEmptyAttrs"
    }

    fn description(&self) -> &'static str {
        "removes empty attributes"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        visit_elements(&mut document.root, remove_empty_attrs);
        Ok(())
    }
}

/// Visit all elements in the AST and apply the transformation function
fn visit_elements(element: &mut Element, transform: fn(&mut Element)) {
    transform(element);
    
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            visit_elements(child_element, transform);
        }
    }
}

/// Remove empty attributes from an element, preserving conditional processing attributes
fn remove_empty_attrs(element: &mut Element) {
    element.attributes.retain(|name, value| {
        // Preserve conditional processing attributes even if empty
        if CONDITIONAL_PROCESSING_ATTRS.contains(name.as_str()) {
            return true;
        }
        
        // Remove attributes with empty values
        !value.is_empty()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};

    #[test]
    fn test_removes_empty_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        // Add some attributes, including empty ones
        element.attributes.insert("width".to_string(), "100".to_string());
        element.attributes.insert("height".to_string(), "".to_string()); // empty
        element.attributes.insert("fill".to_string(), "red".to_string());
        element.attributes.insert("stroke".to_string(), "".to_string()); // empty
        
        document.root = element;

        let mut plugin = RemoveEmptyAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Should keep non-empty attributes
        assert!(document.root.attributes.contains_key("width"));
        assert!(document.root.attributes.contains_key("fill"));
        
        // Should remove empty attributes
        assert!(!document.root.attributes.contains_key("height"));
        assert!(!document.root.attributes.contains_key("stroke"));
    }

    #[test]
    fn test_preserves_conditional_processing_attrs() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        
        // Add conditional processing attributes with empty values
        element.attributes.insert("requiredExtensions".to_string(), "".to_string());
        element.attributes.insert("requiredFeatures".to_string(), "".to_string());
        element.attributes.insert("systemLanguage".to_string(), "".to_string());
        element.attributes.insert("width".to_string(), "".to_string()); // regular empty attr
        
        document.root = element;

        let mut plugin = RemoveEmptyAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Should preserve conditional processing attributes even when empty
        assert!(document.root.attributes.contains_key("requiredExtensions"));
        assert!(document.root.attributes.contains_key("requiredFeatures"));
        assert!(document.root.attributes.contains_key("systemLanguage"));
        
        // Should remove regular empty attributes
        assert!(!document.root.attributes.contains_key("width"));
    }

    #[test]
    fn test_nested_elements() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut child = Element::new("g");
        
        // Add empty attributes to both elements
        root.attributes.insert("width".to_string(), "100".to_string());
        root.attributes.insert("height".to_string(), "".to_string()); // empty
        
        child.attributes.insert("fill".to_string(), "red".to_string());
        child.attributes.insert("stroke".to_string(), "".to_string()); // empty
        
        root.children.push(Node::Element(child));
        document.root = root;

        let mut plugin = RemoveEmptyAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Check root element
        assert!(document.root.attributes.contains_key("width"));
        assert!(!document.root.attributes.contains_key("height"));
        
        // Check child element
        if let Node::Element(child_element) = &document.root.children[0] {
            assert!(child_element.attributes.contains_key("fill"));
            assert!(!child_element.attributes.contains_key("stroke"));
        } else {
            panic!("Expected child element");
        }
    }

    #[test]
    fn test_no_attributes() {
        let mut document = Document::new();
        let element = Element::new("rect");
        document.root = element;

        let mut plugin = RemoveEmptyAttrsPlugin;
        let result = plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None);
        
        assert!(result.is_ok());
        assert!(document.root.attributes.is_empty());
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = RemoveEmptyAttrsPlugin;
        assert_eq!(plugin.name(), "removeEmptyAttrs");
        assert_eq!(plugin.description(), "removes empty attributes");
    }
}