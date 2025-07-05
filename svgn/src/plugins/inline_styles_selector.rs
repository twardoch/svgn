// this_file: svgn/src/plugins/inline_styles_selector.rs

//! SVG Element selector matching implementation for inline styles plugin
//!
//! This module provides the necessary implementations for the selectors crate
//! to work with our SVG DOM structure.

use crate::ast::Element;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::parser::{Selector, SelectorImpl};
use selectors::{Element as SelectorElement, OpaqueElement};
use std::fmt;

/// SVG Selector implementation for use with the selectors crate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgSelectorImpl;

impl SelectorImpl for SvgSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = String;
    type Identifier = String;
    type LocalName = String;
    type NamespacePrefix = String;
    type NamespaceUrl = String;
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

/// Pseudo-element
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElement {}

impl fmt::Display for PseudoElement {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

/// Wrapper around our SVG Element to implement SelectorElement
pub struct SvgElementWrapper<'a> {
    element: &'a Element,
    parent: Option<&'a Element>,
    index: usize,
}

impl<'a> SvgElementWrapper<'a> {
    pub fn new(element: &'a Element, parent: Option<&'a Element>, index: usize) -> Self {
        Self {
            element,
            parent,
            index,
        }
    }
}

impl<'a> SelectorElement for SvgElementWrapper<'a> {
    type Impl = SvgSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.element as *const Element as *const ())
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent.map(|p| SvgElementWrapper::new(p, None, 0))
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
        if let Some(parent) = self.parent {
            if self.index > 0 {
                // Find the previous element sibling
                let mut current_index = 0;
                for child in &parent.children {
                    if let crate::ast::Node::Element(elem) = child {
                        if current_index == self.index - 1 {
                            return Some(SvgElementWrapper::new(elem, self.parent, current_index));
                        }
                        current_index += 1;
                    }
                }
            }
        }
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        if let Some(parent) = self.parent {
            // Find the next element sibling
            let mut current_index = 0;
            let mut found_self = false;
            for child in &parent.children {
                if let crate::ast::Node::Element(elem) = child {
                    if found_self {
                        return Some(SvgElementWrapper::new(elem, self.parent, current_index));
                    }
                    if current_index == self.index {
                        found_self = true;
                    }
                    current_index += 1;
                }
            }
        }
        None
    }

    fn first_element_child(&self) -> Option<Self> {
        for (index, child) in self.element.children.iter().enumerate() {
            if let crate::ast::Node::Element(elem) = child {
                return Some(SvgElementWrapper::new(elem, Some(self.element), index));
            }
        }
        None
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {}

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        false
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(
        &self,
        local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName,
    ) -> bool {
        self.element.name == *local_name
    }

    fn has_namespace(
        &self,
        _ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl,
    ) -> bool {
        // For SVG, we'll assume all elements are in the SVG namespace
        true
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        // Ignore namespace for now
        if !matches!(ns, NamespaceConstraint::Any) {
            return false;
        }

        if let Some(attr_value) = self.element.attributes.get(local_name) {
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity,
                    expected_value,
                } => {
                    let case_insensitive = matches!(case_sensitivity, CaseSensitivity::CaseInsensitive);
                    
                    match operator {
                        selectors::attr::AttrSelectorOperator::Equal => {
                            if case_insensitive {
                                attr_value.to_lowercase() == expected_value.to_lowercase()
                            } else {
                                attr_value == *expected_value
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Includes => {
                            // For whitespace-separated lists (like class)
                            let values: Vec<&str> = attr_value.split_whitespace().collect();
                            if case_insensitive {
                                values.iter().any(|v| v.to_lowercase() == expected_value.to_lowercase())
                            } else {
                                values.contains(&expected_value.as_str())
                            }
                        }
                        selectors::attr::AttrSelectorOperator::DashMatch => {
                            // For hyphen-separated lists (like lang)
                            if case_insensitive {
                                let attr_lower = attr_value.to_lowercase();
                                let expected_lower = expected_value.to_lowercase();
                                attr_lower == expected_lower || attr_lower.starts_with(&format!("{}-", expected_lower))
                            } else {
                                *attr_value == **expected_value || attr_value.starts_with(&format!("{}-", expected_value))
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Prefix => {
                            if case_insensitive {
                                attr_value.to_lowercase().starts_with(&expected_value.to_lowercase())
                            } else {
                                attr_value.starts_with(expected_value)
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Suffix => {
                            if case_insensitive {
                                attr_value.to_lowercase().ends_with(&expected_value.to_lowercase())
                            } else {
                                attr_value.ends_with(expected_value)
                            }
                        }
                        selectors::attr::AttrSelectorOperator::Substring => {
                            if case_insensitive {
                                attr_value.to_lowercase().contains(&expected_value.to_lowercase())
                            } else {
                                attr_value.contains(expected_value)
                            }
                        }
                    }
                }
            }
        } else {
            false
        }
    }

    fn is_root(&self) -> bool {
        self.parent.is_none() && self.element.name == "svg"
    }

    fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(
        &self,
        id: &<Self::Impl as SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.element.attributes.get("id").map_or(false, |elem_id| elem_id == id)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        if let Some(classes) = self.element.attributes.get("class") {
            classes.split_whitespace().any(|c| c == name)
        } else {
            false
        }
    }

    fn is_part(&self, _name: &<Self::Impl as SelectorImpl>::Identifier) -> bool {
        false
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as SelectorImpl>::Identifier> {
        None
    }

    fn is_nth_child(&self, index: i32, step: i32, offset: i32) -> bool {
        if let Some(parent) = self.parent {
            let mut element_index = 0;
            for child in &parent.children {
                if let crate::ast::Node::Element(_) = child {
                    element_index += 1;
                    if element_index == self.index + 1 {
                        return matches_nth(element_index as i32, index, step, offset);
                    }
                }
            }
        }
        false
    }

    fn is_nth_last_child(&self, index: i32, step: i32, offset: i32) -> bool {
        if let Some(parent) = self.parent {
            let total_elements: usize = parent
                .children
                .iter()
                .filter(|child| matches!(child, crate::ast::Node::Element(_)))
                .count();
            
            let mut element_index = 0;
            for child in &parent.children {
                if let crate::ast::Node::Element(_) = child {
                    element_index += 1;
                    if element_index == self.index + 1 {
                        let last_index = total_elements as i32 - element_index + 1;
                        return matches_nth(last_index, index, step, offset);
                    }
                }
            }
        }
        false
    }

    fn is_nth_of_type(&self, index: i32, step: i32, offset: i32) -> bool {
        if let Some(parent) = self.parent {
            let mut type_index = 0;
            for child in &parent.children {
                if let crate::ast::Node::Element(elem) = child {
                    if elem.name == self.element.name {
                        type_index += 1;
                        if std::ptr::eq(elem, self.element) {
                            return matches_nth(type_index, index, step, offset);
                        }
                    }
                }
            }
        }
        false
    }

    fn is_nth_last_of_type(&self, index: i32, step: i32, offset: i32) -> bool {
        if let Some(parent) = self.parent {
            let same_type_elements: Vec<_> = parent
                .children
                .iter()
                .filter_map(|child| {
                    if let crate::ast::Node::Element(elem) = child {
                        if elem.name == self.element.name {
                            Some(elem)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            let total_of_type = same_type_elements.len();
            for (i, elem) in same_type_elements.iter().enumerate() {
                if std::ptr::eq(*elem, self.element) {
                    let last_type_index = total_of_type as i32 - i as i32;
                    return matches_nth(last_type_index, index, step, offset);
                }
            }
        }
        false
    }
}

/// Helper function to check if a position matches nth-child pattern
fn matches_nth(position: i32, index: i32, step: i32, offset: i32) -> bool {
    if step == 0 {
        position == index
    } else {
        let diff = position - offset;
        diff >= 0 && diff % step == 0
    }
}

/// Check if an element matches a parsed selector
pub fn element_matches_selector(element: &Element, selector: &Selector<SvgSelectorImpl>) -> bool {
    let wrapper = SvgElementWrapper::new(element, None, 0);
    let mut context = selectors::matching::MatchingContext::new(
        selectors::matching::MatchingMode::Normal,
        None,
        None,
        selectors::matching::QuirksMode::NoQuirks,
    );
    selectors::matching::matches_selector(selector, 0, None, &wrapper, &mut context)
}

/// Walk the element tree and call a function for each element with its parent and index
pub fn walk_element_tree_with_parent<F>(
    element: &Element,
    parent: Option<&Element>,
    mut func: F,
) where
    F: FnMut(&Element, Option<&Element>, usize) + Clone,
{
    // Process current element
    func(element, parent, 0);
    
    // Process children
    let mut element_index = 0;
    for child in &element.children {
        if let crate::ast::Node::Element(child_elem) = child {
            walk_element_tree_with_parent(child_elem, Some(element), func.clone());
            element_index += 1;
        }
    }
}