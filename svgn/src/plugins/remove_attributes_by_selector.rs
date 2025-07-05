// this_file: svgn/src/plugins/remove_attributes_by_selector.rs

//! Plugin to remove attributes of elements that match a CSS selector
//!
//! This plugin removes attributes from elements that match specified CSS selectors.
//! It supports single selectors or multiple selectors with different attribute removals.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;
use selectors::SelectorList;
use selectors::parser::{Selector, SelectorImpl};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching::{matches_selector_list, MatchingContext, MatchingMode, ElementSelectorFlags};
use selectors::NthIndexCache;
use cssparser::ToCss;

/// Plugin to remove attributes by CSS selector
pub struct RemoveAttributesBySelectorPlugin;

/// Configuration for a single selector
#[derive(Debug, Clone)]
pub struct SelectorConfig {
    /// CSS selector string
    pub selector: String,
    /// Attributes to remove (can be a single attribute or list)
    pub attributes: Vec<String>,
}

/// Configuration parameters for the plugin
#[derive(Debug, Clone)]
pub struct RemoveAttributesBySelectorParams {
    /// List of selector configurations
    pub selectors: Vec<SelectorConfig>,
}

impl RemoveAttributesBySelectorParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> PluginResult<Self> {
        let mut selectors = Vec::new();
        
        if let Some(Value::Object(map)) = value {
            // Check if we have a single selector config or multiple
            if let Some(selector_val) = map.get("selector") {
                // Single selector config
                let selector = selector_val.as_str()
                    .ok_or_else(|| PluginError::InvalidConfig("selector must be a string".to_string()))?
                    .to_string();
                
                let attributes = if let Some(attrs_val) = map.get("attributes") {
                    parse_attributes(attrs_val)?
                } else {
                    return Err(PluginError::InvalidConfig("attributes parameter is required".to_string()));
                };
                
                selectors.push(SelectorConfig { selector, attributes });
            } else if let Some(Value::Array(selector_configs)) = map.get("selectors") {
                // Multiple selector configs
                for config in selector_configs {
                    if let Value::Object(config_map) = config {
                        let selector = config_map.get("selector")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| PluginError::InvalidConfig("selector must be a string".to_string()))?
                            .to_string();
                        
                        let attributes = if let Some(attrs_val) = config_map.get("attributes") {
                            parse_attributes(attrs_val)?
                        } else {
                            return Err(PluginError::InvalidConfig("attributes parameter is required".to_string()));
                        };
                        
                        selectors.push(SelectorConfig { selector, attributes });
                    } else {
                        return Err(PluginError::InvalidConfig("selector config must be an object".to_string()));
                    }
                }
            } else {
                return Err(PluginError::InvalidConfig("either 'selector' or 'selectors' parameter is required".to_string()));
            }
        } else {
            return Err(PluginError::InvalidConfig("parameters must be an object".to_string()));
        }
        
        if selectors.is_empty() {
            return Err(PluginError::InvalidConfig("at least one selector is required".to_string()));
        }
        
        Ok(Self { selectors })
    }
}

/// Parse attributes from JSON value (can be string or array of strings)
fn parse_attributes(value: &Value) -> PluginResult<Vec<String>> {
    match value {
        Value::String(attr) => Ok(vec![attr.clone()]),
        Value::Array(attrs) => {
            let mut result = Vec::new();
            for attr in attrs {
                if let Value::String(s) = attr {
                    result.push(s.clone());
                } else {
                    return Err(PluginError::InvalidConfig("attributes must be strings".to_string()));
                }
            }
            Ok(result)
        }
        _ => Err(PluginError::InvalidConfig("attributes must be a string or array of strings".to_string())),
    }
}

// Use the working SVG selector implementation from inline_styles_selector module
use crate::plugins::inline_styles_selector::{SvgSelectorImpl, SvgElementWrapper};

/// Collect matching elements without mutable borrows
fn collect_matching_paths(
    node: &Node,
    selector_list: &SelectorList<SvgSelectorImpl>,
    current_path: Vec<usize>,
    matching_paths: &mut Vec<Vec<usize>>,
) {
    if let Node::Element(element) = node {
        let wrapper = SvgElementWrapper::new(element, None, 0);
        let mut context = selectors::matching::MatchingContext::new(
            selectors::matching::MatchingMode::Normal,
            None,
            None,
            selectors::matching::QuirksMode::NoQuirks,
        );
        
        // Check if any selector in the list matches
        let mut matches = false;
        for selector in selector_list.iter() {
            if selectors::matching::matches_selector(selector, 0, None, &wrapper, &mut context) {
                matches = true;
                break;
            }
        }
        if matches {
            matching_paths.push(current_path.clone());
        }
        
        // Recursively search children
        for (i, child) in element.children.iter().enumerate() {
            let mut child_path = current_path.clone();
            child_path.push(i);
            collect_matching_paths(child, selector_list, child_path, matching_paths);
        }
    }
}

/// Get mutable element by path
fn get_element_by_path_mut<'a>(node: &'a mut Node, path: &[usize]) -> Option<&'a mut Element> {
    if path.is_empty() {
        if let Node::Element(element) = node {
            return Some(element);
        }
        return None;
    }
    
    if let Node::Element(element) = node {
        if let Some(&index) = path.first() {
            if let Some(child) = element.children.get_mut(index) {
                return get_element_by_path_mut(child, &path[1..]);
            }
        }
    }
    None
}

impl Plugin for RemoveAttributesBySelectorPlugin {
    fn name(&self) -> &'static str {
        "removeAttributesBySelector"
    }
    
    fn description(&self) -> &'static str {
        "removes attributes of elements that match a css selector"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = RemoveAttributesBySelectorParams::from_value(params)?;
        
        // Process each selector configuration
        for config in &params.selectors {
            // Parse the CSS selector
            let mut parser_input = cssparser::ParserInput::new(&config.selector);
            let mut parser = cssparser::Parser::new(&mut parser_input);
            let parsing_mode = selectors::parser::ParseRelative::No;
            
            // Use our SVG selector implementation from the inline_styles_selector module
            // Parse selector using SvgSelectorImpl - simplified parsing for selectors v0.25
            let mut input = cssparser::ParserInput::new(&config.selector);
            let mut parser = cssparser::Parser::new(&mut input);
            
            let selector_list = match SelectorList::<SvgSelectorImpl>::parse(&SvgSelectorImpl, &mut parser, selectors::parser::ParseRelative::No) {
                Ok(list) => list,
                Err(_) => {
                    return Err(PluginError::InvalidConfig(format!(
                        "Invalid CSS selector: {}",
                        config.selector
                    )));
                }
            };
            
            // Collect paths to matching elements
            let mut matching_paths = Vec::new();
            let root_node = Node::Element(document.root.clone());
            collect_matching_paths(&root_node, &selector_list, vec![], &mut matching_paths);
            
            // Remove specified attributes from matching elements
            let mut root_node_mut = Node::Element(document.root.clone());
            for path in matching_paths {
                if let Some(element) = get_element_by_path_mut(&mut root_node_mut, &path) {
                    for attr_name in &config.attributes {
                        element.attributes.shift_remove(attr_name);
                    }
                }
            }
            
            // Update the document root
            if let Node::Element(updated_root) = root_node_mut {
                document.root = updated_root;
            }
        }
        
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        // Try to parse parameters to validate them
        RemoveAttributesBySelectorParams::from_value(params)?;
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
    use std::collections::HashMap;

    fn create_test_document() -> Document {
        let mut doc = Document::default();
        
        // Create a simple SVG structure
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        
        // Add rect with fill="#00ff00"
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
        rect.attributes.insert("fill".to_string(), "#00ff00".to_string());
        rect.attributes.insert("stroke".to_string(), "#00ff00".to_string());
        
        svg.children.push(Node::Element(rect));
        doc.root = svg;
        doc
    }

    #[test]
    fn test_single_attribute_removal() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[fill='#00ff00']",
            "attributes": "fill"
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that fill was removed but stroke remains
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), Some(&"#00ff00".to_string()));
        }
    }

    #[test]
    fn test_multiple_attributes_removal() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[fill='#00ff00']",
            "attributes": ["fill", "stroke"]
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that both fill and stroke were removed
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), None);
            // Other attributes should remain
            assert_eq!(rect.attributes.get("width"), Some(&"100".to_string()));
        }
    }

    #[test]
    fn test_multiple_selectors() {
        let mut doc = create_test_document();
        
        // Add an element with id="remove"
        let mut circle = Element {
            name: "circle".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
            circle.attributes.insert("id".to_string(), "remove".to_string());
            circle.attributes.insert("cx".to_string(), "50".to_string());
            circle.attributes.insert("cy".to_string(), "50".to_string());
            circle.attributes.insert("r".to_string(), "25".to_string());
            circle.attributes.insert("stroke".to_string(), "black".to_string());
            
        doc.root.children.push(Node::Element(circle));
        
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selectors": [
                {
                    "selector": "[fill='#00ff00']",
                    "attributes": "fill"
                },
                {
                    "selector": "#remove",
                    "attributes": ["stroke", "id"]
                }
            ]
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check results
        // Check rect
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), Some(&"#00ff00".to_string()));
        }
        
        // Check circle
        if let Some(Node::Element(ref circle)) = doc.root.children.get(1) {
            assert_eq!(circle.attributes.get("id"), None);
            assert_eq!(circle.attributes.get("stroke"), None);
            assert_eq!(circle.attributes.get("cx"), Some(&"50".to_string()));
        }
    }

    #[test]
    fn test_element_name_selector() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "rect",
            "attributes": "fill"
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that fill was removed from rect
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
        }
    }

    #[test]
    fn test_invalid_selector() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[invalid selector",
            "attributes": "fill"
        });
        
        let result = plugin.apply(&mut doc, &plugin_info, Some(&params));
        assert!(result.is_err());
    }
}