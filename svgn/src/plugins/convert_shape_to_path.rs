// this_file: svgn/src/plugins/convert_shape_to_path.rs

use crate::ast::{Node, NodeType};
use crate::plugins::{Plugin, PluginInfo};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertShapeToPathParams {
    /// Whether to convert circles and ellipses to paths using arc commands
    #[serde(default = "default_false")]
    pub convert_arcs: bool,
    /// Precision for floating point numbers in path data
    #[serde(default)]
    pub float_precision: Option<u8>,
}

fn default_false() -> bool {
    false
}

impl Default for ConvertShapeToPathParams {
    fn default() -> Self {
        Self {
            convert_arcs: false,
            float_precision: None,
        }
    }
}

static NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?").unwrap()
});

pub struct ConvertShapeToPath {
    params: ConvertShapeToPathParams,
}

impl ConvertShapeToPath {
    pub fn new(params: ConvertShapeToPathParams) -> Self {
        Self { params }
    }

    /// Parse a coordinate value, returning None if it's not a valid number (e.g., percentage)
    fn parse_coord(value: &str) -> Option<f64> {
        // Skip if contains non-numeric characters that indicate units or percentages
        if value.contains('%') || value.contains("px") || value.contains("pt") {
            return None;
        }
        value.parse().ok()
    }

    /// Convert a rectangle to a path
    fn convert_rect(&self, node: &mut Node) -> bool {
        let attrs = &node.attributes;
        
        // Don't convert rectangles with rounded corners
        if attrs.contains_key("rx") || attrs.contains_key("ry") {
            return false;
        }

        // Extract required attributes
        let width_str = attrs.get("width")?;
        let height_str = attrs.get("height")?;
        
        let x = Self::parse_coord(attrs.get("x").unwrap_or("0"))?;
        let y = Self::parse_coord(attrs.get("y").unwrap_or("0"))?;
        let width = Self::parse_coord(width_str)?;
        let height = Self::parse_coord(height_str)?;

        // Build path data: M x y H x+width V y+height H x z
        let path_data = format!(
            "M{} {}H{}V{}H{}z",
            format_number(x, self.params.float_precision),
            format_number(y, self.params.float_precision),
            format_number(x + width, self.params.float_precision),
            format_number(y + height, self.params.float_precision),
            format_number(x, self.params.float_precision)
        );

        // Update node
        node.name = "path".to_string();
        node.attributes.insert("d".to_string(), path_data);
        node.attributes.remove("x");
        node.attributes.remove("y");
        node.attributes.remove("width");
        node.attributes.remove("height");

        true
    }

    /// Convert a line to a path
    fn convert_line(&self, node: &mut Node) -> bool {
        let attrs = &node.attributes;
        
        let x1 = Self::parse_coord(attrs.get("x1").unwrap_or("0"))?;
        let y1 = Self::parse_coord(attrs.get("y1").unwrap_or("0"))?;
        let x2 = Self::parse_coord(attrs.get("x2").unwrap_or("0"))?;
        let y2 = Self::parse_coord(attrs.get("y2").unwrap_or("0"))?;

        // Build path data: M x1 y1 L x2 y2
        let path_data = format!(
            "M{} {} {} {}",
            format_number(x1, self.params.float_precision),
            format_number(y1, self.params.float_precision),
            format_number(x2, self.params.float_precision),
            format_number(y2, self.params.float_precision)
        );

        // Update node
        node.name = "path".to_string();
        node.attributes.insert("d".to_string(), path_data);
        node.attributes.remove("x1");
        node.attributes.remove("y1");
        node.attributes.remove("x2");
        node.attributes.remove("y2");

        true
    }

    /// Convert polyline or polygon to a path
    fn convert_poly(&self, node: &mut Node) -> bool {
        let attrs = &node.attributes;
        let is_polygon = node.name == "polygon";
        
        let points_str = attrs.get("points")?;
        
        // Extract all numbers from the points string
        let coords: Vec<f64> = NUMBER_REGEX
            .find_iter(points_str)
            .filter_map(|m| m.as_str().parse().ok())
            .collect();

        // Need at least 2 coordinate pairs (4 numbers)
        if coords.len() < 4 {
            // Remove the node by marking it for deletion
            node.node_type = NodeType::Text; // Mark as invalid
            return false;
        }

        // Build path data
        let mut path_data = String::new();
        
        for (i, chunk) in coords.chunks(2).enumerate() {
            if chunk.len() == 2 {
                if i == 0 {
                    path_data.push_str(&format!(
                        "M{} {}",
                        format_number(chunk[0], self.params.float_precision),
                        format_number(chunk[1], self.params.float_precision)
                    ));
                } else {
                    path_data.push_str(&format!(
                        " {} {}",
                        format_number(chunk[0], self.params.float_precision),
                        format_number(chunk[1], self.params.float_precision)
                    ));
                }
            }
        }

        // Add closing command for polygons
        if is_polygon {
            path_data.push('z');
        }

        // Update node
        node.name = "path".to_string();
        node.attributes.insert("d".to_string(), path_data);
        node.attributes.remove("points");

        true
    }

    /// Convert circle to a path using arc commands
    fn convert_circle(&self, node: &mut Node) -> bool {
        if !self.params.convert_arcs {
            return false;
        }

        let attrs = &node.attributes;
        
        let cx = Self::parse_coord(attrs.get("cx").unwrap_or("0"))?;
        let cy = Self::parse_coord(attrs.get("cy").unwrap_or("0"))?;
        let r = Self::parse_coord(attrs.get("r").unwrap_or("0"))?;

        // Build path data using two arc commands
        let path_data = format!(
            "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
            format_number(cx, self.params.float_precision),
            format_number(cy - r, self.params.float_precision),
            format_number(r, self.params.float_precision),
            format_number(r, self.params.float_precision),
            format_number(cx, self.params.float_precision),
            format_number(cy + r, self.params.float_precision),
            format_number(r, self.params.float_precision),
            format_number(r, self.params.float_precision),
            format_number(cx, self.params.float_precision),
            format_number(cy - r, self.params.float_precision)
        );

        // Update node
        node.name = "path".to_string();
        node.attributes.insert("d".to_string(), path_data);
        node.attributes.remove("cx");
        node.attributes.remove("cy");
        node.attributes.remove("r");

        true
    }

    /// Convert ellipse to a path using arc commands
    fn convert_ellipse(&self, node: &mut Node) -> bool {
        if !self.params.convert_arcs {
            return false;
        }

        let attrs = &node.attributes;
        
        let cx = Self::parse_coord(attrs.get("cx").unwrap_or("0"))?;
        let cy = Self::parse_coord(attrs.get("cy").unwrap_or("0"))?;
        let rx = Self::parse_coord(attrs.get("rx").unwrap_or("0"))?;
        let ry = Self::parse_coord(attrs.get("ry").unwrap_or("0"))?;

        // Build path data using two arc commands
        let path_data = format!(
            "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
            format_number(cx, self.params.float_precision),
            format_number(cy - ry, self.params.float_precision),
            format_number(rx, self.params.float_precision),
            format_number(ry, self.params.float_precision),
            format_number(cx, self.params.float_precision),
            format_number(cy + ry, self.params.float_precision),
            format_number(rx, self.params.float_precision),
            format_number(ry, self.params.float_precision),
            format_number(cx, self.params.float_precision),
            format_number(cy - ry, self.params.float_precision)
        );

        // Update node
        node.name = "path".to_string();
        node.attributes.insert("d".to_string(), path_data);
        node.attributes.remove("cx");
        node.attributes.remove("cy");
        node.attributes.remove("rx");
        node.attributes.remove("ry");

        true
    }
}

impl Plugin for ConvertShapeToPath {
    fn process_node(&mut self, node: &mut Node, _info: &PluginInfo) {
        if node.node_type != NodeType::Element {
            return;
        }

        match node.name.as_str() {
            "rect" => {
                self.convert_rect(node);
            }
            "line" => {
                self.convert_line(node);
            }
            "polyline" | "polygon" => {
                self.convert_poly(node);
            }
            "circle" => {
                self.convert_circle(node);
            }
            "ellipse" => {
                self.convert_ellipse(node);
            }
            _ => {}
        }
    }
}

/// Format a number with optional precision
fn format_number(value: f64, precision: Option<u8>) -> String {
    match precision {
        Some(p) => {
            let formatted = format!("{:.1$}", value, p as usize);
            // Remove trailing zeros and decimal point if integer
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            if trimmed.is_empty() || trimmed == "-" {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        }
        None => {
            // Default formatting - remove .0 from integers
            if value.fract() == 0.0 {
                format!("{}", value as i64)
            } else {
                value.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeType;

    fn create_element(name: &str, attrs: Vec<(&str, &str)>) -> Node {
        let mut attributes = IndexMap::new();
        for (key, value) in attrs {
            attributes.insert(key.to_string(), value.to_string());
        }
        
        Node {
            node_type: NodeType::Element,
            name: name.to_string(),
            attributes,
            children: vec![],
            parent: None,
            text: None,
        }
    }

    #[test]
    fn test_convert_rect_basic() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("rect", vec![
            ("width", "32"),
            ("height", "32"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M0 0H32V32H0z");
        assert!(!node.attributes.contains_key("width"));
        assert!(!node.attributes.contains_key("height"));
    }

    #[test]
    fn test_convert_rect_with_position() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("rect", vec![
            ("x", "20"),
            ("y", "10"),
            ("width", "50"),
            ("height", "40"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M20 10H70V50H20z");
    }

    #[test]
    fn test_rect_with_rounded_corners_not_converted() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("rect", vec![
            ("x", "10"),
            ("y", "10"),
            ("width", "50"),
            ("height", "50"),
            ("rx", "4"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        // Should not be converted
        assert_eq!(node.name, "rect");
        assert!(node.attributes.contains_key("rx"));
    }

    #[test]
    fn test_convert_line() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("line", vec![
            ("x1", "10"),
            ("y1", "10"),
            ("x2", "50"),
            ("y2", "20"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M10 10 50 20");
        assert!(!node.attributes.contains_key("x1"));
        assert!(!node.attributes.contains_key("y1"));
        assert!(!node.attributes.contains_key("x2"));
        assert!(!node.attributes.contains_key("y2"));
    }

    #[test]
    fn test_convert_polyline() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("polyline", vec![
            ("points", "10,80 20,50 50,20 80,10"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M10 80 20 50 50 20 80 10");
        assert!(!node.attributes.contains_key("points"));
    }

    #[test]
    fn test_convert_polygon() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("polygon", vec![
            ("points", "20 10 50 40 30 20"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M20 10 50 40 30 20z");
        assert!(!node.attributes.contains_key("points"));
    }

    #[test]
    fn test_convert_circle_without_arcs() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams::default());
        let mut node = create_element("circle", vec![
            ("cx", "50"),
            ("cy", "50"),
            ("r", "25"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        // Should not be converted without convert_arcs
        assert_eq!(node.name, "circle");
        assert!(node.attributes.contains_key("r"));
    }

    #[test]
    fn test_convert_circle_with_arcs() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams {
            convert_arcs: true,
            float_precision: None,
        });
        let mut node = create_element("circle", vec![
            ("cx", "50"),
            ("cy", "50"),
            ("r", "25"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        assert_eq!(node.attributes.get("d").unwrap(), "M50 25A25 25 0 1 0 50 75A25 25 0 1 0 50 25z");
        assert!(!node.attributes.contains_key("cx"));
        assert!(!node.attributes.contains_key("cy"));
        assert!(!node.attributes.contains_key("r"));
    }

    #[test]
    fn test_precision_formatting() {
        let mut plugin = ConvertShapeToPath::new(ConvertShapeToPathParams {
            convert_arcs: false,
            float_precision: Some(3),
        });
        let mut node = create_element("rect", vec![
            ("x", "10.123456"),
            ("y", "20.987654"),
            ("width", "30.5"),
            ("height", "40.25"),
        ]);

        plugin.process_node(&mut node, &PluginInfo::default());

        assert_eq!(node.name, "path");
        // Should round to 3 decimal places
        let d = node.attributes.get("d").unwrap();
        assert!(d.contains("10.123"));
        assert!(d.contains("20.988"));
    }
}