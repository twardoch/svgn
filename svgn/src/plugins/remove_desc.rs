// this_file: svgn/src/plugins/remove_desc.rs

//! Remove desc plugin
//!
//! This plugin removes <desc> elements from SVG documents.
//! By default, it only removes empty descriptions or those containing standard
//! editor content (e.g., "Created with..."). Can be configured to remove all
//! descriptions.
//! Ported from ref/svgo/plugins/removeDesc.js

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

/// Plugin that removes <desc> elements
pub struct RemoveDescPlugin;

// Regex pattern for standard editor descriptions
static STANDARD_DESCS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(Created with|Created using)").unwrap());

impl Plugin for RemoveDescPlugin {
    fn name(&self) -> &'static str {
        "removeDesc"
    }

    fn description(&self) -> &'static str {
        "Remove <desc> elements"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let remove_any = params
            .and_then(|v| v.get("removeAny"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Process the document
        remove_desc_from_element(&mut document.root, remove_any);

        Ok(())
    }
}

/// Recursively remove <desc> elements from an element and its descendants
fn remove_desc_from_element(element: &mut Element, remove_any: bool) {
    // Filter out desc elements based on criteria
    element.children.retain(|child| {
        match child {
            Node::Element(child_element) if child_element.name == "desc" => {
                // Keep the desc element if we should not remove it
                !should_remove_desc(child_element, remove_any)
            }
            _ => true,
        }
    });

    // Process remaining child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            remove_desc_from_element(child_element, remove_any);
        }
    }
}

/// Check if a desc element should be removed
fn should_remove_desc(desc_element: &Element, remove_any: bool) -> bool {
    if remove_any {
        return true;
    }

    // Remove if empty
    if desc_element.children.is_empty() {
        return true;
    }

    // Check if it contains only standard editor text
    if desc_element.children.len() == 1 {
        if let Some(Node::Text(text)) = desc_element.children.first() {
            if STANDARD_DESCS.is_match(text) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use serde_json::json;

    #[test]
    fn test_remove_empty_desc() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <desc></desc>
            <rect width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = RemoveDescPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Check that empty desc is removed
        assert!(!has_desc_element(&document.root));
    }

    #[test]
    fn test_remove_standard_desc() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <desc>Created with Sketch.</desc>
            <rect width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = RemoveDescPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Check that standard desc is removed
        assert!(!has_desc_element(&document.root));
    }

    #[test]
    fn test_preserve_custom_desc() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <desc>This is a custom description for accessibility</desc>
            <rect width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = RemoveDescPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Check that custom desc is preserved
        assert!(has_desc_element(&document.root));
    }

    #[test]
    fn test_remove_any() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <desc>This is a custom description for accessibility</desc>
            <rect width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = RemoveDescPlugin;
        let params = json!({"removeAny": true});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        // Check that all desc elements are removed
        assert!(!has_desc_element(&document.root));
    }

    fn has_desc_element(element: &Element) -> bool {
        for child in &element.children {
            if let Node::Element(child_element) = child {
                if child_element.name == "desc" {
                    return true;
                }
                if has_desc_element(child_element) {
                    return true;
                }
            }
        }
        false
    }
}
