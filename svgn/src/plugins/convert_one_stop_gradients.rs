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
            if let Some(value) = element.attributes.get_mut(color_prop) {
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
        let mut has_xlink = false;
        
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

        if let Some(root) = &document.root {
            has_xlink = check_xlink(root);
        }

        // Remove xmlns:xlink if no xlink:href attributes remain
        if !has_xlink {
            if let Some(root) = &mut document.root {
                root.namespaces.shift_remove("xlink");
                root.attributes.shift_remove("xmlns:xlink");
            }
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
        if let Some(root) = &mut document.root {
            self.process_element(root, &mut gradients_to_remove, false, &mut affected_defs);
        }

        // Second pass: replace gradient references with solid colors
        if !gradients_to_remove.is_empty() {
            if let Some(root) = &mut document.root {
                self.replace_gradient_references(root, &gradients_to_remove);
            }

            // Third pass: remove the gradient elements
            if let Some(root) = &mut document.root {
                self.remove_gradients(root, &gradients_to_remove);
            }

            // Fourth pass: remove empty defs elements
            if let Some(root) = &mut document.root {
                self.remove_empty_defs(root);
            }

            // Remove unused xlink namespace
            self.remove_unused_xlink_namespace(document);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_one_stop_gradient() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1">
                        <stop stop-color="red"/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            // Check that defs is removed (because it becomes empty)
            assert!(!elem.children.iter().any(|child| {
                if let Node::Element(e) = child {
                    e.name == "defs"
                } else {
                    false
                }
            }));
            
            // Check that rect now has solid fill
            let rect = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "rect" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            assert_eq!(rect.attributes.get("fill"), Some(&"red".to_string()));
        }
    }

    #[test]
    fn test_convert_radial_gradient() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <radialGradient id="grad1">
                        <stop stop-color="#ff0000"/>
                    </radialGradient>
                </defs>
                <circle fill="url(#grad1)" cx="50" cy="50" r="40"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            let circle = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "circle" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            assert_eq!(circle.attributes.get("fill"), Some(&"#ff0000".to_string()));
        }
    }

    #[test]
    fn test_preserve_multi_stop_gradient() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1">
                        <stop offset="0%" stop-color="red"/>
                        <stop offset="100%" stop-color="blue"/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        let original = doc.clone();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        // Document should remain unchanged
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            // Gradient should still be there
            assert!(elem.children.iter().any(|child| {
                if let Node::Element(e) = child {
                    e.name == "defs"
                } else {
                    false
                }
            }));
            
            // Rect should still reference gradient
            let rect = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "rect" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            assert_eq!(rect.attributes.get("fill"), Some(&"url(#grad1)".to_string()));
        }
    }

    #[test]
    fn test_style_attribute() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1">
                        <stop stop-color="blue"/>
                    </linearGradient>
                </defs>
                <rect style="fill: url(#grad1); stroke: black" width="100" height="100"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            let rect = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "rect" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            assert_eq!(rect.attributes.get("style"), Some(&"fill: blue; stroke: black".to_string()));
        }
    }

    #[test]
    fn test_stop_color_in_style() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1">
                        <stop style="stop-color: green"/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            let rect = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "rect" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            assert_eq!(rect.attributes.get("fill"), Some(&"green".to_string()));
        }
    }

    #[test]
    fn test_default_stop_color() {
        let mut doc = parse_svg(r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1">
                        <stop/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#);
        let plugin = ConvertOneStopGradientsPlugin::new();
        plugin.apply(&mut doc, &None, &PluginInfo::default()).unwrap();
        
        let root = doc.root.as_ref().unwrap();
        if let Node::Element(elem) = root {
            let rect = elem.children.iter().find_map(|child| {
                if let Node::Element(e) = child {
                    if e.name == "rect" {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).unwrap();
            
            // Default stop-color is black
            assert_eq!(rect.attributes.get("fill"), Some(&"black".to_string()));
        }
    }
}