// this_file: svgn/src/plugins/inline_styles_converter.rs

//! CSS property to SVG attribute conversion

use lightningcss::properties::Property;

/// Convert a lightningcss Property to SVG attribute name and value (simplified)
pub fn convert_css_property(property: &Property) -> Option<(String, String)> {
    // For now, use a simplified approach that extracts from debug strings
    let debug_str = format!("{:?}", property);
    
    // Try to match common SVG properties
    if debug_str.starts_with("Fill(") {
        Some(("fill".to_string(), extract_color_from_debug(&debug_str)))
    } else if debug_str.starts_with("Stroke(") && !debug_str.contains("Width") && !debug_str.contains("Opacity") {
        Some(("stroke".to_string(), extract_color_from_debug(&debug_str)))
    } else if debug_str.starts_with("Opacity(") {
        Some(("opacity".to_string(), extract_opacity_from_debug(&debug_str)))
    } else if debug_str.starts_with("FillOpacity(") {
        Some(("fill-opacity".to_string(), extract_opacity_from_debug(&debug_str)))
    } else if debug_str.starts_with("StrokeOpacity(") {
        Some(("stroke-opacity".to_string(), extract_opacity_from_debug(&debug_str)))
    } else if debug_str.starts_with("StrokeWidth(") {
        Some(("stroke-width".to_string(), extract_dimension_from_debug(&debug_str)))
    } else if debug_str.starts_with("FontSize(") {
        Some(("font-size".to_string(), extract_dimension_from_debug(&debug_str)))
    } else if debug_str.starts_with("FontFamily(") {
        Some(("font-family".to_string(), extract_font_family_from_debug(&debug_str)))
    } else if debug_str.starts_with("FontStyle(") {
        Some(("font-style".to_string(), extract_font_style_from_debug(&debug_str)))
    } else if debug_str.starts_with("FontWeight(") {
        Some(("font-weight".to_string(), extract_font_weight_from_debug(&debug_str)))
    } else if debug_str.starts_with("Transform(") {
        Some(("transform".to_string(), "none".to_string()))
    } else {
        None
    }
}

/// Extract color from debug string
fn extract_color_from_debug(debug_str: &str) -> String {
    if debug_str.contains("#") {
        // Try to extract hex color
        if let Some(hex_start) = debug_str.find('#') {
            if let Some(hex_end) = debug_str[hex_start..].find([' ', ')', ','].as_ref()) {
                return debug_str[hex_start..hex_start + hex_end].to_string();
            }
        }
    } else if debug_str.contains("RGBA") || debug_str.contains("rgba") {
        // Try to extract RGBA values
        if let (Some(r), Some(g), Some(b)) = (
            extract_number_after(debug_str, "red: "),
            extract_number_after(debug_str, "green: "),
            extract_number_after(debug_str, "blue: ")
        ) {
            return format!("rgb({}, {}, {})", r, g, b);
        }
    }
    "black".to_string()
}

/// Extract opacity from debug string
fn extract_opacity_from_debug(debug_str: &str) -> String {
    // Format: Opacity(AlphaValue(0.75))
    if debug_str.contains("AlphaValue(") {
        if let Some(start) = debug_str.find("AlphaValue(") {
            let after_alpha = &debug_str[start + 11..];
            if let Some(end) = after_alpha.find(')') {
                let num_part = &after_alpha[..end];
                if let Ok(val) = num_part.parse::<f64>() {
                    return val.to_string();
                }
            }
        }
    } else if debug_str.contains("Number(") {
        if let Some(start) = debug_str.find("Number(") {
            if let Some(end) = debug_str[start..].find(')') {
                let num_part = &debug_str[start + 7..start + end];
                if let Ok(val) = num_part.parse::<f64>() {
                    return val.to_string();
                }
            }
        }
    } else if debug_str.contains("Percentage(") {
        if let Some(start) = debug_str.find("Percentage(") {
            if let Some(end) = debug_str[start..].find(')') {
                let pct_part = &debug_str[start + 11..start + end];
                if let Ok(val) = pct_part.parse::<f64>() {
                    return (val / 100.0).to_string();
                }
            }
        }
    }
    "1".to_string()
}

/// Extract a number after a given prefix in a string
fn extract_number_after(s: &str, prefix: &str) -> Option<u8> {
    if let Some(start) = s.find(prefix) {
        let after_prefix = &s[start + prefix.len()..];
        // Take digits until we hit a non-digit
        let num_str: String = after_prefix.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        num_str.parse().ok()
    } else {
        None
    }
}


/// Extract dimension value from debug string (for stroke-width, font-size, etc.)
fn extract_dimension_from_debug(debug_str: &str) -> String {
    // Format: StrokeWidth(Dimension(Px(3.0)))
    // Format: FontSize(Length(Dimension(Px(16.0))))
    if debug_str.contains("Px(") {
        if let Some(start) = debug_str.find("Px(") {
            let after_px = &debug_str[start + 3..];
            if let Some(end) = after_px.find(')') {
                let num_part = &after_px[..end];
                if let Ok(val) = num_part.parse::<f64>() {
                    return format!("{}px", val);
                }
            }
        }
    }
    "1px".to_string()
}

/// Extract font family from debug string
fn extract_font_family_from_debug(_debug_str: &str) -> String {
    // TODO: Implement proper font family extraction
    "sans-serif".to_string()
}

/// Extract font style from debug string
fn extract_font_style_from_debug(_debug_str: &str) -> String {
    // TODO: Implement proper font style extraction
    "normal".to_string()
}

/// Extract font weight from debug string
fn extract_font_weight_from_debug(_debug_str: &str) -> String {
    // TODO: Implement proper font weight extraction
    "normal".to_string()
}