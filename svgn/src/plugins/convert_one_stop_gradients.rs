// this_file: svgn/src/plugins/convert_one_stop_gradients.rs

use crate::ast::{Document, Element, Node};
use crate::collections::COLORS_PROPS;
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub struct ConvertOneStopGradientsPlugin;

impl ConvertOneStopGradientsPlugin {
    pub fn new() -> Self {
        Self
    }

    fn process_element(
        &self,
        element: &mut Element,
        gradients_to_remove: &mut HashMap<String, String>,
        parent_is_defs: bool,
        affected_defs: &mut HashSet<String>,
    ) {
        // Track defs elements
        if element.name == "defs" && element.attributes.contains_key("id") {
            if let Some(id) = element.attributes.get("id") {
                affected_defs.insert(id.clone());
            }
        }

        // Process gradient elements
        if element.name == "linearGradient" || element.name == "radialGradient" {
            if let Some(id) = element.attributes.get("id") {
                // Count stop elements
                let stops: Vec<&Element> = element
                    .children
                    .iter()
                    .filter_map(|child| {
                        if let Node::Element(ref elem) = child {
                            if elem.name == "stop" {
                                Some(elem)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                // Check if this gradient references another gradient
                let href = element
                    .attributes
                    .get("xlink:href")
                    .or_else(|| element.attributes.get("href"));

                // If this gradient has no stops and references another, skip for now
                // (would need to resolve references, which is complex)
                if stops.is_empty() && href.is_some() {
                    return;
                }

                // Only process gradients with exactly one stop
                if stops.len() == 1 {
                    let stop = stops[0];
                    
                    // Get the stop color
                    let stop_color = stop
                        .attributes
                        .get("stop-color")
                        .cloned()
                        .or_else(|| {
                            // Check style attribute for stop-color
                            stop.attributes.get("style").and_then(|style| {
                                // Simple regex-like parsing for stop-color in style
                                if let Some(idx) = style.find("stop-color:") {
                                    let start = idx + 11;
                                    let rest = &style[start..].trim_start();
                                    let end = rest.find(';').unwrap_or(rest.len());
                                    Some(rest[..end].trim().to_string())
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or_else(|| "black".to_string()); // Default stop-color is black

                    // Mark this gradient for removal and store its replacement color
                    gradients_to_remove.insert(id.clone(), stop_color);

                    if parent_is_defs {
                        affected_defs.insert("parent_defs".to_string());
                    }
                }
            }
        }

        // Process child elements recursively
        let is_defs = element.name == "defs";
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.process_element(child_elem, gradients_to_remove, is_defs, affected_defs);
            }
        }
    }

    fn replace_gradient_references(
        &self,
        element: &mut Element,
        gradients_to_remove: &HashMap<String, String>,
    ) {
        // Replace gradient references in color properties
        for color_prop in COLORS_PROPS.iter() {
            if let Some(value) = element.attributes.get_mut(*color_prop) {
                if let Some(gradient_id) = self.extract_gradient_id(value) {
                    if let Some(replacement_color) = gradients_to_remove.get(&gradient_id) {
                        *value = replacement_color.clone();
                    }
                }
            }
        }

        // Replace gradient references in style attribute
        if let Some(style) = element.attributes.get_mut("style") {
            let mut new_style = style.clone();
            for (gradient_id, replacement_color) in gradients_to_remove {
                let url_pattern = format!("url(#{})", gradient_id);
                new_style = new_style.replace(&url_pattern, replacement_color);
            }
            if new_style != *style {
                *style = new_style;
            }
        }

        // Process children
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.replace_gradient_references(child_elem, gradients_to_remove);
            }
        }
    }

    fn extract_gradient_id(&self, value: &str) -> Option<String> {
        if value.starts_with("url(#") && value.ends_with(')') {
            let id = &value[5..value.len() - 1];
            Some(id.to_string())
        } else {
            None
        }
    }

    fn remove_gradients(
        &self,
        element: &mut Element,
        gradients_to_remove: &HashMap<String, String>,
    ) {
        // Remove gradient elements
        element.children.retain(|child| {
            if let Node::Element(ref elem) = child {
                if elem.name == "linearGradient" || elem.name == "radialGradient" {
                    if let Some(id) = elem.attributes.get("id") {
                        return !gradients_to_remove.contains_key(id.as_str());
                    }
                }
            }
            true
        });

        // Process children
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.remove_gradients(child_elem, gradients_to_remove);
            }
        }
    }

    fn remove_empty_defs(&self, element: &mut Element) {
        // Remove empty defs elements
        element.children.retain(|child| {
            if let Node::Element(ref elem) = child {
                if elem.name == "defs" && elem.children.is_empty() {
                    return false;
                }
            }
            true
        });

        // Process children
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.remove_empty_defs(child_elem);
            }
        }
    }

    fn remove_unused_xlink_namespace(&self, document: &mut Document) {
        // Check if any xlink:href attributes remain
        fn check_xlink(element: &Element) -> bool {
            if element.attributes.contains_key("xlink:href") {
                return true;
            }
            
            for child in &element.children {
                if let Node::Element(ref elem) = child {
                    if check_xlink(elem) {
                        return true;
                    }
                }
            }
            false
        }

        let has_xlink = check_xlink(&document.root);

        // Remove xmlns:xlink if no xlink:href attributes remain
        if !has_xlink {
            document.root.namespaces.remove("xlink");
            document.root.attributes.shift_remove("xmlns:xlink");
        }
    }
}

impl Plugin for ConvertOneStopGradientsPlugin {
    fn name(&self) -> &'static str {
        "convertOneStopGradients"
    }

    fn description(&self) -> &'static str {
        "converts one-stop (single color) gradients to a plain color"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        let mut gradients_to_remove = HashMap::new();
        let mut affected_defs = HashSet::new();

        // First pass: identify gradients with only one stop
        self.process_element(&mut document.root, &mut gradients_to_remove, false, &mut affected_defs);

        // Second pass: replace gradient references with solid colors
        if !gradients_to_remove.is_empty() {
            self.replace_gradient_references(&mut document.root, &gradients_to_remove);

            // Third pass: remove the gradient elements
            self.remove_gradients(&mut document.root, &gradients_to_remove);

            // Fourth pass: remove empty defs elements
            self.remove_empty_defs(&mut document.root);

            // Remove unused xlink namespace
            self.remove_unused_xlink_namespace(document);
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::PluginInfo;
    use indexmap::IndexMap;

    fn create_test_document() -> Document {
        use std::collections::HashMap;
        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            prologue: vec![],
            epilogue: vec![],
            metadata: crate::ast::DocumentMetadata {
                path: None,
                encoding: None,
                version: None,
            },
        }
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = ConvertOneStopGradientsPlugin::new();
        assert_eq!(plugin.name(), "convertOneStopGradients");
        assert_eq!(plugin.description(), "converts one-stop (single color) gradients to a plain color");
    }

    #[test]
    fn test_extract_gradient_id() {
        let plugin = ConvertOneStopGradientsPlugin::new();
        
        // Test valid gradient ID extraction
        assert_eq!(plugin.extract_gradient_id("url(#myGradient)"), Some("myGradient".to_string()));
        assert_eq!(plugin.extract_gradient_id("url(#grad1)"), Some("grad1".to_string()));
        
        // Test invalid formats
        assert_eq!(plugin.extract_gradient_id("red"), None);
        assert_eq!(plugin.extract_gradient_id("url(myGradient)"), None);
        assert_eq!(plugin.extract_gradient_id("#myGradient"), None);
    }

    #[test]
    fn test_apply_with_empty_document() {
        let mut plugin = ConvertOneStopGradientsPlugin::new();
        let mut doc = create_test_document();
        let info = PluginInfo::default();
        
        // Should not panic with empty document
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_with_no_gradients() {
        let mut plugin = ConvertOneStopGradientsPlugin::new();
        let mut doc = create_test_document();
        
        // Add a simple rect element
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("fill".to_string(), "red".to_string());
        rect_attrs.insert("width".to_string(), "100".to_string());
        rect_attrs.insert("height".to_string(), "100".to_string());
        
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: std::collections::HashMap::new(),
            children: vec![],
        }));
        
        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());
        
        // Document should remain unchanged
        assert_eq!(doc.root.children.len(), 1);
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.name, "rect");
            assert_eq!(rect.attributes.get("fill"), Some(&"red".to_string()));
        }
    }
}