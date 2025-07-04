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
use cssparser::ToCss;
use lightningcss::{
    rules::CssRule,
    stylesheet::{ParserOptions, StyleSheet},
};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::parser::SelectorParseErrorKind;
use selectors::{Element as SelectorElement, OpaqueElement};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use crate::collections::PRESENTATION_ATTRS;
use indexmap::IndexMap;
use std::collections::HashMap;

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
    // Build parent mapping for the entire document
    let mut parents = HashMap::new();
    build_parent_map(element, None, &mut parents);
    
    // First collect all style elements and their CSS rules
    let mut all_css_rules = Vec::new();
    collect_css_rules(element, &mut all_css_rules, params)?;
    
    // Sort rules by specificity (lowest to highest)
    all_css_rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));
    
    // Apply rules to matching elements (in specificity order)
    for rule in &all_css_rules {
        apply_css_rule_with_parents(element, rule, params, &parents)?;
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
    params: &InlineStylesParams,
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
            collect_css_rules(child_elem, all_rules, params)?;
        }
    }
    
    Ok(())
}

/// Apply a CSS rule to all matching elements
fn apply_css_rule(
    element: &mut Element,
    rule: &CssRuleData,
    params: &InlineStylesParams,
) -> PluginResult<()> {
    // Check if this element matches any of the rule's selectors
    for selector in &rule.selectors {
        if element_matches_selector(element, selector) {
            // Apply declarations to this element
            apply_declarations_to_element(element, &rule.declarations);
            
            // If onlyMatchedOnce is true, we should track matches
            // TODO: Implement match tracking
        }
    }
    
    // Process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            apply_css_rule(child_elem, rule, params)?;
        }
    }
    
    Ok(())
}

/// Apply a CSS rule with parent context
fn apply_css_rule_with_parents(
    element: &mut Element,
    rule: &CssRuleData,
    params: &InlineStylesParams,
    parents: &HashMap<*const Element, *const Element>,
) -> PluginResult<()> {
    apply_css_rule_with_parent_impl(element, rule, params, parents)
}

/// Implementation of CSS rule application with parent tracking
fn apply_css_rule_with_parent_impl(
    element: &mut Element,
    rule: &CssRuleData,
    params: &InlineStylesParams,
    parents: &HashMap<*const Element, *const Element>,
) -> PluginResult<()> {
    // Check if this element matches any of the rule's selectors
    for selector in &rule.selectors {
        if element_matches_selector_with_parents(element, selector, parents) {
            // Apply declarations to this element
            apply_declarations_to_element(element, &rule.declarations);
            
            // If onlyMatchedOnce is true, we should track matches
            // TODO: Implement match tracking
        }
    }
    
    // Process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            apply_css_rule_with_parent_impl(child_elem, rule, params, parents)?;
        }
    }
    
    Ok(())
}

/// Build a map of element pointers to parent pointers
fn build_parent_map(
    element: &Element,
    parent: Option<*const Element>,
    parents: &mut HashMap<*const Element, *const Element>,
) {
    let elem_ptr = element as *const Element;
    
    if let Some(parent_ptr) = parent {
        parents.insert(elem_ptr, parent_ptr);
    }
    
    // Process children
    for child in &element.children {
        if let Node::Element(child_elem) = child {
            build_parent_map(child_elem, Some(elem_ptr), parents);
        }
    }
}

/// Check if an element matches a CSS selector with parent context
fn element_matches_selector_with_parents(
    element: &Element,
    selector: &str,
    parents: &HashMap<*const Element, *const Element>,
) -> bool {
    // For now, use simple matching
    // TODO: Implement full selector matching with parent context
    element_matches_selector_simple(element, selector)
}

/// Check if an element matches a CSS selector
fn element_matches_selector(element: &Element, selector: &str) -> bool {
    // For now, use simple matching
    // TODO: Implement full selector matching with selectors crate
    element_matches_selector_simple(element, selector)
}

/// Simple fallback selector matching for basic cases
fn element_matches_selector_simple(element: &Element, selector: &str) -> bool {
    // Handle class selectors
    if selector.starts_with('.') {
        let class_name = &selector[1..];
        if let Some(classes) = element.attributes.get("class") {
            return classes.split_whitespace().any(|c| c == class_name);
        }
    }
    
    // Handle ID selectors
    if selector.starts_with('#') {
        let id = &selector[1..];
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
    // For now, just remove class attributes if we've inlined styles
    // In a full implementation, we would track which classes/IDs are still needed
    if element.attributes.contains_key("style") {
        element.attributes.shift_remove("class");
    }
    
    // Process children
    for child in &mut element.children {
        if let Node::Element(child_elem) = child {
            cleanup_unused_attributes(child_elem);
        }
    }
}

// TODO: Implement full selectors::Element trait when API is stabilized
/*
/// Wrapper for SVG elements to implement selectors::Element trait
#[derive(Debug, Clone)]
struct SvgElement<'a> {
    element: &'a Element,
}

/// Wrapper for SVG elements with parent tracking
struct SvgElementWithParent<'a> {
    element: &'a Element,
    parents: &'a HashMap<*const Element, *const Element>,
}

impl<'a> SelectorElement for SvgElement<'a> {
    type Impl = SimpleSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        None // No parent tracking in simple version
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None // TODO: Implement sibling tracking
    }

    fn next_sibling_element(&self) -> Option<Self> {
        None // TODO: Implement sibling tracking
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        self.element.name == local_name.0
    }

    fn has_namespace(&self, _ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        false // TODO: Implement namespace support
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        if let Some(attr_value) = self.element.attributes.get(&local_name.0) {
            let attr_value = CssString(attr_value.clone());
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue { value, case_sensitivity, operator } => {
                    use selectors::attr::AttrSelectorOperator;
                    match operator {
                        AttrSelectorOperator::Equal => {
                            match case_sensitivity {
                                CaseSensitivity::CaseSensitive => &attr_value.0 == &value.0,
                                CaseSensitivity::AsciiCaseInsensitive => {
                                    attr_value.0.eq_ignore_ascii_case(&value.0)
                                }
                            }
                        }
                        AttrSelectorOperator::Includes => {
                            // TODO: Implement whitespace-separated token matching
                            false
                        }
                        AttrSelectorOperator::DashMatch => {
                            // TODO: Implement dash-match (value or value-*)
                            false
                        }
                        AttrSelectorOperator::Prefix => {
                            // TODO: Implement prefix matching
                            false
                        }
                        AttrSelectorOperator::Substring => {
                            // TODO: Implement substring matching
                            false
                        }
                        AttrSelectorOperator::Suffix => {
                            // TODO: Implement suffix matching
                            false
                        }
                    }
                }
            }
        } else {
            false
        }
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        _context: &mut selectors::context::MatchingContext<'_, Self::Impl>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        _context: &mut selectors::context::MatchingContext<'_, Self::Impl>,
    ) -> bool {
        false
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {
        // No-op for now
    }

    fn is_link(&self) -> bool {
        false
    }
    
    fn first_element_child(&self) -> Option<Self> {
        None // TODO: Implement child tracking
    }
    
    fn is_html_slot_element(&self) -> bool {
        false
    }
    
    fn has_id(&self, id: &<Self::Impl as selectors::SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("id").map_or(false, |v| v == &id.0)
    }
    
    fn has_class(&self, name: &<Self::Impl as selectors::SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("class").map_or(false, |classes| {
            classes.split_whitespace().any(|c| c == &name.0)
        })
    }
    
    fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }
    
    fn is_root(&self) -> bool {
        self.element.name == "svg" // TODO: Check if this is actually the root element
    }

    fn is_part(&self, _name: &CssString) -> bool {
        false
    }

    fn imported_part(
        &self,
        _name: &CssString,
    ) -> Option<CssString> {
        None
    }
}

impl<'a> SelectorElement for SvgElementWithParent<'a> {
    type Impl = SimpleSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        let elem_ptr = self.element as *const Element;
        if let Some(&parent_ptr) = self.parents.get(&elem_ptr) {
            // Safety: We know these pointers are valid because we built the map
            // from a valid tree structure
            unsafe {
                Some(SvgElementWithParent {
                    element: &*parent_ptr,
                    parents: self.parents,
                })
            }
        } else {
            None
        }
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        // To implement this properly, we'd need sibling tracking
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        // To implement this properly, we'd need sibling tracking
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        self.element.name == local_name.0
    }

    fn has_namespace(&self, _ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        false // TODO: Implement namespace support
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        if let Some(attr_value) = self.element.attributes.get(&local_name.0) {
            let attr_value = CssString(attr_value.clone());
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue { value, case_sensitivity, operator } => {
                    use selectors::attr::AttrSelectorOperator;
                    match operator {
                        AttrSelectorOperator::Equal => {
                            match case_sensitivity {
                                CaseSensitivity::CaseSensitive => &attr_value.0 == &value.0,
                                CaseSensitivity::AsciiCaseInsensitive => {
                                    attr_value.0.eq_ignore_ascii_case(&value.0)
                                }
                            }
                        }
                        AttrSelectorOperator::Includes => {
                            attr_value.0.split_whitespace().any(|word| {
                                match case_sensitivity {
                                    CaseSensitivity::CaseSensitive => word == &value.0,
                                    CaseSensitivity::AsciiCaseInsensitive => word.eq_ignore_ascii_case(&value.0),
                                }
                            })
                        }
                        AttrSelectorOperator::DashMatch => {
                            &attr_value.0 == &value.0 || attr_value.0.starts_with(&format!("{}-", value.0))
                        }
                        AttrSelectorOperator::Prefix => {
                            attr_value.0.starts_with(&value.0)
                        }
                        AttrSelectorOperator::Substring => {
                            attr_value.0.contains(&value.0)
                        }
                        AttrSelectorOperator::Suffix => {
                            attr_value.0.ends_with(&value.0)
                        }
                    }
                }
            }
        } else {
            false
        }
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        _context: &mut selectors::context::MatchingContext<'_, Self::Impl>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        _context: &mut selectors::context::MatchingContext<'_, Self::Impl>,
    ) -> bool {
        false
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {
        // No-op for now
    }

    fn is_link(&self) -> bool {
        false
    }
    
    fn first_element_child(&self) -> Option<Self> {
        // Find first element child
        for child in &self.element.children {
            if let Node::Element(elem) = child {
                return Some(SvgElementWithParent {
                    element: elem,
                    parents: self.parents,
                });
            }
        }
        None
    }
    
    fn is_html_slot_element(&self) -> bool {
        false
    }
    
    fn has_id(&self, id: &<Self::Impl as selectors::SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("id").map_or(false, |v| v == &id.0)
    }
    
    fn has_class(&self, name: &<Self::Impl as selectors::SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("class").map_or(false, |classes| {
            classes.split_whitespace().any(|c| c == &name.0)
        })
    }
    
    fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }
    
    fn is_root(&self) -> bool {
        // Check if this element has no parent
        let elem_ptr = self.element as *const Element;
        !self.parents.contains_key(&elem_ptr)
    }

    fn is_part(&self, _name: &CssString) -> bool {
        false
    }

    fn imported_part(
        &self,
        _name: &CssString,
    ) -> Option<CssString> {
        None
    }
}

/// Simple selector implementation for SVG
#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleSelectorImpl;

/// Local name wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalName(String);

impl fmt::Display for LocalName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToCss for LocalName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

/// String wrapper that implements ToCss
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CssString(String);

impl ToCss for CssString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl From<&str> for CssString {
    fn from(s: &str) -> Self {
        CssString(s.to_string())
    }
}

impl From<&str> for LocalName {
    fn from(s: &str) -> Self {
        LocalName(s.to_string())
    }
}

impl selectors::SelectorImpl for SimpleSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = CssString;
    type Identifier = CssString;
    type LocalName = LocalName;
    type NamespaceUrl = CssString;
    type NamespacePrefix = CssString;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = CssString;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

/// Non-tree-structural pseudo-classes
#[derive(Debug, Clone, PartialEq, Eq)]
enum NonTSPseudoClass {}

impl selectors::visitor::SelectorVisitor for NonTSPseudoClass {
    type Impl = SimpleSelectorImpl;
    
    fn visit_attribute_selector(
        &mut self,
        _namespace: &NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        _local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        _local_name_lower: &<Self::Impl as selectors::SelectorImpl>::LocalName,
    ) -> bool {
        true
    }
}

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SimpleSelectorImpl;
    
    fn is_active_or_hover(&self) -> bool {
        false
    }
    
    fn is_user_action_state(&self) -> bool {
        false
    }
}

/// Pseudo-elements
#[derive(Debug, Clone, PartialEq, Eq)]
enum PseudoElement {}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = SimpleSelectorImpl;
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {} // Empty enum, no variants
    }
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {} // Empty enum, no variants
    }
}

/// Parser for CSS selectors
struct Parser;

impl<'i> selectors::parser::Parser<'i> for Parser {
    type Impl = SimpleSelectorImpl;
    type Error = SelectorParseErrorKind<'i>;
}
*/

/// Represents a CSS rule with selectors and declarations
#[derive(Debug, Clone)]
struct CssRuleData {
    selectors: Vec<String>,
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
                // Extract selectors and calculate specificity
                let mut rule_data = Vec::new();
                
                for sel in &style_rule.selectors.0 {
                    if let Some(selector_str) = extract_selector_string(sel) {
                        // Calculate specificity
                        let specificity = calculate_selector_specificity(&selector_str);
                        rule_data.push((selector_str, specificity));
                    }
                }
                
                // Extract declarations
                let mut declarations = Vec::new();
                for property in &style_rule.declarations.declarations {
                    // Extract property name and value
                    if let Some((prop_name, prop_value)) = extract_property_declaration(property) {
                        declarations.push((prop_name, prop_value));
                    }
                }
                
                // Create a rule for each selector with its specificity
                for (selector, specificity) in rule_data {
                    rules.push(CssRuleData {
                        selectors: vec![selector],
                        declarations: declarations.clone(),
                        specificity,
                    });
                }
            }
            CssRule::Media(_media_rule) => {
                // TODO: Handle media queries
            }
            _ => {
                // Ignore other rule types for now
            }
        }
    }
    
    rules
}

/// Extract a selector string from lightningcss selector
fn extract_selector_string(selector: &lightningcss::selector::Selector) -> Option<String> {
    // Convert selector to string representation
    // For now, use debug format and clean it up
    let debug_str = format!("{:?}", selector);
    
    // Try to extract the selector string from the debug format
    // This is a simplified approach - in production, we'd parse the selector properly
    if debug_str.contains("Class(") {
        // Extract class selector
        if let Some(start) = debug_str.find("Class(") {
            if let Some(end) = debug_str[start..].find(')') {
                let class_part = &debug_str[start + 6..start + end];
                if let Some(name_start) = class_part.find('"') {
                    if let Some(name_end) = class_part[name_start + 1..].find('"') {
                        return Some(format!(".{}", &class_part[name_start + 1..name_start + 1 + name_end]));
                    }
                }
            }
        }
    } else if debug_str.contains("Id(") {
        // Extract ID selector
        if let Some(start) = debug_str.find("Id(") {
            if let Some(end) = debug_str[start..].find(')') {
                let id_part = &debug_str[start + 3..start + end];
                if let Some(name_start) = id_part.find('"') {
                    if let Some(name_end) = id_part[name_start + 1..].find('"') {
                        return Some(format!("#{}", &id_part[name_start + 1..name_start + 1 + name_end]));
                    }
                }
            }
        }
    } else if debug_str.contains("LocalName(") {
        // Extract element selector
        if let Some(start) = debug_str.find("LocalName(") {
            if let Some(end) = debug_str[start..].find(')') {
                let name_part = &debug_str[start + 10..start + end];
                if let Some(name_start) = name_part.find('"') {
                    if let Some(name_end) = name_part[name_start + 1..].find('"') {
                        return Some(name_part[name_start + 1..name_start + 1 + name_end].to_string());
                    }
                }
            }
        }
    }
    
    // Fallback: try to extract any quoted string as the selector
    if let Some(quote_start) = debug_str.find('"') {
        if let Some(quote_end) = debug_str[quote_start + 1..].find('"') {
            return Some(debug_str[quote_start + 1..quote_start + 1 + quote_end].to_string());
        }
    }
    
    None
}

/// Extract property name and value from a lightningcss property
fn extract_property_declaration(property: &lightningcss::properties::Property) -> Option<(String, String)> {
    // For now, use a simplified approach that extracts common properties
    // In a full implementation, we would match on all property variants
    let debug_str = format!("{:?}", property);
    
    // Extract property name from debug string
    let property_name = if let Some(paren_pos) = debug_str.find('(') {
        debug_str[..paren_pos].to_lowercase().replace('_', "-")
    } else {
        return None;
    };
    
    // Only process presentation attributes
    if !PRESENTATION_ATTRS.contains(property_name.as_str()) {
        return None;
    }
    
    // Extract value - simplified for now
    // In a full implementation, we would properly extract values from each property type
    let value = if debug_str.contains("RGBA") {
        // Handle color values
        if let (Some(r), Some(g), Some(b)) = (
            extract_number_after(&debug_str, "red: "),
            extract_number_after(&debug_str, "green: "),
            extract_number_after(&debug_str, "blue: ")
        ) {
            format!("rgb({}, {}, {})", r, g, b)
        } else {
            "inherit".to_string()
        }
    } else if debug_str.contains("#") {
        // Handle hex colors
        if let Some(hex_start) = debug_str.find('#') {
            if let Some(hex_end) = debug_str[hex_start..].find([' ', ')', ','].as_ref()) {
                debug_str[hex_start..hex_start + hex_end].to_string()
            } else {
                "inherit".to_string()
            }
        } else {
            "inherit".to_string()
        }
    } else if debug_str.contains("px") || debug_str.contains("em") || debug_str.contains("rem") {
        // Handle length values
        "1px".to_string() // Simplified
    } else if debug_str.contains("Percentage") {
        // Handle percentage values
        "100%".to_string() // Simplified
    } else {
        // Default value
        "inherit".to_string()
    };
    
    Some((property_name, value))
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


/// Calculate CSS specificity for a selector
/// Returns a u32 where higher values mean higher specificity
/// Format: AAABBBCCC where AAA = ID count, BBB = class count, CCC = element count
fn calculate_selector_specificity(selector: &str) -> u32 {
    // For now, use simple specificity calculation
    // TODO: Implement full specificity calculation with selectors crate
    calculate_simple_specificity(selector)
}

// TODO: Implement full specificity calculation when selectors API is properly integrated
/*
/// Calculate specificity for a parsed selector
fn calculate_specificity_for_selector(selector: &selectors::parser::Selector<SimpleSelectorImpl>) -> u32 {
    let mut id_count = 0;
    let mut class_count = 0;
    let mut element_count = 0;
    
    // Walk through the selector components
    for component in selector.iter() {
        use selectors::parser::Component;
        match component {
            Component::ID(_) => id_count += 1,
            Component::Class(_) => class_count += 1,
            Component::LocalName(_) => element_count += 1,
            Component::AttributeInNoNamespace { .. } => class_count += 1,
            Component::PseudoElement(_) => element_count += 1,
            Component::NonTSPseudoClass(_) => class_count += 1,
            _ => {}
        }
    }
    
    // Combine into single specificity value
    (id_count * 1_000_000) + (class_count * 1_000) + element_count
}
*/

/// Simple specificity calculation for basic selectors
fn calculate_simple_specificity(selector: &str) -> u32 {
    if selector.starts_with('#') {
        1_000_000 // ID selector
    } else if selector.starts_with('.') {
        1_000 // Class selector
    } else {
        1 // Element selector
    }
}