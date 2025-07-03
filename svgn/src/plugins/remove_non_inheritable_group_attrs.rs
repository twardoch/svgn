// this_file: svgn/src/plugins/remove_non_inheritable_group_attrs.rs

//! Plugin to remove non-inheritable attributes from groups
//!
//! This plugin removes presentation attributes from `<g>` elements that are not inheritable.
//! These attributes don't apply to groups themselves and aren't inherited by their children,
//! so they have no effect and can be safely removed.

use crate::ast::{Document, Element, Node};
use crate::collections::{INHERITABLE_ATTRS, PRESENTATION_ATTRS};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin to remove non-inheritable presentation attributes from groups
pub struct RemoveNonInheritableGroupAttrsPlugin;

impl Plugin for RemoveNonInheritableGroupAttrsPlugin {
    fn name(&self) -> &'static str {
        "removeNonInheritableGroupAttrs"
    }

    fn description(&self) -> &'static str {
        "removes non-inheritable group's presentation attributes"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        self.process_element(&mut document.root);
        Ok(())
    }
}

impl RemoveNonInheritableGroupAttrsPlugin {
    fn process_element(&mut self, element: &mut Element) {
        // Only process <g> elements
        if element.name == "g" {
            self.remove_non_inheritable_attrs(element);
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.process_element(child_elem);
            }
        }
    }

    fn remove_non_inheritable_attrs(&self, element: &mut Element) {
        let mut attrs_to_remove = Vec::new();

        for attr_name in element.attributes.keys() {
            // Check if this is a presentation attribute that is NOT inheritable
            if PRESENTATION_ATTRS.contains(attr_name.as_str()) 
                && !INHERITABLE_ATTRS.contains(attr_name.as_str()) {
                attrs_to_remove.push(attr_name.clone());
            }
        }

        // Remove the non-inheritable attributes
        for attr in attrs_to_remove {
            element.attributes.shift_remove(&attr);
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use std::collections::HashMap;
    use indexmap::IndexMap;

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

    fn create_group_with_attrs(attrs: Vec<(&str, &str)>) -> Element {
        let mut attributes = IndexMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }
        Element {
            name: "g".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![],
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = RemoveNonInheritableGroupAttrsPlugin;
        assert_eq!(plugin.name(), "removeNonInheritableGroupAttrs");
        assert_eq!(plugin.description(), "removes non-inheritable group's presentation attributes");
    }

    #[test]
    fn test_remove_non_inheritable_attrs() {
        let mut document = create_test_document();
        
        // Create a group with both inheritable and non-inheritable presentation attributes
        let group = create_group_with_attrs(vec![
            // Non-inheritable presentation attributes (should be removed)
            ("clip-path", "url(#clip)"),
            ("mask", "url(#mask)"),
            ("filter", "url(#blur)"),
            ("opacity", "0.5"),
            ("overflow", "hidden"),
            // Inheritable attributes (should be kept)
            ("fill", "red"),
            ("stroke", "blue"),
            ("font-size", "14px"),
            ("transform", "translate(10,10)"),
            // Non-presentation attributes (should be kept)
            ("id", "myGroup"),
            ("class", "important"),
        ]);
        
        document.root.children = vec![Node::Element(group)];

        let mut plugin = RemoveNonInheritableGroupAttrsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Check that non-inheritable attrs were removed and others kept
        if let Node::Element(elem) = &document.root.children[0] {
            // Non-inheritable should be removed
            assert!(!elem.attributes.contains_key("clip-path"));
            assert!(!elem.attributes.contains_key("mask"));
            assert!(!elem.attributes.contains_key("filter"));
            assert!(!elem.attributes.contains_key("opacity"));
            assert!(!elem.attributes.contains_key("overflow"));
            
            // Inheritable should be kept
            assert_eq!(elem.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(elem.attributes.get("stroke"), Some(&"blue".to_string()));
            assert_eq!(elem.attributes.get("font-size"), Some(&"14px".to_string()));
            assert_eq!(elem.attributes.get("transform"), Some(&"translate(10,10)".to_string()));
            
            // Non-presentation should be kept
            assert_eq!(elem.attributes.get("id"), Some(&"myGroup".to_string()));
            assert_eq!(elem.attributes.get("class"), Some(&"important".to_string()));
        } else {
            panic!("Expected group element");
        }
    }

    #[test]
    fn test_nested_groups() {
        let mut document = create_test_document();
        
        // Create nested groups with non-inheritable attrs
        let inner_group = create_group_with_attrs(vec![
            ("opacity", "0.8"),
            ("fill", "green"),
        ]);
        
        let mut outer_group = create_group_with_attrs(vec![
            ("filter", "url(#blur)"),
            ("stroke", "black"),
        ]);
        outer_group.children = vec![Node::Element(inner_group)];
        
        document.root.children = vec![Node::Element(outer_group)];

        let mut plugin = RemoveNonInheritableGroupAttrsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Check outer group
        if let Node::Element(outer) = &document.root.children[0] {
            assert!(!outer.attributes.contains_key("filter"));
            assert_eq!(outer.attributes.get("stroke"), Some(&"black".to_string()));
            
            // Check inner group
            if let Node::Element(inner) = &outer.children[0] {
                assert!(!inner.attributes.contains_key("opacity"));
                assert_eq!(inner.attributes.get("fill"), Some(&"green".to_string()));
            } else {
                panic!("Expected inner group element");
            }
        } else {
            panic!("Expected outer group element");
        }
    }

    #[test]
    fn test_only_affects_groups() {
        let mut document = create_test_document();
        
        // Create a rect with non-inheritable attrs (should not be affected)
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("opacity".to_string(), "0.5".to_string());
        rect_attrs.insert("filter".to_string(), "url(#blur)".to_string());
        rect_attrs.insert("fill".to_string(), "red".to_string());
        rect_attrs.insert("width".to_string(), "100".to_string());
        rect_attrs.insert("height".to_string(), "100".to_string());
        
        let rect = Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };
        
        document.root.children = vec![Node::Element(rect)];

        let mut plugin = RemoveNonInheritableGroupAttrsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Rect should keep all attributes since plugin only affects groups
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("opacity"), Some(&"0.5".to_string()));
            assert_eq!(elem.attributes.get("filter"), Some(&"url(#blur)".to_string()));
            assert_eq!(elem.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(elem.attributes.len(), 5);
        } else {
            panic!("Expected rect element");
        }
    }

    #[test]
    fn test_empty_group() {
        let mut document = create_test_document();
        
        // Create an empty group with non-inheritable attrs
        let group = create_group_with_attrs(vec![
            ("mask", "url(#mask)"),
            ("id", "emptyGroup"),
        ]);
        
        document.root.children = vec![Node::Element(group)];

        let mut plugin = RemoveNonInheritableGroupAttrsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        if let Node::Element(elem) = &document.root.children[0] {
            assert!(!elem.attributes.contains_key("mask"));
            assert_eq!(elem.attributes.get("id"), Some(&"emptyGroup".to_string()));
        } else {
            panic!("Expected group element");
        }
    }

    #[test]
    fn test_all_non_inheritable_attrs() {
        let mut document = create_test_document();
        
        // Create a group with various non-inheritable presentation attributes
        let group = create_group_with_attrs(vec![
            ("alignment-baseline", "middle"),
            ("baseline-shift", "super"),
            ("clip-path", "url(#clip)"),
            ("clip", "rect(0,0,100,100)"),
            ("enable-background", "new"),
            ("filter", "url(#blur)"),
            ("flood-color", "blue"),
            ("flood-opacity", "0.5"),
            ("lighting-color", "yellow"),
            ("mask", "url(#mask)"),
            ("opacity", "0.8"),
            ("overflow", "hidden"),
            ("stop-color", "green"),
            ("stop-opacity", "0.3"),
            ("text-decoration", "underline"),
            ("unicode-bidi", "embed"),
            // Add one inheritable to verify it's kept
            ("fill", "red"),
        ]);
        
        document.root.children = vec![Node::Element(group)];

        let mut plugin = RemoveNonInheritableGroupAttrsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        if let Node::Element(elem) = &document.root.children[0] {
            // All non-inheritable should be removed
            assert!(!elem.attributes.contains_key("alignment-baseline"));
            assert!(!elem.attributes.contains_key("baseline-shift"));
            assert!(!elem.attributes.contains_key("clip-path"));
            assert!(!elem.attributes.contains_key("clip"));
            assert!(!elem.attributes.contains_key("enable-background"));
            assert!(!elem.attributes.contains_key("filter"));
            assert!(!elem.attributes.contains_key("flood-color"));
            assert!(!elem.attributes.contains_key("flood-opacity"));
            assert!(!elem.attributes.contains_key("lighting-color"));
            assert!(!elem.attributes.contains_key("mask"));
            assert!(!elem.attributes.contains_key("opacity"));
            assert!(!elem.attributes.contains_key("overflow"));
            assert!(!elem.attributes.contains_key("stop-color"));
            assert!(!elem.attributes.contains_key("stop-opacity"));
            assert!(!elem.attributes.contains_key("text-decoration"));
            assert!(!elem.attributes.contains_key("unicode-bidi"));
            
            // Inheritable should be kept
            assert_eq!(elem.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(elem.attributes.len(), 1);
        } else {
            panic!("Expected group element");
        }
    }
}