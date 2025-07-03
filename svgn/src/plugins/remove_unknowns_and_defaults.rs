// this_file: svgn/src/plugins/remove_unknowns_and_defaults.rs

//! Plugin to remove unknown elements and attributes, and attributes with default values
//!
//! This is a simplified implementation that handles common cases without full SVG spec data.
//! Future versions will include complete SVG specification compliance.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Plugin to remove unknown elements and attributes, and default values
pub struct RemoveUnknownsAndDefaultsPlugin;

/// Configuration parameters for removing unknowns and defaults
#[derive(Debug, Clone)]
pub struct RemoveUnknownsAndDefaultsParams {
    /// Remove unknown element content
    pub unknown_content: bool,
    /// Remove unknown attributes
    pub unknown_attrs: bool,
    /// Remove attributes with default values
    pub default_attrs: bool,
    /// Remove default markup declarations
    pub default_markup_declarations: bool,
    /// Remove useless attribute overrides
    pub useless_overrides: bool,
    /// Keep data-* attributes
    pub keep_data_attrs: bool,
    /// Keep aria-* attributes
    pub keep_aria_attrs: bool,
    /// Keep role attribute
    pub keep_role_attr: bool,
}

impl Default for RemoveUnknownsAndDefaultsParams {
    fn default() -> Self {
        Self {
            unknown_content: true,
            unknown_attrs: true,
            default_attrs: true,
            default_markup_declarations: true,
            useless_overrides: true,
            keep_data_attrs: true,
            keep_aria_attrs: true,
            keep_role_attr: false,
        }
    }
}

impl RemoveUnknownsAndDefaultsParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> Self {
        let mut params = Self::default();

        if let Some(Value::Object(map)) = value {
            if let Some(Value::Bool(val)) = map.get("unknownContent") {
                params.unknown_content = *val;
            }
            if let Some(Value::Bool(val)) = map.get("unknownAttrs") {
                params.unknown_attrs = *val;
            }
            if let Some(Value::Bool(val)) = map.get("defaultAttrs") {
                params.default_attrs = *val;
            }
            if let Some(Value::Bool(val)) = map.get("defaultMarkupDeclarations") {
                params.default_markup_declarations = *val;
            }
            if let Some(Value::Bool(val)) = map.get("uselessOverrides") {
                params.useless_overrides = *val;
            }
            if let Some(Value::Bool(val)) = map.get("keepDataAttrs") {
                params.keep_data_attrs = *val;
            }
            if let Some(Value::Bool(val)) = map.get("keepAriaAttrs") {
                params.keep_aria_attrs = *val;
            }
            if let Some(Value::Bool(val)) = map.get("keepRoleAttr") {
                params.keep_role_attr = *val;
            }
        }

        params
    }
}

// Common SVG elements (simplified list)
static KNOWN_ELEMENTS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        // Root
        "svg",
        // Structural
        "defs",
        "g",
        "symbol",
        "use",
        // Shape elements
        "circle",
        "ellipse",
        "line",
        "path",
        "polygon",
        "polyline",
        "rect",
        // Text elements
        "text",
        "tspan",
        "tref",
        "textPath",
        // Paint server elements
        "linearGradient",
        "radialGradient",
        "pattern",
        // Container elements
        "a",
        "marker",
        "mask",
        "clipPath",
        "foreignObject",
        "switch",
        // Filter elements
        "filter",
        "feBlend",
        "feColorMatrix",
        "feComponentTransfer",
        "feComposite",
        "feConvolveMatrix",
        "feDiffuseLighting",
        "feDisplacementMap",
        "feFlood",
        "feGaussianBlur",
        "feImage",
        "feMerge",
        "feMergeNode",
        "feMorphology",
        "feOffset",
        "feSpecularLighting",
        "feTile",
        "feTurbulence",
        // Animation elements
        "animate",
        "animateTransform",
        "animateMotion",
        "set",
        // Descriptive elements
        "desc",
        "title",
        "metadata",
        // Other elements
        "image",
        "stop",
        "script",
        "style",
    ]
    .into_iter()
    .collect()
});

// Common SVG attributes (simplified list)
static KNOWN_ATTRIBUTES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        // Core attributes
        "id",
        "class",
        "style",
        "lang",
        "tabindex",
        // Geometry attributes
        "x",
        "y",
        "x1",
        "y1",
        "x2",
        "y2",
        "cx",
        "cy",
        "r",
        "rx",
        "ry",
        "width",
        "height",
        "viewBox",
        "d",
        "points",
        "pathLength",
        // Presentation attributes
        "fill",
        "stroke",
        "stroke-width",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-opacity",
        "fill-opacity",
        "opacity",
        "color",
        "font-family",
        "font-size",
        "font-weight",
        "text-anchor",
        "visibility",
        "display",
        "overflow",
        "clip",
        "clip-path",
        "clip-rule",
        "mask",
        "filter",
        "transform",
        "transform-origin",
        // Animation attributes
        "begin",
        "dur",
        "end",
        "repeatCount",
        "repeatDur",
        "from",
        "to",
        "by",
        "values",
        "attributeName",
        "attributeType",
        "calcMode",
        "keyTimes",
        "keySplines",
        // Link attributes
        "href",
        "target",
        // Gradient attributes
        "gradientUnits",
        "gradientTransform",
        "x1",
        "y1",
        "x2",
        "y2",
        "fx",
        "fy",
        "spreadMethod",
        "stop-color",
        "stop-opacity",
        "offset",
        // Pattern attributes
        "patternUnits",
        "patternContentUnits",
        "patternTransform",
        // Filter attributes
        "filterUnits",
        "primitiveUnits",
        "result",
        "in",
        "in2",
        // Text attributes
        "text-rendering",
        "font-style",
        "font-variant",
        "text-decoration",
        "writing-mode",
        "glyph-orientation-vertical",
        "glyph-orientation-horizontal",
        // Other common attributes
        "preserveAspectRatio",
        "version",
        "xmlns",
        "xmlns:xlink",
        "xml:space",
    ]
    .into_iter()
    .collect()
});

// Default attribute values (simplified list)
static DEFAULT_ATTRIBUTE_VALUES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        [
            ("x", "0"),
            ("y", "0"),
            ("fill", "#000000"),
            ("fill", "black"),
            ("stroke", "none"),
            ("stroke-width", "1"),
            ("stroke-linecap", "butt"),
            ("stroke-linejoin", "miter"),
            ("stroke-dasharray", "none"),
            ("stroke-dashoffset", "0"),
            ("stroke-opacity", "1"),
            ("fill-opacity", "1"),
            ("opacity", "1"),
            ("visibility", "visible"),
            ("display", "inline"),
            ("overflow", "visible"),
            ("clip-rule", "nonzero"),
            ("font-size", "medium"),
            ("font-weight", "normal"),
            ("font-style", "normal"),
            ("text-anchor", "start"),
            ("text-decoration", "none"),
            ("preserveAspectRatio", "xMidYMid meet"),
            ("gradientUnits", "objectBoundingBox"),
            ("spreadMethod", "pad"),
            ("patternUnits", "objectBoundingBox"),
            ("patternContentUnits", "userSpaceOnUse"),
            ("filterUnits", "objectBoundingBox"),
            ("primitiveUnits", "userSpaceOnUse"),
        ]
        .into_iter()
        .collect()
    });

impl Plugin for RemoveUnknownsAndDefaultsPlugin {
    fn name(&self) -> &'static str {
        "removeUnknownsAndDefaults"
    }

    fn description(&self) -> &'static str {
        "removes unknown elements content and attributes, removes attrs with default values"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = RemoveUnknownsAndDefaultsParams::from_value(params);

        // Process XML declaration if needed
        if config.default_markup_declarations {
            process_xml_declaration(&mut document.metadata);
        }

        visit_elements(&mut document.root, &config, None);
        Ok(())
    }
}

/// Process XML declaration to remove default values
fn process_xml_declaration(metadata: &mut crate::ast::DocumentMetadata) {
    // Remove standalone="no" from version string if present
    if let Some(version) = &metadata.version {
        let cleaned = version
            .replace(" standalone=\"no\"", "")
            .replace(" standalone='no'", "");
        if cleaned != *version {
            metadata.version = Some(cleaned);
        }
    }
}

/// Visit all elements in the AST and remove unknowns and defaults
fn visit_elements(
    element: &mut Element,
    config: &RemoveUnknownsAndDefaultsParams,
    parent_element: Option<&Element>,
) {
    remove_unknown_and_default_attributes(element, config, parent_element);

    // First, collect indices of children to remove and process remaining children
    let mut children_to_remove = Vec::new();
    let mut child_elements_to_process = Vec::new();

    for (index, child) in element.children.iter().enumerate() {
        if let Node::Element(child_element) = child {
            // Check if this element should be removed
            if config.unknown_content && should_remove_unknown_element(child_element) {
                children_to_remove.push(index);
            } else if child_element.name != "foreignObject" {
                // Skip processing children of foreignObject
                child_elements_to_process.push(index);
            }
        }
    }

    // Remove unknown elements (in reverse order to maintain indices)
    for &index in children_to_remove.iter().rev() {
        element.children.remove(index);
    }

    // Now process the remaining child elements
    for &index in &child_elements_to_process {
        if let Some(Node::Element(child_element)) = element.children.get_mut(index) {
            // We need to collect parent attributes first to avoid borrowing issues
            let parent_attrs = element.attributes.clone();
            let parent_name = element.name.clone();

            // Create a temporary parent element for reference
            let mut temp_parent = Element::new(&parent_name);
            temp_parent.attributes = parent_attrs;

            visit_elements(child_element, config, Some(&temp_parent));
        }
    }
}

/// Check if an element should be removed as unknown
fn should_remove_unknown_element(element: &Element) -> bool {
    // Skip namespaced elements
    if element.name.contains(':') {
        return false;
    }

    // Check if it's a known SVG element
    !KNOWN_ELEMENTS.contains(element.name.as_str())
}

/// Remove unknown and default attributes from a single element
fn remove_unknown_and_default_attributes(
    element: &mut Element,
    config: &RemoveUnknownsAndDefaultsParams,
    parent_element: Option<&Element>,
) {
    let mut attrs_to_remove = Vec::new();

    for (attr_name, attr_value) in &element.attributes {
        let should_remove =
            should_remove_attribute(attr_name, attr_value, element, config, parent_element);

        if should_remove {
            attrs_to_remove.push(attr_name.clone());
        }
    }

    // Remove the attributes
    for attr_name in attrs_to_remove {
        element.attributes.shift_remove(&attr_name);
    }
}

/// Determine if an attribute should be removed
fn should_remove_attribute(
    attr_name: &str,
    attr_value: &str,
    element: &Element,
    config: &RemoveUnknownsAndDefaultsParams,
    parent_element: Option<&Element>,
) -> bool {
    // Keep data-* attributes if configured
    if config.keep_data_attrs && attr_name.starts_with("data-") {
        return false;
    }

    // Keep aria-* attributes if configured
    if config.keep_aria_attrs && attr_name.starts_with("aria-") {
        return false;
    }

    // Keep role attribute if configured
    if config.keep_role_attr && attr_name == "role" {
        return false;
    }

    // Always keep xmlns
    if attr_name == "xmlns" {
        return false;
    }

    // Keep namespaced attributes (xml:*, xlink:*)
    if attr_name.contains(':') {
        let prefix = attr_name.split(':').next().unwrap_or("");
        if prefix == "xml" || prefix == "xlink" {
            return false;
        }
    }

    // Check for unknown attributes
    if config.unknown_attrs && !is_known_attribute(attr_name) {
        return true;
    }

    // Check for default values (only if element doesn't have id)
    if config.default_attrs && !element.has_attr("id") && is_default_value(attr_name, attr_value) {
        return true;
    }

    // Check for useless overrides (simplified - only if element doesn't have id)
    if config.useless_overrides && !element.has_attr("id") {
        if let Some(parent) = parent_element {
            if let Some(parent_value) = parent.attr(attr_name) {
                if parent_value == attr_value {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if an attribute is known/valid
fn is_known_attribute(attr_name: &str) -> bool {
    KNOWN_ATTRIBUTES.contains(attr_name)
}

/// Check if an attribute value is the default value
fn is_default_value(attr_name: &str, attr_value: &str) -> bool {
    if let Some(&default_value) = DEFAULT_ATTRIBUTE_VALUES.get(attr_name) {
        default_value == attr_value
    } else {
        false
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use serde_json::json;

    #[test]
    fn test_removes_unknown_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("width".to_string(), "100".to_string());
        element
            .attributes
            .insert("unknown-attr".to_string(), "value".to_string());
        element
            .attributes
            .insert("height".to_string(), "50".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(document.root.has_attr("width"));
        assert!(document.root.has_attr("height"));
        assert!(!document.root.has_attr("unknown-attr"));
    }

    #[test]
    fn test_removes_default_values() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element.attributes.insert("x".to_string(), "0".to_string());
        element.attributes.insert("y".to_string(), "0".to_string());
        element
            .attributes
            .insert("width".to_string(), "100".to_string());
        element
            .attributes
            .insert("fill".to_string(), "black".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(!document.root.has_attr("x")); // default value removed
        assert!(!document.root.has_attr("y")); // default value removed
        assert!(document.root.has_attr("width")); // not default, kept
        assert!(!document.root.has_attr("fill")); // default value removed
    }

    #[test]
    fn test_preserves_data_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("data-test".to_string(), "value".to_string());
        element
            .attributes
            .insert("unknown-attr".to_string(), "value".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(document.root.has_attr("data-test")); // preserved
        assert!(!document.root.has_attr("unknown-attr")); // removed
    }

    #[test]
    fn test_preserves_aria_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("aria-label".to_string(), "test".to_string());
        element
            .attributes
            .insert("unknown-attr".to_string(), "value".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(document.root.has_attr("aria-label")); // preserved
        assert!(!document.root.has_attr("unknown-attr")); // removed
    }

    #[test]
    fn test_role_attribute_handling() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("role".to_string(), "button".to_string());

        document.root = element;

        // Test with keepRoleAttr = false (default)
        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();
        assert!(!document.root.has_attr("role")); // removed

        // Test with keepRoleAttr = true
        let mut document2 = Document::new();
        let mut element2 = Element::new("rect");
        element2
            .attributes
            .insert("role".to_string(), "button".to_string());
        document2.root = element2;

        let params = json!({"keepRoleAttr": true});
        plugin
            .apply(
                &mut document2,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();
        assert!(document2.root.has_attr("role")); // preserved
    }

    #[test]
    fn test_preserves_namespaced_attributes() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("xml:space".to_string(), "preserve".to_string());
        element
            .attributes
            .insert("xlink:href".to_string(), "#test".to_string());
        element.attributes.insert(
            "xmlns".to_string(),
            "http://www.w3.org/2000/svg".to_string(),
        );
        element
            .attributes
            .insert("unknown-attr".to_string(), "value".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(document.root.has_attr("xml:space")); // preserved
        assert!(document.root.has_attr("xlink:href")); // preserved
        assert!(document.root.has_attr("xmlns")); // preserved
        assert!(!document.root.has_attr("unknown-attr")); // removed
    }

    #[test]
    fn test_removes_unknown_elements() {
        let mut document = Document::new();
        let known_child = Element::new("rect");
        let unknown_child = Element::new("unknown-element");

        document.root.children.push(Node::Element(known_child));
        document.root.children.push(Node::Element(unknown_child));

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert_eq!(document.root.children.len(), 1);
        if let Node::Element(remaining_child) = &document.root.children[0] {
            assert_eq!(remaining_child.name, "rect");
        }
    }

    #[test]
    fn test_preserves_id_elements_from_default_removal() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("id".to_string(), "test".to_string());
        element.attributes.insert("x".to_string(), "0".to_string()); // default value
        element
            .attributes
            .insert("fill".to_string(), "black".to_string()); // default value

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        assert!(document.root.has_attr("id"));
        assert!(document.root.has_attr("x")); // preserved because element has id
        assert!(document.root.has_attr("fill")); // preserved because element has id
    }

    #[test]
    fn test_configuration_options() {
        let mut document = Document::new();
        let mut element = Element::new("rect");

        element
            .attributes
            .insert("unknown-attr".to_string(), "value".to_string());
        element.attributes.insert("x".to_string(), "0".to_string());

        document.root = element;

        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        let params = json!({
            "unknownAttrs": false,
            "defaultAttrs": false
        });
        plugin
            .apply(
                &mut document,
                &crate::plugin::PluginInfo::default(),
                Some(&params),
            )
            .unwrap();

        // Both should be preserved due to config
        assert!(document.root.has_attr("unknown-attr"));
        assert!(document.root.has_attr("x"));
    }

    #[test]
    fn test_plugin_name_and_description() {
        let mut plugin = RemoveUnknownsAndDefaultsPlugin;
        assert_eq!(plugin.name(), "removeUnknownsAndDefaults");
        assert_eq!(
            plugin.description(),
            "removes unknown elements content and attributes, removes attrs with default values"
        );
    }
}
