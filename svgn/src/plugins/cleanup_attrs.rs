// this_file: svgn/src/plugins/cleanup_attrs.rs

//! Cleanup attributes plugin
//!
//! This plugin cleans up attributes from newlines, trailing and repeating spaces.
//! Ported from ref/svgo/plugins/cleanupAttrs.js

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

/// Plugin that cleans up attribute values from newlines, trailing and repeating spaces
pub struct CleanupAttrsPlugin;

// Compile regex patterns once at startup
static REG_NEWLINES_NEED_SPACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\S)\r?\n(\S)").unwrap()
});
static REG_NEWLINES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\r?\n").unwrap()
});
static REG_SPACES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s{2,}").unwrap()
});

impl Plugin for CleanupAttrsPlugin {
    fn name(&self) -> &'static str {
        "cleanupAttrs"
    }
    
    fn description(&self) -> &'static str {
        "Cleanup attributes from newlines, trailing and repeating spaces"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &crate::plugin::PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        // Parse parameters with defaults
        let newlines = params
            .and_then(|v| v.get("newlines"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
            
        let trim = params
            .and_then(|v| v.get("trim"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
            
        let spaces = params
            .and_then(|v| v.get("spaces"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        // Process the root element and all its descendants
        cleanup_element_attrs(&mut document.root, newlines, trim, spaces);
        
        Ok(())
    }
}

/// Recursively clean up attributes for an element and all its descendants
fn cleanup_element_attrs(element: &mut Element, newlines: bool, trim: bool, spaces: bool) {
    // Clean up attributes of the current element
    for (_name, value) in element.attributes.iter_mut() {
        if newlines {
            // Replace newlines that need a space (between non-whitespace chars)
            *value = REG_NEWLINES_NEED_SPACE.replace_all(value, "$1 $2").to_string();
            // Remove simple newlines
            *value = REG_NEWLINES.replace_all(value, "").to_string();
        }
        
        if trim {
            *value = value.trim().to_string();
        }
        
        if spaces {
            // Replace multiple spaces with a single space
            *value = REG_SPACES.replace_all(value, " ").to_string();
        }
    }
    
    // Recursively process child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            cleanup_element_attrs(child_element, newlines, trim, spaces);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use serde_json::json;
    
    #[test]
    fn test_cleanup_newlines() {
        let svg = r#"<svg viewBox="0
10
20 30">
            <rect x="1
2" y="3
4"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let plugin = CleanupAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check that newlines are replaced with spaces where needed
        assert_eq!(document.root.attr("viewBox"), Some(&"0 10 20 30".to_string()));
        
        let rect = document.root.child_elements().next().unwrap();
        assert_eq!(rect.attr("x"), Some(&"1 2".to_string()));
        assert_eq!(rect.attr("y"), Some(&"3 4".to_string()));
    }
    
    #[test]
    fn test_cleanup_spaces() {
        let svg = r#"<svg class="  foo    bar  ">
            <rect fill="  red  "/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let plugin = CleanupAttrsPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check that multiple spaces are reduced to single spaces and trimmed
        assert_eq!(document.root.attr("class"), Some(&"foo bar".to_string()));
        
        let rect = document.root.child_elements().next().unwrap();
        assert_eq!(rect.attr("fill"), Some(&"red".to_string()));
    }
    
    #[test]
    fn test_with_params() {
        let svg = r#"<svg class="  foo    bar  ">
            <rect x="1
2"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let plugin = CleanupAttrsPlugin;
        let params = json!({
            "newlines": false,
            "spaces": false,
            "trim": true
        });
        
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();
        
        // Only trim should be applied
        assert_eq!(document.root.attr("class"), Some(&"foo    bar".to_string()));
        
        let rect = document.root.child_elements().next().unwrap();
        // Newline should remain but be trimmed
        assert_eq!(rect.attr("x"), Some(&"1\n2".to_string()));
    }
}