// this_file: svgn/src/plugins/remove_elements_by_attr.rs

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct RemoveElementsByAttrConfig {
    /// IDs of elements to remove
    pub ids: Vec<String>,
    /// Class names of elements to remove
    pub classes: Vec<String>,
}

pub struct RemoveElementsByAttrPlugin;

impl Default for RemoveElementsByAttrPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoveElementsByAttrPlugin {
    pub fn new() -> Self {
        Self
    }

    fn parse_config(&self, params: Option<&Value>) -> RemoveElementsByAttrConfig {
        let mut config = RemoveElementsByAttrConfig::default();

        if let Some(Value::Object(obj)) = params {
            // Parse IDs
            if let Some(id_value) = obj.get("id") {
                match id_value {
                    Value::String(id) => config.ids.push(id.clone()),
                    Value::Array(ids) => {
                        for id in ids {
                            if let Value::String(id_str) = id {
                                config.ids.push(id_str.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Parse classes
            if let Some(class_value) = obj.get("class") {
                match class_value {
                    Value::String(class) => config.classes.push(class.clone()),
                    Value::Array(classes) => {
                        for class in classes {
                            if let Value::String(class_str) = class {
                                config.classes.push(class_str.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        config
    }

    fn should_remove_element(
        &self,
        element: &Element,
        config: &RemoveElementsByAttrConfig,
    ) -> bool {
        // Check if element ID matches any configured IDs
        if !config.ids.is_empty() {
            if let Some(id) = element.attributes.get("id") {
                if config.ids.contains(id) {
                    return true;
                }
            }
        }

        // Check if element class contains any of the configured classes
        if !config.classes.is_empty() {
            if let Some(class_attr) = element.attributes.get("class") {
                let class_list: HashSet<&str> = class_attr.split_whitespace().collect();
                for config_class in &config.classes {
                    if class_list.contains(config_class.as_str()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn process_element(&self, element: &mut Element, config: &RemoveElementsByAttrConfig) {
        // Process children, removing elements that match the criteria
        element.children.retain_mut(|child| {
            match child {
                Node::Element(ref mut child_elem) => {
                    // Check if this element should be removed
                    if self.should_remove_element(child_elem, config) {
                        return false; // Remove this element
                    }

                    // Recursively process this child element
                    self.process_element(child_elem, config);
                    true // Keep this element
                }
                _ => true, // Keep non-element nodes
            }
        });
    }
}

impl Plugin for RemoveElementsByAttrPlugin {
    fn name(&self) -> &'static str {
        "removeElementsByAttr"
    }

    fn description(&self) -> &'static str {
        "removes arbitrary elements by ID or className (disabled by default)"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = self.parse_config(params);

        // Only proceed if we have something to remove
        if config.ids.is_empty() && config.classes.is_empty() {
            return Ok(());
        }

        self.process_element(&mut document.root, &config);

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
        let plugin = RemoveElementsByAttrPlugin::new();
        assert_eq!(plugin.name(), "removeElementsByAttr");
        assert_eq!(
            plugin.description(),
            "removes arbitrary elements by ID or className (disabled by default)"
        );
    }

    #[test]
    fn test_parse_config_single_id() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config_json = serde_json::json!({
            "id": "elementToRemove"
        });

        let config = plugin.parse_config(Some(&config_json));
        assert_eq!(config.ids, vec!["elementToRemove"]);
        assert!(config.classes.is_empty());
    }

    #[test]
    fn test_parse_config_multiple_ids() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config_json = serde_json::json!({
            "id": ["elementToRemove1", "elementToRemove2"]
        });

        let config = plugin.parse_config(Some(&config_json));
        assert_eq!(config.ids, vec!["elementToRemove1", "elementToRemove2"]);
        assert!(config.classes.is_empty());
    }

    #[test]
    fn test_parse_config_single_class() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config_json = serde_json::json!({
            "class": "classToRemove"
        });

        let config = plugin.parse_config(Some(&config_json));
        assert!(config.ids.is_empty());
        assert_eq!(config.classes, vec!["classToRemove"]);
    }

    #[test]
    fn test_parse_config_multiple_classes() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config_json = serde_json::json!({
            "class": ["classToRemove1", "classToRemove2"]
        });

        let config = plugin.parse_config(Some(&config_json));
        assert!(config.ids.is_empty());
        assert_eq!(config.classes, vec!["classToRemove1", "classToRemove2"]);
    }

    #[test]
    fn test_parse_config_mixed() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config_json = serde_json::json!({
            "id": "elementToRemove",
            "class": ["classToRemove1", "classToRemove2"]
        });

        let config = plugin.parse_config(Some(&config_json));
        assert_eq!(config.ids, vec!["elementToRemove"]);
        assert_eq!(config.classes, vec!["classToRemove1", "classToRemove2"]);
    }

    #[test]
    fn test_should_remove_element_by_id() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config = RemoveElementsByAttrConfig {
            ids: vec!["removeMe".to_string()],
            classes: vec![],
        };

        let mut element_attrs = IndexMap::new();
        element_attrs.insert("id".to_string(), "removeMe".to_string());
        let element = Element {
            name: "rect".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        assert!(plugin.should_remove_element(&element, &config));

        // Test element that shouldn't be removed
        let mut element_attrs2 = IndexMap::new();
        element_attrs2.insert("id".to_string(), "keepMe".to_string());
        let element2 = Element {
            name: "rect".to_string(),
            attributes: element_attrs2,
            namespaces: HashMap::new(),
            children: vec![],
        };

        assert!(!plugin.should_remove_element(&element2, &config));
    }

    #[test]
    fn test_should_remove_element_by_class() {
        let plugin = RemoveElementsByAttrPlugin::new();
        let config = RemoveElementsByAttrConfig {
            ids: vec![],
            classes: vec!["removeMe".to_string()],
        };

        // Test element with matching class
        let mut element_attrs = IndexMap::new();
        element_attrs.insert(
            "class".to_string(),
            "someClass removeMe anotherClass".to_string(),
        );
        let element = Element {
            name: "rect".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        assert!(plugin.should_remove_element(&element, &config));

        // Test element that shouldn't be removed
        let mut element_attrs2 = IndexMap::new();
        element_attrs2.insert(
            "class".to_string(),
            "someClass keepMe anotherClass".to_string(),
        );
        let element2 = Element {
            name: "rect".to_string(),
            attributes: element_attrs2,
            namespaces: HashMap::new(),
            children: vec![],
        };

        assert!(!plugin.should_remove_element(&element2, &config));
    }

    #[test]
    fn test_apply_removes_by_id() {
        let mut plugin = RemoveElementsByAttrPlugin::new();
        let mut doc = create_test_document();

        // Add element to remove
        let mut attrs_remove = IndexMap::new();
        attrs_remove.insert("id".to_string(), "elementToRemove".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: attrs_remove,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        // Add element to keep
        let mut attrs_keep = IndexMap::new();
        attrs_keep.insert("id".to_string(), "elementToKeep".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "circle".to_string(),
            attributes: attrs_keep,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let config = serde_json::json!({
            "id": "elementToRemove"
        });

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, Some(&config));
        assert!(result.is_ok());

        // Should have only one element remaining
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(element) = &doc.root.children[0] {
            assert_eq!(element.name, "circle");
            assert_eq!(
                element.attributes.get("id"),
                Some(&"elementToKeep".to_string())
            );
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_apply_removes_by_class() {
        let mut plugin = RemoveElementsByAttrPlugin::new();
        let mut doc = create_test_document();

        // Add element to remove
        let mut attrs_remove = IndexMap::new();
        attrs_remove.insert(
            "class".to_string(),
            "some-class removeMe another-class".to_string(),
        );
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: attrs_remove,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        // Add element to keep
        let mut attrs_keep = IndexMap::new();
        attrs_keep.insert(
            "class".to_string(),
            "some-class keep-me another-class".to_string(),
        );
        doc.root.children.push(Node::Element(Element {
            name: "circle".to_string(),
            attributes: attrs_keep,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let config = serde_json::json!({
            "class": "removeMe"
        });

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, Some(&config));
        assert!(result.is_ok());

        // Should have only one element remaining
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(element) = &doc.root.children[0] {
            assert_eq!(element.name, "circle");
            assert_eq!(
                element.attributes.get("class"),
                Some(&"some-class keep-me another-class".to_string())
            );
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_apply_no_config_does_nothing() {
        let mut plugin = RemoveElementsByAttrPlugin::new();
        let mut doc = create_test_document();

        // Add some elements
        let mut attrs = IndexMap::new();
        attrs.insert("id".to_string(), "someId".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());

        // Should still have the element
        assert_eq!(doc.root.children.len(), 1);
    }

    #[test]
    fn test_apply_recursive() {
        let mut plugin = RemoveElementsByAttrPlugin::new();
        let mut doc = create_test_document();

        // Create nested structure
        let mut nested_attrs = IndexMap::new();
        nested_attrs.insert("id".to_string(), "removeMe".to_string());
        let nested_element = Element {
            name: "rect".to_string(),
            attributes: nested_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        let mut group_attrs = IndexMap::new();
        group_attrs.insert("id".to_string(), "group".to_string());
        let group = Element {
            name: "g".to_string(),
            attributes: group_attrs,
            namespaces: HashMap::new(),
            children: vec![Node::Element(nested_element)],
        };

        doc.root.children.push(Node::Element(group));

        let config = serde_json::json!({
            "id": "removeMe"
        });

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, Some(&config));
        assert!(result.is_ok());

        // Group should remain but nested element should be removed
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(group) = &doc.root.children[0] {
            assert_eq!(group.name, "g");
            assert_eq!(group.children.len(), 0); // Nested element removed
        } else {
            panic!("Expected group element");
        }
    }
}
