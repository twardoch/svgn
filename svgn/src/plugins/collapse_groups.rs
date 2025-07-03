// this_file: svgn/src/plugins/collapse_groups.rs

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use crate::collections::{ANIMATION_ELEMS, INHERITABLE_ATTRS};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollapseGroupsConfig {}

pub struct CollapseGroupsPlugin;

impl CollapseGroupsPlugin {
    /// Check if an element or its descendants have animated attributes
    fn has_animated_attr(node: &Node, attr_name: &str) -> bool {
        match node {
            Node::Element(element) => {
                // Check if this is an animation element targeting the attribute
                if ANIMATION_ELEMS.contains(element.name.as_str()) {
                    if let Some(attribute_name) = element.attributes.get("attributeName") {
                        if attribute_name == attr_name {
                            return true;
                        }
                    }
                }
                
                // Check children recursively
                for child in &element.children {
                    if Self::has_animated_attr(child, attr_name) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Process a group element to potentially collapse it
    fn process_group(&self, element: &mut Element, parent_name: Option<&str>) -> bool {
        // Skip if parent is root or switch
        if parent_name.is_none() || parent_name == Some("switch") {
            return false;
        }

        // Only process <g> elements with children
        if element.name != "g" || element.children.is_empty() {
            return false;
        }

        // Move group attributes to single child element
        if !element.attributes.is_empty() && element.children.len() == 1 {
            if let Node::Element(first_child) = &mut element.children[0] {
                // TODO: Add style computation for filter check when style support is implemented
                let node_has_filter = element.attributes.contains_key("filter");
                
                // Check conditions for moving attributes
                if first_child.attributes.get("id").is_none()
                    && !node_has_filter
                    && (element.attributes.get("class").is_none() 
                        || first_child.attributes.get("class").is_none())
                    && ((element.attributes.get("clip-path").is_none() 
                        && element.attributes.get("mask").is_none())
                        || (first_child.name == "g" 
                            && element.attributes.get("transform").is_none()
                            && first_child.attributes.get("transform").is_none()))
                {
                    // Check if any attribute would conflict with animation
                    for (name, _) in &element.attributes {
                        if Self::has_animated_attr(&Node::Element(first_child.clone()), name) {
                            return false;
                        }
                    }

                    // Move attributes from group to child
                    for (name, value) in element.attributes.drain(..) {
                        match first_child.attributes.get(&name) {
                            None => {
                                first_child.attributes.insert(name, value);
                            }
                            Some(existing_value) => {
                                if name == "transform" {
                                    // Concatenate transforms
                                    let new_value = format!("{} {}", value, existing_value);
                                    first_child.attributes.insert(name, new_value);
                                } else if existing_value == "inherit" {
                                    // Replace inherit with actual value
                                    first_child.attributes.insert(name, value);
                                } else if !INHERITABLE_ATTRS.contains(name.as_str()) 
                                    && existing_value != &value {
                                    // Non-inheritable attributes must match
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check if we can collapse groups without attributes
        if element.attributes.is_empty() {
            // Check if any child is an animation element
            for child in &element.children {
                if let Node::Element(child_elem) = child {
                    if ANIMATION_ELEMS.contains(child_elem.name.as_str()) {
                        return false;
                    }
                }
            }
            // This group can be collapsed
            return true;
        }

        false
    }

    fn process_node(&self, node: &mut Node, parent_name: Option<&str>) -> Vec<Node> {
        match node {
            Node::Element(element) => {
                // First, process all children recursively
                let mut new_children = Vec::new();
                for mut child in element.children.drain(..) {
                    let collapsed = self.process_node(&mut child, Some(&element.name));
                    new_children.extend(collapsed);
                }
                element.children = new_children;

                // Then check if this element should be collapsed
                if element.name == "g" && self.process_group(element, parent_name) {
                    // Return the children directly, effectively removing this group
                    element.children.drain(..).collect()
                } else {
                    vec![Node::Element(element.clone())]
                }
            }
            _ => vec![node.clone()],
        }
    }
}

impl Plugin for CollapseGroupsPlugin {
    fn name(&self) -> &'static str {
        "collapseGroups"
    }

    fn description(&self) -> &'static str {
        "collapses useless groups"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&serde_json::Value>) -> PluginResult<()> {
        let _config: CollapseGroupsConfig = if let Some(p) = params {
            serde_json::from_value(p.clone()).map_err(|e| PluginError::InvalidConfig(e.to_string()))?
        } else {
            CollapseGroupsConfig {}
        };

        // Process the root element
        let mut root_node = Node::Element(document.root.clone());
        let processed = self.process_node(&mut root_node, None);
        if let Some(Node::Element(new_root)) = processed.into_iter().next() {
            document.root = new_root;
        }

        Ok(())
    }

    fn validate_params(&self, params: Option<&serde_json::Value>) -> PluginResult<()> {
        if params.is_none() || params.unwrap().is_object() {
            Ok(())
        } else {
            Err(PluginError::InvalidConfig("Configuration must be an object or null".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::stringifier::Stringifier;

    #[test]
    fn test_collapse_empty_group() {
        let input = r#"<svg><g><rect width="10" height="10"/></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Check that the group was collapsed
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.get("width"), Some(&"10".to_string()));
            assert_eq!(rect.attributes.get("height"), Some(&"10".to_string()));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_move_attributes_to_single_child() {
        let input = r#"<svg><g fill="red"><rect width="10" height="10"/></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Check that the group was collapsed and attributes moved
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(rect.attributes.get("width"), Some(&"10".to_string()));
            assert_eq!(rect.attributes.get("height"), Some(&"10".to_string()));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_preserve_group_with_multiple_children() {
        let input = r#"<svg><g fill="red"><rect width="10" height="10"/><circle r="5"/></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Group should be preserved because it has multiple children
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(group) = &doc.root.children[0] {
            assert_eq!(group.name, "g");
            assert_eq!(group.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(group.children.len(), 2);
        } else {
            panic!("Expected group element");
        }
    }

    #[test]
    fn test_concatenate_transforms() {
        let input = r#"<svg><g transform="translate(10,10)"><rect transform="scale(2)" width="10" height="10"/></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Check that transforms were concatenated
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.get("transform"), Some(&"translate(10,10) scale(2)".to_string()));
            assert_eq!(rect.attributes.get("width"), Some(&"10".to_string()));
            assert_eq!(rect.attributes.get("height"), Some(&"10".to_string()));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_nested_groups() {
        let input = r#"<svg><g><g><rect width="10" height="10"/></g></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Both groups should be collapsed
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.get("width"), Some(&"10".to_string()));
            assert_eq!(rect.attributes.get("height"), Some(&"10".to_string()));
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_preserve_group_with_id() {
        let input = r#"<svg><g id="mygroup"><rect id="myrect" width="10" height="10"/></g></svg>"#;
        
        let parser = Parser::new();
        let mut doc = parser.parse(input).unwrap();
        let mut plugin = CollapseGroupsPlugin;
        let config = serde_json::json!({});
        let info = PluginInfo::default();
        
        plugin.apply(&mut doc, &info, Some(&config)).unwrap();
        
        // Group should NOT be preserved just because child has id
        // The group itself has id, so it can't move attributes to child
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(group) = &doc.root.children[0] {
            assert_eq!(group.name, "g");
            assert_eq!(group.attributes.get("id"), Some(&"mygroup".to_string()));
            assert_eq!(group.children.len(), 1);
        } else {
            panic!("Expected group element");
        }
    }
}