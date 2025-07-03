// this_file: svgn/src/plugins/remove_view_box.rs

//! Plugin to remove viewBox attribute when possible
//!
//! This plugin removes the viewBox attribute when it coincides with width/height box.
//! For example, `<svg width="100" height="50" viewBox="0 0 100 50">` becomes
//! `<svg width="100" height="50">`.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Elements that can have viewBox attributes
const VIEWBOX_ELEMENTS: &[&str] = &["pattern", "svg", "symbol"];

/// Plugin to remove redundant viewBox attributes
pub struct RemoveViewBoxPlugin;

impl Plugin for RemoveViewBoxPlugin {
    fn name(&self) -> &'static str {
        "removeViewBox"
    }

    fn description(&self) -> &'static str {
        "removes viewBox attribute when possible"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        self.process_element(&mut document.root, true);
        Ok(())
    }
}

impl RemoveViewBoxPlugin {
    fn process_element(&self, element: &mut Element, is_root: bool) {
        // Check if this element can have a viewBox
        if VIEWBOX_ELEMENTS.contains(&element.name.as_str()) {
            self.check_and_remove_viewbox(element, is_root);
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem, false);
            }
        }
    }

    fn check_and_remove_viewbox(&self, element: &mut Element, is_root: bool) {
        // Skip nested SVG elements (keep their viewBox)
        if element.name == "svg" && !is_root {
            return;
        }

        // Check if element has viewBox, width, and height attributes
        if let (Some(view_box), Some(width), Some(height)) = (
            element.attributes.get("viewBox"),
            element.attributes.get("width"),
            element.attributes.get("height"),
        ) {
            if self.can_remove_viewbox(view_box, width, height) {
                element.attributes.shift_remove("viewBox");
            }
        }
    }

    fn can_remove_viewbox(&self, view_box: &str, width: &str, height: &str) -> bool {
        // Parse viewBox values (format: "min-x min-y width height")
        let view_box_parts: Vec<&str> = view_box
            .split(&[' ', ','][..])
            .filter(|s| !s.is_empty())
            .collect();

        if view_box_parts.len() != 4 {
            return false;
        }

        // Check if viewBox starts at origin (0,0)
        if view_box_parts[0] != "0" || view_box_parts[1] != "0" {
            return false;
        }

        // Remove 'px' suffix from width/height if present
        let width_value = width.strip_suffix("px").unwrap_or(width);
        let height_value = height.strip_suffix("px").unwrap_or(height);

        // Check if viewBox width/height matches element width/height
        view_box_parts[2] == width_value && view_box_parts[3] == height_value
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
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
            ..Default::default()
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = RemoveViewBoxPlugin;
        assert_eq!(plugin.name(), "removeViewBox");
        assert_eq!(
            plugin.description(),
            "removes viewBox attribute when possible"
        );
    }

    #[test]
    fn test_remove_redundant_viewbox() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 100 50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be removed
        assert!(!document.root.attributes.contains_key("viewBox"));
        assert_eq!(document.root.attributes["width"], "100");
        assert_eq!(document.root.attributes["height"], "50");
    }

    #[test]
    fn test_keep_viewbox_with_px_units() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100px".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50px".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 100 50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be removed (px suffix is ignored)
        assert!(!document.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_keep_viewbox_different_origin() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "10 10 100 50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be kept (different origin)
        assert!(document.root.attributes.contains_key("viewBox"));
        assert_eq!(document.root.attributes["viewBox"], "10 10 100 50");
    }

    #[test]
    fn test_keep_viewbox_different_dimensions() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 200 100".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be kept (different dimensions)
        assert!(document.root.attributes.contains_key("viewBox"));
        assert_eq!(document.root.attributes["viewBox"], "0 0 200 100");
    }

    #[test]
    fn test_keep_viewbox_missing_width_or_height() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        // Missing height
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 100 50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be kept (missing height)
        assert!(document.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_comma_separated_viewbox() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0,0,100,50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be removed (comma-separated format)
        assert!(!document.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_mixed_separator_viewbox() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "50".to_string());
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0, 0 100,50".to_string());

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // viewBox should be removed (mixed separators)
        assert!(!document.root.attributes.contains_key("viewBox"));
    }

    #[test]
    fn test_pattern_element() {
        let mut document = create_test_document();

        let mut pattern_attrs = IndexMap::new();
        pattern_attrs.insert("width".to_string(), "20".to_string());
        pattern_attrs.insert("height".to_string(), "30".to_string());
        pattern_attrs.insert("viewBox".to_string(), "0 0 20 30".to_string());

        let pattern_element = Element {
            name: "pattern".to_string(),
            attributes: pattern_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(pattern_element)];

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Pattern viewBox should be removed
        if let Node::Element(ref pattern) = document.root.children[0] {
            assert_eq!(pattern.name, "pattern");
            assert!(!pattern.attributes.contains_key("viewBox"));
            assert_eq!(pattern.attributes["width"], "20");
            assert_eq!(pattern.attributes["height"], "30");
        } else {
            panic!("Expected pattern element");
        }
    }

    #[test]
    fn test_nested_svg_preserved() {
        let mut document = create_test_document();

        let mut nested_svg_attrs = IndexMap::new();
        nested_svg_attrs.insert("width".to_string(), "50".to_string());
        nested_svg_attrs.insert("height".to_string(), "25".to_string());
        nested_svg_attrs.insert("viewBox".to_string(), "0 0 50 25".to_string());

        let nested_svg = Element {
            name: "svg".to_string(),
            attributes: nested_svg_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(nested_svg)];

        let mut plugin = RemoveViewBoxPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Nested SVG viewBox should be preserved
        if let Node::Element(ref nested) = document.root.children[0] {
            assert_eq!(nested.name, "svg");
            assert!(nested.attributes.contains_key("viewBox"));
            assert_eq!(nested.attributes["viewBox"], "0 0 50 25");
        } else {
            panic!("Expected nested svg element");
        }
    }
}
