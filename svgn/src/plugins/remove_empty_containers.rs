// this_file: svgn/src/plugins/remove_empty_containers.rs

//! Plugin to remove empty container elements
//!
//! Removes container elements that have no children, with special handling
//! for certain cases where empty containers have semantic meaning.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Set of SVG container elements that can be removed when empty
/// Based on https://www.w3.org/TR/SVG11/intro.html#TermContainerElement
static CONTAINER_ELEMENTS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "a",
        "defs", 
        "foreignObject",
        "g",
        "marker",
        "mask",
        "missing-glyph",
        "pattern",
        "svg",
        "switch",
        "symbol",
    ]
    .into_iter()
    .collect()
});

/// Plugin to remove empty container elements
pub struct RemoveEmptyContainersPlugin;

impl Plugin for RemoveEmptyContainersPlugin {
    fn name(&self) -> &'static str {
        "removeEmptyContainers"
    }

    fn description(&self) -> &'static str {
        "removes empty container elements"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Process the tree bottom-up to handle nested containers correctly
        remove_empty_containers(&mut document.root, None);
        Ok(())
    }
}

/// Remove empty containers from the tree, processing children first
fn remove_empty_containers(element: &mut Element, _parent_name: Option<&str>) {
    // First, recursively process all child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            remove_empty_containers(child_element, Some(&element.name));
        }
    }
    
    // Then remove empty container children from this element
    element.children.retain(|child| {
        if let Node::Element(child_element) = child {
            !should_remove_empty_container(child_element, Some(&element.name))
        } else {
            true // Keep non-element nodes (text, comments)
        }
    });
}

/// Determine if an empty container element should be removed
fn should_remove_empty_container(element: &Element, parent_name: Option<&str>) -> bool {
    // Only consider container elements
    if !CONTAINER_ELEMENTS.contains(element.name.as_str()) {
        return false;
    }
    
    // Must be empty (no children)
    if !element.children.is_empty() {
        return false;
    }
    
    // Don't remove root SVG elements
    if element.name == "svg" {
        return false;
    }
    
    // Empty patterns may contain reusable configuration
    if element.name == "pattern" && !element.attributes.is_empty() {
        return false;
    }
    
    // Empty <mask> with ID hides masked element
    if element.name == "mask" && element.attributes.contains_key("id") {
        return false;
    }
    
    // Don't remove elements that are direct children of <switch>
    if parent_name == Some("switch") {
        return false;
    }
    
    // The <g> may not have content, but the filter may cause a rectangle
    // to be created and filled with pattern
    if element.name == "g" && element.attributes.contains_key("filter") {
        return false;
    }
    
    // If we get here, it's safe to remove this empty container
    true
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};

    #[test]
    fn test_removes_empty_defs() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_defs = Element::new("defs");
        
        root.children.push(Node::Element(empty_defs));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Empty defs should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_removes_empty_g() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let empty_g = Element::new("g");
        
        root.children.push(Node::Element(empty_g));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Empty g should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_preserves_svg_root() {
        let mut document = Document::new();
        let root = Element::new("svg");
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // SVG root should never be removed even if empty
        assert_eq!(document.root.name, "svg");
    }

    #[test]
    fn test_preserves_pattern_with_attributes() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut pattern = Element::new("pattern");
        pattern.attributes.insert("id".to_string(), "mypattern".to_string());
        
        root.children.push(Node::Element(pattern));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Pattern with attributes should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "pattern");
        }
    }

    #[test]
    fn test_preserves_mask_with_id() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut mask = Element::new("mask");
        mask.attributes.insert("id".to_string(), "mymask".to_string());
        
        root.children.push(Node::Element(mask));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Mask with ID should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "mask");
        }
    }

    #[test]
    fn test_preserves_g_with_filter() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut g = Element::new("g");
        g.attributes.insert("filter".to_string(), "url(#myfilter)".to_string());
        
        root.children.push(Node::Element(g));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Group with filter should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "g");
        }
    }

    #[test]
    fn test_preserves_elements_in_switch() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut switch = Element::new("switch");
        let empty_g = Element::new("g");
        
        switch.children.push(Node::Element(empty_g));
        root.children.push(Node::Element(switch));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Elements in switch should be preserved
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(switch_elem) = &document.root.children[0] {
            assert_eq!(switch_elem.name, "switch");
            assert_eq!(switch_elem.children.len(), 1);
        }
    }

    #[test]
    fn test_removes_nested_empty_containers() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let mut outer_g = Element::new("g");
        let inner_g = Element::new("g");
        
        outer_g.children.push(Node::Element(inner_g));
        root.children.push(Node::Element(outer_g));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Both nested empty containers should be removed
        assert!(document.root.children.is_empty());
    }

    #[test]
    fn test_keeps_non_container_elements() {
        let mut document = Document::new();
        let mut root = Element::new("svg");
        let rect = Element::new("rect");
        let empty_g = Element::new("g");
        
        root.children.push(Node::Element(rect));
        root.children.push(Node::Element(empty_g));
        document.root = root;

        let mut plugin = RemoveEmptyContainersPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // Should keep rect and remove empty g
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(child) = &document.root.children[0] {
            assert_eq!(child.name, "rect");
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = RemoveEmptyContainersPlugin;
        assert_eq!(plugin.name(), "removeEmptyContainers");
        assert_eq!(plugin.description(), "removes empty container elements");
    }
}