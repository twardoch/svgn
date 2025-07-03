// this_file: svgn/src/plugins/add_attributes_to_svg_element.rs

//! Add attributes to SVG element plugin
//!
//! This plugin adds attributes to the outer <svg> element.

use crate::ast::{Document, Element};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;

/// Plugin that adds attributes to the outer <svg> element
pub struct AddAttributesToSVGElementPlugin;

impl Plugin for AddAttributesToSVGElementPlugin {
    fn name(&self) -> &'static str {
        "addAttributesToSVGElement"
    }
    
    fn description(&self) -> &'static str {
        "adds attributes to an outer <svg> element"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = params.ok_or_else(|| {
            PluginError::InvalidConfig(
                "addAttributesToSVGElement plugin requires parameters".to_string()
            )
        })?;
        
        let config = AddAttributesConfig::from_params(params)?;
        
        // Only modify if the root element is an SVG
        if document.root.name == "svg" {
            add_attributes_to_element(&mut document.root, &config);
        }
        
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        if let Some(params) = params {
            AddAttributesConfig::from_params(params)?;
        } else {
            return Err(PluginError::InvalidConfig(
                "addAttributesToSVGElement plugin requires parameters".to_string()
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct AddAttributesConfig {
    attributes: Vec<AttributeSpec>,
}

#[derive(Debug)]
enum AttributeSpec {
    /// Simple attribute name (value will be empty string)
    Name(String),
    /// Attribute with name and value
    NameValue(String, String),
}

impl AddAttributesConfig {
    fn from_params(params: &Value) -> PluginResult<Self> {
        let mut attributes = Vec::new();
        
        // Handle single attribute
        if let Some(attribute) = params.get("attribute") {
            attributes.push(Self::parse_attribute_spec(attribute)?);
        }
        
        // Handle multiple attributes
        if let Some(attrs) = params.get("attributes") {
            if let Some(array) = attrs.as_array() {
                for attr in array {
                    attributes.push(Self::parse_attribute_spec(attr)?);
                }
            } else {
                return Err(PluginError::InvalidConfig(
                    "attributes parameter must be an array".to_string()
                ));
            }
        }
        
        if attributes.is_empty() {
            return Err(PluginError::InvalidConfig(
                "addAttributesToSVGElement plugin requires either 'attribute' or 'attributes' parameter".to_string()
            ));
        }
        
        Ok(Self { attributes })
    }
    
    fn parse_attribute_spec(spec: &Value) -> PluginResult<AttributeSpec> {
        match spec {
            Value::String(name) => Ok(AttributeSpec::Name(name.clone())),
            Value::Object(obj) => {
                if obj.len() != 1 {
                    return Err(PluginError::InvalidConfig(
                        "Attribute object must have exactly one key-value pair".to_string()
                    ));
                }
                
                let (key, value) = obj.iter().next().unwrap();
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Null => "".to_string(),
                    _ => return Err(PluginError::InvalidConfig(
                        "Attribute value must be string, number, boolean, or null".to_string()
                    )),
                };
                
                Ok(AttributeSpec::NameValue(key.clone(), value_str))
            }
            _ => Err(PluginError::InvalidConfig(
                "Attribute specification must be string or object".to_string()
            ))
        }
    }
}

fn add_attributes_to_element(element: &mut Element, config: &AddAttributesConfig) {
    for attr_spec in &config.attributes {
        match attr_spec {
            AttributeSpec::Name(name) => {
                // Only add if it doesn't already exist
                if !element.attributes.contains_key(name) {
                    element.attributes.insert(name.clone(), "".to_string());
                }
            }
            AttributeSpec::NameValue(name, value) => {
                // Only add if it doesn't already exist
                if !element.attributes.contains_key(name) {
                    element.attributes.insert(name.clone(), value.clone());
                }
            }
        }
    }
}

#[cfg(test)]
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

    #[test]
    fn test_add_single_attribute() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "attribute": "focusable"
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        assert_eq!(document.root.attributes.get("focusable"), Some(&"".to_string()));
        // Existing attributes should remain
        assert_eq!(document.root.attributes.get("width"), Some(&"100".to_string()));
    }

    #[test]
    fn test_add_multiple_attributes() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "attributes": ["focusable", "role"]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        assert_eq!(document.root.attributes.get("focusable"), Some(&"".to_string()));
        assert_eq!(document.root.attributes.get("role"), Some(&"".to_string()));
    }

    #[test]
    fn test_add_attributes_with_values() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "attributes": [
                {"focusable": false},
                {"data-image": "icon"},
                {"role": "img"}
            ]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        assert_eq!(document.root.attributes.get("focusable"), Some(&"false".to_string()));
        assert_eq!(document.root.attributes.get("data-image"), Some(&"icon".to_string()));
        assert_eq!(document.root.attributes.get("role"), Some(&"img".to_string()));
    }

    #[test]
    fn test_dont_overwrite_existing_attributes() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        // Add an attribute that already exists
        document.root.attributes.insert("width".to_string(), "200".to_string());
        
        let params = json!({
            "attributes": [{"width": "300"}]
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        // Should not overwrite existing attribute
        assert_eq!(document.root.attributes.get("width"), Some(&"200".to_string()));
    }

    #[test]
    fn test_non_svg_element() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = Document::new();
        document.root = Element::new("rect"); // Not an SVG element
        
        let params = json!({
            "attribute": "focusable"
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        // Should not add attributes to non-svg elements
        assert_eq!(document.root.attributes.get("focusable"), None);
    }

    #[test]
    fn test_missing_params() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let result = plugin.apply(&mut document, &plugin_info, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_params() {
        let mut plugin = AddAttributesToSVGElementPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_svg_document();
        
        let params = json!({
            "invalid": "param"
        });
        
        let result = plugin.apply(&mut document, &plugin_info, Some(&params));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_params() {
        let plugin = AddAttributesToSVGElementPlugin;
        
        // Valid params
        let valid_params = json!({
            "attribute": "focusable"
        });
        assert!(plugin.validate_params(Some(&valid_params)).is_ok());
        
        // Invalid params - missing required fields
        let invalid_params = json!({
            "invalid": "param"
        });
        assert!(plugin.validate_params(Some(&invalid_params)).is_err());
        
        // No params
        assert!(plugin.validate_params(None).is_err());
    }
}