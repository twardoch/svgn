// this_file: svgn/src/plugins/remove_hidden_elems.rs

//! Plugin to remove hidden elements
//!
//! This plugin removes elements that are hidden through various means:
//! - display="none" 
//! - visibility="hidden" or visibility="collapse"
//! - opacity="0" (optional)
//! - Zero width/height rectangles, ellipses, images
//! - Zero radius circles
//! - Empty paths

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RemoveHiddenElemsConfig {
    /// Whether to treat display="none" as hidden
    pub display_none: bool,
    /// Whether to treat opacity="0" as hidden
    pub opacity_zero: bool,
    /// Whether to remove empty groups
    pub empty_groups: bool,
    /// Whether to remove elements with no meaningful size
    pub zero_size: bool,
}

impl Default for RemoveHiddenElemsConfig {
    fn default() -> Self {
        Self {
            display_none: true,
            opacity_zero: true,
            empty_groups: true,
            zero_size: true,
        }
    }
}

/// Plugin to remove hidden elements
pub struct RemoveHiddenElemsPlugin;

impl Plugin for RemoveHiddenElemsPlugin {
    fn name(&self) -> &'static str {
        "removeHiddenElems"
    }

    fn description(&self) -> &'static str {
        "removes hidden elements (disabled by default)"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = self.parse_config(params);
        self.process_element(&mut document.root, &config);
        Ok(())
    }
}

impl RemoveHiddenElemsPlugin {
    fn parse_config(&self, params: Option<&Value>) -> RemoveHiddenElemsConfig {
        let mut config = RemoveHiddenElemsConfig::default();
        
        if let Some(Value::Object(obj)) = params {
            if let Some(Value::Bool(v)) = obj.get("displayNone") {
                config.display_none = *v;
            }
            if let Some(Value::Bool(v)) = obj.get("opacity0") {
                config.opacity_zero = *v;
            }
            if let Some(Value::Bool(v)) = obj.get("emptyGroups") {
                config.empty_groups = *v;
            }
            if let Some(Value::Bool(v)) = obj.get("zeroSize") {
                config.zero_size = *v;
            }
        }
        
        config
    }

    fn is_hidden(&self, element: &Element, config: &RemoveHiddenElemsConfig) -> bool {
        // Check display="none"
        if config.display_none {
            if let Some(display) = element.attributes.get("display") {
                if display == "none" {
                    return true;
                }
            }
        }

        // Check visibility="hidden" or visibility="collapse"
        if let Some(visibility) = element.attributes.get("visibility") {
            if visibility == "hidden" || visibility == "collapse" {
                return true;
            }
        }

        // Check opacity="0"
        if config.opacity_zero {
            if let Some(opacity) = element.attributes.get("opacity") {
                if let Ok(opacity_val) = opacity.parse::<f64>() {
                    if opacity_val == 0.0 {
                        return true;
                    }
                }
            }
        }

        // Check for zero-size elements
        if config.zero_size {
            match element.name.as_str() {
                "rect" | "image" | "pattern" => {
                    if self.is_zero_dimension(element, "width") || self.is_zero_dimension(element, "height") {
                        return true;
                    }
                }
                "circle" => {
                    if self.is_zero_dimension(element, "r") {
                        return true;
                    }
                }
                "ellipse" => {
                    if self.is_zero_dimension(element, "rx") || self.is_zero_dimension(element, "ry") {
                        return true;
                    }
                }
                "line" => {
                    // Line is hidden if start and end points are the same
                    let x1 = self.get_numeric_attr(element, "x1").unwrap_or(0.0);
                    let y1 = self.get_numeric_attr(element, "y1").unwrap_or(0.0);
                    let x2 = self.get_numeric_attr(element, "x2").unwrap_or(0.0);
                    let y2 = self.get_numeric_attr(element, "y2").unwrap_or(0.0);
                    if x1 == x2 && y1 == y2 {
                        return true;
                    }
                }
                "polyline" | "polygon" => {
                    if let Some(points) = element.attributes.get("points") {
                        if points.trim().is_empty() {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
                "path" => {
                    if let Some(d) = element.attributes.get("d") {
                        if d.trim().is_empty() {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
                _ => {}
            }
        }

        // Check for empty groups
        if config.empty_groups && element.name == "g" && element.children.is_empty() {
            return true;
        }

        false
    }

    fn is_zero_dimension(&self, element: &Element, attr_name: &str) -> bool {
        if let Some(value) = element.attributes.get(attr_name) {
            if let Ok(num_val) = value.parse::<f64>() {
                return num_val == 0.0;
            }
        }
        false
    }

    fn get_numeric_attr(&self, element: &Element, attr_name: &str) -> Option<f64> {
        element.attributes.get(attr_name)?.parse::<f64>().ok()
    }

    fn process_element(&self, element: &mut Element, config: &RemoveHiddenElemsConfig) {
        // Process children first (bottom-up) and remove hidden elements
        element.children.retain_mut(|child| {
            if let Node::Element(ref mut child_elem) = child {
                // First process the child's children
                self.process_element(child_elem, config);
                
                // Then check if the child itself should be removed
                !self.is_hidden(child_elem, config)
            } else {
                true // Keep non-element nodes
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use std::collections::HashMap;
    use indexmap::IndexMap;

    fn create_test_document() -> Document {
        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            ..Default::default()
        }
    }

    fn create_element(name: &str, attrs: Vec<(&str, &str)>) -> Element {
        let mut attributes = IndexMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }
        Element {
            name: name.to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![],
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = RemoveHiddenElemsPlugin;
        assert_eq!(plugin.name(), "removeHiddenElems");
        assert_eq!(plugin.description(), "removes hidden elements (disabled by default)");
    }

    #[test]
    fn test_remove_display_none() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("rect", vec![("display", "none"), ("width", "100"), ("height", "100")])),
            Node::Element(create_element("circle", vec![("display", "block"), ("r", "50")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Element with display="none" should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.name, "circle");
        }
    }

    #[test]
    fn test_remove_visibility_hidden() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("rect", vec![("visibility", "hidden"), ("width", "100")])),
            Node::Element(create_element("rect", vec![("visibility", "collapse"), ("width", "100")])),
            Node::Element(create_element("rect", vec![("visibility", "visible"), ("width", "100")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Elements with visibility="hidden" or "collapse" should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("visibility"), Some(&"visible".to_string()));
        }
    }

    #[test]
    fn test_remove_opacity_zero() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("rect", vec![("opacity", "0"), ("width", "100")])),
            Node::Element(create_element("rect", vec![("opacity", "0.0"), ("width", "100")])),
            Node::Element(create_element("rect", vec![("opacity", "0.5"), ("width", "100")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Elements with opacity="0" should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("opacity"), Some(&"0.5".to_string()));
        }
    }

    #[test]
    fn test_remove_zero_width_rect() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("rect", vec![("width", "0"), ("height", "100")])),
            Node::Element(create_element("rect", vec![("width", "100"), ("height", "0")])),
            Node::Element(create_element("rect", vec![("width", "100"), ("height", "100")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Rects with zero width or height should be removed
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_remove_zero_radius_circle() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("circle", vec![("r", "0"), ("cx", "50"), ("cy", "50")])),
            Node::Element(create_element("circle", vec![("r", "50"), ("cx", "50"), ("cy", "50")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Circle with zero radius should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("r"), Some(&"50".to_string()));
        }
    }

    #[test]
    fn test_remove_zero_radius_ellipse() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("ellipse", vec![("rx", "0"), ("ry", "50")])),
            Node::Element(create_element("ellipse", vec![("rx", "50"), ("ry", "0")])),
            Node::Element(create_element("ellipse", vec![("rx", "50"), ("ry", "30")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Ellipses with zero rx or ry should be removed
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_remove_empty_path() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("path", vec![("d", "")])),
            Node::Element(create_element("path", vec![("d", "  ")])),
            Node::Element(create_element("path", vec![("d", "M10,10 L20,20")])),
            Node::Element(create_element("path", vec![])), // No d attribute
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Empty paths should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("d"), Some(&"M10,10 L20,20".to_string()));
        }
    }

    #[test]
    fn test_remove_empty_groups() {
        let mut document = create_test_document();
        
        let empty_group = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        let mut filled_group = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![Node::Element(create_element("rect", vec![("width", "100"), ("height", "100")]))],
        };

        document.root.children = vec![
            Node::Element(empty_group),
            Node::Element(filled_group),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Empty group should be removed
        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.children.len(), 1);
        }
    }

    #[test]
    fn test_config_display_none_disabled() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("rect", vec![("display", "none"), ("width", "100")])),
        ];

        let config = serde_json::json!({
            "displayNone": false
        });

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&config));
        assert!(result.is_ok());

        // Element should not be removed when displayNone is disabled
        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_nested_hidden_elements() {
        let mut document = create_test_document();
        
        let mut group = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("rect", vec![("display", "none"), ("width", "100")])),
                Node::Element(create_element("circle", vec![("r", "50")])),
                Node::Element(create_element("rect", vec![("visibility", "hidden"), ("width", "100")])),
            ],
        };

        document.root.children = vec![Node::Element(group)];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Hidden elements within group should be removed
        if let Node::Element(group) = &document.root.children[0] {
            assert_eq!(group.children.len(), 1);
            if let Node::Element(elem) = &group.children[0] {
                assert_eq!(elem.name, "circle");
            }
        }
    }

    #[test]
    fn test_zero_line() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_element("line", vec![("x1", "10"), ("y1", "10"), ("x2", "10"), ("y2", "10")])),
            Node::Element(create_element("line", vec![("x1", "10"), ("y1", "10"), ("x2", "20"), ("y2", "20")])),
        ];

        let mut plugin = RemoveHiddenElemsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Line with same start and end points should be removed
        assert_eq!(document.root.children.len(), 1);
    }
}