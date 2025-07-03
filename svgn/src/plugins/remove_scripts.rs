// this_file: svgn/src/plugins/remove_scripts.rs

//! Plugin to remove script elements and script-related attributes
//!
//! This plugin removes `<script>` elements and all event attributes that could contain scripts.
//! It also handles `<a>` elements with `javascript:` URLs by replacing them with their children.

use crate::ast::{Document, Element, Node};
use crate::collections::{
    ANIMATION_EVENT_ATTRS, DOCUMENT_ELEMENT_EVENT_ATTRS, DOCUMENT_EVENT_ATTRS, GLOBAL_EVENT_ATTRS,
    GRAPHICAL_EVENT_ATTRS,
};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashMap;

/// Plugin to remove scripts and script-related attributes
pub struct RemoveScriptsPlugin;

impl Plugin for RemoveScriptsPlugin {
    fn name(&self) -> &'static str {
        "removeScripts"
    }

    fn description(&self) -> &'static str {
        "removes scripts (disabled by default)"
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

impl RemoveScriptsPlugin {
    fn process_element(&self, element: &mut Element) {
        // Remove script elements from children
        element.children.retain(|child| {
            if let Node::Element(ref elem) = child {
                elem.name != "script"
            } else {
                true
            }
        });

        // Process remaining children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem);
            }
        }

        // Remove event attributes from all elements
        self.remove_event_attributes(element);

        // Handle anchor elements with javascript: URLs
        if element.name == "a" {
            self.process_anchor_element(element);
        }
    }

    fn remove_event_attributes(&self, element: &mut Element) {
        let event_attrs: Vec<&str> = ANIMATION_EVENT_ATTRS
            .iter()
            .chain(DOCUMENT_EVENT_ATTRS.iter())
            .chain(DOCUMENT_ELEMENT_EVENT_ATTRS.iter())
            .chain(GLOBAL_EVENT_ATTRS.iter())
            .chain(GRAPHICAL_EVENT_ATTRS.iter())
            .copied()
            .collect();

        // Remove event attributes
        for attr in &event_attrs {
            element.attributes.shift_remove(*attr);
        }
    }

    fn process_anchor_element(&self, element: &mut Element) {
        // Check for javascript: URLs in href attributes
        let has_javascript_url = element.attributes.iter().any(|(attr, value)| {
            (attr == "href" || attr.ends_with(":href"))
                && value.trim_start().starts_with("javascript:")
        });

        if has_javascript_url {
            // Replace the anchor element with its non-text children
            let useful_children: Vec<Node> = element
                .children
                .drain(..)
                .filter(|child| !matches!(child, Node::Text(_)))
                .collect();

            // Clear attributes and replace with children
            element.attributes = IndexMap::new();
            element.namespaces = HashMap::new();
            element.children = useful_children;
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
        let plugin = RemoveScriptsPlugin;
        assert_eq!(plugin.name(), "removeScripts");
        assert_eq!(
            plugin.description(),
            "removes scripts (disabled by default)"
        );
    }

    #[test]
    fn test_remove_script_elements() {
        let mut document = create_test_document();

        let script_element = Element {
            name: "script".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Text("alert('hello')".to_string())],
        };

        let circle_element = Element {
            name: "circle".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![
            Node::Element(script_element),
            Node::Element(circle_element.clone()),
        ];

        let mut plugin = RemoveScriptsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should only have circle element left
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(ref elem) = document.root.children[0] {
            assert_eq!(elem.name, "circle");
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_remove_event_attributes() {
        let mut document = create_test_document();

        let mut attributes = IndexMap::new();
        attributes.insert("onclick".to_string(), "alert('click')".to_string());
        attributes.insert("onload".to_string(), "doSomething()".to_string());
        attributes.insert("fill".to_string(), "red".to_string());

        let element = Element {
            name: "rect".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveScriptsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        if let Node::Element(ref elem) = document.root.children[0] {
            // Event attributes should be removed
            assert!(!elem.attributes.contains_key("onclick"));
            assert!(!elem.attributes.contains_key("onload"));
            // Regular attributes should remain
            assert!(elem.attributes.contains_key("fill"));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_javascript_url_in_anchor() {
        let mut document = create_test_document();

        let mut attributes = IndexMap::new();
        attributes.insert("href".to_string(), "javascript:alert('test')".to_string());

        let child_text = Node::Text("Click me".to_string());

        let child_element = Node::Element(Element {
            name: "span".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        });

        let anchor_element = Element {
            name: "a".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![child_text, child_element.clone()],
        };

        document.root.children = vec![Node::Element(anchor_element)];

        let mut plugin = RemoveScriptsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        if let Node::Element(ref elem) = document.root.children[0] {
            // Href attribute should be removed
            assert!(!elem.attributes.contains_key("href"));
            // Only non-text children should remain
            assert_eq!(elem.children.len(), 1);
            if let Node::Element(ref child) = elem.children[0] {
                assert_eq!(child.name, "span");
            } else {
                panic!("Expected element child");
            }
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_normal_anchor_preserved() {
        let mut document = create_test_document();

        let mut attributes = IndexMap::new();
        attributes.insert("href".to_string(), "https://example.com".to_string());

        let anchor_element = Element {
            name: "a".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![Node::Text("Normal link".to_string())],
        };

        document.root.children = vec![Node::Element(anchor_element.clone())];

        let mut plugin = RemoveScriptsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        if let Node::Element(ref elem) = document.root.children[0] {
            // Normal href should be preserved
            assert!(elem.attributes.contains_key("href"));
            assert_eq!(elem.attributes["href"], "https://example.com");
            // Children should be preserved
            assert_eq!(elem.children.len(), 1);
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_xlink_href_with_javascript() {
        let mut document = create_test_document();

        let mut attributes = IndexMap::new();
        attributes.insert("xlink:href".to_string(), "javascript:void(0)".to_string());

        let anchor_element = Element {
            name: "a".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![Node::Element(Element {
                name: "rect".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            })],
        };

        document.root.children = vec![Node::Element(anchor_element)];

        let mut plugin = RemoveScriptsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        if let Node::Element(ref elem) = document.root.children[0] {
            // xlink:href should be removed
            assert!(!elem.attributes.contains_key("xlink:href"));
            // Child element should remain
            assert_eq!(elem.children.len(), 1);
            if let Node::Element(ref child) = elem.children[0] {
                assert_eq!(child.name, "rect");
            }
        } else {
            panic!("Expected element");
        }
    }
}
