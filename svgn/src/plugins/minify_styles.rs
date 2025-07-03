// this_file: svgn/src/plugins/minify_styles.rs

//! Minify CSS in style elements and style attributes
//!
//! This plugin performs basic CSS minification by removing whitespace,
//! comments, and unnecessary semicolons from CSS content.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

// Regular expressions for CSS minification
static CSS_COMMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/").unwrap());

static CSS_WHITESPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

static CSS_SEMICOLON_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";\s*}").unwrap());

static CSS_TRAILING_SEMICOLON_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r";\s*$").unwrap());

static CSS_COLON_SPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r":\s+").unwrap());

static CSS_BRACKET_SPACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*([{}:;,>+~])\s*").unwrap());

/// Plugin that minifies CSS in style elements and style attributes
pub struct MinifyStylesPlugin;

impl Plugin for MinifyStylesPlugin {
    fn name(&self) -> &'static str {
        "minifyStyles"
    }

    fn description(&self) -> &'static str {
        "Minifies CSS in style elements and style attributes"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let remove_comments = params
            .and_then(|v| v.get("comments"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Process root element
        minify_styles_in_element(&mut document.root, remove_comments);

        Ok(())
    }
}

/// Recursively minify styles in an element and its children
fn minify_styles_in_element(element: &mut Element, remove_comments: bool) {
    // Process child elements first
    for child in &mut element.children {
        if let Node::Element(child_element) = child {
            minify_styles_in_element(child_element, remove_comments);
        }
    }

    // Minify style elements
    if element.name == "style" {
        minify_style_element_content(element, remove_comments);
    }

    // Minify style attributes
    if element.has_attr("style") {
        if let Some(style_value) = element.attr("style") {
            let minified = minify_css_block(style_value, remove_comments);
            element.set_attr("style".to_string(), minified);
        }
    }
}

/// Minify CSS content in a style element
fn minify_style_element_content(element: &mut Element, remove_comments: bool) {
    let mut modified = false;

    for child in &mut element.children {
        match child {
            Node::Text(text) => {
                let minified = minify_css(text, remove_comments);
                if minified != *text {
                    *text = minified;
                    modified = true;
                }
            }
            Node::CData(cdata) => {
                let minified = minify_css(cdata, remove_comments);
                if minified != *cdata {
                    *cdata = minified;
                    modified = true;
                }
            }
            _ => {}
        }
    }

    // If all CSS content is empty after minification, mark for removal
    if modified {
        let all_empty = element.children.iter().all(|child| match child {
            Node::Text(text) => text.trim().is_empty(),
            Node::CData(cdata) => cdata.trim().is_empty(),
            _ => true,
        });

        if all_empty {
            // Clear the element by removing all children
            element.clear_children();
        }
    }
}

/// Perform basic CSS minification
fn minify_css(css: &str, remove_comments: bool) -> String {
    let mut result = css.to_string();

    // Remove comments if requested
    if remove_comments {
        result = CSS_COMMENT_REGEX.replace_all(&result, "").to_string();
    }

    // Normalize whitespace
    result = CSS_WHITESPACE_REGEX.replace_all(&result, " ").to_string();

    // Remove space around special characters
    result = CSS_BRACKET_SPACE_REGEX
        .replace_all(&result, "$1")
        .to_string();

    // Remove space after colons
    result = CSS_COLON_SPACE_REGEX.replace_all(&result, ":").to_string();

    // Remove unnecessary semicolons before closing braces
    result = CSS_SEMICOLON_REGEX.replace_all(&result, "}").to_string();

    // Remove trailing semicolons
    result = CSS_TRAILING_SEMICOLON_REGEX
        .replace_all(&result, "")
        .to_string();

    // Trim leading and trailing whitespace
    result.trim().to_string()
}

/// Minify a CSS block (for style attributes)
fn minify_css_block(css: &str, remove_comments: bool) -> String {
    let mut result = minify_css(css, remove_comments);

    // For style attributes, ensure there's no trailing semicolon
    if result.ends_with(';') {
        result.pop();
    }

    result
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

    fn create_style_element(css: &str) -> Element {
        let mut element = create_element("style", vec![]);
        element.add_child(Node::Text(css.to_string()));
        element
    }

    #[test]
    fn test_minify_css_basic() {
        let css = "  body  {  margin : 0 ;  padding : 10px  ;  }  ";
        let expected = "body{margin:0;padding:10px}";
        assert_eq!(minify_css(css, true), expected);
    }

    #[test]
    fn test_minify_css_with_comments() {
        let css = "/* comment */ body { margin: 0; /* another comment */ }";
        let expected = "body{margin:0}";
        assert_eq!(minify_css(css, true), expected);
    }

    #[test]
    fn test_minify_css_preserve_comments() {
        let css = "/* comment */ body { margin: 0; }";
        let result = minify_css(css, false);
        assert!(result.contains("/* comment */"));
        assert!(result.contains("body{margin:0}"));
    }

    #[test]
    fn test_minify_css_block() {
        let css = "margin: 10px; padding: 5px;";
        let expected = "margin:10px;padding:5px";
        assert_eq!(minify_css_block(css, true), expected);
    }

    #[test]
    fn test_minify_css_block_trailing_semicolon() {
        let css = "margin: 10px;";
        let expected = "margin:10px";
        assert_eq!(minify_css_block(css, true), expected);
    }

    #[test]
    fn test_minify_style_element() {
        let mut element = create_style_element("  body  {  margin : 0 ;  }  ");
        minify_style_element_content(&mut element, true);

        if let Some(Node::Text(text)) = element.children.first() {
            assert_eq!(text, "body{margin:0}");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_empty_style_element_removal() {
        let mut element = create_style_element("/* only comments */");
        minify_style_element_content(&mut element, true);

        // Should be empty after removing comments
        assert!(element.children.is_empty());
    }

    #[test]
    fn test_style_attribute_minification() {
        let mut element = create_element("rect", vec![("style", "margin: 10px; padding: 5px;")]);

        minify_styles_in_element(&mut element, true);

        assert_eq!(element.attr("style").unwrap(), "margin:10px;padding:5px");
    }

    #[test]
    fn test_complex_css_minification() {
        let css = r#"
            .class1 { 
                color: red; 
                font-size: 12px; 
            }
            .class2 { 
                background: blue; 
            }
        "#;
        let expected = ".class1{color:red;font-size:12px}.class2{background:blue}";
        assert_eq!(minify_css(css, true), expected);
    }

    #[test]
    fn test_preserve_important() {
        let css = "color: red !important; margin: 0;";
        let result = minify_css(css, true);
        assert!(result.contains("!important"));
        // Note: spaces around !important are preserved (this is correct behavior)
        // The trailing semicolon is removed because it's the end of the declaration
        assert_eq!(result, "color:red !important;margin:0");
    }
}
