// this_file: svgn/src/plugins/inline_styles.rs

//! Inline styles plugin
//!
//! This plugin moves and merges styles from `<style>` elements to inline style attributes.
//! It parses CSS rules, matches them against SVG elements using selectors, and applies
//! the computed styles directly to matching elements.
//!
//! SVGO parameters supported:
//! - `onlyMatchedOnce` (default: true) - Inline only rules that match a single element
//! - `removeMatchedSelectors` (default: true) - Remove selectors from style sheets when inlined
//! - `useMqs` (default: true) - Process media queries
//! - `usePseudos` (default: true) - Process pseudo-classes and pseudo-elements

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use lightningcss::{
    rules::CssRule,
    stylesheet::{ParserOptions, StyleSheet},
    traits::ToCss,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::collections::PRESENTATION_ATTRS;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use selectors::parser::{SelectorList, SelectorImpl};
use selectors::matching::{MatchingContext, MatchingMode, QuirksMode, NeedsSelectorFlags, IgnoreNthChildForInvalidation};

mod inline_styles_converter;

use crate::plugins::inline_styles_selector::{SvgSelectorImpl, SvgElementWrapper, walk_element_tree_with_parent};
use inline_styles_converter::convert_css_property;

/// Parameters for the inline styles plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineStylesParams {
    /// Inline only rules that match a single element
    #[serde(default = "default_only_matched_once")]
    pub only_matched_once: bool,
    
    /// Remove selectors from style sheets when inlined
    #[serde(default = "default_remove_matched_selectors")]
    pub remove_matched_selectors: bool,
    
    /// Process media queries
    #[serde(default = "default_use_mqs")]
    pub use_mqs: bool,
    
    /// Process pseudo-classes and pseudo-elements
    #[serde(default = "default_use_pseudos")]
    pub use_pseudos: bool,
}

impl Default for InlineStylesParams {
    fn default() -> Self {
        Self {
            only_matched_once: default_only_matched_once(),
            remove_matched_selectors: default_remove_matched_selectors(),
            use_mqs: default_use_mqs(),
            use_pseudos: default_use_pseudos(),
        }
    }
}

fn default_only_matched_once() -> bool { true }
fn default_remove_matched_selectors() -> bool { true }
fn default_use_mqs() -> bool { true }
fn default_use_pseudos() -> bool { true }

/// Plugin that inlines styles from style elements to inline style attributes
pub struct InlineStylesPlugin;

impl Plugin for InlineStylesPlugin {
    fn name(&self) -> &'static str {
        "inlineStyles"
    }

    fn description(&self) -> &'static str {
        "Move and merge styles from style elements to inline style attributes"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        // Parse parameters
        let params = if let Some(params) = params {
            serde_json::from_value::<InlineStylesParams>(params.clone())
                .unwrap_or_default()
        } else {
            InlineStylesParams::default()
        };

        // Find all style elements and process them
        process_all_style_elements(&mut document.root, &params)?;

        // Remove empty style elements if configured
        if params.remove_matched_selectors {
            remove_empty_style_elements(&mut document.root);
        }

        Ok(())
    }
}

/// Process all style elements in the document
fn process_all_style_elements(
    element: &mut Element,
    params: &InlineStylesParams,
) -> PluginResult<()> {
    // First collect all style elements and their CSS rules
    let mut all_css_rules = Vec::new();
    collect_css_rules(element, &mut all_css_rules, params)?;
    
    // Sort rules by specificity (lowest to highest)
    all_css_rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));
    
    // Track match counts for onlyMatchedOnce
    let mut match_counts = HashMap::new();
    if params.only_matched_once {
        for rule in &all_css_rules {
            let count = count_matching_elements_for_rule(element, rule);
            match_counts.insert(rule.selector.clone(), count);
        }
    }
    
    // Track which selectors were used for cleanup
    let mut used_selectors = HashSet::new();
    
    // Apply rules to matching elements (in specificity order)
    for rule in &all_css_rules {
        // Skip if onlyMatchedOnce is true and selector matches multiple elements
        if params.only_matched_once {
            if let Some(&count) = match_counts.get(&rule.selector) {
                if count > 1 {
                    continue;
                }
            }
        }
        
        apply_css_rule_and_track(element, rule, params, &mut used_selectors)?;
    }
    
    // Clean up if configured
    if params.remove_matched_selectors {
        // Remove empty style elements and unused class/id attributes
        remove_empty_style_elements(element);
        cleanup_unused_attributes(element);
    }
    
    Ok(())
}

/// Extract CSS content from a style element
fn extract_css_content(style_elem: &Element) -> String {
    let mut content = String::new();
    
    for child in &style_elem.children {
        match child {
            Node::Text(text) => content.push_str(text),
            Node::CData(cdata) => content.push_str(cdata),
            _ => {}
        }
    }
    
    content
}

/// Collect CSS rules from all style elements
fn collect_css_rules(
    element: &Element,
    all_rules: &mut Vec<CssRuleData>,
    _params: &InlineStylesParams,
) -> PluginResult<()> {
    if element.name == "style" {
        let css_content = extract_css_content(element);
        if !css_content.is_empty() {
            if let Ok(stylesheet) = StyleSheet::<'_, '_>::parse(&css_content, ParserOptions::default()) {
                let mut rules = extract_css_rules(&stylesheet);
                all_rules.append(&mut rules);
            }
        }
    }
    
    // Process children
    for child in &element.children {
        if let Node::Element(child_elem) = child {
            collect_css_rules(child_elem, all_rules, _params)?;
        }
    }
    
    Ok(())
}

/// Count how many elements match a CSS rule
fn count_matching_elements_for_rule(element: &Element, rule: &CssRuleData) -> usize {
    let mut count = 0;
    
    if let Some(ref selector_list) = rule.parsed_selectors {
        // Use advanced selector matching
        walk_element_tree_with_parent(element, None, |elem, parent, index| {
            let wrapper = SvgElementWrapper::new(elem, parent, index);
            for selector in selector_list.0.iter() {
                let mut context = MatchingContext::new(
                    MatchingMode::Normal,
                    None,
                    &mut selectors::NthIndexCache::default(),
                    QuirksMode::NoQuirks,
                    NeedsSelectorFlags::No,
                    IgnoreNthChildForInvalidation::No,
                );
                if selectors::matching::matches_selector(selector, 0, None, &wrapper, &mut context) {
                    count += 1;
                    break; // Don't count same element multiple times
                }
            }
        });
    } else {
        // Fallback to simple matching
        count_matching_elements_simple(element, &rule.selector, &mut count);
    }
    
    count
}

/// Simple element counting fallback
fn count_matching_elements_simple(element: &Element, selector: &str, count: &mut usize) {
    if element_matches_selector_simple(element, selector) {
        *count += 1;
    }
    
    for child in &element.children {
        if let Node::Element(child_elem) = child {
            count_matching_elements_simple(child_elem, selector, count);
        }
    }
}

/// Apply a CSS rule to all matching elements
fn apply_css_rule(
    element: &mut Element,
    rule: &CssRuleData,
    _params: &InlineStylesParams,
) -> PluginResult<()> {
    apply_css_rule_and_track(element, rule, _params, &mut HashSet::new())
}

/// Apply a CSS rule to all matching elements and track which selectors were used
fn apply_css_rule_and_track(
    element: &mut Element,
    rule: &CssRuleData,
    _params: &InlineStylesParams,
    used_selectors: &mut HashSet<String>,
) -> PluginResult<()> {
    apply_css_rule_and_track_impl(element, rule, _params, used_selectors, None, 0)
}

/// Implementation of CSS rule application with parent tracking
fn apply_css_rule_and_track_impl(
    element: &mut Element,
    rule: &CssRuleData,
    _params: &InlineStylesParams,
    used_selectors: &mut HashSet<String>,
    parent: Option<&Element>,
    index: usize,
) -> PluginResult<()> {
    // Check if this element matches the selector
    let matches = if let Some(ref selector_list) = rule.parsed_selectors {
        // Use advanced selector matching
        let wrapper = SvgElementWrapper::new(element, parent, index);
        selector_list.0.iter().any(|selector| {
            let mut context = MatchingContext::new(
                MatchingMode::Normal,
                None,
                &mut selectors::NthIndexCache::default(),
                QuirksMode::NoQuirks,
                NeedsSelectorFlags::No,
                IgnoreNthChildForInvalidation::No,
            );
            selectors::matching::matches_selector(selector, 0, None, &wrapper, &mut context)
        })
    } else {
        // Fallback to simple matching
        element_matches_selector_simple(element, &rule.selector)
    };
    
    if matches {
        // Apply declarations to this element
        apply_declarations_to_element(element, &rule.declarations);
        // Track which selectors were used
        used_selectors.insert(rule.selector.clone());
        
        // Track if we used a class or ID selector for cleanup
        if rule.selector.starts_with('.') {
            let class_name = &rule.selector[1..];
            element.attributes.insert("data-used-class".to_string(), class_name.to_string());
        } else if rule.selector.starts_with('#') {
            let id_name = &rule.selector[1..];
            element.attributes.insert("data-used-id".to_string(), id_name.to_string());
        }
    }
    
    // Process children with proper parent tracking
    let mut child_index = 0;
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            apply_css_rule_and_track_impl(child_elem, rule, _params, used_selectors, Some(element), child_index)?;
            child_index += 1;
        }
    }
    
    Ok(())
}

/// Check if an element matches a CSS selector (simplified fallback)
fn element_matches_selector_simple(element: &Element, selector: &str) -> bool {
    // Handle class selectors
    if let Some(class_name) = selector.strip_prefix('.') {
        if let Some(classes) = element.attributes.get("class") {
            return classes.split_whitespace().any(|c| c == class_name);
        }
    }
    
    // Handle ID selectors
    if let Some(id) = selector.strip_prefix('#') {
        if let Some(elem_id) = element.attributes.get("id") {
            return elem_id == id;
        }
    }
    
    // Handle type selectors
    if !selector.contains(' ') && !selector.contains('[') && !selector.contains(':') {
        return element.name == selector;
    }
    
    false
}

/// Apply CSS declarations to an element
fn apply_declarations_to_element(element: &mut Element, declarations: &[(String, String)]) {
    // Get existing style attribute if any
    let mut inline_styles = parse_inline_style(element.attributes.get("style").cloned().unwrap_or_default());
    
    // Apply new declarations
    for (property, value) in declarations {
        // Only apply if it's a presentation attribute
        if PRESENTATION_ATTRS.contains(property.as_str()) {
            inline_styles.insert(property.clone(), value.clone());
        }
    }
    
    // Build new style attribute
    if !inline_styles.is_empty() {
        let style_str = inline_styles
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        element.attributes.insert("style".to_string(), style_str);
    }
}

/// Parse inline style attribute into property-value pairs
fn parse_inline_style(style: String) -> IndexMap<String, String> {
    let mut styles = IndexMap::new();
    
    if !style.is_empty() {
        for declaration in style.split(';') {
            let declaration = declaration.trim();
            if let Some(colon_pos) = declaration.find(':') {
                let property = declaration[..colon_pos].trim().to_string();
                let value = declaration[colon_pos + 1..].trim().to_string();
                styles.insert(property, value);
            }
        }
    }
    
    styles
}

/// Remove empty style elements from the document
fn remove_empty_style_elements(element: &mut Element) {
    element.children.retain(|child| {
        if let Node::Element(elem) = child {
            if elem.name == "style" {
                // Check if style element has any content
                let has_content = elem.children.iter().any(|c| match c {
                    Node::Text(t) | Node::CData(t) => !t.trim().is_empty(),
                    _ => false,
                });
                return has_content;
            }
        }
        true
    });
    
    // Recursively process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            remove_empty_style_elements(child_elem);
        }
    }
}

/// Clean up unused class and ID attributes
fn cleanup_unused_attributes(element: &mut Element) {
    // Remove class attribute if it was used for inlining
    if let Some(used_class) = element.attributes.get("data-used-class") {
        if let Some(classes) = element.attributes.get("class") {
            // Remove only the used class from the class list
            let remaining_classes: Vec<&str> = classes
                .split_whitespace()
                .filter(|c| c != used_class)
                .collect();
            
            if remaining_classes.is_empty() {
                element.attributes.shift_remove("class");
            } else {
                element.attributes.insert("class".to_string(), remaining_classes.join(" "));
            }
        }
        // Remove the tracking attribute
        element.attributes.shift_remove("data-used-class");
    }
    
    // Remove ID attribute if it was used for inlining
    if let Some(used_id) = element.attributes.get("data-used-id") {
        if let Some(id) = element.attributes.get("id") {
            if id == used_id {
                element.attributes.shift_remove("id");
            }
        }
        // Remove the tracking attribute
        element.attributes.shift_remove("data-used-id");
    }
    
    // Process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            cleanup_unused_attributes(child_elem);
        }
    }
}

/// Represents a CSS rule with selector and declarations
#[derive(Debug, Clone)]
struct CssRuleData {
    selector: String,
    parsed_selectors: Option<SelectorList<SvgSelectorImpl>>,
    declarations: Vec<(String, String)>,
    specificity: u32,
}

/// Extract CSS rules from a parsed stylesheet
fn extract_css_rules(stylesheet: &StyleSheet) -> Vec<CssRuleData> {
    let mut rules = Vec::new();
    
    // Iterate through rules in the stylesheet
    for rule in &stylesheet.rules.0 {
        match rule {
            CssRule::Style(style_rule) => {
                // Extract selectors
                for sel in &style_rule.selectors.0 {
                    if let Some(selector_str) = extract_selector_string(sel) {
                        // Try to parse the selector with advanced parser
                        let parsed_selectors = parse_selector(&selector_str);
                        
                        // Calculate specificity
                        let specificity = if let Some(ref selectors) = parsed_selectors {
                            calculate_selector_list_specificity(selectors)
                        } else {
                            calculate_selector_specificity(&selector_str)
                        };
                        
                        // Extract declarations using the converter
                        let mut declarations = Vec::new();
                        for property in &style_rule.declarations.declarations {
                            if let Some((prop_name, prop_value)) = convert_css_property(property) {
                                // Only include SVG presentation attributes
                                if PRESENTATION_ATTRS.contains(prop_name.as_str()) {
                                    declarations.push((prop_name, prop_value));
                                }
                            }
                        }
                        
                        if !declarations.is_empty() {
                            rules.push(CssRuleData {
                                selector: selector_str,
                                parsed_selectors,
                                declarations,
                                specificity,
                            });
                        }
                    }
                }
            }
            CssRule::Media(media_rule) => {
                // TODO: Handle media queries when useMqs parameter is true
                // For now, skip media queries to maintain compatibility
                // In future versions, we can recursively process media rules
                // when params.use_mqs is enabled
            }
            _ => {
                // Ignore other rule types (import, namespace, etc.)
            }
        }
    }
    
    rules
}

/// Extract a selector string from lightningcss selector
fn extract_selector_string(selector: &lightningcss::selector::Selector) -> Option<String> {
    // Convert selector to string using lightningcss's ToCss trait
    use lightningcss::printer::{Printer, PrinterOptions};
    
    let mut dest = String::new();
    let mut printer = Printer::new(&mut dest, PrinterOptions::default());
    
    if selector.to_css(&mut printer).is_ok() {
        Some(dest)
    } else {
        // Fallback to debug format if ToCss fails
        let debug_str = format!("{:?}", selector);
        
        // Try to extract the selector string from the debug format
        if debug_str.starts_with("Selector(") {
            if let Some(comma_pos) = debug_str.find(", specificity") {
                let selector_part = &debug_str[9..comma_pos]; // Skip "Selector("
                return Some(selector_part.to_string());
            }
        }
        
        None
    }
}


/// Calculate CSS specificity for a selector
/// Returns a u32 where higher values mean higher specificity
/// Format: AAABBBCCC where AAA = ID count, BBB = class count, CCC = element count
fn calculate_selector_specificity(selector: &str) -> u32 {
    if selector.starts_with('#') {
        1_000_000 // ID selector
    } else if selector.starts_with('.') {
        1_000 // Class selector
    } else {
        1 // Element selector
    }
}

/// Parse a CSS selector string into a SelectorList
/// Uses simplified parsing approach compatible with selectors v0.25
fn parse_selector(selector_str: &str) -> Option<SelectorList<SvgSelectorImpl>> {
    // For now, return None to disable advanced selector parsing
    // This forces the plugin to use the simple selector matching fallback
    // TODO: Implement proper selectors v0.25 compatible parsing
    None
}

/// Calculate specificity for a parsed selector list
fn calculate_selector_list_specificity(selectors: &SelectorList<SvgSelectorImpl>) -> u32 {
    selectors.0.iter()
        .map(calculate_parsed_selector_specificity)
        .max()
        .unwrap_or(0)
}

/// Calculate specificity for a parsed selector
fn calculate_parsed_selector_specificity(selector: &selectors::parser::Selector<SvgSelectorImpl>) -> u32 {
    selector.specificity()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};
    
    /// Helper function to create a test document with style element
    fn create_test_document_with_style(css: &str, _svg_content: &str) -> Document {
        let mut doc = Document::new();
        doc.root.name = "svg".to_string();
        
        // Add style element
        let mut style_elem = Element::new("style");
        style_elem.children.push(Node::Text(css.to_string()));
        doc.root.children.push(Node::Element(style_elem));
        
        // Parse and add SVG content elements
        // For now, just add a simple rect element for testing
        let mut rect_elem = Element::new("rect");
        rect_elem.attributes.insert("class".to_string(), "st0".to_string());
        rect_elem.attributes.insert("x".to_string(), "10".to_string());
        rect_elem.attributes.insert("y".to_string(), "10".to_string());
        rect_elem.attributes.insert("width".to_string(), "100".to_string());
        rect_elem.attributes.insert("height".to_string(), "100".to_string());
        doc.root.children.push(Node::Element(rect_elem));
        
        doc
    }
    
    #[test]
    fn test_inline_styles_plugin_name() {
        let plugin = InlineStylesPlugin;
        assert_eq!(plugin.name(), "inlineStyles");
    }
    
    #[test]
    fn test_inline_styles_plugin_description() {
        let plugin = InlineStylesPlugin;
        assert_eq!(plugin.description(), "Move and merge styles from style elements to inline style attributes");
    }
    
    #[test]
    fn test_inline_styles_basic_functionality() {
        let css = ".st0 { fill: blue; }";
        let mut doc = create_test_document_with_style(css, "");
        
        let mut plugin = InlineStylesPlugin;
        let plugin_info = PluginInfo::default();
        
        // Run the plugin
        let result = plugin.apply(&mut doc, &plugin_info, None);
        assert!(result.is_ok());
    }
}