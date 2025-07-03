// this_file: svgn/src/plugins/convert_style_to_attrs.rs

//! Convert style to attributes plugin
//!
//! This plugin converts inline styles to SVG presentation attributes
//! where possible. It parses the style attribute and extracts any
//! properties that are valid presentation attributes.
//! Ported from ref/svgo/plugins/convertStyleToAttrs.js

use crate::ast::{Document, Element, Node};
use crate::collections::PRESENTATION_ATTRS;
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

/// Plugin that converts inline styles to SVG presentation attributes
pub struct ConvertStyleToAttrsPlugin;

// Regex for parsing CSS declarations
// This handles CSS comments, strings, escape sequences, and declarations
static CSS_DECLARATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
        (?:
            /\*[^]*?\*/  # CSS comments
            |
            (?:
                ([-\w]+)  # property name
                \s*:\s*
                (
                    (?:
                        /\*[^]*?\*/  # inline comments
                        |
                        '(?:[^'\\]|\\.)*'  # single-quoted strings
                        |
                        "(?:[^"\\]|\\.)*"  # double-quoted strings
                        |
                        [^;'"/]  # any other character
                    )+
                )
            )
            \s*(?:;|$)  # declaration end
        )
        "#
    )
    .unwrap()
});

impl Plugin for ConvertStyleToAttrsPlugin {
    fn name(&self) -> &'static str {
        "convertStyleToAttrs"
    }
    
    fn description(&self) -> &'static str {
        "Convert inline styles to SVG presentation attributes"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Process the root element and all its descendants
        convert_styles(&mut document.root);
        
        Ok(())
    }
}

/// Recursively convert styles to attributes for an element and its descendants
fn convert_styles(element: &mut Element) {
    // Check if element has a style attribute
    if let Some(style_value) = element.attributes.get("style").cloned() {
        let mut remaining_styles = Vec::new();
        let mut new_attributes = Vec::new();
        
        // Parse CSS declarations
        for cap in CSS_DECLARATION_RE.captures_iter(&style_value) {
            if let (Some(prop_match), Some(value_match)) = (cap.get(1), cap.get(2)) {
                let property = prop_match.as_str().trim();
                let value = value_match.as_str().trim();
                
                // Check if this is a presentation attribute
                if PRESENTATION_ATTRS.contains(property) {
                    // Don't override existing attributes
                    if !element.attributes.contains_key(property) {
                        new_attributes.push((property.to_string(), value.to_string()));
                    } else {
                        // Keep in style if attribute already exists
                        remaining_styles.push(format!("{}: {}", property, value));
                    }
                } else {
                    // Not a presentation attribute, keep in style
                    remaining_styles.push(format!("{}: {}", property, value));
                }
            }
        }
        
        // Add new attributes
        for (name, value) in new_attributes {
            element.attributes.insert(name, value);
        }
        
        // Update or remove style attribute
        if remaining_styles.is_empty() {
            element.attributes.shift_remove("style");
        } else {
            element.attributes.insert("style".to_string(), remaining_styles.join("; "));
        }
    }
    
    // Recursively process child elements
    for child in &mut element.children {
        if let Node::Element(ref mut child_elem) = child {
            convert_styles(child_elem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_convert_style_to_attrs() {
        let svg = r#"<svg>
            <rect style="fill: red; stroke: blue; opacity: 0.5" width="100" height="100"/>
            <circle style="fill: green; custom-prop: value" r="50"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = ConvertStyleToAttrsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check first rect
        if let Node::Element(rect) = &document.root.children[1] {
            assert_eq!(rect.attributes.get("fill"), Some(&"red".to_string()));
            assert_eq!(rect.attributes.get("stroke"), Some(&"blue".to_string()));
            assert_eq!(rect.attributes.get("opacity"), Some(&"0.5".to_string()));
            assert!(!rect.attributes.contains_key("style"));
        }
        
        // Check circle - custom-prop should remain in style
        if let Node::Element(circle) = &document.root.children[3] {
            assert_eq!(circle.attributes.get("fill"), Some(&"green".to_string()));
            assert_eq!(circle.attributes.get("style"), Some(&"custom-prop: value".to_string()));
        }
    }
    
    #[test]
    fn test_preserve_existing_attributes() {
        let svg = r#"<svg>
            <rect style="fill: red; stroke: blue" fill="green" width="100"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = ConvertStyleToAttrsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that existing fill attribute is preserved
        if let Node::Element(rect) = &document.root.children[1] {
            assert_eq!(rect.attributes.get("fill"), Some(&"green".to_string()));
            assert_eq!(rect.attributes.get("stroke"), Some(&"blue".to_string()));
            assert_eq!(rect.attributes.get("style"), Some(&"fill: red".to_string()));
        }
    }
    
    #[test]
    fn test_complex_css_parsing() {
        let svg = r#"<svg>
            <rect style="/* comment */ fill: url(#grad); stroke: /* inline */ blue; font-family: 'Arial', sans-serif"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = ConvertStyleToAttrsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check parsing of complex CSS
        if let Node::Element(rect) = &document.root.children[1] {
            assert_eq!(rect.attributes.get("fill"), Some(&"url(#grad)".to_string()));
            assert_eq!(rect.attributes.get("stroke"), Some(&"blue".to_string()));
            assert_eq!(rect.attributes.get("font-family"), Some(&"'Arial', sans-serif".to_string()));
            assert!(!rect.attributes.contains_key("style"));
        }
    }
}