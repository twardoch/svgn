// this_file: svgn/src/plugins/remove_off_canvas_paths.rs

//! Plugin to remove paths that are drawn outside of the viewBox
//!
//! This plugin removes paths and shapes that are completely outside the viewBox,
//! making them invisible and unnecessary.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
struct ViewBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Plugin to remove paths drawn outside viewBox
pub struct RemoveOffCanvasPathsPlugin;

impl Plugin for RemoveOffCanvasPathsPlugin {
    fn name(&self) -> &'static str {
        "removeOffCanvasPaths"
    }

    fn description(&self) -> &'static str {
        "removes elements that are drawn outside of the viewBox (disabled by default)"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        // First, find the viewBox from the root SVG element
        if let Some(viewbox) = self.get_viewbox(&document.root) {
            self.process_element(&mut document.root, &viewbox);
        }
        Ok(())
    }
}

impl RemoveOffCanvasPathsPlugin {
    fn get_viewbox(&self, svg_element: &Element) -> Option<ViewBox> {
        if svg_element.name != "svg" {
            return None;
        }

        let viewbox_str = svg_element.attributes.get("viewBox")?;
        let parts: Vec<f64> = viewbox_str
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        if parts.len() == 4 {
            Some(ViewBox {
                x: parts[0],
                y: parts[1],
                width: parts[2],
                height: parts[3],
            })
        } else {
            None
        }
    }

    fn process_element(&self, element: &mut Element, viewbox: &ViewBox) {
        // Process children and remove those outside viewBox
        element.children.retain_mut(|child| {
            if let Node::Element(ref mut child_elem) = child {
                // First check if this element is outside viewBox
                if self.is_outside_viewbox(child_elem, viewbox) {
                    return false; // Remove this element
                }

                // Recursively process children
                self.process_element(child_elem, viewbox);
                true
            } else {
                true // Keep non-element nodes
            }
        });
    }

    fn is_outside_viewbox(&self, element: &Element, viewbox: &ViewBox) -> bool {
        match element.name.as_str() {
            "rect" => self.is_rect_outside(element, viewbox),
            "circle" => self.is_circle_outside(element, viewbox),
            "ellipse" => self.is_ellipse_outside(element, viewbox),
            "line" => self.is_line_outside(element, viewbox),
            "polygon" | "polyline" => self.is_polygon_outside(element, viewbox),
            "path" => self.is_path_outside(element, viewbox),
            _ => false, // Don't remove other elements
        }
    }

    fn is_rect_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        let x = self.get_numeric_attr(element, "x").unwrap_or(0.0);
        let y = self.get_numeric_attr(element, "y").unwrap_or(0.0);
        let width = self.get_numeric_attr(element, "width").unwrap_or(0.0);
        let height = self.get_numeric_attr(element, "height").unwrap_or(0.0);

        // Check if rect is completely outside viewBox
        x + width < viewbox.x
            || x > viewbox.x + viewbox.width
            || y + height < viewbox.y
            || y > viewbox.y + viewbox.height
    }

    fn is_circle_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        let cx = self.get_numeric_attr(element, "cx").unwrap_or(0.0);
        let cy = self.get_numeric_attr(element, "cy").unwrap_or(0.0);
        let r = self.get_numeric_attr(element, "r").unwrap_or(0.0);

        // Check if circle is completely outside viewBox
        cx + r < viewbox.x
            || cx - r > viewbox.x + viewbox.width
            || cy + r < viewbox.y
            || cy - r > viewbox.y + viewbox.height
    }

    fn is_ellipse_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        let cx = self.get_numeric_attr(element, "cx").unwrap_or(0.0);
        let cy = self.get_numeric_attr(element, "cy").unwrap_or(0.0);
        let rx = self.get_numeric_attr(element, "rx").unwrap_or(0.0);
        let ry = self.get_numeric_attr(element, "ry").unwrap_or(0.0);

        // Check if ellipse is completely outside viewBox
        cx + rx < viewbox.x
            || cx - rx > viewbox.x + viewbox.width
            || cy + ry < viewbox.y
            || cy - ry > viewbox.y + viewbox.height
    }

    fn is_line_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        let x1 = self.get_numeric_attr(element, "x1").unwrap_or(0.0);
        let y1 = self.get_numeric_attr(element, "y1").unwrap_or(0.0);
        let x2 = self.get_numeric_attr(element, "x2").unwrap_or(0.0);
        let y2 = self.get_numeric_attr(element, "y2").unwrap_or(0.0);

        // Check if both endpoints are outside the same edge of viewBox
        (x1 < viewbox.x && x2 < viewbox.x)
            || (x1 > viewbox.x + viewbox.width && x2 > viewbox.x + viewbox.width)
            || (y1 < viewbox.y && y2 < viewbox.y)
            || (y1 > viewbox.y + viewbox.height && y2 > viewbox.y + viewbox.height)
    }

    fn is_polygon_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        if let Some(points_str) = element.attributes.get("points") {
            let points = self.parse_points(points_str);
            if points.is_empty() {
                return false;
            }

            // Check if all points are outside the same edge
            let all_left = points.iter().all(|(x, _)| *x < viewbox.x);
            let all_right = points.iter().all(|(x, _)| *x > viewbox.x + viewbox.width);
            let all_above = points.iter().all(|(_, y)| *y < viewbox.y);
            let all_below = points.iter().all(|(_, y)| *y > viewbox.y + viewbox.height);

            all_left || all_right || all_above || all_below
        } else {
            false
        }
    }

    fn is_path_outside(&self, element: &Element, viewbox: &ViewBox) -> bool {
        if let Some(d) = element.attributes.get("d") {
            // Simple heuristic: extract numeric values from path data
            // This is a simplified check - a full implementation would parse the path
            let bounds = self.get_path_bounds(d);
            if let Some((min_x, min_y, max_x, max_y)) = bounds {
                max_x < viewbox.x
                    || min_x > viewbox.x + viewbox.width
                    || max_y < viewbox.y
                    || min_y > viewbox.y + viewbox.height
            } else {
                false
            }
        } else {
            false
        }
    }

    fn parse_points(&self, points_str: &str) -> Vec<(f64, f64)> {
        let numbers: Vec<f64> = points_str
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        numbers
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some((chunk[0], chunk[1]))
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_path_bounds(&self, d: &str) -> Option<(f64, f64, f64, f64)> {
        // Extract numbers from path data (simplified)
        let numbers: Vec<f64> = d
            .split(|c: char| c.is_alphabetic() || c.is_whitespace() || c == ',')
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        if numbers.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        // Process numbers in pairs (x, y)
        for chunk in numbers.chunks(2) {
            if !chunk.is_empty() {
                let x = chunk[0];
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
            if chunk.len() >= 2 {
                let y = chunk[1];
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }

        Some((min_x, min_y, max_x, max_y))
    }

    fn get_numeric_attr(&self, element: &Element, attr_name: &str) -> Option<f64> {
        element.attributes.get(attr_name)?.parse::<f64>().ok()
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_test_document_with_viewbox(viewbox: &str) -> Document {
        let mut root_attrs = IndexMap::new();
        root_attrs.insert("viewBox".to_string(), viewbox.to_string());

        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: root_attrs,
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
        let plugin = RemoveOffCanvasPathsPlugin;
        assert_eq!(plugin.name(), "removeOffCanvasPaths");
        assert_eq!(
            plugin.description(),
            "removes elements that are drawn outside of the viewBox (disabled by default)"
        );
    }

    #[test]
    fn test_remove_rect_outside_viewbox() {
        let mut document = create_test_document_with_viewbox("0 0 100 100");

        document.root.children = vec![
            // Rect completely outside (left)
            Node::Element(create_element(
                "rect",
                vec![("x", "-50"), ("y", "10"), ("width", "40"), ("height", "40")],
            )),
            // Rect completely outside (right)
            Node::Element(create_element(
                "rect",
                vec![("x", "110"), ("y", "10"), ("width", "40"), ("height", "40")],
            )),
            // Rect inside viewBox
            Node::Element(create_element(
                "rect",
                vec![("x", "10"), ("y", "10"), ("width", "40"), ("height", "40")],
            )),
            // Rect partially outside (should keep)
            Node::Element(create_element(
                "rect",
                vec![("x", "90"), ("y", "10"), ("width", "20"), ("height", "40")],
            )),
        ];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Only rects completely outside should be removed
        assert_eq!(document.root.children.len(), 2);
    }

    #[test]
    fn test_remove_circle_outside_viewbox() {
        let mut document = create_test_document_with_viewbox("0 0 100 100");

        document.root.children = vec![
            // Circle completely outside
            Node::Element(create_element(
                "circle",
                vec![("cx", "-20"), ("cy", "50"), ("r", "10")],
            )),
            // Circle inside
            Node::Element(create_element(
                "circle",
                vec![("cx", "50"), ("cy", "50"), ("r", "30")],
            )),
            // Circle touching edge (should keep)
            Node::Element(create_element(
                "circle",
                vec![("cx", "95"), ("cy", "50"), ("r", "10")],
            )),
        ];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        assert_eq!(document.root.children.len(), 2);
    }

    #[test]
    fn test_remove_line_outside_viewbox() {
        let mut document = create_test_document_with_viewbox("0 0 100 100");

        document.root.children = vec![
            // Line completely outside (both points left)
            Node::Element(create_element(
                "line",
                vec![("x1", "-20"), ("y1", "10"), ("x2", "-10"), ("y2", "20")],
            )),
            // Line crossing viewBox (should keep)
            Node::Element(create_element(
                "line",
                vec![("x1", "-10"), ("y1", "50"), ("x2", "110"), ("y2", "50")],
            )),
            // Line inside
            Node::Element(create_element(
                "line",
                vec![("x1", "10"), ("y1", "10"), ("x2", "90"), ("y2", "90")],
            )),
        ];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        assert_eq!(document.root.children.len(), 2);
    }

    #[test]
    fn test_remove_polygon_outside_viewbox() {
        let mut document = create_test_document_with_viewbox("0 0 100 100");

        document.root.children = vec![
            // Polygon completely outside
            Node::Element(create_element(
                "polygon",
                vec![("points", "110,10 120,20 115,30")],
            )),
            // Polygon inside
            Node::Element(create_element(
                "polygon",
                vec![("points", "10,10 50,10 30,50")],
            )),
        ];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        assert_eq!(document.root.children.len(), 1);
    }

    #[test]
    fn test_no_viewbox_no_removal() {
        let mut document = Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            ..Default::default()
        };

        document.root.children = vec![
            Node::Element(create_element(
                "rect",
                vec![("x", "-50"), ("y", "10"), ("width", "40"), ("height", "40")],
            )),
            Node::Element(create_element(
                "circle",
                vec![("cx", "1000"), ("cy", "1000"), ("r", "50")],
            )),
        ];

        let initial_count = document.root.children.len();

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Without viewBox, nothing should be removed
        assert_eq!(document.root.children.len(), initial_count);
    }

    #[test]
    fn test_nested_elements() {
        let mut document = create_test_document_with_viewbox("0 0 100 100");

        let group = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                // Rect outside
                Node::Element(create_element(
                    "rect",
                    vec![("x", "-50"), ("y", "10"), ("width", "40"), ("height", "40")],
                )),
                // Rect inside
                Node::Element(create_element(
                    "rect",
                    vec![("x", "10"), ("y", "10"), ("width", "40"), ("height", "40")],
                )),
            ],
        };

        document.root.children = vec![Node::Element(group)];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        // Group should remain with only the inside rect
        if let Node::Element(group) = &document.root.children[0] {
            assert_eq!(group.children.len(), 1);
        }
    }

    #[test]
    fn test_viewbox_with_offset() {
        let mut document = create_test_document_with_viewbox("50 50 100 100");

        document.root.children = vec![
            // Rect outside the offset viewBox
            Node::Element(create_element(
                "rect",
                vec![("x", "0"), ("y", "0"), ("width", "40"), ("height", "40")],
            )),
            // Rect inside the offset viewBox
            Node::Element(create_element(
                "rect",
                vec![("x", "60"), ("y", "60"), ("width", "40"), ("height", "40")],
            )),
        ];

        let mut plugin = RemoveOffCanvasPathsPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), None);
        assert!(result.is_ok());

        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(elem) = &document.root.children[0] {
            assert_eq!(elem.attributes.get("x"), Some(&"60".to_string()));
        }
    }
}
