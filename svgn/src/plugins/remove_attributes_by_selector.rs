// this_file: svgn/src/plugins/remove_attributes_by_selector.rs

//! Plugin to remove attributes of elements that match a CSS selector
//!
//! This plugin removes attributes from elements that match specified CSS selectors.
//! It supports single selectors or multiple selectors with different attribute removals.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginError, PluginInfo, PluginResult};
use selectors::SelectorList;
use serde_json::Value;

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
                let selector = selector_val
                    .as_str()
                    .ok_or_else(|| {
                        PluginError::InvalidConfig("selector must be a string".to_string())
                    })?
                    .to_string();

                let attributes = if let Some(attrs_val) = map.get("attributes") {
                    parse_attributes(attrs_val)?
                } else {
                    return Err(PluginError::InvalidConfig(
                        "attributes parameter is required".to_string(),
                    ));
                };

                selectors.push(SelectorConfig {
                    selector,
                    attributes,
                });
            } else if let Some(Value::Array(selector_configs)) = map.get("selectors") {
                // Multiple selector configs
                for config in selector_configs {
                    if let Value::Object(config_map) = config {
                        let selector = config_map
                            .get("selector")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                PluginError::InvalidConfig("selector must be a string".to_string())
                            })?
                            .to_string();

                        let attributes = if let Some(attrs_val) = config_map.get("attributes") {
                            parse_attributes(attrs_val)?
                        } else {
                            return Err(PluginError::InvalidConfig(
                                "attributes parameter is required".to_string(),
                            ));
                        };

                        selectors.push(SelectorConfig {
                            selector,
                            attributes,
                        });
                    } else {
                        return Err(PluginError::InvalidConfig(
                            "selector config must be an object".to_string(),
                        ));
                    }
                }
            } else {
                return Err(PluginError::InvalidConfig(
                    "either 'selector' or 'selectors' parameter is required".to_string(),
                ));
            }
        } else {
            return Err(PluginError::InvalidConfig(
                "parameters must be an object".to_string(),
            ));
        }

        if selectors.is_empty() {
            return Err(PluginError::InvalidConfig(
                "at least one selector is required".to_string(),
            ));
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
                    return Err(PluginError::InvalidConfig(
                        "attributes must be strings".to_string(),
                    ));
                }
            }
            Ok(result)
        }
        _ => Err(PluginError::InvalidConfig(
            "attributes must be a string or array of strings".to_string(),
        )),
    }
}

// Use the working SVG selector implementation from inline_styles_selector module
use crate::plugins::inline_styles_selector::{SvgElementWrapper, SvgSelectorImpl};

/// Collect matching elements without mutable borrows
fn collect_matching_paths(
    node: &Node,
    selector_list: &SelectorList<SvgSelectorImpl>,
    current_path: Vec<usize>,
    matching_paths: &mut Vec<Vec<usize>>,
) {
    if let Node::Element(element) = node {
        let _wrapper = SvgElementWrapper::new(element);
        let mut selector_caches = selectors::matching::SelectorCaches::default();
        let _context: selectors::matching::MatchingContext<'_, SvgSelectorImpl> = selectors::matching::MatchingContext::new(
            selectors::matching::MatchingMode::Normal,
            None,
            &mut selector_caches,
            selectors::matching::QuirksMode::NoQuirks,
            selectors::matching::NeedsSelectorFlags::No,
            selectors::matching::MatchingForInvalidation::No,
        );

        // Check if any selector in the list matches
        let matches = false;
        // TODO: Fix SelectorList iteration - using fallback for now
        // for selector in &*selector_list {
        //     if selectors::matching::matches_selector(selector, 0, None, &wrapper, &mut context) {
        //         matches = true;
        //         break;
        //     }
        // }
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

/// Simple fallback function to remove attributes using basic selector matching
fn remove_attributes_simple(element: &mut Element, selector: &str, attributes: &[String]) -> PluginResult<bool> {
    // Simple element matching - check if element matches selector
    let matches = if let Some(class_name) = selector.strip_prefix('.') {
        // Class selector: .className
        element.attributes.get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|c| c == class_name))
    } else if let Some(id) = selector.strip_prefix('#') {
        // ID selector: #elementId
        element.attributes.get("id").is_some_and(|elem_id| elem_id == id)
    } else if selector.starts_with('[') && selector.ends_with(']') {
        // Attribute selector: [attr='value'] or [attr="value"] or [attr=value]
        parse_and_match_attribute_selector(element, selector)?
    } else if selector.contains('[') || selector.contains(']') {
        // Malformed attribute selector - should error
        return Err(PluginError::InvalidConfig(
            format!("Malformed CSS selector: {}", selector)
        ));
    } else {
        // Element selector: elementName
        element.name == selector
    };

    if matches {
        // Remove specified attributes
        for attr in attributes {
            element.attributes.shift_remove(attr);
        }
        return Ok(true);
    }

    // Recursively process children
    let mut found = false;
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            if remove_attributes_simple(child_element, selector, attributes)? {
                found = true;
            }
        }
    }

    Ok(found)
}

/// Parse and match attribute selectors like [attr='value']
fn parse_and_match_attribute_selector(element: &Element, selector: &str) -> PluginResult<bool> {
    // Remove brackets and parse content
    let content = &selector[1..selector.len()-1];
    
    // Handle different attribute selector formats:
    // [attr='value'], [attr="value"], [attr=value]
    if let Some(eq_pos) = content.find('=') {
        let attr_name = content[..eq_pos].trim();
        let attr_value = content[eq_pos+1..].trim();
        
        // Remove quotes if present
        let attr_value = if (attr_value.starts_with('"') && attr_value.ends_with('"')) ||
                            (attr_value.starts_with('\'') && attr_value.ends_with('\'')) {
            &attr_value[1..attr_value.len()-1]
        } else {
            attr_value
        };
        
        // Check if element has the attribute with the specified value
        Ok(element.attributes.get(attr_name)
            .is_some_and(|elem_value| elem_value == attr_value))
    } else {
        // Simple attribute existence check: [attr]
        let attr_name = content.trim();
        Ok(element.attributes.contains_key(attr_name))
    }
}

impl Plugin for RemoveAttributesBySelectorPlugin {
    fn name(&self) -> &'static str {
        "removeAttributesBySelector"
    }

    fn description(&self) -> &'static str {
        "removes attributes of elements that match a css selector"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let params = RemoveAttributesBySelectorParams::from_value(params)?;

        // Process each selector configuration
        for config in &params.selectors {
            // Parse the CSS selector
            let mut parser_input = cssparser::ParserInput::new(&config.selector);
            let _parser = cssparser::Parser::new(&mut parser_input);
            let _parsing_mode = selectors::parser::ParseRelative::No;

            // Use our SVG selector implementation from the inline_styles_selector module
            // Parse selector using SvgSelectorImpl - simplified parsing for selectors v0.25
            let mut input = cssparser::ParserInput::new(&config.selector);
            let _parser = cssparser::Parser::new(&mut input);

            // For now, disable advanced selector parsing and use simple matching
            // TODO: Implement proper Parser trait for SvgSelectorImpl
            
            // Use simple fallback matching instead
            remove_attributes_simple(&mut document.root, &config.selector, &config.attributes)?;
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
        rect.attributes
            .insert("width".to_string(), "100".to_string());
        rect.attributes
            .insert("height".to_string(), "100".to_string());
        rect.attributes
            .insert("fill".to_string(), "#00ff00".to_string());
        rect.attributes
            .insert("stroke".to_string(), "#00ff00".to_string());

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
        circle
            .attributes
            .insert("id".to_string(), "remove".to_string());
        circle.attributes.insert("cx".to_string(), "50".to_string());
        circle.attributes.insert("cy".to_string(), "50".to_string());
        circle.attributes.insert("r".to_string(), "25".to_string());
        circle
            .attributes
            .insert("stroke".to_string(), "black".to_string());

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
