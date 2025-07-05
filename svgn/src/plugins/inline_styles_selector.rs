// this_file: svgn/src/plugins/inline_styles_selector.rs

//! SVG Element selector matching implementation for inline styles plugin
//!
//! This module provides the necessary implementations for the selectors crate
//! to work with our SVG DOM structure. It uses wrapper types to avoid orphan
//! rule violations when implementing external traits.

use crate::ast::Element;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::parser::{Selector, SelectorImpl};
use selectors::{Element as SelectorElement, OpaqueElement};
use std::fmt;
use std::borrow::Borrow;

/// Wrapper types to implement required traits for selectors crate
/// These types wrap String to avoid orphan rule violations (E0117)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgAttrValue(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgIdentifier(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SvgLocalName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SvgNamespacePrefix(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SvgNamespaceUrl(pub String);

// Implement required traits for our wrapper types
impl cssparser::ToCss for SvgAttrValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for SvgAttrValue {
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl cssparser::ToCss for SvgIdentifier {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for SvgIdentifier {
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl cssparser::ToCss for SvgLocalName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for SvgLocalName {
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl cssparser::ToCss for SvgNamespacePrefix {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for SvgNamespacePrefix {
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl cssparser::ToCss for SvgNamespaceUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl PrecomputedHash for SvgNamespaceUrl {
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish() as u32
    }
}

// Implement Borrow<str> for types that need it
impl Borrow<str> for SvgLocalName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SvgNamespaceUrl {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// Implement From<&str> for our wrapper types as required by selectors crate
impl<'a> From<&'a str> for SvgAttrValue {
    fn from(s: &'a str) -> Self {
        SvgAttrValue(s.to_string())
    }
}

impl<'a> From<&'a str> for SvgIdentifier {
    fn from(s: &'a str) -> Self {
        SvgIdentifier(s.to_string())
    }
}

impl<'a> From<&'a str> for SvgLocalName {
    fn from(s: &'a str) -> Self {
        SvgLocalName(s.to_string())
    }
}

impl<'a> From<&'a str> for SvgNamespacePrefix {
    fn from(s: &'a str) -> Self {
        SvgNamespacePrefix(s.to_string())
    }
}

impl<'a> From<&'a str> for SvgNamespaceUrl {
    fn from(s: &'a str) -> Self {
        SvgNamespaceUrl(s.to_string())
    }
}

/// SVG Selector implementation for use with the selectors crate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgSelectorImpl;

impl SelectorImpl for SvgSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = SvgAttrValue;
    type Identifier = SvgIdentifier;
    type LocalName = SvgLocalName;
    type NamespacePrefix = SvgNamespacePrefix;
    type NamespaceUrl = SvgNamespaceUrl;
    type BorrowedNamespaceUrl = str;
    type BorrowedLocalName = str;

    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

/// Non-tree-structural pseudo-class
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonTSPseudoClass {}

impl fmt::Display for NonTSPseudoClass {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl cssparser::ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {}
    }
}

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SvgSelectorImpl;

    fn is_active_or_hover(&self) -> bool {
        match *self {}
    }

    fn is_user_action_state(&self) -> bool {
        match *self {}
    }
}

/// Pseudo-element
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElement {}

impl fmt::Display for PseudoElement {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {}
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = SvgSelectorImpl;
}

/// Wrapper around our SVG Element to implement the selectors::Element trait
#[derive(Debug)]
pub struct SvgElement<'a> {
    pub element: &'a Element,
}

/// Type alias for compatibility with other modules
pub type SvgElementWrapper<'a> = SvgElement<'a>;

impl<'a> SvgElement<'a> {
    pub fn new(element: &'a Element) -> Self {
        SvgElement { element }
    }
}

impl<'a> Clone for SvgElement<'a> {
    fn clone(&self) -> Self {
        SvgElement {
            element: self.element,
        }
    }
}

impl<'a> SelectorElement for SvgElement<'a> {
    type Impl = SvgSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.element)
    }
    
    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {
        // No-op for SVG elements - we don't need to track selector flags
    }

    fn parent_element(&self) -> Option<Self> {
        None // Simplified implementation for now
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
        None // Simplified implementation for now
    }

    fn next_sibling_element(&self) -> Option<Self> {
        None // Simplified implementation for now
    }

    fn first_element_child(&self) -> Option<Self> {
        self.element.children.iter().find_map(|child| {
            if let crate::ast::Node::Element(element) = child {
                Some(SvgElement::new(element))
            } else {
                None
            }
        })
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &str) -> bool {
        self.element.name == local_name
    }

    fn has_namespace(&self, _namespace: &str) -> bool {
        true // SVG elements are in the SVG namespace
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&String>,
        local_name: &String,
        operation: &AttrSelectorOperation<&String>,
    ) -> bool {
        // Only match attributes without namespace for now
        if !matches!(ns, NamespaceConstraint::Specific(ns_val) if ns_val.is_empty())
            && !matches!(ns, NamespaceConstraint::Any) {
            return false;
        }

        if let Some(attr_value) = self.element.attributes.get(local_name) {
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity,
                    value,
                } => {
                    let case_insensitive = matches!(case_sensitivity, CaseSensitivity::AsciiCaseInsensitive);
                    
                    match operator {
                        selectors::attr::AttrSelectorOperator::Equal => {
                            if case_insensitive {
                                attr_value.to_lowercase() == value.to_lowercase()
                            } else {
                                attr_value == value
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Includes => {
                            let values: Vec<&str> = attr_value.split_whitespace().collect();
                            if case_insensitive {
                                values.iter().any(|v| v.to_lowercase() == value.to_lowercase())
                            } else {
                                values.contains(value)
                            }
                        }
                        selectors::attr::AttrSelectorOperator::DashMatch => {
                            if case_insensitive {
                                let attr_lower = attr_value.to_lowercase();
                                let expected_lower = value.to_lowercase();
                                attr_lower == expected_lower || attr_lower.starts_with(&format!("{}-", expected_lower))
                            } else {
                                attr_value == value || attr_value.starts_with(&format!("{}-", value))
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Prefix => {
                            if case_insensitive {
                                attr_value.to_lowercase().starts_with(&value.to_lowercase())
                            } else {
                                attr_value.starts_with(value)
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Suffix => {
                            if case_insensitive {
                                attr_value.to_lowercase().ends_with(&value.to_lowercase())
                            } else {
                                attr_value.ends_with(value)
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Substring => {
                            if case_insensitive {
                                attr_value.to_lowercase().contains(&value.to_lowercase())
                            } else {
                                attr_value.contains(value)
                            }
                        }
                    }
                }
            }
        } else {
            false
        }
    }

    fn match_non_ts_pseudo_class(&self, _pc: &NonTSPseudoClass, _context: &mut selectors::matching::MatchingContext<SvgSelectorImpl>) -> bool {
        match *_pc {}
    }

    fn match_pseudo_element(&self, _pe: &PseudoElement, _context: &mut selectors::matching::MatchingContext<SvgSelectorImpl>) -> bool {
        match *_pe {}
    }

    fn is_link(&self) -> bool {
        false
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &String, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("id").map_or(false, |elem_id| elem_id == id)
    }

    fn has_class(&self, name: &String, _case_sensitivity: CaseSensitivity) -> bool {
        if let Some(class_attr) = self.element.attributes.get("class") {
            let classes = class_attr;
            classes.split_whitespace().any(|c| c == name)
        } else {
            false
        }
    }

    fn imported_part(&self, _name: &String) -> Option<String> {
        None
    }

    fn is_part(&self, _name: &String) -> bool {
        false
    }

    fn has_custom_state(&self, _name: &String) -> bool {
        false
    }

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::CountingBloomFilter<selectors::bloom::BloomStorageU8>) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }

    fn is_root(&self) -> bool {
        self.element.name == "svg"
    }
}

/// Walk through the element tree with parent context
/// This function is used by other modules for traversing SVG elements
pub fn walk_element_tree_with_parent<F>(element: &Element, parent: Option<&Element>, mut visitor: F)
where
    F: FnMut(&Element, Option<&Element>),
{
    visitor(element, parent);
    
    for child in &element.children {
        if let crate::ast::Node::Element(child_element) = child {
            walk_element_tree_with_parent(child_element, Some(element), &mut visitor);
        }
    }
}

/// Check if a CSS selector matches an SVG element
pub fn matches_selector(element: &Element, selector: &Selector<SvgSelectorImpl>) -> bool {
    use selectors::matching::{MatchingContext, MatchingMode, QuirksMode, NeedsSelectorFlags, MatchingForInvalidation};
    use selectors::matching::SelectorCaches;
    
    let svg_element = SvgElement::new(element);
    let mut selector_caches = SelectorCaches::default();
    let mut context = selectors::matching::MatchingContext::new(
        MatchingMode::Normal,
        None,
        &mut selector_caches,
        QuirksMode::NoQuirks,
        NeedsSelectorFlags::No,
        MatchingForInvalidation::No,
    );

    selectors::matching::matches_selector(selector, 0, None, &svg_element, &mut context)
}