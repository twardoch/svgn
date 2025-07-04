// this_file: svgn/src/plugins/inline_styles_selector.rs

//! CSS selector matching implementation for SVG elements

use crate::ast::{Element, Node};
use selectors::{Element as SelectorElement, OpaqueElement};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::parser::{SelectorImpl, NonTSPseudoClass, PseudoElement};
use selectors::matching::{ElementSelectorFlags, MatchingContext};
use cssparser::ToCss;
use std::fmt;

/// Wrapper for SVG elements to implement selectors::Element trait
#[derive(Debug, Clone)]
pub struct SvgElementWrapper<'a> {
    pub element: &'a Element,
    parent: Option<&'a Element>,
    index_in_parent: Option<usize>,
}

impl<'a> SvgElementWrapper<'a> {
    /// Create a new wrapper for an element
    pub fn new(element: &'a Element) -> Self {
        Self {
            element,
            parent: None,
            index_in_parent: None,
        }
    }
    
    /// Create a wrapper with parent information
    pub fn with_parent(element: &'a Element, parent: &'a Element, index: usize) -> Self {
        Self {
            element,
            parent: Some(parent),
            index_in_parent: Some(index),
        }
    }
}

impl<'a> SelectorElement for SvgElementWrapper<'a> {
    type Impl = SvgSelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.element)
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent.map(SvgElementWrapper::new)
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
        if let (Some(parent), Some(index)) = (self.parent, self.index_in_parent) {
            if index > 0 {
                // Find previous element sibling
                for i in (0..index).rev() {
                    if let Node::Element(elem) = &parent.children[i] {
                        return Some(SvgElementWrapper::with_parent(elem, parent, i));
                    }
                }
            }
        }
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        if let (Some(parent), Some(index)) = (self.parent, self.index_in_parent) {
            // Find next element sibling
            for i in (index + 1)..parent.children.len() {
                if let Node::Element(elem) = &parent.children[i] {
                    return Some(SvgElementWrapper::with_parent(elem, parent, i));
                }
            }
        }
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName) -> bool {
        self.element.name == local_name.0
    }

    fn has_namespace(&self, _ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        // TODO: Implement namespace support if needed
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        if let Some(attr_value) = self.element.attributes.get(&local_name.0) {
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue { operator, case_sensitivity, value } => {
                    use selectors::attr::AttrSelectorOperator;
                    match operator {
                        AttrSelectorOperator::Equal => {
                            match case_sensitivity {
                                CaseSensitivity::CaseSensitive => attr_value == &value.0,
                                CaseSensitivity::AsciiCaseInsensitive => {
                                    attr_value.eq_ignore_ascii_case(&value.0)
                                }
                            }
                        }
                        AttrSelectorOperator::Includes => {
                            attr_value.split_whitespace().any(|word| {
                                match case_sensitivity {
                                    CaseSensitivity::CaseSensitive => word == value.0,
                                    CaseSensitivity::AsciiCaseInsensitive => word.eq_ignore_ascii_case(&value.0),
                                }
                            })
                        }
                        AttrSelectorOperator::DashMatch => {
                            attr_value == &value.0 || attr_value.starts_with(&format!("{}-", value.0))
                        }
                        AttrSelectorOperator::Prefix => {
                            attr_value.starts_with(&value.0)
                        }
                        AttrSelectorOperator::Substring => {
                            attr_value.contains(&value.0)
                        }
                        AttrSelectorOperator::Suffix => {
                            attr_value.ends_with(&value.0)
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
        _pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<'_, Self::Impl>,
    ) -> bool {
        // SVG doesn't support pseudo-classes like :hover, :active etc.
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<'_, Self::Impl>,
    ) -> bool {
        // SVG doesn't support pseudo-elements
        false
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {
        // No-op for SVG
    }

    fn is_link(&self) -> bool {
        // SVG elements are not links in the CSS sense
        false
    }
    
    fn has_id(&self, id: &<Self::Impl as SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("id") == Some(&id.0)
    }
    
    fn has_class(&self, name: &<Self::Impl as SelectorImpl>::Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.element.attributes.get("class").is_some_and(|classes| {
            classes.split_whitespace().any(|c| c == name.0)
        })
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
    
    fn is_empty(&self) -> bool {
        // An element is empty if it has no children
        self.element.children.is_empty()
    }
    
    fn is_root(&self) -> bool {
        // Root is svg element with no parent
        self.element.name == "svg" && self.parent.is_none()
    }
    
    fn first_element_child(&self) -> Option<Self> {
        // Find first element child
        for (index, child) in self.element.children.iter().enumerate() {
            if let Node::Element(elem) = child {
                return Some(SvgElementWrapper::with_parent(elem, self.element, index));
            }
        }
        None
    }
    
    fn is_html_slot_element(&self) -> bool {
        false
    }
}

/// SVG selector implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgSelectorImpl;

/// String wrapper for attribute values
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SvgString(pub String);

impl ToCss for SvgString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        cssparser::serialize_string(&self.0, dest)
    }
}

impl<'a> From<&'a str> for SvgString {
    fn from(s: &'a str) -> Self {
        SvgString(s.to_string())
    }
}

/// Local name wrapper
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalName(pub String);

impl ToCss for LocalName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl<'a> From<&'a str> for LocalName {
    fn from(s: &'a str) -> Self {
        LocalName(s.to_string())
    }
}

impl SelectorImpl for SvgSelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = SvgString;
    type Identifier = SvgString;
    type LocalName = LocalName;
    type NamespaceUrl = SvgString;
    type NamespacePrefix = SvgString;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = SvgString;
    type NonTSPseudoClass = SvgNonTSPseudoClass;
    type PseudoElement = SvgPseudoElement;
}

/// Non-tree-structural pseudo-classes (empty for SVG)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgNonTSPseudoClass {}

impl NonTSPseudoClass for SvgNonTSPseudoClass {
    type Impl = SvgSelectorImpl;
    
    fn is_active_or_hover(&self) -> bool {
        match *self {} // Empty enum
    }
    
    fn is_user_action_state(&self) -> bool {
        match *self {} // Empty enum
    }
}

impl ToCss for SvgNonTSPseudoClass {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {} // Empty enum
    }
}

/// Pseudo-elements (empty for SVG)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgPseudoElement {}

impl PseudoElement for SvgPseudoElement {
    type Impl = SvgSelectorImpl;
}

impl ToCss for SvgPseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {} // Empty enum
    }
}

/// Check if an element matches a selector
pub fn element_matches_selector(element: &Element, selector: &selectors::parser::Selector<SvgSelectorImpl>) -> bool {
    let wrapper = SvgElementWrapper::new(element);
    
    // For selectors 0.25, we need to manually check selector components
    // This is a simplified implementation
    use selectors::parser::Component;
    
    for component in selector.iter() {
        match component {
            Component::LocalName(local_name) => {
                if wrapper.element.name != local_name.name.0 {
                    return false;
                }
            }
            Component::ID(id) => {
                let svg_id = SvgString(id.0.to_string());
                if !wrapper.has_id(&svg_id, CaseSensitivity::CaseSensitive) {
                    return false;
                }
            }
            Component::Class(class) => {
                let svg_class = SvgString(class.0.to_string());
                if !wrapper.has_class(&svg_class, CaseSensitivity::CaseSensitive) {
                    return false;
                }
            }
            // For now, ignore other components
            _ => {}
        }
    }
    
    true
}

/// Walk the element tree with parent tracking
pub fn walk_element_tree_with_parent<F>(element: &Element, parent: Option<&Element>, mut f: F)
where
    F: FnMut(&Element, Option<&Element>, Option<usize>),
{
    f(element, parent, None);
    
    for (index, child) in element.children.iter().enumerate() {
        if let Node::Element(child_elem) = child {
            walk_element_tree_with_parent_impl(child_elem, Some(element), index, &mut f);
        }
    }
}

fn walk_element_tree_with_parent_impl<F>(
    element: &Element,
    parent: Option<&Element>,
    index: usize,
    f: &mut F,
)
where
    F: FnMut(&Element, Option<&Element>, Option<usize>),
{
    f(element, parent, Some(index));
    
    for (child_index, child) in element.children.iter().enumerate() {
        if let Node::Element(child_elem) = child {
            walk_element_tree_with_parent_impl(child_elem, Some(element), child_index, f);
        }
    }
}