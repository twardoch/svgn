// this_file: svgn/src/plugins/cleanup_numeric_values.rs

//! Plugin to round numeric values to fixed precision and clean up units
//!
//! Rounds numeric values to the specified precision, removes default "px" units,
//! and optionally converts absolute units to pixels for optimization.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Regular expression to match numeric values with optional units
static NUMERIC_VALUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([-+]?\d*\.?\d+([eE][-+]?\d+)?)(px|pt|pc|mm|cm|m|in|ft|em|ex|%)?$").unwrap()
});

/// Conversion factors from absolute units to pixels
static ABSOLUTE_LENGTHS: LazyLock<HashMap<&'static str, f64>> = LazyLock::new(|| {
    [
        ("cm", 96.0 / 2.54), // 96 DPI / 2.54 cm per inch
        ("mm", 96.0 / 25.4), // 96 DPI / 25.4 mm per inch
        ("in", 96.0),        // 96 pixels per inch
        ("pt", 4.0 / 3.0),   // 4/3 pixels per point
        ("pc", 16.0),        // 16 pixels per pica
        ("px", 1.0),         // 1 pixel per pixel
    ]
    .into_iter()
    .collect()
});

/// Plugin to clean up numeric values
pub struct CleanupNumericValuesPlugin;

/// Configuration parameters for the plugin
#[derive(Debug)]
pub struct CleanupNumericValuesParams {
    /// Number of decimal places to round to (default: 3)
    pub float_precision: usize,
    /// Remove leading zeros from decimals (default: true)
    pub leading_zero: bool,
    /// Remove default "px" units (default: true)  
    pub default_px: bool,
    /// Convert absolute units to px when beneficial (default: true)
    pub convert_to_px: bool,
}

impl Default for CleanupNumericValuesParams {
    fn default() -> Self {
        Self {
            float_precision: 3,
            leading_zero: true,
            default_px: true,
            convert_to_px: true,
        }
    }
}

impl CleanupNumericValuesParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut params = Self::default();

        if let Some(Value::Object(map)) = value {
            if let Some(Value::Number(precision)) = map.get("floatPrecision") {
                if let Some(precision) = precision.as_u64() {
                    params.float_precision = precision as usize;
                }
            }
            if let Some(Value::Bool(leading_zero)) = map.get("leadingZero") {
                params.leading_zero = *leading_zero;
            }
            if let Some(Value::Bool(default_px)) = map.get("defaultPx") {
                params.default_px = *default_px;
            }
            if let Some(Value::Bool(convert_to_px)) = map.get("convertToPx") {
                params.convert_to_px = *convert_to_px;
            }
        }

        params
    }
}

impl Plugin for CleanupNumericValuesPlugin {
    fn name(&self) -> &'static str {
        "cleanupNumericValues"
    }

    fn description(&self) -> &'static str {
        "rounds numeric values to the fixed precision, removes default \"px\" units"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = CleanupNumericValuesParams::from_value(params);
        visit_elements(&mut document.root, &config);
        Ok(())
    }
}

/// Visit all elements in the AST and apply numeric value cleanup
fn visit_elements(element: &mut Element, config: &CleanupNumericValuesParams) {
    cleanup_numeric_values_in_element(element, config);

    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            visit_elements(child_element, config);
        }
    }
}

/// Clean up numeric values in a single element
fn cleanup_numeric_values_in_element(element: &mut Element, config: &CleanupNumericValuesParams) {
    // Handle viewBox specially - it contains space/comma-separated numbers
    if let Some(viewbox) = element.attributes.get_mut("viewBox") {
        cleanup_viewbox_value(viewbox, config);
    }

    // Process all other attributes
    for (name, value) in &mut element.attributes {
        // Skip version attribute - it's a text string
        if name == "version" {
            continue;
        }

        // Skip viewBox as we handled it above
        if name == "viewBox" {
            continue;
        }

        if let Some(cleaned) = cleanup_numeric_value(value, config) {
            *value = cleaned;
        }
    }
}

/// Clean up viewBox value which contains multiple space/comma-separated numbers
fn cleanup_viewbox_value(viewbox: &mut String, config: &CleanupNumericValuesParams) {
    let nums: Vec<String> = viewbox
        .trim()
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|value| {
            if let Ok(num) = value.parse::<f64>() {
                let rounded = round_to_precision(num, config.float_precision);
                format_number(rounded, config)
            } else {
                value.to_string()
            }
        })
        .collect();

    *viewbox = nums.join(" ");
}

/// Clean up a single numeric value with optional units
fn cleanup_numeric_value(value: &str, config: &CleanupNumericValuesParams) -> Option<String> {
    let captures = NUMERIC_VALUE_REGEX.captures(value)?;

    let number_str = captures.get(1)?.as_str();
    let mut number: f64 = number_str.parse().ok()?;
    let unit = captures.get(3).map(|m| m.as_str()).unwrap_or("");
    let mut final_unit = unit;

    // Convert absolute units to pixels if beneficial
    if config.convert_to_px && !unit.is_empty() && ABSOLUTE_LENGTHS.contains_key(unit) {
        let conversion_factor = ABSOLUTE_LENGTHS[unit];
        let original_number: f64 = number_str.parse().unwrap();
        let px_value = original_number * conversion_factor;
        let px_rounded = round_to_precision(px_value, config.float_precision);

        // Format the px value and see if it's shorter
        let px_formatted = format_number(px_rounded, config);
        let px_str = if config.default_px {
            px_formatted.clone() // Remove px unit
        } else {
            px_formatted.clone() + "px"
        };

        // Use px conversion if it results in a shorter or equal string
        if px_str.len() <= value.len() {
            number = px_rounded;
            final_unit = "px";
        } else {
            number = round_to_precision(original_number, config.float_precision);
        }
    } else {
        number = round_to_precision(number, config.float_precision);
    }

    // Format the number
    let mut result = format_number(number, config);

    // Remove default 'px' units if enabled
    if config.default_px && final_unit == "px" {
        final_unit = "";
    }

    result.push_str(final_unit);
    Some(result)
}

/// Round a number to the specified precision
fn round_to_precision(num: f64, precision: usize) -> f64 {
    let multiplier = 10_f64.powi(precision as i32);
    (num * multiplier).round() / multiplier
}

/// Format a number according to the configuration
fn format_number(num: f64, config: &CleanupNumericValuesParams) -> String {
    let formatted = if num.fract() == 0.0 {
        // Integer - no decimal places needed
        (num as i64).to_string()
    } else {
        // Format with precision and remove trailing zeros
        format!("{:.precision$}", num, precision = config.float_precision)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };

    if config.leading_zero {
        remove_leading_zero(&formatted)
    } else {
        formatted
    }
}

/// Remove leading zero from decimal numbers (0.5 → .5, -0.5 → -.5)
fn remove_leading_zero(value: &str) -> String {
    if value.len() > 1 && value.starts_with("0.") {
        return value[1..].to_string();
    }

    if value.len() > 2 && value.starts_with("-0.") {
        return format!("-{}", &value[2..]);
    }

    value.to_string()
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use serde_json::json;

    #[test]
    fn test_rounds_decimal_values() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("x".to_string(), "1.23456".to_string());
        element
            .attributes
            .insert("y".to_string(), "2.7891011".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("x"),
            Some(&"1.235".to_string())
        );
        assert_eq!(
            document.root.attributes.get("y"),
            Some(&"2.789".to_string())
        );
    }

    #[test]
    fn test_removes_leading_zeros() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("opacity".to_string(), "0.5".to_string());
        element
            .attributes
            .insert("fill-opacity".to_string(), "-0.25".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("opacity"),
            Some(&".5".to_string())
        );
        assert_eq!(
            document.root.attributes.get("fill-opacity"),
            Some(&"-.25".to_string())
        );
    }

    #[test]
    fn test_removes_default_px_units() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("width".to_string(), "100px".to_string());
        element
            .attributes
            .insert("height".to_string(), "50.5px".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("width"),
            Some(&"100".to_string())
        );
        assert_eq!(
            document.root.attributes.get("height"),
            Some(&"50.5".to_string())
        );
    }

    #[test]
    fn test_converts_units_to_px() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("width".to_string(), "1in".to_string()); // 96px -> "96" (shorter)
        element
            .attributes
            .insert("height".to_string(), "1pt".to_string()); // 4/3 px -> "1.333" (longer, don't convert)
        element
            .attributes
            .insert("x".to_string(), "2in".to_string()); // 192px -> "192" (shorter)
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("width"),
            Some(&"96".to_string())
        );
        assert_eq!(
            document.root.attributes.get("height"),
            Some(&"1pt".to_string())
        ); // Not converted due to length
        assert_eq!(document.root.attributes.get("x"), Some(&"192".to_string()));
    }

    #[test]
    fn test_preserves_version_attribute() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        element
            .attributes
            .insert("version".to_string(), "1.1".to_string());
        element
            .attributes
            .insert("width".to_string(), "1.1".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Version should be unchanged, width should be rounded
        assert_eq!(
            document.root.attributes.get("version"),
            Some(&"1.1".to_string())
        );
        assert_eq!(
            document.root.attributes.get("width"),
            Some(&"1.1".to_string())
        );
    }

    #[test]
    fn test_cleans_viewbox() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        element.attributes.insert(
            "viewBox".to_string(),
            "0.12345 1.6789 100.555 50.9999".to_string(),
        );
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("viewBox"),
            Some(&".123 1.679 100.555 51".to_string())
        );
    }

    #[test]
    fn test_configurable_precision() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("x".to_string(), "1.23456".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        let params = json!({"floatPrecision": 2});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert_eq!(document.root.attributes.get("x"), Some(&"1.23".to_string()));
    }

    #[test]
    fn test_configurable_leading_zero() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("opacity".to_string(), "0.5".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        let params = json!({"leadingZero": false});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert_eq!(
            document.root.attributes.get("opacity"),
            Some(&"0.5".to_string())
        );
    }

    #[test]
    fn test_configurable_default_px() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("width".to_string(), "100px".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        let params = json!({"defaultPx": false});
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        assert_eq!(
            document.root.attributes.get("width"),
            Some(&"100px".to_string())
        );
    }

    #[test]
    fn test_preserves_non_numeric_values() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element
            .attributes
            .insert("fill".to_string(), "red".to_string());
        element
            .attributes
            .insert("id".to_string(), "test123".to_string());
        document.root = element;

        let mut plugin = CleanupNumericValuesPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(
            document.root.attributes.get("fill"),
            Some(&"red".to_string())
        );
        assert_eq!(
            document.root.attributes.get("id"),
            Some(&"test123".to_string())
        );
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = CleanupNumericValuesPlugin;
        assert_eq!(plugin.name(), "cleanupNumericValues");
        assert_eq!(
            plugin.description(),
            "rounds numeric values to the fixed precision, removes default \"px\" units"
        );
    }
}
