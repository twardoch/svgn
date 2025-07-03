// this_file: svgn/src/plugins/remove_editors_ns_data.rs

use crate::ast::{Document, Element, Node};
use crate::collections::EDITOR_NAMESPACES;
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct RemoveEditorsNSDataConfig {
    /// Additional namespaces to remove besides the default editor namespaces
    pub additional_namespaces: Vec<String>,
}

impl Default for RemoveEditorsNSDataConfig {
    fn default() -> Self {
        Self {
            additional_namespaces: Vec::new(),
        }
    }
}

pub struct RemoveEditorsNSDataPlugin;

impl RemoveEditorsNSDataPlugin {
    pub fn new() -> Self {
        Self
    }

    fn parse_config(&self, params: Option<&Value>) -> RemoveEditorsNSDataConfig {
        let mut config = RemoveEditorsNSDataConfig::default();
        
        if let Some(Value::Object(obj)) = params {
            if let Some(Value::Array(additional)) = obj.get("additionalNamespaces") {
                config.additional_namespaces = additional
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        
        config
    }

    fn collect_editor_prefixes(&self, element: &mut Element, config: &RemoveEditorsNSDataConfig) -> Vec<String> {
        let mut prefixes = Vec::new();
        
        if element.name == "svg" {
            // Create a combined set of namespaces to check
            let mut namespaces_to_remove = HashSet::new();
            for ns in EDITOR_NAMESPACES.iter() {
                namespaces_to_remove.insert(*ns);
            }
            for ns in &config.additional_namespaces {
                namespaces_to_remove.insert(ns.as_str());
            }
            
            // Collect namespace prefixes and remove xmlns declarations
            let mut attrs_to_remove = Vec::new();
            
            for (name, value) in &element.attributes {
                if name.starts_with("xmlns:") && namespaces_to_remove.contains(value.as_str()) {
                    let prefix = &name[6..]; // Remove "xmlns:" prefix
                    prefixes.push(prefix.to_string());
                    attrs_to_remove.push(name.clone());
                }
            }
            
            // Remove the xmlns declarations
            for attr in attrs_to_remove {
                element.attributes.shift_remove(&attr);
            }
        }
        
        prefixes
    }

    fn remove_editor_attributes(&self, element: &mut Element, prefixes: &[String]) {
        let mut attrs_to_remove = Vec::new();
        
        for name in element.attributes.keys() {
            if name.contains(':') {
                if let Some(colon_pos) = name.find(':') {
                    let prefix = &name[..colon_pos];
                    if prefixes.contains(&prefix.to_string()) {
                        attrs_to_remove.push(name.clone());
                    }
                }
            }
        }
        
        // Remove the editor attributes
        for attr in attrs_to_remove {
            element.attributes.shift_remove(&attr);
        }
    }

    fn should_remove_element(&self, element: &Element, prefixes: &[String]) -> bool {
        if element.name.contains(':') {
            if let Some(colon_pos) = element.name.find(':') {
                let prefix = &element.name[..colon_pos];
                return prefixes.contains(&prefix.to_string());
            }
        }
        false
    }

    fn process_element(&self, element: &mut Element, prefixes: &mut Vec<String>, config: &RemoveEditorsNSDataConfig) {
        // If this is the SVG root element, collect editor prefixes
        if element.name == "svg" && prefixes.is_empty() {
            *prefixes = self.collect_editor_prefixes(element, config);
        }

        // Remove editor attributes from this element
        self.remove_editor_attributes(element, prefixes);

        // Process children, removing editor elements
        element.children.retain_mut(|child| {
            match child {
                Node::Element(ref mut child_elem) => {
                    // Check if this element should be removed
                    if self.should_remove_element(child_elem, prefixes) {
                        return false; // Remove this element
                    }
                    
                    // Recursively process this child element
                    self.process_element(child_elem, prefixes, config);
                    true // Keep this element
                }
                _ => true, // Keep non-element nodes
            }
        });
    }
}

impl Plugin for RemoveEditorsNSDataPlugin {
    fn name(&self) -> &'static str {
        "removeEditorsNSData"
    }

    fn description(&self) -> &'static str {
        "removes editors namespaces, elements and attributes"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = self.parse_config(params);
        let mut prefixes = Vec::new();
        
        self.process_element(&mut document.root, &mut prefixes, &config);
        
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
        let plugin = RemoveEditorsNSDataPlugin::new();
        assert_eq!(plugin.name(), "removeEditorsNSData");
        assert_eq!(plugin.description(), "removes editors namespaces, elements and attributes");
    }

    #[test]
    fn test_collect_editor_prefixes() {
        let plugin = RemoveEditorsNSDataPlugin::new();
        let config = RemoveEditorsNSDataConfig::default();
        
        let mut svg_element = Element {
            name: "svg".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        
        // Add some editor namespace declarations
        svg_element.attributes.insert("xmlns:sodipodi".to_string(), "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd".to_string());
        svg_element.attributes.insert("xmlns:inkscape".to_string(), "http://www.inkscape.org/namespaces/inkscape".to_string());
        svg_element.attributes.insert("xmlns:normal".to_string(), "http://normal.namespace".to_string());
        
        let prefixes = plugin.collect_editor_prefixes(&mut svg_element, &config);
        
        // Should have collected the editor prefixes and removed xmlns declarations
        assert!(prefixes.contains(&"sodipodi".to_string()));
        assert!(prefixes.contains(&"inkscape".to_string()));
        assert!(!prefixes.contains(&"normal".to_string()));
        
        // Editor xmlns declarations should be removed
        assert!(!svg_element.attributes.contains_key("xmlns:sodipodi"));
        assert!(!svg_element.attributes.contains_key("xmlns:inkscape"));
        
        // Normal xmlns should remain
        assert!(svg_element.attributes.contains_key("xmlns:normal"));
    }

    #[test]
    fn test_remove_editor_attributes() {
        let plugin = RemoveEditorsNSDataPlugin::new();
        let prefixes = vec!["sodipodi".to_string(), "inkscape".to_string()];
        
        let mut element = Element {
            name: "path".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        
        // Add various attributes
        element.attributes.insert("d".to_string(), "M0,0 L10,10".to_string());
        element.attributes.insert("sodipodi:nodetypes".to_string(), "cc".to_string());
        element.attributes.insert("inkscape:connector-curvature".to_string(), "0".to_string());
        element.attributes.insert("fill".to_string(), "red".to_string());
        
        plugin.remove_editor_attributes(&mut element, &prefixes);
        
        // Editor attributes should be removed
        assert!(!element.attributes.contains_key("sodipodi:nodetypes"));
        assert!(!element.attributes.contains_key("inkscape:connector-curvature"));
        
        // Normal attributes should remain
        assert!(element.attributes.contains_key("d"));
        assert!(element.attributes.contains_key("fill"));
    }

    #[test]
    fn test_should_remove_element() {
        let plugin = RemoveEditorsNSDataPlugin::new();
        let prefixes = vec!["sodipodi".to_string()];
        
        let editor_element = Element {
            name: "sodipodi:namedview".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        
        let normal_element = Element {
            name: "path".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        
        assert!(plugin.should_remove_element(&editor_element, &prefixes));
        assert!(!plugin.should_remove_element(&normal_element, &prefixes));
    }

    #[test]
    fn test_apply_removes_editor_content() {
        let mut plugin = RemoveEditorsNSDataPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG root with editor namespace
        doc.root.attributes.insert("xmlns:sodipodi".to_string(), "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd".to_string());
        
        // Add editor element
        let sodipodi_element = Element {
            name: "sodipodi:namedview".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        doc.root.children.push(Node::Element(sodipodi_element));
        
        // Add normal element with editor attribute
        let mut path_attrs = IndexMap::new();
        path_attrs.insert("d".to_string(), "M0,0 L10,10".to_string());
        path_attrs.insert("sodipodi:nodetypes".to_string(), "cc".to_string());
        let path_element = Element {
            name: "path".to_string(),
            attributes: path_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };
        doc.root.children.push(Node::Element(path_element));
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Editor namespace should be removed
        assert!(!doc.root.attributes.contains_key("xmlns:sodipodi"));
        
        // Should have only one child (path element, sodipodi:namedview removed)
        assert_eq!(doc.root.children.len(), 1);
        
        // Path element should have editor attribute removed
        if let Node::Element(path) = &doc.root.children[0] {
            assert_eq!(path.name, "path");
            assert!(path.attributes.contains_key("d"));
            assert!(!path.attributes.contains_key("sodipodi:nodetypes"));
        } else {
            panic!("Expected path element");
        }
    }

    #[test]
    fn test_apply_with_additional_namespaces() {
        let mut plugin = RemoveEditorsNSDataPlugin::new();
        let mut doc = create_test_document();
        
        // Set up SVG root with custom namespace
        doc.root.attributes.insert("xmlns:custom".to_string(), "http://custom.editor/ns".to_string());
        
        // Add custom editor element
        let custom_element = Element {
            name: "custom:element".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };
        doc.root.children.push(Node::Element(custom_element));
        
        let config = serde_json::json!({
            "additionalNamespaces": ["http://custom.editor/ns"]
        });
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, Some(&config));
        assert!(result.is_ok());
        
        // Custom namespace should be removed
        assert!(!doc.root.attributes.contains_key("xmlns:custom"));
        
        // Custom element should be removed
        assert_eq!(doc.root.children.len(), 0);
    }

    #[test]
    fn test_apply_preserves_normal_content() {
        let mut plugin = RemoveEditorsNSDataPlugin::new();
        let mut doc = create_test_document();
        
        // Add normal element
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("x".to_string(), "10".to_string());
        rect_attrs.insert("y".to_string(), "10".to_string());
        rect_attrs.insert("width".to_string(), "100".to_string());
        rect_attrs.insert("height".to_string(), "100".to_string());
        let rect_element = Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };
        doc.root.children.push(Node::Element(rect_element));
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Normal element should be preserved
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.len(), 4);
        } else {
            panic!("Expected rect element");
        }
    }
}