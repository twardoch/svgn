// this_file: svgn/src/plugins/convert_colors.rs

//! Convert colors plugin
//!
//! This plugin converts colors between different formats:
//! - Color names to hex (fuchsia → #ff00ff)
//! - RGB to hex (rgb(255, 0, 255) → #ff00ff)
//! - Long hex to short hex (#aabbcc → #abc)
//! - Hex to short names (#000080 → navy)

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Plugin that converts colors between different formats
pub struct ConvertColorsPlugin;

impl Plugin for ConvertColorsPlugin {
    fn name(&self) -> &'static str {
        "convertColors"
    }
    
    fn description(&self) -> &'static str {
        "converts colors: rgb() to #rrggbb and #rrggbb to #rgb"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let config = ConvertColorsConfig::from_params(params);
        let mut mask_counter = 0;
        
        convert_colors_in_element(&mut document.root, &config, &mut mask_counter);
        
        Ok(())
    }
}

#[derive(Debug)]
struct ConvertColorsConfig {
    current_color: Option<String>, // false, string, or regex - simplified to string for now
    names2hex: bool,
    rgb2hex: bool,
    convert_case: Option<String>, // "lower", "upper", or None
    shorthex: bool,
    shortname: bool,
}

impl ConvertColorsConfig {
    fn from_params(params: Option<&Value>) -> Self {
        let params = params.unwrap_or(&Value::Null);
        
        Self {
            current_color: params.get("currentColor")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            names2hex: params.get("names2hex")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            rgb2hex: params.get("rgb2hex")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            convert_case: params.get("convertCase")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            shorthex: params.get("shorthex")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            shortname: params.get("shortname")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        }
    }
}

fn convert_colors_in_element(element: &mut Element, config: &ConvertColorsConfig, mask_counter: &mut usize) {
    // Track mask elements
    if element.name == "mask" {
        *mask_counter += 1;
    }
    
    convert_colors_in_element_attrs(element, config, *mask_counter);
    
    // Process children
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            convert_colors_in_element(child_element, config, mask_counter);
        }
    }
    
    // Untrack mask elements
    if element.name == "mask" {
        *mask_counter -= 1;
    }
}

fn convert_colors_in_element_attrs(element: &mut Element, config: &ConvertColorsConfig, mask_counter: usize) {
    let color_props = get_color_properties();
    
    for (name, value) in element.attributes.iter_mut() {
        if color_props.contains(name.as_str()) {
            let mut val = value.clone();
            
            // Convert colors to currentColor
            if let Some(ref current_color_pattern) = config.current_color {
                if mask_counter == 0 {
                    if val == *current_color_pattern || val != "none" {
                        val = "currentColor".to_string();
                    }
                }
            }
            
            // Convert color names to hex
            if config.names2hex {
                let color_name = val.to_lowercase();
                if let Some(hex_value) = get_color_names().get(&color_name) {
                    val = hex_value.clone();
                }
            }
            
            // Convert rgb() to hex
            if config.rgb2hex {
                if let Some(hex_color) = convert_rgb_to_hex(&val) {
                    val = hex_color;
                }
            }
            
            // Apply case conversion
            if let Some(ref case) = config.convert_case {
                if !includes_url_reference(&val) && val != "currentColor" {
                    match case.as_str() {
                        "lower" => val = val.to_lowercase(),
                        "upper" => val = val.to_uppercase(),
                        _ => {}
                    }
                }
            }
            
            // Convert long hex to short hex
            if config.shorthex {
                if let Some(short_hex) = convert_to_short_hex(&val) {
                    val = short_hex;
                }
            }
            
            // Convert hex to short name (but not when we just converted from RGB)
            if config.shortname {
                let color_name = val.to_lowercase();
                if let Some(short_name) = get_color_short_names().get(&color_name) {
                    val = short_name.clone();
                }
            }
            
            *value = val;
        }
    }
}

fn get_color_properties() -> HashSet<&'static str> {
    let mut props = HashSet::new();
    props.insert("color");
    props.insert("fill");
    props.insert("flood-color");
    props.insert("lighting-color");
    props.insert("stop-color");
    props.insert("stroke");
    props
}

fn get_color_names() -> HashMap<String, String> {
    let mut colors = HashMap::new();
    
    // Basic color names (subset from SVG spec)
    colors.insert("aliceblue".to_string(), "#f0f8ff".to_string());
    colors.insert("antiquewhite".to_string(), "#faebd7".to_string());
    colors.insert("aqua".to_string(), "#0ff".to_string());
    colors.insert("aquamarine".to_string(), "#7fffd4".to_string());
    colors.insert("azure".to_string(), "#f0ffff".to_string());
    colors.insert("beige".to_string(), "#f5f5dc".to_string());
    colors.insert("bisque".to_string(), "#ffe4c4".to_string());
    colors.insert("black".to_string(), "#000".to_string());
    colors.insert("blanchedalmond".to_string(), "#ffebcd".to_string());
    colors.insert("blue".to_string(), "#00f".to_string());
    colors.insert("blueviolet".to_string(), "#8a2be2".to_string());
    colors.insert("brown".to_string(), "#a52a2a".to_string());
    colors.insert("burlywood".to_string(), "#deb887".to_string());
    colors.insert("cadetblue".to_string(), "#5f9ea0".to_string());
    colors.insert("chartreuse".to_string(), "#7fff00".to_string());
    colors.insert("chocolate".to_string(), "#d2691e".to_string());
    colors.insert("coral".to_string(), "#ff7f50".to_string());
    colors.insert("cornflowerblue".to_string(), "#6495ed".to_string());
    colors.insert("cornsilk".to_string(), "#fff8dc".to_string());
    colors.insert("crimson".to_string(), "#dc143c".to_string());
    colors.insert("cyan".to_string(), "#0ff".to_string());
    colors.insert("darkblue".to_string(), "#00008b".to_string());
    colors.insert("darkcyan".to_string(), "#008b8b".to_string());
    colors.insert("darkgoldenrod".to_string(), "#b8860b".to_string());
    colors.insert("darkgray".to_string(), "#a9a9a9".to_string());
    colors.insert("darkgreen".to_string(), "#006400".to_string());
    colors.insert("darkgrey".to_string(), "#a9a9a9".to_string());
    colors.insert("darkkhaki".to_string(), "#bdb76b".to_string());
    colors.insert("darkmagenta".to_string(), "#8b008b".to_string());
    colors.insert("darkolivegreen".to_string(), "#556b2f".to_string());
    colors.insert("darkorange".to_string(), "#ff8c00".to_string());
    colors.insert("darkorchid".to_string(), "#9932cc".to_string());
    colors.insert("darkred".to_string(), "#8b0000".to_string());
    colors.insert("darksalmon".to_string(), "#e9967a".to_string());
    colors.insert("darkseagreen".to_string(), "#8fbc8f".to_string());
    colors.insert("darkslateblue".to_string(), "#483d8b".to_string());
    colors.insert("darkslategray".to_string(), "#2f4f4f".to_string());
    colors.insert("darkslategrey".to_string(), "#2f4f4f".to_string());
    colors.insert("darkturquoise".to_string(), "#00ced1".to_string());
    colors.insert("darkviolet".to_string(), "#9400d3".to_string());
    colors.insert("deeppink".to_string(), "#ff1493".to_string());
    colors.insert("deepskyblue".to_string(), "#00bfff".to_string());
    colors.insert("dimgray".to_string(), "#696969".to_string());
    colors.insert("dimgrey".to_string(), "#696969".to_string());
    colors.insert("dodgerblue".to_string(), "#1e90ff".to_string());
    colors.insert("firebrick".to_string(), "#b22222".to_string());
    colors.insert("floralwhite".to_string(), "#fffaf0".to_string());
    colors.insert("forestgreen".to_string(), "#228b22".to_string());
    colors.insert("fuchsia".to_string(), "#f0f".to_string());
    colors.insert("gainsboro".to_string(), "#dcdcdc".to_string());
    colors.insert("ghostwhite".to_string(), "#f8f8ff".to_string());
    colors.insert("gold".to_string(), "#ffd700".to_string());
    colors.insert("goldenrod".to_string(), "#daa520".to_string());
    colors.insert("gray".to_string(), "#808080".to_string());
    colors.insert("green".to_string(), "#008000".to_string());
    colors.insert("greenyellow".to_string(), "#adff2f".to_string());
    colors.insert("grey".to_string(), "#808080".to_string());
    colors.insert("honeydew".to_string(), "#f0fff0".to_string());
    colors.insert("hotpink".to_string(), "#ff69b4".to_string());
    colors.insert("indianred".to_string(), "#cd5c5c".to_string());
    colors.insert("indigo".to_string(), "#4b0082".to_string());
    colors.insert("ivory".to_string(), "#fffff0".to_string());
    colors.insert("khaki".to_string(), "#f0e68c".to_string());
    colors.insert("lavender".to_string(), "#e6e6fa".to_string());
    colors.insert("lavenderblush".to_string(), "#fff0f5".to_string());
    colors.insert("lawngreen".to_string(), "#7cfc00".to_string());
    colors.insert("lemonchiffon".to_string(), "#fffacd".to_string());
    colors.insert("lightblue".to_string(), "#add8e6".to_string());
    colors.insert("lightcoral".to_string(), "#f08080".to_string());
    colors.insert("lightcyan".to_string(), "#e0ffff".to_string());
    colors.insert("lightgoldenrodyellow".to_string(), "#fafad2".to_string());
    colors.insert("lightgray".to_string(), "#d3d3d3".to_string());
    colors.insert("lightgreen".to_string(), "#90ee90".to_string());
    colors.insert("lightgrey".to_string(), "#d3d3d3".to_string());
    colors.insert("lightpink".to_string(), "#ffb6c1".to_string());
    colors.insert("lightsalmon".to_string(), "#ffa07a".to_string());
    colors.insert("lightseagreen".to_string(), "#20b2aa".to_string());
    colors.insert("lightskyblue".to_string(), "#87cefa".to_string());
    colors.insert("lightslategray".to_string(), "#778899".to_string());
    colors.insert("lightslategrey".to_string(), "#778899".to_string());
    colors.insert("lightsteelblue".to_string(), "#b0c4de".to_string());
    colors.insert("lightyellow".to_string(), "#ffffe0".to_string());
    colors.insert("lime".to_string(), "#0f0".to_string());
    colors.insert("limegreen".to_string(), "#32cd32".to_string());
    colors.insert("linen".to_string(), "#faf0e6".to_string());
    colors.insert("magenta".to_string(), "#f0f".to_string());
    colors.insert("maroon".to_string(), "#800000".to_string());
    colors.insert("mediumaquamarine".to_string(), "#66cdaa".to_string());
    colors.insert("mediumblue".to_string(), "#0000cd".to_string());
    colors.insert("mediumorchid".to_string(), "#ba55d3".to_string());
    colors.insert("mediumpurple".to_string(), "#9370db".to_string());
    colors.insert("mediumseagreen".to_string(), "#3cb371".to_string());
    colors.insert("mediumslateblue".to_string(), "#7b68ee".to_string());
    colors.insert("mediumspringgreen".to_string(), "#00fa9a".to_string());
    colors.insert("mediumturquoise".to_string(), "#48d1cc".to_string());
    colors.insert("mediumvioletred".to_string(), "#c71585".to_string());
    colors.insert("midnightblue".to_string(), "#191970".to_string());
    colors.insert("mintcream".to_string(), "#f5fffa".to_string());
    colors.insert("mistyrose".to_string(), "#ffe4e1".to_string());
    colors.insert("moccasin".to_string(), "#ffe4b5".to_string());
    colors.insert("navajowhite".to_string(), "#ffdead".to_string());
    colors.insert("navy".to_string(), "#000080".to_string());
    colors.insert("oldlace".to_string(), "#fdf5e6".to_string());
    colors.insert("olive".to_string(), "#808000".to_string());
    colors.insert("olivedrab".to_string(), "#6b8e23".to_string());
    colors.insert("orange".to_string(), "#ffa500".to_string());
    colors.insert("orangered".to_string(), "#ff4500".to_string());
    colors.insert("orchid".to_string(), "#da70d6".to_string());
    colors.insert("palegoldenrod".to_string(), "#eee8aa".to_string());
    colors.insert("palegreen".to_string(), "#98fb98".to_string());
    colors.insert("paleturquoise".to_string(), "#afeeee".to_string());
    colors.insert("palevioletred".to_string(), "#db7093".to_string());
    colors.insert("papayawhip".to_string(), "#ffefd5".to_string());
    colors.insert("peachpuff".to_string(), "#ffdab9".to_string());
    colors.insert("peru".to_string(), "#cd853f".to_string());
    colors.insert("pink".to_string(), "#ffc0cb".to_string());
    colors.insert("plum".to_string(), "#dda0dd".to_string());
    colors.insert("powderblue".to_string(), "#b0e0e6".to_string());
    colors.insert("purple".to_string(), "#800080".to_string());
    colors.insert("red".to_string(), "#f00".to_string());
    colors.insert("rosybrown".to_string(), "#bc8f8f".to_string());
    colors.insert("royalblue".to_string(), "#4169e1".to_string());
    colors.insert("saddlebrown".to_string(), "#8b4513".to_string());
    colors.insert("salmon".to_string(), "#fa8072".to_string());
    colors.insert("sandybrown".to_string(), "#f4a460".to_string());
    colors.insert("seagreen".to_string(), "#2e8b57".to_string());
    colors.insert("seashell".to_string(), "#fff5ee".to_string());
    colors.insert("sienna".to_string(), "#a0522d".to_string());
    colors.insert("silver".to_string(), "#c0c0c0".to_string());
    colors.insert("skyblue".to_string(), "#87ceeb".to_string());
    colors.insert("slateblue".to_string(), "#6a5acd".to_string());
    colors.insert("slategray".to_string(), "#708090".to_string());
    colors.insert("slategrey".to_string(), "#708090".to_string());
    colors.insert("snow".to_string(), "#fffafa".to_string());
    colors.insert("springgreen".to_string(), "#00ff7f".to_string());
    colors.insert("steelblue".to_string(), "#4682b4".to_string());
    colors.insert("tan".to_string(), "#d2b48c".to_string());
    colors.insert("teal".to_string(), "#008080".to_string());
    colors.insert("thistle".to_string(), "#d8bfd8".to_string());
    colors.insert("tomato".to_string(), "#ff6347".to_string());
    colors.insert("turquoise".to_string(), "#40e0d0".to_string());
    colors.insert("violet".to_string(), "#ee82ee".to_string());
    colors.insert("wheat".to_string(), "#f5deb3".to_string());
    colors.insert("white".to_string(), "#fff".to_string());
    colors.insert("whitesmoke".to_string(), "#f5f5f5".to_string());
    colors.insert("yellow".to_string(), "#ff0".to_string());
    colors.insert("yellowgreen".to_string(), "#9acd32".to_string());
    
    colors
}

fn get_color_short_names() -> HashMap<String, String> {
    let mut colors = HashMap::new();
    
    // Hex to short names (reverse mapping)
    colors.insert("#000080".to_string(), "navy".to_string());
    colors.insert("#008000".to_string(), "green".to_string());
    colors.insert("#008080".to_string(), "teal".to_string());
    colors.insert("#800000".to_string(), "maroon".to_string());
    colors.insert("#800080".to_string(), "purple".to_string());
    colors.insert("#808000".to_string(), "olive".to_string());
    colors.insert("#808080".to_string(), "gray".to_string());
    colors.insert("#c0c0c0".to_string(), "silver".to_string());
    colors.insert("#ff0000".to_string(), "red".to_string());
    colors.insert("#00ff00".to_string(), "lime".to_string());
    colors.insert("#0000ff".to_string(), "blue".to_string());
    colors.insert("#ffff00".to_string(), "yellow".to_string());
    colors.insert("#00ffff".to_string(), "cyan".to_string());
    colors.insert("#ff00ff".to_string(), "magenta".to_string());
    colors.insert("#ffffff".to_string(), "white".to_string());
    colors.insert("#000000".to_string(), "black".to_string());
    
    colors
}

fn convert_rgb_to_hex(value: &str) -> Option<String> {
    let re = Regex::new(r"^rgb\(\s*([+-]?(?:\d*\.\d+|\d+\.?)%?)\s*[,\s]+\s*([+-]?(?:\d*\.\d+|\d+\.?)%?)\s*[,\s]+\s*([+-]?(?:\d*\.\d+|\d+\.?)%?)\s*\)$").ok()?;
    
    let caps = re.captures(value)?;
    let mut nums = Vec::new();
    
    for i in 1..=3 {
        let m = caps.get(i)?.as_str();
        let n = if m.contains('%') {
            (m.trim_end_matches('%').parse::<f64>().ok()? * 2.55).round() as i32
        } else {
            m.parse::<f64>().ok()? as i32
        };
        nums.push(n.max(0).min(255) as u8);
    }
    
    Some(format!("#{:02x}{:02x}{:02x}", nums[0], nums[1], nums[2]))
}

fn convert_to_short_hex(value: &str) -> Option<String> {
    // Check if it's a 6-character hex code that can be shortened
    if value.len() == 7 && value.starts_with('#') {
        let chars: Vec<char> = value.chars().collect();
        if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
            return Some(format!("#{}{}{}", chars[1], chars[3], chars[5]));
        }
    }
    None
}

fn includes_url_reference(value: &str) -> bool {
    value.contains("url(")
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use serde_json::json;
    use indexmap::IndexMap;

    fn create_test_document_with_colors() -> Document {
        let mut doc = Document::new();
        let mut element = Element::new("rect");
        
        let mut attrs = IndexMap::new();
        attrs.insert("fill".to_string(), "red".to_string());
        attrs.insert("stroke".to_string(), "rgb(255, 0, 255)".to_string());
        element.attributes = attrs;
        
        doc.root = element;
        doc
    }

    #[test]
    fn test_color_name_to_hex() {
        let mut plugin = ConvertColorsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_document_with_colors();
        
        plugin.apply(&mut document, &plugin_info, None).unwrap();
        
        assert_eq!(document.root.attributes.get("fill"), Some(&"#f00".to_string()));
    }

    #[test]
    fn test_rgb_to_hex() {
        let mut plugin = ConvertColorsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_document_with_colors();
        
        // Disable shortname and shorthex conversion to keep long hex format
        let params = json!({
            "shortname": false,
            "shorthex": false
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        assert_eq!(document.root.attributes.get("stroke"), Some(&"#ff00ff".to_string()));
    }

    #[test]
    fn test_convert_rgb_to_hex_function() {
        assert_eq!(convert_rgb_to_hex("rgb(255, 0, 255)"), Some("#ff00ff".to_string()));
        assert_eq!(convert_rgb_to_hex("rgb(100%, 0%, 100%)"), Some("#ff00ff".to_string()));
        assert_eq!(convert_rgb_to_hex("rgb(50%, 100, 100%)"), Some("#7f64ff".to_string()));
        assert_eq!(convert_rgb_to_hex("invalid"), None);
    }

    #[test]
    fn test_convert_to_short_hex() {
        assert_eq!(convert_to_short_hex("#aabbcc"), Some("#abc".to_string()));
        assert_eq!(convert_to_short_hex("#112233"), Some("#123".to_string()));
        assert_eq!(convert_to_short_hex("#123456"), None); // Not shortable
        assert_eq!(convert_to_short_hex("#abc"), None); // Already short
    }

    #[test]
    fn test_color_params() {
        let mut plugin = ConvertColorsPlugin;
        let plugin_info = PluginInfo { path: None, multipass_count: 0 };
        let mut document = create_test_document_with_colors();
        
        let params = json!({
            "names2hex": false,
            "rgb2hex": true,
            "shortname": false,
            "shorthex": false
        });
        
        plugin.apply(&mut document, &plugin_info, Some(&params)).unwrap();
        
        // names2hex is disabled, so "red" should stay as "red"
        assert_eq!(document.root.attributes.get("fill"), Some(&"red".to_string()));
        // rgb2hex is enabled, so RGB should convert to hex
        assert_eq!(document.root.attributes.get("stroke"), Some(&"#ff00ff".to_string()));
    }
}