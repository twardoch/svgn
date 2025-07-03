// this_file: svgn/src/plugins/convert_shape_to_path.rs

//! Convert basic shapes to path elements
//!
//! This plugin converts rect, line, polyline, polygon, circle and ellipse elements
//! to path elements for better optimization potential.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?").unwrap());

/// Plugin that converts basic shapes to path elements
pub struct ConvertShapeToPathPlugin;

impl Plugin for ConvertShapeToPathPlugin {
    fn name(&self) -> &'static str {
        "convertShapeToPath"
    }

    fn description(&self) -> &'static str {
        "Converts basic shapes to more compact path form"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let convert_arcs = params
            .and_then(|v| v.get("convertArcs"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let float_precision = params
            .and_then(|v| v.get("floatPrecision"))
            .and_then(|v| v.as_u64())
            .map(|p| p as u8);

        // Process root element
        convert_shapes_in_element(&mut document.root, convert_arcs, float_precision);

        Ok(())
    }
}

/// Recursively convert shapes in an element and its children
fn convert_shapes_in_element(
    element: &mut Element,
    convert_arcs: bool,
    float_precision: Option<u8>,
) {
    // Process child elements
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            convert_shapes_in_element(child_element, convert_arcs, float_precision);
        }
    }

    // Convert current element if it's a shape
    convert_shape_element(element, convert_arcs, float_precision);
}

/// Convert a shape element to a path if applicable
fn convert_shape_element(element: &mut Element, convert_arcs: bool, float_precision: Option<u8>) {
    match element.name.as_str() {
        "rect" => convert_rect(element, float_precision),
        "line" => convert_line(element, float_precision),
        "polyline" => convert_polyline(element, float_precision),
        "polygon" => convert_polygon(element, float_precision),
        "circle" if convert_arcs => convert_circle(element, float_precision),
        "ellipse" if convert_arcs => convert_ellipse(element, float_precision),
        _ => {}
    }
}

/// Parse a coordinate value, returning None if it's not a valid number
fn parse_coord(value: &str) -> Option<f64> {
    // Skip if contains non-numeric characters that indicate units or percentages
    if value.contains('%') || value.contains("px") || value.contains("pt") {
        return None;
    }
    value.parse().ok()
}

/// Convert a rectangle to a path
fn convert_rect(element: &mut Element, float_precision: Option<u8>) {
    // Don't convert rectangles with rounded corners
    if element.has_attr("rx") || element.has_attr("ry") {
        return;
    }

    // Extract required attributes
    let width_str = match element.attr("width") {
        Some(w) => w,
        None => return,
    };
    let height_str = match element.attr("height") {
        Some(h) => h,
        None => return,
    };

    let x = match parse_coord(element.attr("x").map_or("0", |v| v)) {
        Some(x) => x,
        None => return,
    };
    let y = match parse_coord(element.attr("y").map_or("0", |v| v)) {
        Some(y) => y,
        None => return,
    };
    let width = match parse_coord(width_str) {
        Some(w) => w,
        None => return,
    };
    let height = match parse_coord(height_str) {
        Some(h) => h,
        None => return,
    };

    // Build path data: M x y H x+width V y+height H x z
    let path_data = format!(
        "M{} {}H{}V{}H{}z",
        format_number(x, float_precision),
        format_number(y, float_precision),
        format_number(x + width, float_precision),
        format_number(y + height, float_precision),
        format_number(x, float_precision)
    );

    // Update element
    element.name = "path".to_string();
    element.set_attr("d".to_string(), path_data);
    element.remove_attr("x");
    element.remove_attr("y");
    element.remove_attr("width");
    element.remove_attr("height");
}

/// Convert a line to a path
fn convert_line(element: &mut Element, float_precision: Option<u8>) {
    let x1 = match parse_coord(element.attr("x1").map_or("0", |v| v)) {
        Some(x) => x,
        None => return,
    };
    let y1 = match parse_coord(element.attr("y1").map_or("0", |v| v)) {
        Some(y) => y,
        None => return,
    };
    let x2 = match parse_coord(element.attr("x2").map_or("0", |v| v)) {
        Some(x) => x,
        None => return,
    };
    let y2 = match parse_coord(element.attr("y2").map_or("0", |v| v)) {
        Some(y) => y,
        None => return,
    };

    // Build path data: M x1 y1 L x2 y2
    let path_data = format!(
        "M{} {} {} {}",
        format_number(x1, float_precision),
        format_number(y1, float_precision),
        format_number(x2, float_precision),
        format_number(y2, float_precision)
    );

    // Update element
    element.name = "path".to_string();
    element.set_attr("d".to_string(), path_data);
    element.remove_attr("x1");
    element.remove_attr("y1");
    element.remove_attr("x2");
    element.remove_attr("y2");
}

/// Convert polyline to a path
fn convert_polyline(element: &mut Element, float_precision: Option<u8>) {
    convert_poly(element, false, float_precision);
}

/// Convert polygon to a path
fn convert_polygon(element: &mut Element, float_precision: Option<u8>) {
    convert_poly(element, true, float_precision);
}

/// Convert polyline or polygon to a path
fn convert_poly(element: &mut Element, is_polygon: bool, float_precision: Option<u8>) {
    let points_str = match element.attr("points") {
        Some(p) => p,
        None => return,
    };

    // Extract all numbers from the points string
    let coords: Vec<f64> = NUMBER_REGEX
        .find_iter(points_str)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    // Need at least 2 coordinate pairs (4 numbers)
    if coords.len() < 4 {
        // Remove the element by removing all its children and marking name as empty
        element.clear_children();
        element.name = "g".to_string(); // Convert to empty group that will be removed by other plugins
        element.attributes.clear();
        return;
    }

    // Build path data
    let mut path_data = String::new();

    for (i, chunk) in coords.chunks(2).enumerate() {
        if chunk.len() == 2 {
            if i == 0 {
                path_data.push_str(&format!(
                    "M{} {}",
                    format_number(chunk[0], float_precision),
                    format_number(chunk[1], float_precision)
                ));
            } else {
                path_data.push_str(&format!(
                    " {} {}",
                    format_number(chunk[0], float_precision),
                    format_number(chunk[1], float_precision)
                ));
            }
        }
    }

    // Add closing command for polygons
    if is_polygon {
        path_data.push('z');
    }

    // Update element
    element.name = "path".to_string();
    element.set_attr("d".to_string(), path_data);
    element.remove_attr("points");
}

/// Convert circle to a path using arc commands
fn convert_circle(element: &mut Element, float_precision: Option<u8>) {
    let cx = match parse_coord(element.attr("cx").map_or("0", |v| v)) {
        Some(x) => x,
        None => return,
    };
    let cy = match parse_coord(element.attr("cy").map_or("0", |v| v)) {
        Some(y) => y,
        None => return,
    };
    let r = match parse_coord(element.attr("r").map_or("0", |v| v)) {
        Some(r) => r,
        None => return,
    };

    // Build path data using two arc commands
    let path_data = format!(
        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
        format_number(cx, float_precision),
        format_number(cy - r, float_precision),
        format_number(r, float_precision),
        format_number(r, float_precision),
        format_number(cx, float_precision),
        format_number(cy + r, float_precision),
        format_number(r, float_precision),
        format_number(r, float_precision),
        format_number(cx, float_precision),
        format_number(cy - r, float_precision)
    );

    // Update element
    element.name = "path".to_string();
    element.set_attr("d".to_string(), path_data);
    element.remove_attr("cx");
    element.remove_attr("cy");
    element.remove_attr("r");
}

/// Convert ellipse to a path using arc commands
fn convert_ellipse(element: &mut Element, float_precision: Option<u8>) {
    let cx = match parse_coord(element.attr("cx").map_or("0", |v| v)) {
        Some(x) => x,
        None => return,
    };
    let cy = match parse_coord(element.attr("cy").map_or("0", |v| v)) {
        Some(y) => y,
        None => return,
    };
    let rx = match parse_coord(element.attr("rx").map_or("0", |v| v)) {
        Some(r) => r,
        None => return,
    };
    let ry = match parse_coord(element.attr("ry").map_or("0", |v| v)) {
        Some(r) => r,
        None => return,
    };

    // Build path data using two arc commands
    let path_data = format!(
        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
        format_number(cx, float_precision),
        format_number(cy - ry, float_precision),
        format_number(rx, float_precision),
        format_number(ry, float_precision),
        format_number(cx, float_precision),
        format_number(cy + ry, float_precision),
        format_number(rx, float_precision),
        format_number(ry, float_precision),
        format_number(cx, float_precision),
        format_number(cy - ry, float_precision)
    );

    // Update element
    element.name = "path".to_string();
    element.set_attr("d".to_string(), path_data);
    element.remove_attr("cx");
    element.remove_attr("cy");
    element.remove_attr("rx");
    element.remove_attr("ry");
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
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::Element;
    use indexmap::IndexMap;

    fn create_element(name: &str, attrs: Vec<(&str, &str)>) -> Element {
        let mut attributes = IndexMap::new();
        for (key, value) in attrs {
            attributes.insert(key.to_string(), value.to_string());
        }

        Element {
            name: name.to_string(),
            attributes,
            children: vec![],
            namespaces: Default::default(),
        }
    }

    #[test]
    fn test_convert_rect_basic() {
        let mut element = create_element("rect", vec![("width", "32"), ("height", "32")]);

        convert_rect(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(element.attr("d").unwrap(), "M0 0H32V32H0z");
        assert!(!element.has_attr("width"));
        assert!(!element.has_attr("height"));
    }

    #[test]
    fn test_convert_rect_with_position() {
        let mut element = create_element(
            "rect",
            vec![("x", "20"), ("y", "10"), ("width", "50"), ("height", "40")],
        );

        convert_rect(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(element.attr("d").unwrap(), "M20 10H70V50H20z");
    }

    #[test]
    fn test_rect_with_rounded_corners_not_converted() {
        let mut element = create_element(
            "rect",
            vec![
                ("x", "10"),
                ("y", "10"),
                ("width", "50"),
                ("height", "50"),
                ("rx", "4"),
            ],
        );

        convert_rect(&mut element, None);

        // Should not be converted
        assert_eq!(element.name, "rect");
        assert!(element.has_attr("rx"));
    }

    #[test]
    fn test_convert_line() {
        let mut element = create_element(
            "line",
            vec![("x1", "10"), ("y1", "10"), ("x2", "50"), ("y2", "20")],
        );

        convert_line(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(element.attr("d").unwrap(), "M10 10 50 20");
        assert!(!element.has_attr("x1"));
        assert!(!element.has_attr("y1"));
        assert!(!element.has_attr("x2"));
        assert!(!element.has_attr("y2"));
    }

    #[test]
    fn test_convert_polyline() {
        let mut element = create_element("polyline", vec![("points", "10,80 20,50 50,20 80,10")]);

        convert_polyline(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(element.attr("d").unwrap(), "M10 80 20 50 50 20 80 10");
        assert!(!element.has_attr("points"));
    }

    #[test]
    fn test_convert_polygon() {
        let mut element = create_element("polygon", vec![("points", "20 10 50 40 30 20")]);

        convert_polygon(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(element.attr("d").unwrap(), "M20 10 50 40 30 20z");
        assert!(!element.has_attr("points"));
    }

    #[test]
    fn test_convert_circle() {
        let mut element = create_element("circle", vec![("cx", "50"), ("cy", "50"), ("r", "25")]);

        convert_circle(&mut element, None);

        assert_eq!(element.name, "path");
        assert_eq!(
            element.attr("d").unwrap(),
            "M50 25A25 25 0 1 0 50 75A25 25 0 1 0 50 25z"
        );
        assert!(!element.has_attr("cx"));
        assert!(!element.has_attr("cy"));
        assert!(!element.has_attr("r"));
    }

    #[test]
    fn test_precision_formatting() {
        assert_eq!(format_number(10.123456, Some(3)), "10.123");
        assert_eq!(format_number(20.987654, Some(3)), "20.988");
        assert_eq!(format_number(30.0, Some(3)), "30");
        assert_eq!(format_number(40.5, None), "40.5");
        assert_eq!(format_number(50.0, None), "50");
    }
}
