// this_file: svgn/src/plugins/remove_style_element.rs

//! Remove style element plugin
//!
//! This plugin removes all `<style>` elements from the SVG document.
//! It's useful when you want to completely strip CSS styling from SVGs.

use crate::ast::{Document, Node, Element};
use crate::plugin::{Plugin, PluginResult, PluginInfo};
use serde_json::Value;

/// Plugin that removes all `<style>` elements from the SVG
pub struct RemoveStyleElement;

impl Plugin for RemoveStyleElement {
    fn name(&self) -> &'static str {
        "removeStyleElement"
    }
    
    fn description(&self) -> &'static str {
        "Remove all <style> elements from SVG document"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Remove style elements from the main document tree
        remove_style_elements(&mut document.root);
        
        Ok(())
    }
}

/// Recursively remove style elements from an element and its children
fn remove_style_elements(element: &mut Element) {
    // Remove style elements from children
    element.children.retain(|child| {
        if let Node::Element(ref elem) = child {
            elem.name != "style"
        } else {
            true
        }
    });
    
    // Recursively process child elements
    for child in &mut element.children {
        if let Node::Element(ref mut elem) = child {
            remove_style_elements(elem);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_remove_style_elements() {
        let svg = r#"<svg>
            <style>.cls-1{fill:red;}</style>
            <rect class="cls-1" width="10" height="10"/>
            <g>
                <style type="text/css">.cls-2{stroke:blue;}</style>
                <circle class="cls-2" r="5"/>
            </g>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveStyleElement;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that style elements are removed
        assert!(!contains_style_element(&document.root));
    }
    
    #[test]
    fn test_remove_multiple_style_elements() {
        let svg = r#"<svg>
            <style>/* First style */</style>
            <style>/* Second style */</style>
            <rect/>
            <style>/* Third style */</style>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveStyleElement;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that all style elements are removed
        assert!(!contains_style_element(&document.root));
    }
    
    fn contains_style_element(element: &Element) -> bool {
        for child in &element.children {
            match child {
                Node::Element(elem) => {
                    if elem.name == "style" {
                        return true;
                    }
                    if contains_style_element(elem) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}