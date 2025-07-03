// this_file: svgn/src/plugins/cleanup_enable_background.rs

//! Cleanup enable-background plugin
//!
//! This plugin removes or cleans up the enable-background attribute when possible.
//! The enable-background attribute is used with filters, but when it matches the
//! SVG dimensions, it can often be simplified or removed.
//! Ported from ref/svgo/plugins/cleanupEnableBackground.js

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

/// Plugin that cleans up or removes the enable-background attribute
pub struct CleanupEnableBackgroundPlugin;

// Regex to match the enable-background format: "new 0 0 <width> <height>"
static REG_ENABLE_BACKGROUND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^new\s0\s0\s([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)\s([-+]?\d*\.?\d+(?:[eE][-+]?\d+)?)$").unwrap()
});

impl Plugin for CleanupEnableBackgroundPlugin {
    fn name(&self) -> &'static str {
        "cleanupEnableBackground"
    }
    
    fn description(&self) -> &'static str {
        "Remove or cleanup enable-background attribute when possible"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &crate::plugin::PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // First check if there are any filter elements in the document
        let has_filter = has_filter_element(&document.root);
        
        // Process the document
        cleanup_enable_background(&mut document.root, has_filter);
        
        Ok(())
    }
}

/// Check if the document contains any filter elements
fn has_filter_element(element: &Element) -> bool {
    if element.name == "filter" {
        return true;
    }
    
    for child in &element.children {
        if let Node::Element(child_element) = child {
            if has_filter_element(child_element) {
                return true;
            }
        }
    }
    
    false
}

/// Recursively process elements to clean up enable-background attributes
fn cleanup_enable_background(element: &mut Element, has_filter: bool) {
    // If there are no filters in the document, we can remove all enable-background attributes
    if !has_filter {
        element.remove_attr("enable-background");
        // TODO: Also handle style attribute with enable-background property
    } else {
        // Check if this is an element that can have enable-background
        let is_valid_element = matches!(element.name.as_str(), "svg" | "mask" | "pattern");
        let has_dimensions = element.has_attr("width") && element.has_attr("height");
        
        if is_valid_element && has_dimensions {
            if let Some(enable_bg) = element.attr("enable-background") {
                if let (Some(width), Some(height)) = (element.attr("width"), element.attr("height")) {
                    // Clean up the value
                    if let Some(cleaned) = cleanup_value(enable_bg, &element.name, width, height) {
                        element.set_attr("enable-background".to_string(), cleaned);
                    } else {
                        element.remove_attr("enable-background");
                    }
                }
            }
        }
    }
    
    // Process child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            cleanup_enable_background(child_element, has_filter);
        }
    }
}

/// Clean up the enable-background value
/// Returns None if the attribute should be removed, Some(value) if it should be kept
fn cleanup_value(value: &str, node_name: &str, width: &str, height: &str) -> Option<String> {
    if let Some(captures) = REG_ENABLE_BACKGROUND.captures(value) {
        if let (Some(bg_width), Some(bg_height)) = (captures.get(1), captures.get(2)) {
            // If the enable-background dimensions match the element dimensions
            if bg_width.as_str() == width && bg_height.as_str() == height {
                // For svg elements, we can remove it entirely
                if node_name == "svg" {
                    return None;
                } else {
                    // For mask and pattern, simplify to just "new"
                    return Some("new".to_string());
                }
            }
        }
    }
    
    // Keep the original value if it doesn't match our pattern or dimensions
    Some(value.to_string())
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_remove_without_filter() {
        let svg = r#"<svg width="100" height="50" enable-background="new 0 0 100 50">
            <rect x="10" y="10" width="80" height="30"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = CleanupEnableBackgroundPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Should remove enable-background when no filter is present
        assert!(!document.root.has_attr("enable-background"));
    }
    
    #[test]
    fn test_cleanup_with_filter() {
        let svg = r#"<svg width="100" height="50" enable-background="new 0 0 100 50">
            <filter id="blur">
                <feGaussianBlur stdDeviation="2"/>
            </filter>
            <rect x="10" y="10" width="80" height="30" filter="url(#blur)"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = CleanupEnableBackgroundPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Should remove enable-background even with filter when dimensions match
        assert!(!document.root.has_attr("enable-background"));
    }
    
    #[test]
    fn test_keep_non_matching() {
        let svg = r#"<svg width="100" height="50" enable-background="new 0 0 200 100">
            <filter id="blur">
                <feGaussianBlur stdDeviation="2"/>
            </filter>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = CleanupEnableBackgroundPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Should keep enable-background when dimensions don't match
        assert_eq!(
            document.root.attr("enable-background"),
            Some(&"new 0 0 200 100".to_string())
        );
    }
    
    #[test]
    fn test_simplify_mask_pattern() {
        let svg = r#"<svg>
            <filter id="f"/>
            <mask width="100" height="50" enable-background="new 0 0 100 50"/>
            <pattern width="200" height="100" enable-background="new 0 0 200 100"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = CleanupEnableBackgroundPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Check mask - should be simplified to "new"
        let mask = document.root.child_elements().find(|e| e.name == "mask").unwrap();
        assert_eq!(mask.attr("enable-background"), Some(&"new".to_string()));
        
        // Check pattern - should be simplified to "new"
        let pattern = document.root.child_elements().find(|e| e.name == "pattern").unwrap();
        assert_eq!(pattern.attr("enable-background"), Some(&"new".to_string()));
    }
}