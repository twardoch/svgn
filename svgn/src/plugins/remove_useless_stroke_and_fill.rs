// this_file: svgn/src/plugins/remove_useless_stroke_and_fill.rs

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;

/// SVG shape elements that can have stroke and fill attributes
static SHAPE_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "rect", "circle", "ellipse", "line", "polyline", "polygon", 
        "path", "text", "tspan", "textPath", "altGlyph", "glyph", "missing-glyph"
    ])
});

/// Remove useless stroke and fill attributes
///
/// This plugin removes stroke and fill attributes that are either:
/// - Set to "none" when no parent element has these attributes
/// - Set to transparent (opacity 0)
/// - Stroke width set to 0
/// 
/// It also handles inheritance and can optionally remove elements that have
/// no visible stroke or fill (removeNone parameter).
pub struct RemoveUselessStrokeAndFillPlugin;

#[derive(Debug)]
struct RemoveUselessStrokeAndFillParams {
    stroke: bool,
    fill: bool,
    remove_none: bool,
}

impl Default for RemoveUselessStrokeAndFillParams {
    fn default() -> Self {
        Self {
            stroke: true,
            fill: true,
            remove_none: false,
        }
    }
}

impl RemoveUselessStrokeAndFillPlugin {
    pub fn new() -> Self {
        Self
    }

    fn parse_params(&self, params: Option<&Value>) -> RemoveUselessStrokeAndFillParams {
        let mut result = RemoveUselessStrokeAndFillParams::default();
        
        if let Some(Value::Object(obj)) = params {
            if let Some(Value::Bool(stroke)) = obj.get("stroke") {
                result.stroke = *stroke;
            }
            if let Some(Value::Bool(fill)) = obj.get("fill") {
                result.fill = *fill;
            }
            if let Some(Value::Bool(remove_none)) = obj.get("removeNone") {
                result.remove_none = *remove_none;
            }
        }
        
        result
    }

    fn has_style_or_script(&self, element: &Element) -> bool {
        if element.name == "style" || element.name == "script" {
            return true;
        }
        
        for child in &element.children {
            if let Node::Element(child_elem) = child {
                if self.has_style_or_script(child_elem) {
                    return true;
                }
            }
        }
        
        false
    }

    fn process_element(
        &self,
        element: &mut Element,
        params: &RemoveUselessStrokeAndFillParams,
        parent_styles: &HashMap<String, String>,
        nodes_to_remove: &mut Vec<*mut Element>,
    ) -> HashMap<String, String> {
        // Skip elements with ID (they might be referenced by CSS)
        if element.attributes.contains_key("id") {
            return HashMap::new();
        }
        
        // Only process shape elements
        if !SHAPE_ELEMENTS.contains(&element.name.as_str()) {
            return HashMap::new();
        }
        
        // Compute current element styles
        let current_styles = self.compute_element_styles(element, parent_styles);
        
        // Process stroke attributes
        if params.stroke {
            self.process_stroke_attributes(element, &current_styles, parent_styles);
        }
        
        // Process fill attributes
        if params.fill {
            self.process_fill_attributes(element, &current_styles);
        }
        
        // Remove element if it has no visible stroke or fill
        if params.remove_none {
            if self.should_remove_element(element, &current_styles) {
                nodes_to_remove.push(element as *mut Element);
            }
        }
        
        current_styles
    }

    fn compute_element_styles(
        &self,
        element: &Element,
        parent_styles: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut styles = parent_styles.clone();
        
        // Override with element's own attributes
        for (attr, value) in &element.attributes {
            if attr.starts_with("stroke") || attr.starts_with("fill") || attr.starts_with("marker") {
                styles.insert(attr.clone(), value.clone());
            }
        }
        
        // Parse style attribute
        if let Some(style_attr) = element.attributes.get("style") {
            for part in style_attr.split(';') {
                if let Some((key, value)) = part.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    if key.starts_with("stroke") || key.starts_with("fill") || key.starts_with("marker") {
                        styles.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        styles
    }

    fn process_stroke_attributes(
        &self,
        element: &mut Element,
        current_styles: &HashMap<String, String>,
        parent_styles: &HashMap<String, String>,
    ) {
        let stroke = current_styles.get("stroke");
        let stroke_opacity = current_styles.get("stroke-opacity");
        let stroke_width = current_styles.get("stroke-width");
        let marker_end = current_styles.get("marker-end");
        
        let should_remove_stroke = stroke.map_or(true, |s| s == "none")
            || stroke_opacity.map_or(false, |op| op == "0")
            || stroke_width.map_or(false, |w| w == "0");
        
        if should_remove_stroke {
            // Check if stroke-width affects marker visibility
            let can_remove = stroke_width.map_or(true, |w| w == "0") || marker_end.is_none();
            
            if can_remove {
                // Remove all stroke-related attributes
                let stroke_attrs: Vec<String> = element
                    .attributes
                    .keys()
                    .filter(|k| k.starts_with("stroke"))
                    .cloned()
                    .collect();
                
                for attr in stroke_attrs {
                    element.attributes.shift_remove(&attr);
                }
                
                // Set explicit "none" if parent has non-none stroke
                let parent_stroke = parent_styles.get("stroke");
                if parent_stroke.map_or(false, |s| s != "none") {
                    element.attributes.insert("stroke".to_string(), "none".to_string());
                }
            }
        }
    }

    fn process_fill_attributes(
        &self,
        element: &mut Element,
        current_styles: &HashMap<String, String>,
    ) {
        let fill = current_styles.get("fill");
        let fill_opacity = current_styles.get("fill-opacity");
        
        let should_remove_fill = fill.map_or(false, |f| f == "none")
            || fill_opacity.map_or(false, |op| op == "0");
        
        if should_remove_fill {
            // Remove all fill-related attributes except fill itself
            let fill_attrs: Vec<String> = element
                .attributes
                .keys()
                .filter(|k| k.starts_with("fill-"))
                .cloned()
                .collect();
            
            for attr in fill_attrs {
                element.attributes.shift_remove(&attr);
            }
            
            // Set explicit "none" if not already set
            if fill.map_or(true, |f| f != "none") {
                element.attributes.insert("fill".to_string(), "none".to_string());
            }
        }
    }

    fn should_remove_element(
        &self,
        element: &Element,
        current_styles: &HashMap<String, String>,
    ) -> bool {
        let stroke = current_styles.get("stroke");
        let fill = current_styles.get("fill");
        
        let no_stroke = stroke.map_or(true, |s| s == "none") || element.attributes.get("stroke").map_or(false, |s| s == "none");
        let no_fill = fill.map_or(false, |f| f == "none") || element.attributes.get("fill").map_or(false, |f| f == "none");
        
        no_stroke && no_fill
    }

    fn remove_marked_elements(&self, element: &mut Element, nodes_to_remove: &[*mut Element]) {
        element.children.retain(|child| {
            if let Node::Element(child_elem) = child {
                let child_ptr = child_elem as *const Element as *mut Element;
                !nodes_to_remove.contains(&child_ptr)
            } else {
                true
            }
        });
        
        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(child_elem) = child {
                self.remove_marked_elements(child_elem, nodes_to_remove);
            }
        }
    }
}

impl Default for RemoveUselessStrokeAndFillPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RemoveUselessStrokeAndFillPlugin {
    fn name(&self) -> &'static str {
        "removeUselessStrokeAndFill"
    }

    fn description(&self) -> &'static str {
        "removes useless stroke and fill attributes"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let params = self.parse_params(params);
        
        // Skip optimization if there are style or script elements
        if self.has_style_or_script(&document.root) {
            return Ok(());
        }
        
        let mut nodes_to_remove = Vec::new();
        self.process_element_recursive(&mut document.root, &params, &HashMap::new(), &mut nodes_to_remove);
        
        // Remove marked elements
        if !nodes_to_remove.is_empty() {
            self.remove_marked_elements(&mut document.root, &nodes_to_remove);
        }
        
        Ok(())
    }
}

impl RemoveUselessStrokeAndFillPlugin {
    fn process_element_recursive(
        &self,
        element: &mut Element,
        params: &RemoveUselessStrokeAndFillParams,
        parent_styles: &HashMap<String, String>,
        nodes_to_remove: &mut Vec<*mut Element>,
    ) {
        // Process current element
        let current_styles = self.process_element(element, params, parent_styles, nodes_to_remove);
        
        // Process children with updated styles
        for child in &mut element.children {
            if let Node::Element(child_elem) = child {
                self.process_element_recursive(child_elem, params, &current_styles, nodes_to_remove);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::parse_svg;

    #[test]
    fn test_remove_stroke_none() {
        let mut doc = parse_svg(r#"
            <svg>
                <rect stroke="none" fill="red" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let mut plugin = RemoveUselessStrokeAndFillPlugin::new();
        let plugin_info = PluginInfo::default();
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        let output = doc.to_string();
        assert!(!output.contains(r#"stroke="none""#));
        assert!(output.contains(r#"fill="red""#));
    }

    #[test]
    fn test_remove_fill_none() {
        let mut doc = parse_svg(r#"
            <svg>
                <rect stroke="blue" fill="none" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let mut plugin = RemoveUselessStrokeAndFillPlugin::new();
        let plugin_info = PluginInfo::default();
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        let output = doc.to_string();
        assert!(output.contains(r#"stroke="blue""#));
        assert!(output.contains(r#"fill="none""#)); // Should keep explicit none
    }

    #[test]
    fn test_remove_zero_opacity() {
        let mut doc = parse_svg(r#"
            <svg>
                <rect stroke-opacity="0" fill-opacity="0" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let mut plugin = RemoveUselessStrokeAndFillPlugin::new();
        let plugin_info = PluginInfo::default();
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        let output = doc.to_string();
        assert!(!output.contains("stroke-opacity"));
        assert!(output.contains(r#"fill="none""#));
    }

    #[test]
    fn test_preserve_with_id() {
        let mut doc = parse_svg(r#"
            <svg>
                <rect id="test" stroke="none" fill="none" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let mut plugin = RemoveUselessStrokeAndFillPlugin::new();
        let plugin_info = PluginInfo::default();
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        let output = doc.to_string();
        // Should preserve attributes because element has ID
        assert!(output.contains(r#"stroke="none""#));
        assert!(output.contains(r#"fill="none""#));
    }

    #[test]
    fn test_skip_with_style_element() {
        let mut doc = parse_svg(r#"
            <svg>
                <style>.test { fill: red; }</style>
                <rect stroke="none" fill="none" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let mut plugin = RemoveUselessStrokeAndFillPlugin::new();
        let plugin_info = PluginInfo::default();
        plugin.apply(&mut doc, &plugin_info, None).unwrap();
        
        let output = doc.to_string();
        // Should preserve attributes because of style element
        assert!(output.contains(r#"stroke="none""#));
        assert!(output.contains(r#"fill="none""#));
    }
}