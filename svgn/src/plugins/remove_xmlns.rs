// this_file: svgn/src/plugins/remove_xmlns.rs

//! Plugin to remove xmlns attribute from SVG elements
//!
//! This plugin removes the xmlns attribute when present, which is useful for inline SVG
//! where the namespace declaration is not needed. This plugin is disabled by default.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin to remove xmlns attribute from SVG elements
pub struct RemoveXMLNSPlugin;

impl Plugin for RemoveXMLNSPlugin {
    fn name(&self) -> &'static str {
        "removeXMLNS"
    }

    fn description(&self) -> &'static str {
        "removes xmlns attribute (for inline svg, disabled by default)"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        self.process_element(&mut document.root);
        Ok(())
    }
}

impl RemoveXMLNSPlugin {
    fn process_element(&self, element: &mut Element) {
        // Remove xmlns attribute from SVG elements
        if element.name == "svg" {
            element.attributes.shift_remove("xmlns");
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem);
            }
        }
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
        let plugin = RemoveXMLNSPlugin;
        assert_eq!(plugin.name(), "removeXMLNS");
        assert_eq!(
            plugin.description(),
            "removes xmlns attribute (for inline svg, disabled by default)"
        );
    }

    #[test]
    fn test_remove_xmlns_from_svg() {
        let mut document = create_test_document();

        document.root.attributes.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 100 100".to_string());

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xmlns should be removed, other attributes preserved
        assert!(!document.root.attributes.contains_key("xmlns"));
        assert!(document.root.attributes.contains_key("viewBox"));
        assert_eq!(document.root.attributes["viewBox"], "0 0 100 100");
    }

    #[test]
    fn test_remove_xmlns_from_nested_svg() {
        let mut document = create_test_document();

        // Add nested SVG element with xmlns
        let mut nested_svg_attrs = IndexMap::new();
        nested_svg_attrs.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        nested_svg_attrs.insert("width".to_string(), "50".to_string());

        let nested_svg = Element {
            name: "svg".to_string(),
            attributes: nested_svg_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(nested_svg)];

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xmlns should be removed from nested SVG
        if let Node::Element(ref nested) = document.root.children[0] {
            assert_eq!(nested.name, "svg");
            assert!(!nested.attributes.contains_key("xmlns"));
            assert!(nested.attributes.contains_key("width"));
            assert_eq!(nested.attributes["width"], "50");
        } else {
            panic!("Expected nested svg element");
        }
    }

    #[test]
    fn test_preserve_other_xmlns_attributes() {
        let mut document = create_test_document();

        document.root.attributes.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:xlink".to_string(),
            "http://www.w3.org/1999/xlink".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:custom".to_string(),
            "http://example.com/custom".to_string(),
        );

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Only xmlns should be removed, namespaced xmlns attributes preserved
        assert!(!document.root.attributes.contains_key("xmlns"));
        assert!(document.root.attributes.contains_key("xmlns:xlink"));
        assert!(document.root.attributes.contains_key("xmlns:custom"));
    }

    #[test]
    fn test_ignore_non_svg_elements() {
        let mut document = create_test_document();

        // Add a non-SVG element with xmlns (shouldn't happen but test anyway)
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        rect_attrs.insert("width".to_string(), "100".to_string());

        let rect_element = Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(rect_element)];

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xmlns should be preserved on non-SVG elements
        if let Node::Element(ref rect) = document.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert!(rect.attributes.contains_key("xmlns"));
            assert_eq!(rect.attributes["xmlns"], "http://www.w3.org/2000/svg");
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_no_xmlns_attribute() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("viewBox".to_string(), "0 0 100 100".to_string());
        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should work fine even without xmlns attribute
        assert!(document.root.attributes.contains_key("viewBox"));
        assert!(document.root.attributes.contains_key("width"));
        assert_eq!(document.root.attributes.len(), 2);
    }

    #[test]
    fn test_complex_nested_structure() {
        let mut document = create_test_document();

        // Root SVG with xmlns
        document.root.attributes.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );

        // Nested structure: svg -> g -> svg
        let mut inner_svg_attrs = IndexMap::new();
        inner_svg_attrs.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        inner_svg_attrs.insert("x".to_string(), "10".to_string());

        let inner_svg = Element {
            name: "svg".to_string(),
            attributes: inner_svg_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let g_element = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(inner_svg)],
        };

        document.root.children = vec![Node::Element(g_element)];

        let mut plugin = RemoveXMLNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Both root and nested SVG should have xmlns removed
        assert!(!document.root.attributes.contains_key("xmlns"));

        if let Node::Element(ref g) = document.root.children[0] {
            if let Node::Element(ref inner_svg) = g.children[0] {
                assert_eq!(inner_svg.name, "svg");
                assert!(!inner_svg.attributes.contains_key("xmlns"));
                assert_eq!(inner_svg.attributes["x"], "10");
            } else {
                panic!("Expected inner svg element");
            }
        } else {
            panic!("Expected g element");
        }
    }
}
