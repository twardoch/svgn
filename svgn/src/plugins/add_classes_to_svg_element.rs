// this_file: svgn/src/plugins/add_classes_to_svg_element.rs

//! Add class names to SVG element plugin
//!
//! This plugin adds class names to the outer <svg> element.

use crate::ast::{Document, Element};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;
use std::collections::HashSet;

/// Plugin that adds class names to the outer <svg> element
pub struct AddClassesToSVGElementPlugin;

impl Plugin for AddClassesToSVGElementPlugin {
    fn name(&self) -> &'static str {
        "addClassesToSVGElement"
    }
    
    fn description(&self) -> &'static str {
        "adds classnames to an outer <svg> element"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = params.ok_or_else(|| {
            PluginError::InvalidConfig(
                "addClassesToSVGElement plugin requires parameters".to_string()
            )
        })?;
        
        let config = AddClassesConfig::from_params(params)?;
        
        // Only modify if the root element is an SVG
        if document.root.name == "svg" {
            add_classes_to_element(&mut document.root, &config);
        }
        
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        if let Some(params) = params {
            AddClassesConfig::from_params(params)?;
        } else {
            return Err(PluginError::InvalidConfig(
                "addClassesToSVGElement plugin requires parameters".to_string()
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct AddClassesConfig {
    class_names: Vec<String>,
}

impl AddClassesConfig {
    fn from_params(params: &Value) -> PluginResult<Self> {
        let mut class_names = Vec::new();
        
        // Handle single className
        if let Some(class_name) = params.get("className") {
            if let Some(name) = class_name.as_str() {
                class_names.push(name.to_string());
            } else {
                return Err(PluginError::InvalidConfig(
                    "className parameter must be a string".to_string()
                ));
            }
        }
        
        // Handle multiple classNames
        if let Some(classes) = params.get("classNames") {
            if let Some(array) = classes.as_array() {
                for class in array {
                    if let Some(name) = class.as_str() {
                        class_names.push(name.to_string());
                    } else {
                        return Err(PluginError::InvalidConfig(
                            "All classNames must be strings".to_string()
                        ));
                    }
                }
            } else {
                return Err(PluginError::InvalidConfig(
                    "classNames parameter must be an array".to_string()
                ));
            }
        }
        
        if class_names.is_empty() {
            return Err(PluginError::InvalidConfig(
                "addClassesToSVGElement plugin requires either 'className' or 'classNames' parameter".to_string()
            ));
        }
        
        Ok(Self { class_names })
    }
}

fn add_classes_to_element(element: &mut Element, config: &AddClassesConfig) {
    // Get existing classes or create new set
    let mut class_set = HashSet::new();
    
    // Parse existing class attribute if it exists
    if let Some(existing_class) = element.attributes.get("class") {
        for class in existing_class.split_whitespace() {
            if !class.is_empty() {
                class_set.insert(class.to_string());
            }
        }
    }
    
    // Add new classes
    for class_name in &config.class_names {
        if !class_name.is_empty() {
            class_set.insert(class_name.clone());
        }
    }
    
    // Convert back to space-separated string
    let mut classes: Vec<_> = class_set.into_iter().collect();
    classes.sort(); // Sort for consistent output
    let class_string = classes.join(" ");
    
    // Update the class attribute
    element.attributes.insert("class".to_string(), class_string);
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use serde_json::json;
    use indexmap::IndexMap;

    fn create_test_svg_document() -> Document {
        let mut doc = Document::new();
        let mut element = Element::new("svg");
        
        let mut attrs = IndexMap::new();
        attrs.insert("width".to_string(), "100".to_string());
        attrs.insert("height".to_string(), "100".to_string());
        element.attributes = attrs;
        
        doc.root = element;
        doc
    }

    fn create_test_svg_document_with_classes() -> Document {
        let mut doc = Document::new();
        let mut element = Element::new("svg");
        
        let mut attrs = IndexMap::new();
        attrs.insert("width".to_string(), "100".to_string());
        attrs.insert("class".to_string(), "existing-class".to_string());
        element.attributes = attrs;
        
        doc.root = element;
        doc
    }

    #[test]
    fn test_add_single_class() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "className": "my-svg"
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        assert_eq!(document.root.attributes.get("class"), Some(&"my-svg".to_string()));
        // Existing attributes should remain
        assert_eq!(document.root.attributes.get("width"), Some(&"100".to_string()));
    }

    #[test]
    fn test_add_multiple_classes() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "classNames": ["my-svg", "size-large", "theme-dark"]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        let class_attr = document.root.attributes.get("class").unwrap();
        let classes: HashSet<&str> = class_attr.split_whitespace().collect();
        
        assert!(classes.contains("my-svg"));
        assert!(classes.contains("size-large"));
        assert!(classes.contains("theme-dark"));
        assert_eq!(classes.len(), 3);
    }

    #[test]
    fn test_add_classes_to_existing() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document_with_classes();
        
        let params = json!({
            "classNames": ["new-class", "another-class"]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        let class_attr = document.root.attributes.get("class").unwrap();
        let classes: HashSet<&str> = class_attr.split_whitespace().collect();
        
        assert!(classes.contains("existing-class"));
        assert!(classes.contains("new-class"));
        assert!(classes.contains("another-class"));
        assert_eq!(classes.len(), 3);
    }

    #[test]
    fn test_duplicate_classes() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document_with_classes();
        
        let params = json!({
            "classNames": ["existing-class", "new-class"]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        let class_attr = document.root.attributes.get("class").unwrap();
        let classes: HashSet<&str> = class_attr.split_whitespace().collect();
        
        // Should not duplicate existing class
        assert!(classes.contains("existing-class"));
        assert!(classes.contains("new-class"));
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn test_empty_class_names() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "classNames": ["valid-class", "", "  ", "another-valid"]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        let class_attr = document.root.attributes.get("class").unwrap();
        let classes: HashSet<&str> = class_attr.split_whitespace().collect();
        
        assert!(classes.contains("valid-class"));
        assert!(classes.contains("another-valid"));
        assert_eq!(classes.len(), 2); // Empty strings should be filtered out
    }

    #[test]
    fn test_non_svg_element() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = Document::new();
        document.root = Element::new("rect"); // Not an SVG element
        
        let params = json!({
            "className": "my-class"
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        // Should not add classes to non-svg elements
        assert_eq!(document.root.attributes.get("class"), None);
    }

    #[test]
    fn test_missing_params() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let result = plugin.apply(&mut document, &plugin_info, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_params() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "invalid": "param"
        });
        
        let result = plugin.apply(&mut document, &plugin_info, Some(&params));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_class_name_type() {
        let mut plugin = AddClassesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "className": 123
        });
        
        let result = plugin.apply(&mut document, &plugin_info, Some(&params));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_params() {
        let plugin = AddClassesToSVGElementPlugin;
        
        // Valid params
        let valid_params = json!({
            "className": "my-class"
        });
        assert!(plugin.validate_params(Some(&valid_params)).is_ok());
        
        let valid_params2 = json!({
            "classNames": ["class1", "class2"]
        });
        assert!(plugin.validate_params(Some(&valid_params2)).is_ok());
        
        // Invalid params - missing required fields
        let invalid_params = json!({
            "invalid": "param"
        });
        assert!(plugin.validate_params(Some(&invalid_params)).is_err());
        
        // No params
        assert!(plugin.validate_params(None).is_err());
    }
}