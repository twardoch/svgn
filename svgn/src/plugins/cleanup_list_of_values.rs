// this_file: svgn/src/plugins/cleanup_list_of_values.rs

//! Plugin to round list of values to fixed precision
//!
//! Rounds lists of numeric values (like viewBox, points, stroke-dasharray) 
//! to the specified precision and optimizes units.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use regex::Regex;

/// Regular expression to match numeric values with optional units
static NUMERIC_VALUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([-+]?\d*\.?\d+([eE][-+]?\d+)?)(px|pt|pc|mm|cm|m|in|ft|em|ex|%)?$").unwrap()
});

/// Regular expression to split lists of values (space and/or comma separated)
static SEPARATOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s+,?\s*|,\s*").unwrap()
});

/// Regular expression to match the "new" keyword (for enable-background)
static NEW_KEYWORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^new$").unwrap()
});

/// Conversion factors from absolute units to pixels
static ABSOLUTE_LENGTHS: LazyLock<HashMap<&'static str, f64>> = LazyLock::new(|| {
    [
        ("cm", 96.0 / 2.54),  // 96 DPI / 2.54 cm per inch
        ("mm", 96.0 / 25.4),  // 96 DPI / 25.4 mm per inch
        ("in", 96.0),         // 96 pixels per inch
        ("pt", 4.0 / 3.0),    // 4/3 pixels per point
        ("pc", 16.0),         // 16 pixels per pica
        ("px", 1.0),          // 1 pixel per pixel
    ]
    .into_iter()
    .collect()
});

/// Attributes that contain lists of values
static LIST_ATTRIBUTES: LazyLock<[&'static str; 8]> = LazyLock::new(|| {
    [
        "points",
        "enable-background", 
        "viewBox",
        "stroke-dasharray",
        "dx",
        "dy", 
        "x",
        "y",
    ]
});

/// Plugin to clean up lists of numeric values
pub struct CleanupListOfValuesPlugin;

/// Configuration parameters for the plugin
#[derive(Debug)]
pub struct CleanupListOfValuesParams {
    /// Number of decimal places to round to (default: 3)
    pub float_precision: usize,
    /// Remove leading zeros from decimals (default: true)
    pub leading_zero: bool,
    /// Remove default "px" units (default: true)  
    pub default_px: bool,
    /// Convert absolute units to px when beneficial (default: true)
    pub convert_to_px: bool,
}

impl Default for CleanupListOfValuesParams {
    fn default() -> Self {
        Self {
            float_precision: 3,
            leading_zero: true,
            default_px: true,
            convert_to_px: true,
        }
    }
}

impl CleanupListOfValuesParams {
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

impl Plugin for CleanupListOfValuesPlugin {
    fn name(&self) -> &'static str {
        "cleanupListOfValues"
    }

    fn description(&self) -> &'static str {
        "rounds list of values to the fixed precision"
    }

    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = CleanupListOfValuesParams::from_value(params);
        visit_elements(&mut document.root, &config);
        Ok(())
    }
}

/// Visit all elements in the AST and apply list value cleanup
fn visit_elements(element: &mut Element, config: &CleanupListOfValuesParams) {
    cleanup_list_values_in_element(element, config);
    
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            visit_elements(child_element, config);
        }
    }
}

/// Clean up list values in a single element
fn cleanup_list_values_in_element(element: &mut Element, config: &CleanupListOfValuesParams) {
    for attr_name in LIST_ATTRIBUTES.iter() {
        if let Some(value) = element.attributes.get_mut(*attr_name) {
            *value = round_values(value, config);
        }
    }
}

/// Round all values in a space/comma-separated list
fn round_values(lists: &str, config: &CleanupListOfValuesParams) -> String {
    let mut rounded_list = Vec::new();
    
    for elem in SEPARATOR_REGEX.split(lists) {
        if elem.is_empty() {
            continue;
        }
        
        // Check for "new" keyword (enable-background)
        if NEW_KEYWORD_REGEX.is_match(elem) {
            rounded_list.push("new".to_string());
            continue;
        }
        
        // Check for numeric value
        if let Some(captures) = NUMERIC_VALUE_REGEX.captures(elem) {
            if let Some(rounded) = process_numeric_value(&captures, elem, config) {
                rounded_list.push(rounded);
            }
        } else {
            // Non-numeric value, keep as is
            rounded_list.push(elem.to_string());
        }
    }
    
    rounded_list.join(" ")
}

/// Process a single numeric value from the regex captures
fn process_numeric_value(
    captures: &regex::Captures,
    original: &str,
    config: &CleanupListOfValuesParams,
) -> Option<String> {
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
        
        // Use px conversion if it results in a shorter string
        if px_str.len() < original.len() {
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
fn format_number(num: f64, config: &CleanupListOfValuesParams) -> String {
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
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use serde_json::json;

    #[test]
    fn test_rounds_points_list() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "208.250977 77.1308594 223.069336 92.456789".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("points"), Some(&"208.251 77.131 223.069 92.457".to_string()));
    }

    #[test]
    fn test_rounds_viewbox() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        element.attributes.insert("viewBox".to_string(), "0.12345 1.6789 200.28423 200.28423".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("viewBox"), Some(&".123 1.679 200.284 200.284".to_string()));
    }

    #[test]
    fn test_rounds_stroke_dasharray() {
        let mut document = Document::new();
        let mut element = Element::new("path");
        element.attributes.insert("stroke-dasharray".to_string(), "5.555 10.9999 15.123456".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("stroke-dasharray"), Some(&"5.555 11 15.123".to_string()));
    }

    #[test]
    fn test_handles_enable_background_with_new() {
        let mut document = Document::new();
        let mut element = Element::new("svg");
        element.attributes.insert("enable-background".to_string(), "new 0.12345 1.6789 200.28423 200.28423".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("enable-background"), Some(&"new .123 1.679 200.284 200.284".to_string()));
    }

    #[test]
    fn test_rounds_text_positioning() {
        let mut document = Document::new();
        let mut element = Element::new("text");
        element.attributes.insert("dx".to_string(), "1.234567 2.567890 3.999999".to_string());
        element.attributes.insert("dy".to_string(), "0.555 -0.333 0.125".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("dx"), Some(&"1.235 2.568 4".to_string()));
        assert_eq!(document.root.attributes.get("dy"), Some(&".555 -.333 .125".to_string()));
    }

    #[test]
    fn test_handles_comma_separated_values() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "1.234567,2.567890,3.999999,4.12345".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("points"), Some(&"1.235 2.568 4 4.123".to_string()));
    }

    #[test]
    fn test_handles_mixed_separators() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "1.234567, 2.567890 , 3.999999  4.12345".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        assert_eq!(document.root.attributes.get("points"), Some(&"1.235 2.568 4 4.123".to_string()));
    }

    #[test]
    fn test_converts_units_in_lists() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "1in 96pt 3cm 4px".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // 1in = 96px (shorter: "96" vs "1in"), 
        // 96pt = 128px (shorter: "128" vs "96pt", do convert)
        // 3cm ≈ 113.386px (longer: "113.386" vs "3cm", don't convert) 
        // 4px = 4 (remove px)
        assert_eq!(document.root.attributes.get("points"), Some(&"96 128 3cm 4".to_string()));
    }

    #[test]
    fn test_configurable_precision() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "1.23456 2.78901".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        let params = json!({"floatPrecision": 2});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        assert_eq!(document.root.attributes.get("points"), Some(&"1.23 2.79".to_string()));
    }

    #[test]
    fn test_configurable_leading_zero() {
        let mut document = Document::new();
        let mut element = Element::new("polygon");
        element.attributes.insert("points".to_string(), "0.5 -0.25".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        let params = json!({"leadingZero": false});
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), Some(&params)).unwrap();

        assert_eq!(document.root.attributes.get("points"), Some(&"0.5 -0.25".to_string()));
    }

    #[test]
    fn test_preserves_non_list_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");
        element.attributes.insert("fill".to_string(), "red".to_string());
        element.attributes.insert("width".to_string(), "100.555".to_string());
        document.root = element;

        let mut plugin = CleanupListOfValuesPlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();

        // These shouldn't be affected by this plugin
        assert_eq!(document.root.attributes.get("fill"), Some(&"red".to_string()));
        assert_eq!(document.root.attributes.get("width"), Some(&"100.555".to_string()));
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = CleanupListOfValuesPlugin;
        assert_eq!(plugin.name(), "cleanupListOfValues");
        assert_eq!(plugin.description(), "rounds list of values to the fixed precision");
    }
}