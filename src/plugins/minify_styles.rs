// this_file: svgn/src/plugins/minify_styles.rs

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use lightningcss::{stylesheet::{StyleSheet, MinifyOptions}, printer::PrinterOptions};
use crate::ast::*;
use crate::plugin::{Plugin, PluginInfo};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageConfig {
    #[serde(default)]
    pub force: bool,
    #[serde(default = "default_true")]
    pub ids: bool,
    #[serde(default = "default_true")]
    pub classes: bool,
    #[serde(default = "default_true")]
    pub tags: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UsageValue {
    Bool(bool),
    Config(UsageConfig),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinifyStylesConfig {
    #[serde(default = "default_true")]
    pub restructure: bool,
    #[serde(default)]
    pub force_media_merge: bool,
    #[serde(default)]
    pub comments: Option<String>, // "exclamation", "first-exclamation", or remove all
    #[serde(default)]
    pub usage: Option<UsageValue>,
}

fn default_true() -> bool {
    true
}

pub struct MinifyStyles {
    config: MinifyStylesConfig,
}

impl MinifyStyles {
    pub fn new(config: MinifyStylesConfig) -> Self {
        Self { config }
    }

    fn has_scripts(&self, element: &Element) -> bool {
        // Check if element contains scripts or has event handlers
        element.name == "script" || 
        element.attributes.keys().any(|k| k.starts_with("on"))
    }

    fn minify_css(&self, css_text: &str, _usage_data: Option<&UsageData>) -> Result<String, String> {
        // Use lightningcss to minify CSS
        match StyleSheet::parse(css_text, Default::default()) {
            Ok(mut stylesheet) => {
                let minify_options = MinifyOptions::default();
                if let Err(e) = stylesheet.minify(minify_options) {
                    return Err(format!("CSS minification error: {:?}", e));
                }

                let printer_options = PrinterOptions {
                    minify: true,
                    ..Default::default()
                };

                match stylesheet.to_css(printer_options) {
                    Ok(result) => Ok(result.code),
                    Err(e) => Err(format!("CSS printing error: {:?}", e)),
                }
            }
            Err(e) => Err(format!("CSS parsing error: {:?}", e)),
        }
    }

    fn minify_style_attribute(&self, style_text: &str) -> Result<String, String> {
        // For style attributes, wrap in a dummy rule and extract the declarations
        let dummy_css = format!("dummy {{ {} }}", style_text);
        match self.minify_css(&dummy_css, None) {
            Ok(minified) => {
                // Extract the content between the braces
                if let Some(start) = minified.find('{') {
                    if let Some(end) = minified.rfind('}') {
                        let content = &minified[start + 1..end].trim();
                        return Ok(content.to_string());
                    }
                }
                Err("Failed to extract style declarations".to_string())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
struct UsageData {
    tags: HashSet<String>,
    ids: HashSet<String>,
    classes: HashSet<String>,
    deoptimized: bool,
    force_usage_deoptimized: bool,
}

impl Plugin for MinifyStyles {
    fn name(&self) -> &str {
        "minifyStyles"
    }

    fn description(&self) -> &'static str {
        "minifies styles and removes unused styles"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&serde_json::Value>) -> PluginResult<()> {
        let mut style_elements = Vec::new();
        let mut elements_with_style_attrs = Vec::new();
        let mut usage_data = UsageData {
            tags: HashSet::new(),
            ids: HashSet::new(),
            classes: HashSet::new(),
            deoptimized: false,
            force_usage_deoptimized: false,
        };

        // Configure usage tracking based on config
        let enable_tags_usage;
        let enable_ids_usage;
        let enable_classes_usage;

        match &self.config.usage {
            Some(UsageValue::Bool(b)) => {
                enable_tags_usage = *b;
                enable_ids_usage = *b;
                enable_classes_usage = *b;
            }
            Some(UsageValue::Config(config)) => {
                enable_tags_usage = config.tags;
                enable_ids_usage = config.ids;
                enable_classes_usage = config.classes;
                usage_data.force_usage_deoptimized = config.force;
            }
            None => {
                enable_tags_usage = true;
                enable_ids_usage = true;
                enable_classes_usage = true;
            }
        }

        // First pass: collect style elements and usage data
        for node_idx in document.tree.traverse() {
            if let Some(node) = document.tree.get(node_idx) {
                if let Node::Element(element) = node {
                    // Check for deoptimizations (scripts)
                    if self.has_scripts(element) {
                        usage_data.deoptimized = true;
                    }

                    // Collect usage data
                    if enable_tags_usage {
                        usage_data.tags.insert(element.name.clone());
                    }

                    if enable_ids_usage {
                        if let Some(id) = element.attributes.get("id") {
                            usage_data.ids.insert(id.clone());
                        }
                    }

                    if enable_classes_usage {
                        if let Some(class) = element.attributes.get("class") {
                            for class_name in class.split_whitespace() {
                                usage_data.classes.insert(class_name.to_string());
                            }
                        }
                    }

                    // Collect style elements and elements with style attributes
                    if element.name == "style" {
                        if !element.children.is_empty() {
                            style_elements.push(node_idx);
                        }
                    } else if element.attributes.contains_key("style") {
                        elements_with_style_attrs.push(node_idx);
                    }
                }
            }
        }

        // Process style elements
        let mut elements_to_remove = Vec::new();
        for &style_node_idx in &style_elements {
            if let Some(Node::Element(style_element)) = document.tree.get_mut(style_node_idx) {
                if let Some(&first_child_idx) = style_element.children.first() {
                    if let Some(child_node) = document.tree.get_mut(first_child_idx) {
                        match child_node {
                            Node::Text(text_node) | Node::CData(cdata_node) => {
                                let css_text = match child_node {
                                    Node::Text(text_node) => &text_node.value,
                                    Node::CData(cdata_node) => &cdata_node.value,
                                    _ => unreachable!(),
                                };

                                // Only use usage data if not deoptimized or force is enabled
                                let usage_ref = if !usage_data.deoptimized || usage_data.force_usage_deoptimized {
                                    Some(&usage_data)
                                } else {
                                    None
                                };

                                match self.minify_css(css_text, usage_ref) {
                                    Ok(minified) => {
                                        if minified.is_empty() {
                                            // Mark for removal if CSS is empty after minification
                                            elements_to_remove.push(style_node_idx);
                                        } else {
                                            // Update the text/cdata content
                                            let needs_cdata = css_text.contains('>') || css_text.contains('<');
                                            
                                            match child_node {
                                                Node::Text(text_node) => {
                                                    if needs_cdata {
                                                        // Convert to CDATA if necessary
                                                        *child_node = Node::CData(CDataNode {
                                                            value: minified,
                                                        });
                                                    } else {
                                                        text_node.value = minified;
                                                    }
                                                }
                                                Node::CData(cdata_node) => {
                                                    if needs_cdata {
                                                        cdata_node.value = minified;
                                                    } else {
                                                        // Convert to text if CDATA no longer needed
                                                        *child_node = Node::Text(TextNode {
                                                            value: minified,
                                                        });
                                                    }
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // If minification fails, leave the CSS unchanged
                                        continue;
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }

        // Remove empty style elements
        for &node_idx in &elements_to_remove {
            document.tree.remove(node_idx);
        }

        // Process style attributes
        for &element_idx in &elements_with_style_attrs {
            if let Some(Node::Element(element)) = document.tree.get_mut(element_idx) {
                if let Some(style_value) = element.attributes.get("style") {
                    match self.minify_style_attribute(style_value) {
                        Ok(minified) => {
                            if minified.is_empty() {
                                element.attributes.remove("style");
                            } else {
                                element.attributes.insert("style".to_string(), minified);
                            }
                        }
                        Err(_) => {
                            // If minification fails, leave the style unchanged
                            continue;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}