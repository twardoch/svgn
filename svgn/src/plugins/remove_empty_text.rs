// this_file: svgn/src/plugins/remove_empty_text.rs

//! Plugin to remove empty text elements
//!
//! Removes empty `<text>`, `<tspan>`, and `<tref>` elements that serve no purpose.
//! For `<tref>` elements, they are removed if they don't have a valid `xlink:href` attribute.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin to remove empty text elements
pub struct RemoveEmptyTextPlugin;

/// Configuration parameters for the plugin
#[derive(Debug)]
pub struct RemoveEmptyTextParams {
    /// Remove empty `<text>` elements (default: true)
    pub text: bool,
    /// Remove empty `<tspan>` elements (default: true)
    pub tspan: bool,
    /// Remove `<tref>` elements without xlink:href (default: true)
    pub tref: bool,
}

impl Default for RemoveEmptyTextParams {
    fn default() -> Self {
        Self {
            text: true,
            tspan: true,
            tref: true,
        }
    }
}

impl RemoveEmptyTextParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut params = Self::default();
        
        if let Some(Value::Object(map)) = value {
            if let Some(Value::Bool(text)) = map.get("text") {
                params.text = *text;
            }
            if let Some(Value::Bool(tspan)) = map.get("tspan") {
                params.tspan = *tspan;
            }
            if let Some(Value::Bool(tref)) = map.get("tref") {
                params.tref = *tref;
            }
        }
        
        params
    }
}

impl Plugin for RemoveEmptyTextPlugin {
    fn name(&self) -> &'static str {
        "removeEmptyText"
    }

    fn description(&self) -> &'static str {
        "removes empty <text> elements"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = RemoveEmptyTextParams::from_value(params);
        remove_empty_text(&mut document.root, &config);
        Ok(())
    }
}

/// Remove empty text elements from the tree
fn remove_empty_text(element: &mut Element, config: &RemoveEmptyTextParams) {
    // First, recursively process all child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            remove_empty_text(child_element, config);
        }
    }
    
    // Then remove empty text elements from this element's children
    element.children.retain(|child| {
        if let Node::Element(child_element) = child {
            !should_remove_empty_text_element(child_element, config)
        } else {
            true // Keep non-element nodes
        }
    });
}

/// Determine if a text element should be removed
fn should_remove_empty_text_element(element: &Element, config: &RemoveEmptyTextParams) -> bool {
    match element.name.as_str() {
        "text" => {
            // Remove empty text elements if enabled
            config.text && element.children.is_empty()
        }
        "tspan" => {
            // Remove empty tspan elements if enabled
            config.tspan && element.children.is_empty()
        }
        "tref" => {
            // Remove tref elements without xlink:href if enabled
            config.tref && !element.attributes.contains_key("xlink:href")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use serde_json::json;

    #[test]
    fn test_removes_empty_text() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_text = Element::new("text");
        
        root.children.push(Node::Element(empty_text));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Empty text should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_removes_empty_tspan() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_tspan = Element::new("tspan");
        
        root.children.push(Node::Element(empty_tspan));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Empty tspan should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_removes_tref_without_href() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let tref = Element::new("tref");
        
        root.children.push(Node::Element(tref));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // tref without xlink:href should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_preserves_tref_with_href() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut tref = Element::new("tref");
        tref.attributes.insert("xlink:href".to_string(), "#someref".to_string());
        
        root.children.push(Node::Element(tref));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // tref with xlink:href should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "tref");
        }
    }

    #[test]
    fn test_preserves_text_with_content() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut text = Element::new("text");
        text.children.push(Node::Text("Hello".to_string()));
        
        root.children.push(Node::Element(text));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Text with content should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "text");
            assert!(!child.children.is_empty());
        }
    }

    #[test]
    fn test_configurable_text_removal() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_text = Element::new("text");
        
        root.children.push(Node::Element(empty_text));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        let params = json!({"text": false});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        // Empty text should be preserved when text=false
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_configurable_tspan_removal() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_tspan = Element::new("tspan");
        
        root.children.push(Node::Element(empty_tspan));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        let params = json!({"tspan": false});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        // Empty tspan should be preserved when tspan=false
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_configurable_tref_removal() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let tref = Element::new("tref");
        
        root.children.push(Node::Element(tref));
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        let params = json!({"tref": false});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        // tref without href should be preserved when tref=false
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_mixed_text_elements() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        
        // Empty text (should be removed)
        let empty_text = Element::new("text");
        root.children.push(Node::Element(empty_text));
        
        // Non-empty tspan (should be preserved)
        let mut tspan_with_content = Element::new("tspan");
        tspan_with_content.children.push(Node::Text("Content".to_string()));
        root.children.push(Node::Element(tspan_with_content));
        
        // tref without href (should be removed)
        let tref_no_href = Element::new("tref");
        root.children.push(Node::Element(tref_no_href));
        
        // tref with href (should be preserved)
        let mut tref_with_href = Element::new("tref");
        tref_with_href.attributes.insert("xlink:href".to_string(), "#ref".to_string());
        root.children.push(Node::Element(tref_with_href));
        
        document.root = root;

        let mut plugin = RemoveEmptyTextPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Should have 2 elements remaining: tspan with content and tref with href
        assert_eq!(document.root.children.len(), 2);
        
        if let Node::Element(child1) = &document.root.children[0] {
            assert_eq!(child1.name, "tspan");
        }
        if let Node::Element(child2) = &document.root.children[1] {
            assert_eq!(child2.name, "tref");
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = RemoveEmptyTextPlugin;
        assert_eq!(plugin.name(), "removeEmptyText");
        assert_eq!(plugin.description(), "removes empty <text> elements");
    }
}