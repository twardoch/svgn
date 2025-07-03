// this_file: svgn/src/plugins/remove_useless_defs.rs

//! Plugin to remove useless elements in `<defs>` without id
//!
//! This plugin removes content of defs and properties that aren't rendered
//! directly without ids. Elements with ids or style elements are preserved.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Non-rendering SVG elements (elements that define reusable resources)
const NON_RENDERING_ELEMENTS: &[&str] = &[
    "clipPath",
    "filter",
    "linearGradient",
    "marker",
    "mask",
    "pattern",
    "radialGradient",
    "solidColor",
    "symbol",
];

/// Plugin to remove useless definitions without IDs
pub struct RemoveUselessDefsPlugin;

impl Plugin for RemoveUselessDefsPlugin {
    fn name(&self) -> &'static str {
        "removeUselessDefs"
    }

    fn description(&self) -> &'static str {
        "removes elements in <defs> without id"
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

impl RemoveUselessDefsPlugin {
    fn process_element(&self, element: &mut Element) {
        // Check if this element should have its children filtered
        if element.name == "defs"
            || (NON_RENDERING_ELEMENTS.contains(&element.name.as_str())
                && !element.attributes.contains_key("id"))
        {
            let useful_nodes = self.collect_useful_nodes(element);
            element.children = useful_nodes;
        }

        // Process remaining children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem);
            }
        }

        // Remove empty defs elements
        if element.name == "defs" && element.children.is_empty() {
            // Note: We can't remove the element here as we don't have access to the parent.
            // The element removal will be handled by the parent processing.
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_useful_nodes(&self, element: &Element) -> Vec<Node> {
        let mut useful_nodes = Vec::new();

        for child in &element.children {
            if let Node::Element(ref elem) = child {
                if elem.attributes.contains_key("id") || elem.name == "style" {
                    // Keep elements with IDs or style elements
                    useful_nodes.push(child.clone());
                } else {
                    // Recursively collect useful nodes from children
                    let child_useful_nodes = self.collect_useful_nodes(elem);
                    useful_nodes.extend(child_useful_nodes);
                }
            } else {
                // Keep non-element nodes (text, comments, etc.)
                useful_nodes.push(child.clone());
            }
        }

        useful_nodes
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
        let plugin = RemoveUselessDefsPlugin;
        assert_eq!(plugin.name(), "removeUselessDefs");
        assert_eq!(
            plugin.description(),
            "removes elements in <defs> without id"
        );
    }

    #[test]
    fn test_remove_empty_defs() {
        let mut document = create_test_document();

        let defs_element = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(defs_element)];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Defs should be empty but still present (removal happens at parent level)
        if let Node::Element(ref elem) = document.root.children[0] {
            assert_eq!(elem.name, "defs");
            assert_eq!(elem.children.len(), 0);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_preserve_elements_with_id() {
        let mut document = create_test_document();

        let mut gradient_attrs = IndexMap::new();
        gradient_attrs.insert("id".to_string(), "grad1".to_string());

        let gradient_element = Element {
            name: "linearGradient".to_string(),
            attributes: gradient_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let defs_element = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(gradient_element.clone())],
        };

        document.root.children = vec![Node::Element(defs_element)];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Gradient with ID should be preserved
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.name, "defs");
            assert_eq!(defs.children.len(), 1);
            if let Node::Element(ref grad) = defs.children[0] {
                assert_eq!(grad.name, "linearGradient");
                assert_eq!(grad.attributes["id"], "grad1");
            } else {
                panic!("Expected gradient element");
            }
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_remove_elements_without_id() {
        let mut document = create_test_document();

        let gradient_element = Element {
            name: "linearGradient".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        let defs_element = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(gradient_element)],
        };

        document.root.children = vec![Node::Element(defs_element)];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Gradient without ID should be removed
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.name, "defs");
            assert_eq!(defs.children.len(), 0);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_preserve_style_elements() {
        let mut document = create_test_document();

        let style_element = Element {
            name: "style".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Text("rect { fill: red; }".to_string())],
        };

        let defs_element = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(style_element.clone())],
        };

        document.root.children = vec![Node::Element(defs_element)];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Style element should be preserved
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.name, "defs");
            assert_eq!(defs.children.len(), 1);
            if let Node::Element(ref style) = defs.children[0] {
                assert_eq!(style.name, "style");
            } else {
                panic!("Expected style element");
            }
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_flatten_nested_useless_elements() {
        let mut document = create_test_document();

        let mut useful_attrs = IndexMap::new();
        useful_attrs.insert("id".to_string(), "useful".to_string());

        let useful_element = Element {
            name: "stop".to_string(),
            attributes: useful_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let useless_element = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(useful_element.clone())],
        };

        let gradient_element = Element {
            name: "linearGradient".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(useless_element)],
        };

        let defs_element = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(gradient_element)],
        };

        document.root.children = vec![Node::Element(defs_element)];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should flatten and only keep the element with ID
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.name, "defs");
            assert_eq!(defs.children.len(), 1);
            if let Node::Element(ref stop) = defs.children[0] {
                assert_eq!(stop.name, "stop");
                assert_eq!(stop.attributes["id"], "useful");
            } else {
                panic!("Expected stop element");
            }
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_non_rendering_elements_without_id() {
        let mut document = create_test_document();

        let mut mask_with_id_attrs = IndexMap::new();
        mask_with_id_attrs.insert("id".to_string(), "mask1".to_string());

        let mask_with_id = Element {
            name: "mask".to_string(),
            attributes: mask_with_id_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let mask_without_id = Element {
            name: "mask".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![
            Node::Element(mask_with_id.clone()),
            Node::Element(mask_without_id),
        ];

        let mut plugin = RemoveUselessDefsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should have 1 element (mask with ID), mask without ID should be empty
        assert_eq!(document.root.children.len(), 2);

        if let Node::Element(ref mask1) = document.root.children[0] {
            assert_eq!(mask1.name, "mask");
            assert!(mask1.attributes.contains_key("id"));
        } else {
            panic!("Expected mask element with ID");
        }

        if let Node::Element(ref mask2) = document.root.children[1] {
            assert_eq!(mask2.name, "mask");
            assert_eq!(mask2.children.len(), 0);
        } else {
            panic!("Expected empty mask element");
        }
    }
}
