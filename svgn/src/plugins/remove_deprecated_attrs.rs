// this_file: svgn/src/plugins/remove_deprecated_attrs.rs

//! Plugin to remove deprecated attributes
//!
//! This plugin removes deprecated SVG attributes from elements. It has a safe mode
//! that removes attributes known to be safe to remove, and an unsafe mode that
//! removes additional deprecated attributes that might affect rendering.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;

/// Plugin to remove deprecated attributes
pub struct RemoveDeprecatedAttrsPlugin;

/// Configuration parameters for the plugin
#[derive(Debug, Clone)]
pub struct RemoveDeprecatedAttrsParams {
    /// Whether to remove unsafe deprecated attributes
    pub remove_unsafe: bool,
}

impl Default for RemoveDeprecatedAttrsParams {
    fn default() -> Self {
        Self {
            remove_unsafe: false,
        }
    }
}

impl RemoveDeprecatedAttrsParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> PluginResult<Self> {
        let mut params = Self::default();
        
        if let Some(Value::Object(map)) = value {
            if let Some(remove_unsafe) = map.get("removeUnsafe") {
                params.remove_unsafe = remove_unsafe.as_bool()
                    .ok_or_else(|| PluginError::InvalidConfig("removeUnsafe must be a boolean".to_string()))?;
            }
        }
        
        Ok(params)
    }
}

/// Deprecated attributes grouped by attribute group
static ATTRS_GROUPS_DEPRECATED: Lazy<HashMap<&'static str, DeprecatedAttrs>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    map.insert("animationAttributeTarget", DeprecatedAttrs {
        safe: HashSet::new(),
        unsafe_attrs: vec!["attributeType"].into_iter().map(String::from).collect(),
    });
    
    map.insert("conditionalProcessing", DeprecatedAttrs {
        safe: HashSet::new(),
        unsafe_attrs: vec!["requiredFeatures"].into_iter().map(String::from).collect(),
    });
    
    map.insert("core", DeprecatedAttrs {
        safe: HashSet::new(),
        unsafe_attrs: vec!["xml:base", "xml:lang", "xml:space"].into_iter().map(String::from).collect(),
    });
    
    map.insert("presentation", DeprecatedAttrs {
        safe: HashSet::new(),
        unsafe_attrs: vec![
            "clip",
            "color-profile",
            "enable-background",
            "glyph-orientation-horizontal",
            "glyph-orientation-vertical",
            "kerning",
        ].into_iter().map(String::from).collect(),
    });
    
    map
});

/// Element configurations with their attribute groups
static ELEMENT_CONFIGS: Lazy<HashMap<&'static str, ElementConfig>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    // Common attribute groups
    let common_groups = vec!["conditionalProcessing", "core", "graphicalEvent", "presentation"];
    
    // Define configurations for various elements
    map.insert("a", ElementConfig {
        attrs_groups: common_groups.iter().copied().chain(vec!["xlink"]).collect(),
        deprecated: None,
    });
    
    map.insert("circle", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("ellipse", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("g", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("image", ElementConfig {
        attrs_groups: common_groups.iter().copied().chain(vec!["xlink"]).collect(),
        deprecated: None,
    });
    
    map.insert("line", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("path", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("polygon", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("polyline", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("rect", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("svg", ElementConfig {
        attrs_groups: vec!["conditionalProcessing", "core", "documentEvent", "graphicalEvent", "presentation"].into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("text", ElementConfig {
        attrs_groups: common_groups.clone().into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("use", ElementConfig {
        attrs_groups: common_groups.iter().copied().chain(vec!["xlink"]).collect(),
        deprecated: None,
    });
    
    // Animation elements
    map.insert("animate", ElementConfig {
        attrs_groups: vec!["conditionalProcessing", "core", "animationEvent", "xlink", "animationAttributeTarget", "animationTiming", "animationValue", "animationAddition", "presentation"].into_iter().collect(),
        deprecated: None,
    });
    
    map.insert("animateTransform", ElementConfig {
        attrs_groups: vec!["conditionalProcessing", "core", "animationEvent", "xlink", "animationAttributeTarget", "animationTiming", "animationValue", "animationAddition"].into_iter().collect(),
        deprecated: None,
    });
    
    // Add more elements as needed
    map
});

/// Deprecated attributes structure
#[derive(Debug, Clone)]
struct DeprecatedAttrs {
    safe: HashSet<String>,
    unsafe_attrs: HashSet<String>,
}

/// Element configuration
#[derive(Debug, Clone)]
struct ElementConfig {
    attrs_groups: HashSet<&'static str>,
    deprecated: Option<DeprecatedAttrs>,
}

/// Process deprecated attributes on an element
fn process_attributes(
    element: &mut Element,
    deprecated_attrs: &DeprecatedAttrs,
    params: &RemoveDeprecatedAttrsParams,
) {
    // Remove safe deprecated attributes
    for attr_name in &deprecated_attrs.safe {
        element.attributes.shift_remove(attr_name);
    }
    
    // Remove unsafe deprecated attributes if requested
    if params.remove_unsafe {
        for attr_name in &deprecated_attrs.unsafe_attrs {
            element.attributes.shift_remove(attr_name);
        }
    }
}

/// Process a single element
fn process_element(element: &mut Element, params: &RemoveDeprecatedAttrsParams) {
    // Get element configuration
    if let Some(elem_config) = ELEMENT_CONFIGS.get(element.name.as_str()) {
        // Special case: Remove xml:lang if lang attribute exists
        if elem_config.attrs_groups.contains("core") &&
           element.attributes.contains_key("xml:lang") &&
           element.attributes.contains_key("lang") {
            element.attributes.shift_remove("xml:lang");
        }
        
        // Process deprecated attributes from attribute groups
        for attrs_group in &elem_config.attrs_groups {
            if let Some(deprecated_attrs) = ATTRS_GROUPS_DEPRECATED.get(attrs_group) {
                process_attributes(element, deprecated_attrs, params);
            }
        }
        
        // Process element-specific deprecated attributes
        if let Some(ref deprecated) = elem_config.deprecated {
            process_attributes(element, deprecated, params);
        }
    }
}

/// Recursively process nodes
fn process_node(node: &mut Node, params: &RemoveDeprecatedAttrsParams) {
    if let Node::Element(ref mut element) = node {
        process_element(element, params);
        
        // Process children
        for child in &mut element.children {
            process_node(child, params);
        }
    }
}

impl Plugin for RemoveDeprecatedAttrsPlugin {
    fn name(&self) -> &'static str {
        "removeDeprecatedAttrs"
    }
    
    fn description(&self) -> &'static str {
        "removes deprecated attributes"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = RemoveDeprecatedAttrsParams::from_value(params)?;
        
        // Process root element
        process_element(&mut document.root, &params);
        
        // Process children
        for child in &mut document.root.children {
            process_node(child, &params);
        }
        
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        RemoveDeprecatedAttrsParams::from_value(params)?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};
    use indexmap::IndexMap;
    use serde_json::json;

    fn create_test_document() -> Document {
        let mut doc = Document::default();
        
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        svg.attributes.insert("xml:lang".to_string(), "en".to_string());
        svg.attributes.insert("lang".to_string(), "en".to_string());
        svg.attributes.insert("xml:space".to_string(), "preserve".to_string());
        
        let mut rect = Element {
            name: "rect".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        rect.attributes.insert("x".to_string(), "0".to_string());
        rect.attributes.insert("y".to_string(), "0".to_string());
        rect.attributes.insert("width".to_string(), "100".to_string());
        rect.attributes.insert("height".to_string(), "100".to_string());
        rect.attributes.insert("enable-background".to_string(), "new".to_string());
        rect.attributes.insert("clip".to_string(), "rect(0 0 100 100)".to_string());
        
        svg.children.push(Node::Element(rect));
        doc.root = svg;
        doc
    }

    #[test]
    fn test_remove_xml_lang_when_lang_exists() {
        let mut doc = create_test_document();
        let mut plugin = RemoveDeprecatedAttrsPlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        // xml:lang should be removed because lang exists
        assert_eq!(doc.root.attributes.get("xml:lang"), None);
        assert_eq!(doc.root.attributes.get("lang"), Some(&"en".to_string()));
        // xml:space should still exist (unsafe attribute)
        assert_eq!(doc.root.attributes.get("xml:space"), Some(&"preserve".to_string()));
    }

    #[test]
    fn test_remove_unsafe_attributes() {
        let mut doc = create_test_document();
        let mut plugin = RemoveDeprecatedAttrsPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "removeUnsafe": true
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // xml:space should be removed with removeUnsafe
        assert_eq!(doc.root.attributes.get("xml:space"), None);
        
        // Check rect element - unsafe presentation attributes should be removed
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("enable-background"), None);
            assert_eq!(rect.attributes.get("clip"), None);
            // Regular attributes should remain
            assert_eq!(rect.attributes.get("width"), Some(&"100".to_string()));
        }
    }

    #[test]
    fn test_keep_xml_lang_without_lang() {
        let mut doc = Document::default();
        
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        svg.attributes.insert("xml:lang".to_string(), "en".to_string());
        // No lang attribute
        
        doc.root = svg;
        
        let mut plugin = RemoveDeprecatedAttrsPlugin;
        let plugin_info = PluginInfo::default();
        
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        // xml:lang should be kept because lang doesn't exist
        assert_eq!(doc.root.attributes.get("xml:lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_animation_attribute_target() {
        let mut doc = Document::default();
        
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        
        let mut animate = Element {
            name: "animate".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        animate.attributes.insert("attributeType".to_string(), "XML".to_string());
        animate.attributes.insert("attributeName".to_string(), "x".to_string());
        
        svg.children.push(Node::Element(animate));
        doc.root = svg;
        
        let mut plugin = RemoveDeprecatedAttrsPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "removeUnsafe": true
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // attributeType is an unsafe deprecated attribute
        if let Some(Node::Element(ref animate)) = doc.root.children.first() {
            assert_eq!(animate.attributes.get("attributeType"), None);
            assert_eq!(animate.attributes.get("attributeName"), Some(&"x".to_string()));
        }
    }
}