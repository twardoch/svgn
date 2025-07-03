// this_file: svgn/src/plugins/remove_attrs.rs

//! Plugin to remove specified attributes based on patterns
//!
//! This plugin removes attributes from elements based on flexible pattern matching.
//! Patterns can specify element names, attribute names, and attribute values using regex.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginError, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;

/// Plugin to remove specified attributes
pub struct RemoveAttrsPlugin;

/// Configuration parameters for attribute removal
#[derive(Debug, Clone)]
pub struct RemoveAttrsParams {
    /// Attribute patterns to remove (element:attribute:value format)
    pub attrs: Vec<String>,
    /// Element separator for patterns (default ":")
    pub elem_separator: String,
    /// Whether to preserve currentColor values in fill/stroke attributes
    pub preserve_current_color: bool,
}

impl Default for RemoveAttrsParams {
    fn default() -> Self {
        Self {
            attrs: Vec::new(),
            elem_separator: ":".to_string(),
            preserve_current_color: false,
        }
    }
}

impl RemoveAttrsParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> PluginResult<Self> {
        let mut params = Self::default();

        if let Some(Value::Object(map)) = value {
            // Parse attrs parameter (required)
            if let Some(attrs_value) = map.get("attrs") {
                match attrs_value {
                    Value::String(pattern) => {
                        params.attrs = vec![pattern.clone()];
                    }
                    Value::Array(patterns) => {
                        for pattern in patterns {
                            if let Value::String(s) = pattern {
                                params.attrs.push(s.clone());
                            }
                        }
                    }
                    _ => {
                        return Err(PluginError::InvalidConfig(
                            "attrs parameter must be a string or array of strings".to_string(),
                        ));
                    }
                }
            } else {
                return Err(PluginError::InvalidConfig(
                    "removeAttrs plugin requires 'attrs' parameter".to_string(),
                ));
            }

            // Parse elemSeparator
            if let Some(Value::String(separator)) = map.get("elemSeparator") {
                params.elem_separator = separator.clone();
            }

            // Parse preserveCurrentColor
            if let Some(Value::Bool(preserve)) = map.get("preserveCurrentColor") {
                params.preserve_current_color = *preserve;
            }
        } else if value.is_some() {
            return Err(PluginError::InvalidConfig(
                "removeAttrs plugin parameters must be an object".to_string(),
            ));
        } else {
            return Err(PluginError::InvalidConfig(
                "removeAttrs plugin requires 'attrs' parameter".to_string(),
            ));
        }

        Ok(params)
    }
}

/// Compiled pattern for attribute removal
#[derive(Debug)]
struct CompiledPattern {
    element_regex: Regex,
    attribute_regex: Regex,
    value_regex: Regex,
}

impl CompiledPattern {
    /// Compile a pattern string into regex components
    fn compile(pattern: &str, separator: &str) -> PluginResult<Self> {
        let mut parts: Vec<String> = pattern.split(separator).map(|s| s.to_string()).collect();

        // Expand pattern based on number of parts
        match parts.len() {
            1 => {
                // Just attribute name - apply to all elements with any value
                parts.insert(0, ".*".to_string());
                parts.push(".*".to_string());
            }
            2 => {
                // Element and attribute - apply with any value
                parts.push(".*".to_string());
            }
            3 => {
                // Full pattern - use as is
            }
            _ => {
                return Err(PluginError::InvalidConfig(format!(
                    "Invalid pattern format: {}",
                    pattern
                )));
            }
        }

        // Convert single * to .*
        for part in &mut parts {
            if part == "*" {
                *part = ".*".to_string();
            }
        }

        // Compile regexes (case-insensitive for better compatibility)
        let element_regex = Regex::new(&format!("^{}$", parts[0]))
            .map_err(|e| PluginError::InvalidConfig(format!("Invalid element regex: {}", e)))?;
        let attribute_regex = Regex::new(&format!("^{}$", parts[1]))
            .map_err(|e| PluginError::InvalidConfig(format!("Invalid attribute regex: {}", e)))?;
        let value_regex = Regex::new(&format!("^{}$", parts[2]))
            .map_err(|e| PluginError::InvalidConfig(format!("Invalid value regex: {}", e)))?;

        Ok(Self {
            element_regex,
            attribute_regex,
            value_regex,
        })
    }

    /// Check if this pattern matches the given element, attribute, and value
    fn matches(&self, element_name: &str, attr_name: &str, attr_value: &str) -> bool {
        self.element_regex.is_match(element_name)
            && self.attribute_regex.is_match(attr_name)
            && self.value_regex.is_match(attr_value)
    }
}

impl Plugin for RemoveAttrsPlugin {
    fn name(&self) -> &'static str {
        "removeAttrs"
    }

    fn description(&self) -> &'static str {
        "removes specified attributes"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = RemoveAttrsParams::from_value(params)?;

        if config.attrs.is_empty() {
            return Err(PluginError::InvalidConfig(
                "removeAttrs plugin requires non-empty 'attrs' parameter".to_string(),
            ));
        }

        // Compile all patterns
        let mut compiled_patterns = Vec::new();
        for pattern in &config.attrs {
            compiled_patterns.push(CompiledPattern::compile(pattern, &config.elem_separator)?);
        }

        visit_elements(&mut document.root, &compiled_patterns, &config);
        Ok(())
    }
}

/// Visit all elements in the AST and remove matching attributes
fn visit_elements(element: &mut Element, patterns: &[CompiledPattern], config: &RemoveAttrsParams) {
    remove_matching_attributes(element, patterns, config);

    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            visit_elements(child_element, patterns, config);
        }
    }
}

/// Remove attributes from a single element that match the patterns
fn remove_matching_attributes(
    element: &mut Element,
    patterns: &[CompiledPattern],
    config: &RemoveAttrsParams,
) {
    let mut attrs_to_remove = Vec::new();

    for (attr_name, attr_value) in &element.attributes {
        for pattern in patterns {
            if pattern.matches(&element.name, attr_name, attr_value) {
                // Check for currentColor preservation
                if config.preserve_current_color {
                    let is_current_color = attr_value.to_lowercase() == "currentcolor";
                    let is_fill_or_stroke = attr_name == "fill" || attr_name == "stroke";

                    if is_fill_or_stroke && is_current_color {
                        continue; // Skip removal
                    }
                }

                attrs_to_remove.push(attr_name.clone());
                break; // No need to check other patterns for this attribute
            }
        }
    }

    // Remove the attributes
    for attr_name in attrs_to_remove {
        element.attributes.shift_remove(&attr_name);
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use serde_json::json;

    #[test]
    fn test_simple_attribute_removal() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "blue".to_string());
        element
            .attributes
            .insert("width".to_string(), "100".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": "fill"});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(!document.root.has_attr("fill"));
        assert!(document.root.has_attr("stroke"));
        assert!(document.root.has_attr("width"));
    }

    #[test]
    fn test_multiple_attribute_removal() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "blue".to_string());
        element
            .attributes
            .insert("width".to_string(), "100".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": ["fill", "stroke"]});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(!document.root.has_attr("fill"));
        assert!(!document.root.has_attr("stroke"));
        assert!(document.root.has_attr("width"));
    }

    #[test]
    fn test_regex_pattern_removal() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "blue".to_string());
        element
            .attributes
            .insert("stroke-width".to_string(), "2".to_string());
        element
            .attributes
            .insert("stroke-opacity".to_string(), "0.5".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": "stroke.*"});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(document.root.has_attr("fill"));
        assert!(!document.root.has_attr("stroke"));
        assert!(!document.root.has_attr("stroke-width"));
        assert!(!document.root.has_attr("stroke-opacity"));
    }

    #[test]
    fn test_element_specific_removal() {
        let mut document = Document::new();
        let mut circle = Element::new("circle");
        let mut rect = Element::new("rect");

        circle
            .attributes
            .insert("fill".to_string(), "red".to_string());
        rect.attributes
            .insert("fill".to_string(), "blue".to_string());

        document.root.children.push(Node::Element(circle));
        document.root.children.push(Node::Element(rect));

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": "circle:fill"});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        // Circle should have fill removed, rect should keep it
        if let Node::Element(circle_elem) = &document.root.children[0] {
            assert!(!circle_elem.has_attr("fill"));
        }
        if let Node::Element(rect_elem) = &document.root.children[1] {
            assert!(rect_elem.has_attr("fill"));
        }
    }

    #[test]
    fn test_value_specific_removal() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "blue".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": "*:fill:red"});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(!document.root.has_attr("fill"));
        assert!(document.root.has_attr("stroke"));
    }

    #[test]
    fn test_preserve_current_color() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "currentColor".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "red".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({
            "attrs": "(fill|stroke)",
            "preserveCurrentColor": true
        });
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(document.root.has_attr("fill")); // currentColor preserved
        assert!(!document.root.has_attr("stroke")); // red removed
    }

    #[test]
    fn test_preserve_current_color_case_insensitive() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "currentcolor".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "CURRENTCOLOR".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({
            "attrs": "(fill|stroke)",
            "preserveCurrentColor": true
        });
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(document.root.has_attr("fill")); // currentcolor preserved
        assert!(document.root.has_attr("stroke")); // CURRENTCOLOR preserved
    }

    #[test]
    fn test_custom_separator() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("stroke".to_string(), "blue".to_string());

        document.root = element;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({
            "attrs": "rect|fill",
            "elemSeparator": "|"
        });
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(!document.root.has_attr("fill"));
        assert!(document.root.has_attr("stroke"));
    }

    #[test]
    fn test_no_attrs_parameter_error() {
        let mut document = Document::new();
        let mut plugin = RemoveAttrsPlugin;

        let result = plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None);
        assert!(result.is_err());

        let result = plugin.apply(
            &mut document,
            &crate::plugin::PluginInfo::default(),
            Some(&json!({})),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_elements() {
        let mut document = Document::new();
        let mut parent = Element::new("g");
        let mut child = Element::new("rect");

        parent
            .attributes
            .insert("fill".to_string(), "red".to_string());
        child
            .attributes
            .insert("fill".to_string(), "blue".to_string());
        child
            .attributes
            .insert("stroke".to_string(), "green".to_string());

        parent.children.push(Node::Element(child));
        document.root = parent;

        let mut plugin = RemoveAttrsPlugin;
        let params = json!({"attrs": "fill"});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert!(!document.root.has_attr("fill"));
        if let Node::Element(child_element) = &document.root.children[0] {
            assert!(!child_element.has_attr("fill"));
            assert!(child_element.has_attr("stroke"));
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = RemoveAttrsPlugin;
        assert_eq!(plugin.name(), "removeAttrs");
        assert_eq!(plugin.description(), "removes specified attributes");
    }
}
