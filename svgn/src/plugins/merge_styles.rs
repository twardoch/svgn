// this_file: svgn/src/plugins/merge_styles.rs

//! Merge styles plugin
//!
//! This plugin merges multiple `<style>` elements into one.
//! Ported from ref/svgo/plugins/mergeStyles.js

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that merges multiple `<style>` elements into one
pub struct MergeStylesPlugin;

impl Plugin for MergeStylesPlugin {
    fn name(&self) -> &'static str {
        "mergeStyles"
    }
    
    fn description(&self) -> &'static str {
        "Merge multiple <style> elements into one"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Process the root element
        merge_styles(&mut document.root);
        
        Ok(())
    }
}

/// Recursively merge style elements
fn merge_styles(element: &mut Element) {
    let mut style_contents: Vec<(Option<String>, String)> = Vec::new();
    let mut first_style_index: Option<usize> = None;
    
    // First pass: collect all style elements
    for (index, child) in element.children.iter().enumerate() {
        if let Node::Element(ref elem) = child {
            if elem.name == "style" {
                // Save the first style element index
                if first_style_index.is_none() {
                    first_style_index = Some(index);
                }
                
                // Get media attribute if present
                let media = elem.attributes.get("media").cloned();
                
                // Extract text content from style element
                let mut content = String::new();
                for child in &elem.children {
                    if let Node::Text(text) = child {
                        content.push_str(text);
                    } else if let Node::CData(cdata) = child {
                        content.push_str(cdata);
                    }
                }
                
                if !content.trim().is_empty() {
                    style_contents.push((media, content));
                }
            }
        }
    }
    
    // If we have multiple style elements, merge them
    if style_contents.len() > 1 {
        // Create merged content
        let mut merged_content = String::new();
        let mut needs_cdata = false;
        
        for (media, content) in &style_contents {
            // Check if we need CDATA wrapper
            if content.contains('<') || content.contains('&') {
                needs_cdata = true;
            }
            
            if let Some(media_value) = media {
                // Wrap content with @media if media attribute exists
                merged_content.push_str(&format!("@media {} {{\n{}\n}}\n", media_value, content.trim()));
            } else {
                merged_content.push_str(content);
                merged_content.push('\n');
            }
        }
        
        // Create the merged style element
        if let Some(first_index) = first_style_index {
            let mut merged_style = Element::new("style");
            
            // Add the merged content
            if needs_cdata {
                merged_style.children.push(Node::CData(merged_content));
            } else {
                merged_style.children.push(Node::Text(merged_content));
            }
            
            // Replace the first style element with the merged one
            element.children[first_index] = Node::Element(merged_style);
            
            // Mark indices of style elements to remove (all except the first one)
            let mut indices_to_remove = Vec::new();
            for (index, child) in element.children.iter().enumerate() {
                if let Node::Element(ref elem) = child {
                    if elem.name == "style" && index != first_index {
                        indices_to_remove.push(index);
                    }
                }
            }
            
            // Remove style elements in reverse order to maintain correct indices
            for &index in indices_to_remove.iter().rev() {
                element.children.remove(index);
            }
        }
    }
    
    // Remove empty style elements
    element.children.retain(|child| {
        if let Node::Element(ref elem) = child {
            if elem.name == "style" {
                // Check if style has any content
                for child in &elem.children {
                    match child {
                        Node::Text(text) if !text.trim().is_empty() => return true,
                        Node::CData(cdata) if !cdata.trim().is_empty() => return true,
                        _ => {}
                    }
                }
                false
            } else {
                true
            }
        } else {
            true
        }
    });
    
    // Recursively process child elements
    for child in &mut element.children {
        if let Node::Element(ref mut elem) = child {
            merge_styles(elem);
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_merge_styles() {
        let svg = r#"<svg>
            <style>.a{fill:red}</style>
            <style>.b{fill:blue}</style>
            <rect class="a"/>
            <rect class="b"/>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = MergeStylesPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that only one style element remains
        assert_eq!(count_style_elements(&document.root), 1);
    }
    
    #[test]
    fn test_merge_styles_with_media() {
        let svg = r#"<svg>
            <style media="print">.a{fill:red}</style>
            <style>.b{fill:blue}</style>
            <style media="screen">.c{fill:green}</style>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = MergeStylesPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that only one style element remains
        assert_eq!(count_style_elements(&document.root), 1);
    }
    
    #[test]
    fn test_remove_empty_styles() {
        let svg = r#"<svg>
            <style></style>
            <style>  </style>
            <style>.a{fill:red}</style>
            <style>
            
            </style>
        </svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = MergeStylesPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        // Check that only one style element remains (the non-empty one)
        assert_eq!(count_style_elements(&document.root), 1);
    }
    
    fn count_style_elements(element: &Element) -> usize {
        let mut count = 0;
        for child in &element.children {
            if let Node::Element(elem) = child {
                if elem.name == "style" {
                    count += 1;
                }
                count += count_style_elements(elem);
            }
        }
        count
    }
}