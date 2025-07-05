// this_file: svgn/src/plugins/inline_styles_converter.rs

//! CSS property to SVG attribute converter for inline styles plugin
//!
//! This module handles the conversion of CSS properties from lightningcss
//! to SVG attribute name-value pairs.

use lightningcss::properties::Property;
use lightningcss::values::color::CssColor;
use lightningcss::values::length::LengthValue;
use lightningcss::values::number::CSSNumber;

/// Convert a CSS property to an SVG attribute name-value pair
pub fn convert_css_property(property: &Property) -> Option<(String, String)> {
    match property {
        // Fill properties
        Property::Fill(fill_value) => {
            Some(("fill".to_string(), format_fill_value(fill_value)))
        }
        
        // Stroke properties
        Property::Stroke(stroke_value) => {
            Some(("stroke".to_string(), format_stroke_value(stroke_value)))
        }
        
        Property::StrokeWidth(width) => {
            Some(("stroke-width".to_string(), format_length_value(width)))
        }
        
        Property::StrokeLinecap(linecap) => {
            Some(("stroke-linecap".to_string(), format!("{:?}", linecap).to_lowercase()))
        }
        
        Property::StrokeLinejoin(linejoin) => {
            Some(("stroke-linejoin".to_string(), format!("{:?}", linejoin).to_lowercase()))
        }
        
        Property::StrokeMiterlimit(limit) => {
            Some(("stroke-miterlimit".to_string(), format_css_number(limit)))
        }
        
        Property::StrokeDasharray(dasharray) => {
            Some(("stroke-dasharray".to_string(), format_dasharray(dasharray)))
        }
        
        Property::StrokeDashoffset(offset) => {
            Some(("stroke-dashoffset".to_string(), format_length_value(offset)))
        }
        
        // Opacity properties
        Property::Opacity(opacity) => {
            Some(("opacity".to_string(), format_css_number(opacity)))
        }
        
        Property::FillOpacity(opacity) => {
            Some(("fill-opacity".to_string(), format_css_number(opacity)))
        }
        
        Property::StrokeOpacity(opacity) => {
            Some(("stroke-opacity".to_string(), format_css_number(opacity)))
        }
        
        // Display and visibility
        Property::Display(display) => {
            Some(("display".to_string(), format!("{:?}", display).to_lowercase()))
        }
        
        Property::Visibility(visibility) => {
            Some(("visibility".to_string(), format!("{:?}", visibility).to_lowercase()))
        }
        
        // Transform properties (experimental support)
        Property::Transform(transform) => {
            Some(("transform".to_string(), format_transform(transform)))
        }
        
        // Color property (can be used for fill in some contexts)
        Property::Color(color) => {
            Some(("color".to_string(), format_color(color)))
        }
        
        // Font properties that map to SVG text attributes
        Property::FontFamily(font_family) => {
            Some(("font-family".to_string(), format_font_family(font_family)))
        }
        
        Property::FontSize(font_size) => {
            Some(("font-size".to_string(), format_font_size(font_size)))
        }
        
        Property::FontWeight(font_weight) => {
            Some(("font-weight".to_string(), format_font_weight(font_weight)))
        }
        
        Property::FontStyle(font_style) => {
            Some(("font-style".to_string(), format!("{:?}", font_style).to_lowercase()))
        }
        
        // Text properties
        Property::TextAnchor(text_anchor) => {
            Some(("text-anchor".to_string(), format!("{:?}", text_anchor).to_lowercase().replace('_', "-")))
        }
        
        Property::DominantBaseline(baseline) => {
            Some(("dominant-baseline".to_string(), format!("{:?}", baseline).to_lowercase().replace('_', "-")))
        }
        
        // Clip and mask properties
        Property::ClipPath(clip_path) => {
            Some(("clip-path".to_string(), format_clip_path(clip_path)))
        }
        
        Property::Mask(mask) => {
            Some(("mask".to_string(), format_mask(mask)))
        }
        
        // Filter properties
        Property::Filter(filter) => {
            Some(("filter".to_string(), format_filter(filter)))
        }
        
        // Overflow
        Property::Overflow(overflow) => {
            Some(("overflow".to_string(), format_overflow(overflow)))
        }
        
        // Marker properties
        Property::MarkerStart(marker) => {
            Some(("marker-start".to_string(), format_marker(marker)))
        }
        
        Property::MarkerMid(marker) => {
            Some(("marker-mid".to_string(), format_marker(marker)))
        }
        
        Property::MarkerEnd(marker) => {
            Some(("marker-end".to_string(), format_marker(marker)))
        }
        
        // Unparsed properties (fallback)
        Property::Unparsed(unparsed) => {
            let prop_name = &unparsed.property_id.name();
            let value = format_unparsed_value(unparsed);
            Some((prop_name.to_string(), value))
        }
        
        // Ignore properties that don't map to SVG attributes
        _ => None,
    }
}

/// Format a fill value from lightningcss
fn format_fill_value(fill: &lightningcss::properties::svg::SVGPaint) -> String {
    match fill {
        lightningcss::properties::svg::SVGPaint::None => "none".to_string(),
        lightningcss::properties::svg::SVGPaint::Color(color) => format_color(color),
        lightningcss::properties::svg::SVGPaint::Url { url, fallback } => {
            let mut result = format!("url({})", url.url);
            if let Some(fallback) = fallback {
                result.push(' ');
                result.push_str(&format_fill_value(fallback));
            }
            result
        }
    }
}

/// Format a stroke value from lightningcss
fn format_stroke_value(stroke: &lightningcss::properties::svg::SVGPaint) -> String {
    format_fill_value(stroke) // Same format as fill
}

/// Format a CSS color value
fn format_color(color: &CssColor) -> String {
    match color {
        CssColor::RGBA(rgba) => {
            if rgba.alpha == 1.0 {
                format!("#{:02x}{:02x}{:02x}", 
                    (rgba.red * 255.0) as u8,
                    (rgba.green * 255.0) as u8,
                    (rgba.blue * 255.0) as u8
                )
            } else {
                format!("rgba({}, {}, {}, {})", 
                    (rgba.red * 255.0) as u8,
                    (rgba.green * 255.0) as u8,
                    (rgba.blue * 255.0) as u8,
                    rgba.alpha
                )
            }
        }
        CssColor::CurrentColor => "currentColor".to_string(),
        _ => {
            // For other color types, use debug format as fallback
            format!("{:?}", color)
        }
    }
}

/// Format a length value
fn format_length_value(length: &LengthValue) -> String {
    match length {
        LengthValue::Px(px) => format!("{}px", px),
        LengthValue::Em(em) => format!("{}em", em),
        LengthValue::Rem(rem) => format!("{}rem", rem),
        LengthValue::Ex(ex) => format!("{}ex", ex),
        LengthValue::Ch(ch) => format!("{}ch", ch),
        LengthValue::Vw(vw) => format!("{}vw", vw),
        LengthValue::Vh(vh) => format!("{}vh", vh),
        LengthValue::Vmin(vmin) => format!("{}vmin", vmin),
        LengthValue::Vmax(vmax) => format!("{}vmax", vmax),
        LengthValue::Cm(cm) => format!("{}cm", cm),
        LengthValue::Mm(mm) => format!("{}mm", mm),
        LengthValue::In(inches) => format!("{}in", inches),
        LengthValue::Pt(pt) => format!("{}pt", pt),
        LengthValue::Pc(pc) => format!("{}pc", pc),
        _ => format!("{:?}", length), // Fallback for other units
    }
}

/// Format a CSS number
fn format_css_number(number: &CSSNumber) -> String {
    // Remove trailing zeros and unnecessary decimal points
    let formatted = format!("{}", number);
    if formatted.contains('.') {
        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        formatted
    }
}

/// Format stroke dasharray
fn format_dasharray(dasharray: &lightningcss::properties::svg::StrokeDasharray) -> String {
    match dasharray {
        lightningcss::properties::svg::StrokeDasharray::None => "none".to_string(),
        lightningcss::properties::svg::StrokeDasharray::Values(values) => {
            values.iter()
                .map(|v| format_length_value(v))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Format transform property (basic implementation)
fn format_transform(transform: &lightningcss::properties::transform::Transform) -> String {
    // This is a simplified implementation
    // In a full implementation, you would need to handle all transform functions
    format!("{:?}", transform)
}

/// Format font family
fn format_font_family(font_family: &lightningcss::properties::font::FontFamily) -> String {
    match font_family {
        lightningcss::properties::font::FontFamily::FamilyName(name) => name.to_string(),
        lightningcss::properties::font::FontFamily::Generic(generic) => {
            format!("{:?}", generic).to_lowercase()
        }
    }
}

/// Format font size
fn format_font_size(font_size: &lightningcss::properties::font::FontSize) -> String {
    match font_size {
        lightningcss::properties::font::FontSize::Length(length) => format_length_value(length),
        lightningcss::properties::font::FontSize::Absolute(absolute) => {
            format!("{:?}", absolute).to_lowercase().replace('_', "-")
        }
        lightningcss::properties::font::FontSize::Relative(relative) => {
            format!("{:?}", relative).to_lowercase()
        }
    }
}

/// Format font weight
fn format_font_weight(font_weight: &lightningcss::properties::font::FontWeight) -> String {
    match font_weight {
        lightningcss::properties::font::FontWeight::Absolute(weight) => {
            match weight {
                lightningcss::properties::font::AbsoluteFontWeight::Weight(num) => num.to_string(),
                lightningcss::properties::font::AbsoluteFontWeight::Normal => "normal".to_string(),
                lightningcss::properties::font::AbsoluteFontWeight::Bold => "bold".to_string(),
            }
        }
        lightningcss::properties::font::FontWeight::Bolder => "bolder".to_string(),
        lightningcss::properties::font::FontWeight::Lighter => "lighter".to_string(),
    }
}

/// Format clip path (simplified)
fn format_clip_path(clip_path: &lightningcss::properties::masking::ClipPath) -> String {
    // Simplified implementation
    format!("{:?}", clip_path)
}

/// Format mask (simplified)
fn format_mask(mask: &lightningcss::properties::masking::Mask) -> String {
    // Simplified implementation
    format!("{:?}", mask)
}

/// Format filter (simplified)
fn format_filter(filter: &lightningcss::properties::effects::Filter) -> String {
    // Simplified implementation
    format!("{:?}", filter)
}

/// Format overflow
fn format_overflow(overflow: &lightningcss::properties::overflow::Overflow) -> String {
    format!("{:?}", overflow.x).to_lowercase()
}

/// Format marker property
fn format_marker(marker: &lightningcss::properties::svg::Marker) -> String {
    match marker {
        lightningcss::properties::svg::Marker::None => "none".to_string(),
        lightningcss::properties::svg::Marker::Url { url, fallback: _ } => {
            format!("url({})", url.url)
        }
    }
}

/// Format unparsed property value
fn format_unparsed_value(unparsed: &lightningcss::properties::custom::UnparsedProperty) -> String {
    // Convert the token sequence to a string
    // This is a simplified implementation
    unparsed.value.to_css_string(lightningcss::printer::PrinterOptions::default()).unwrap_or_default()
}