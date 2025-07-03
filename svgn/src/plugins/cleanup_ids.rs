// this_file: svgn/src/plugins/cleanup_ids.rs

//! Cleanup IDs plugin
//!
//! This plugin removes unused IDs and minifies used IDs to save space.
//! It's careful not to break references and respects various preservation options.
//! Ported from ref/svgo/plugins/cleanupIds.js

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Plugin that removes unused IDs and minifies used IDs
pub struct CleanupIdsPlugin;

// Regex patterns for finding ID references
static REG_REFERENCES_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\burl\(#([^)]+)\)"#).unwrap());
static REG_REFERENCES_URL_QUOTED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\burl\(["']#([^"']+)["']\)"#).unwrap());
static REG_REFERENCES_HREF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#(.+?)$").unwrap());
static REG_REFERENCES_BEGIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w+)\.[a-zA-Z]").unwrap());

// Characters used for generating minified IDs
const GENERATE_ID_CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

// Properties that can contain URL references
const REFERENCES_PROPS: &[&str] = &[
    "clip-path",
    "color-profile",
    "fill",
    "filter",
    "marker-end",
    "marker-mid",
    "marker-start",
    "mask",
    "stroke",
    "style",
];

impl Plugin for CleanupIdsPlugin {
    fn name(&self) -> &'static str {
        "cleanupIds"
    }

    fn description(&self) -> &'static str {
        "Remove unused IDs and minify used IDs"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &crate::plugin::PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let remove = params
            .and_then(|v| v.get("remove"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let minify = params
            .and_then(|v| v.get("minify"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let preserve: HashSet<String> = params
            .and_then(|v| v.get("preserve"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let preserve_prefixes: Vec<String> = params
            .and_then(|v| v.get("preservePrefixes"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let force = params
            .and_then(|v| v.get("force"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if we should deoptimize (skip processing)
        if !force && (has_scripts(&document.root) || has_styles(&document.root)) {
            return Ok(());
        }

        // Collect all IDs and their references
        let mut node_by_id: HashMap<String, *mut Element> = HashMap::new();
        let mut references_by_id: HashMap<String, Vec<(String, String)>> = HashMap::new();

        // First pass: collect IDs and references
        collect_ids_and_refs(&mut document.root, &mut node_by_id, &mut references_by_id);

        // Helper to check if an ID should be preserved
        let is_id_preserved = |id: &str| -> bool {
            preserve.contains(id)
                || preserve_prefixes
                    .iter()
                    .any(|prefix| id.starts_with(prefix))
        };

        // Second pass: process IDs
        let mut current_id = None;
        let mut id_mappings: HashMap<String, String> = HashMap::new();

        // Process referenced IDs first
        for id in references_by_id.keys() {
            if let Some(&node_ptr) = node_by_id.get(id) {
                unsafe {
                    let node = &mut *node_ptr;
                    if minify && !is_id_preserved(id) {
                        // Generate new minified ID
                        let mut new_id;
                        loop {
                            current_id = generate_id(current_id);
                            new_id = get_id_string(current_id.as_ref().unwrap());

                            // Make sure the new ID is unique and not preserved
                            if !is_id_preserved(&new_id)
                                && !node_by_id.contains_key(&new_id)
                                && !id_mappings.values().any(|v| v == &new_id)
                            {
                                break;
                            }
                        }

                        // Update the ID
                        node.set_attr("id".to_string(), new_id.clone());
                        id_mappings.insert(id.clone(), new_id);
                    }
                }

                // Mark this ID as processed (referenced)
                node_by_id.remove(id);
            }
        }

        // Update all references with new IDs
        if !id_mappings.is_empty() {
            update_references(&mut document.root, &id_mappings);
        }

        // Remove unreferenced IDs if requested
        if remove {
            for (id, node_ptr) in node_by_id {
                if !is_id_preserved(&id) {
                    unsafe {
                        let node = &mut *node_ptr;
                        node.remove_attr("id");
                    }
                }
            }
        }

        Ok(())
    }
}

/// Check if the document contains scripts
fn has_scripts(element: &Element) -> bool {
    if element.name == "script" && !element.is_empty() {
        return true;
    }

    // Check for javascript: links
    if element.name == "a" {
        if let Some(href) = element.attr("href") {
            if href.trim_start().starts_with("javascript:") {
                return true;
            }
        }
    }

    // Check for event attributes (onclick, onload, etc.)
    for (attr_name, _) in &element.attributes {
        if attr_name.starts_with("on") {
            return true;
        }
    }

    // Check children
    for child in &element.children {
        if let Node::Element(child_element) = child {
            if has_scripts(child_element) {
                return true;
            }
        }
    }

    false
}

/// Check if the document contains style elements with content
fn has_styles(element: &Element) -> bool {
    if element.name == "style" && !element.is_empty() {
        return true;
    }

    // Check children
    for child in &element.children {
        if let Node::Element(child_element) = child {
            if has_styles(child_element) {
                return true;
            }
        }
    }

    false
}

/// Collect all IDs and references to them
fn collect_ids_and_refs(
    element: &mut Element,
    node_by_id: &mut HashMap<String, *mut Element>,
    references_by_id: &mut HashMap<String, Vec<(String, String)>>,
) {
    // Check for ID attribute
    if let Some(id) = element.attr("id").cloned() {
        let element_ptr: *mut Element = element;
        // Only keep the first occurrence of each ID
        node_by_id.entry(id).or_insert(element_ptr);
    }

    // Check for references in attributes
    for (attr_name, attr_value) in &element.attributes {
        let ids = find_references(attr_name, attr_value);
        for id in ids {
            references_by_id
                .entry(id)
                .or_default()
                .push((attr_name.clone(), attr_value.clone()));
        }
    }

    // Process children
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            collect_ids_and_refs(child_element, node_by_id, references_by_id);
        }
    }
}

/// Find ID references in attribute values
fn find_references(attribute: &str, value: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Check for URL references: url(#id) and url('#id')
    if REFERENCES_PROPS.contains(&attribute) {
        // Try unquoted URL references
        for cap in REG_REFERENCES_URL.captures_iter(value) {
            if let Some(id_match) = cap.get(1) {
                results.push(id_match.as_str().to_string());
            }
        }
        // Try quoted URL references
        for cap in REG_REFERENCES_URL_QUOTED.captures_iter(value) {
            if let Some(id_match) = cap.get(1) {
                results.push(id_match.as_str().to_string());
            }
        }
    }

    // Check for href references: #id
    if attribute == "href" || attribute.ends_with(":href") {
        if let Some(cap) = REG_REFERENCES_HREF.captures(value) {
            if let Some(id_match) = cap.get(1) {
                results.push(id_match.as_str().to_string());
            }
        }
    }

    // Check for begin attribute references: elementId.event
    if attribute == "begin" {
        if let Some(cap) = REG_REFERENCES_BEGIN.captures(value) {
            if let Some(id_match) = cap.get(1) {
                results.push(id_match.as_str().to_string());
            }
        }
    }

    results
}

/// Generate the next ID in sequence
fn generate_id(current_id: Option<Vec<usize>>) -> Option<Vec<usize>> {
    let mut id = current_id.unwrap_or_else(|| vec![usize::MAX]); // Start before 'a'
    let max_index = GENERATE_ID_CHARS.len() - 1;

    // Increment the ID
    let last_idx = id.len() - 1;
    if id[last_idx] == usize::MAX {
        id[last_idx] = 0; // First call: go from MAX to 0 ('a')
    } else {
        id[last_idx] += 1;
    }

    // Handle carry-over
    for i in (1..id.len()).rev() {
        if id[i] > max_index {
            id[i] = 0;
            id[i - 1] += 1;
        }
    }

    // Add new position if needed
    if id[0] > max_index {
        id[0] = 0;
        id.insert(0, 0);
    }

    Some(id)
}

/// Convert ID array to string
fn get_id_string(id: &[usize]) -> String {
    id.iter().map(|&i| GENERATE_ID_CHARS[i]).collect()
}

/// Update all references with new IDs
fn update_references(element: &mut Element, id_mappings: &HashMap<String, String>) {
    // Update attributes
    for (attr_name, attr_value) in element.attributes.iter_mut() {
        let mut new_value = attr_value.clone();

        // Update URL references
        if REFERENCES_PROPS.contains(&attr_name.as_str()) {
            for (old_id, new_id) in id_mappings {
                // Handle both encoded and non-encoded IDs
                new_value = new_value
                    .replace(
                        &format!("#{}\"", urlencoding::encode(old_id)),
                        &format!("#{}\"", new_id),
                    )
                    .replace(
                        &format!("#{}'", urlencoding::encode(old_id)),
                        &format!("#{}'", new_id),
                    )
                    .replace(
                        &format!("#{})", urlencoding::encode(old_id)),
                        &format!("#{})", new_id),
                    )
                    .replace(&format!("#{}\"", old_id), &format!("#{}\"", new_id))
                    .replace(&format!("#{}'", old_id), &format!("#{}'", new_id))
                    .replace(&format!("#{})", old_id), &format!("#{})", new_id));
            }
        }

        // Update href references
        if attr_name == "href" || attr_name.ends_with(":href") {
            for (old_id, new_id) in id_mappings {
                if new_value == format!("#{}", old_id) {
                    new_value = format!("#{}", new_id);
                }
            }
        }

        // Update begin attribute references
        if attr_name == "begin" {
            for (old_id, new_id) in id_mappings {
                new_value = new_value.replace(&format!("{}.", old_id), &format!("{}.", new_id));
            }
        }

        *attr_value = new_value;
    }

    // Process children
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            update_references(child_element, id_mappings);
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use serde_json::json;

    #[test]
    fn test_remove_unused_ids() {
        let svg = r#"<svg>
            <defs>
                <linearGradient id="unused-gradient">
                    <stop offset="0%" stop-color="red"/>
                </linearGradient>
                <linearGradient id="used-gradient">
                    <stop offset="0%" stop-color="blue"/>
                </linearGradient>
            </defs>
            <rect id="unused-rect" width="100" height="100"/>
            <rect fill="url(#used-gradient)" width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = CleanupIdsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Check that unused IDs are removed
        let defs = document.root.child_elements().next().unwrap();
        let unused_gradient = defs.child_elements().next().unwrap();
        assert!(!unused_gradient.has_attr("id"));

        let unused_rect = document.root.child_elements().nth(1).unwrap();
        assert!(!unused_rect.has_attr("id"));

        // Check that used ID is kept (and minified)
        let used_gradient = defs.child_elements().nth(1).unwrap();
        assert!(used_gradient.has_attr("id"));
    }

    #[test]
    fn test_minify_ids() {
        let svg = r#"<svg>
            <defs>
                <linearGradient id="myVeryLongGradientId">
                    <stop offset="0%" stop-color="red"/>
                </linearGradient>
            </defs>
            <rect fill="url(#myVeryLongGradientId)" width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = CleanupIdsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Check that ID is minified
        let defs = document.root.child_elements().next().unwrap();
        let gradient = defs.child_elements().next().unwrap();
        let new_id = gradient.attr("id").unwrap();
        assert!(new_id.len() < "myVeryLongGradientId".len());

        // Check that reference is updated
        let rect = document.root.child_elements().nth(1).unwrap();
        let fill = rect.attr("fill").unwrap();
        assert!(fill.contains(&format!("#{}", new_id)));
    }

    #[test]
    fn test_preserve_ids() {
        let svg = r#"<svg>
            <rect id="preserve-me" width="100" height="100"/>
            <rect id="icon-rect" width="50" height="50"/>
            <rect id="normal-rect" width="25" height="25"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = CleanupIdsPlugin;
        let params = json!({
            "preserve": ["preserve-me"],
            "preservePrefixes": ["icon-"]
        });

        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        // Check that preserved IDs are kept
        let rect1 = document.root.child_elements().next().unwrap();
        assert_eq!(rect1.attr("id"), Some(&"preserve-me".to_string()));

        let rect2 = document.root.child_elements().nth(1).unwrap();
        assert_eq!(rect2.attr("id"), Some(&"icon-rect".to_string()));

        // Check that non-preserved ID is removed (no references to it)
        let rect3 = document.root.child_elements().nth(2).unwrap();
        assert!(!rect3.has_attr("id"));
    }

    #[test]
    fn test_skip_with_scripts() {
        let svg = r#"<svg>
            <script>console.log('test');</script>
            <rect id="unused-rect" width="100" height="100"/>
        </svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = CleanupIdsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Should not remove ID when scripts are present
        let rect = document.root.child_elements().nth(1).unwrap();
        assert!(rect.has_attr("id"));
    }
}
