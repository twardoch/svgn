// this_file: svgn/src/plugins/remove_unused_ns.rs

//! Plugin to remove unused namespace declarations
//!
//! This plugin removes unused namespace declarations from the root SVG element
//! which are not used in elements or attributes throughout the document.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashSet;

/// Plugin to remove unused namespace declarations
pub struct RemoveUnusedNSPlugin;

impl Plugin for RemoveUnusedNSPlugin {
    fn name(&self) -> &'static str {
        "removeUnusedNS"
    }

    fn description(&self) -> &'static str {
        "removes unused namespaces declaration"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        // First, collect all namespace declarations from the root SVG element
        let mut unused_namespaces = HashSet::new();

        // Collect xmlns: attributes from root element
        for attr_name in document.root.attributes.keys() {
            if attr_name.starts_with("xmlns:") {
                let local = attr_name.strip_prefix("xmlns:").unwrap();
                unused_namespaces.insert(local.to_string());
            }
        }

        // Traverse the document and remove used namespaces from the unused set
        self.check_usage(&document.root, &mut unused_namespaces);

        // Remove unused namespace declarations from root element
        for ns in &unused_namespaces {
            let xmlns_attr = format!("xmlns:{}", ns);
            document.root.attributes.shift_remove(&xmlns_attr);
        }

        Ok(())
    }
}

impl RemoveUnusedNSPlugin {
    #[allow(clippy::only_used_in_recursion)]
    fn check_usage(&self, element: &Element, unused_namespaces: &mut HashSet<String>) {
        // Check if element name uses a namespace
        if element.name.contains(':') {
            let parts: Vec<&str> = element.name.split(':').collect();
            if parts.len() >= 2 {
                let ns = parts[0];
                unused_namespaces.remove(ns);
            }
        }

        // Check if any attributes use namespaces
        for attr_name in element.attributes.keys() {
            if attr_name.contains(':') {
                let parts: Vec<&str> = attr_name.split(':').collect();
                if parts.len() >= 2 {
                    let ns = parts[0];
                    unused_namespaces.remove(ns);
                }
            }
        }

        // Recursively check children
        for child in &element.children {
            if let Node::Element(ref elem) = child {
                self.check_usage(elem, unused_namespaces);
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
        let plugin = RemoveUnusedNSPlugin;
        assert_eq!(plugin.name(), "removeUnusedNS");
        assert_eq!(
            plugin.description(),
            "removes unused namespaces declaration"
        );
    }

    #[test]
    fn test_remove_unused_namespace() {
        let mut document = create_test_document();

        // Add unused namespace
        document.root.attributes.insert(
            "xmlns:unused".to_string(),
            "http://example.com/unused".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:xlink".to_string(),
            "http://www.w3.org/1999/xlink".to_string(),
        );

        // Add an element that uses xlink
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("xlink:href".to_string(), "#test".to_string());

        let rect_element = Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(rect_element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // unused namespace should be removed, xlink should remain
        assert!(!document.root.attributes.contains_key("xmlns:unused"));
        assert!(document.root.attributes.contains_key("xmlns:xlink"));
    }

    #[test]
    fn test_preserve_used_namespace_in_element_name() {
        let mut document = create_test_document();

        // Add namespace
        document.root.attributes.insert(
            "xmlns:svg".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );

        // Add a child element with namespaced name
        let ns_element = Element {
            name: "svg:g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(ns_element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // svg namespace should be preserved
        assert!(document.root.attributes.contains_key("xmlns:svg"));
    }

    #[test]
    fn test_preserve_used_namespace_in_attributes() {
        let mut document = create_test_document();

        // Add namespace
        document.root.attributes.insert(
            "xmlns:custom".to_string(),
            "http://example.com/custom".to_string(),
        );

        // Add an element with namespaced attribute
        let mut element_attrs = IndexMap::new();
        element_attrs.insert("custom:data".to_string(), "value".to_string());

        let element = Element {
            name: "rect".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // custom namespace should be preserved
        assert!(document.root.attributes.contains_key("xmlns:custom"));
    }

    #[test]
    fn test_remove_all_unused_namespaces() {
        let mut document = create_test_document();

        // Add multiple unused namespaces
        document.root.attributes.insert(
            "xmlns:ns1".to_string(),
            "http://example.com/ns1".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:ns2".to_string(),
            "http://example.com/ns2".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:ns3".to_string(),
            "http://example.com/ns3".to_string(),
        );

        // Add an element without any namespace usage
        let element = Element {
            name: "rect".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // All unused namespaces should be removed
        assert!(!document.root.attributes.contains_key("xmlns:ns1"));
        assert!(!document.root.attributes.contains_key("xmlns:ns2"));
        assert!(!document.root.attributes.contains_key("xmlns:ns3"));
    }

    #[test]
    fn test_no_namespaces_to_remove() {
        let mut document = create_test_document();

        // No xmlns: attributes
        document
            .root
            .attributes
            .insert("width".to_string(), "100".to_string());
        document
            .root
            .attributes
            .insert("height".to_string(), "100".to_string());

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should still have original attributes
        assert_eq!(document.root.attributes["width"], "100");
        assert_eq!(document.root.attributes["height"], "100");
    }

    #[test]
    fn test_nested_element_namespace_usage() {
        let mut document = create_test_document();

        // Add namespace
        document.root.attributes.insert(
            "xmlns:deep".to_string(),
            "http://example.com/deep".to_string(),
        );

        // Create nested structure where namespace is used deep in the tree
        let mut deep_attrs = IndexMap::new();
        deep_attrs.insert("deep:attr".to_string(), "value".to_string());

        let deep_element = Element {
            name: "text".to_string(),
            attributes: deep_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let middle_element = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(deep_element)],
        };

        let container_element = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(middle_element)],
        };

        document.root.children = vec![Node::Element(container_element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // deep namespace should be preserved (used in nested element)
        assert!(document.root.attributes.contains_key("xmlns:deep"));
    }

    #[test]
    fn test_mixed_used_and_unused_namespaces() {
        let mut document = create_test_document();

        // Add multiple namespaces
        document.root.attributes.insert(
            "xmlns:used".to_string(),
            "http://example.com/used".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:unused".to_string(),
            "http://example.com/unused".to_string(),
        );
        document.root.attributes.insert(
            "xmlns:alsounused".to_string(),
            "http://example.com/alsounused".to_string(),
        );

        // Add an element that uses only one namespace
        let mut element_attrs = IndexMap::new();
        element_attrs.insert("used:data".to_string(), "value".to_string());

        let element = Element {
            name: "rect".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveUnusedNSPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Only used namespace should remain
        assert!(document.root.attributes.contains_key("xmlns:used"));
        assert!(!document.root.attributes.contains_key("xmlns:unused"));
        assert!(!document.root.attributes.contains_key("xmlns:alsounused"));
    }
}
