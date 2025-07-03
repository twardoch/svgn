// this_file: svgn/src/plugins/remove_attributes_by_selector.rs

//! Plugin to remove attributes of elements that match a CSS selector
//!
//! This plugin removes attributes from elements that match specified CSS selectors.
//! It supports single selectors or multiple selectors with different attribute removals.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;
use selectors::{parser::Parser, SelectorList};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching::{matches_selector_list, MatchingContext, MatchingMode, ElementSelectorFlags};
use selectors::NthIndexCache;
use cssparser::ToCss;
use std::collections::HashMap;

/// Plugin to remove attributes by CSS selector
pub struct RemoveAttributesBySelectorPlugin;

/// Configuration for a single selector
#[derive(Debug, Clone)]
pub struct SelectorConfig {
    /// CSS selector string
    pub selector: String,
    /// Attributes to remove (can be a single attribute or list)
    pub attributes: Vec<String>,
}

/// Configuration parameters for the plugin
#[derive(Debug, Clone)]
pub struct RemoveAttributesBySelectorParams {
    /// List of selector configurations
    pub selectors: Vec<SelectorConfig>,
}

impl RemoveAttributesBySelectorParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> PluginResult<Self> {
        let mut selectors = Vec::new();
        
        if let Some(Value::Object(map)) = value {
            // Check if we have a single selector config or multiple
            if let Some(selector_val) = map.get("selector") {
                // Single selector config
                let selector = selector_val.as_str()
                    .ok_or_else(|| PluginError::InvalidConfig("selector must be a string".to_string()))?
                    .to_string();
                
                let attributes = if let Some(attrs_val) = map.get("attributes") {
                    parse_attributes(attrs_val)?
                } else {
                    return Err(PluginError::InvalidConfig("attributes parameter is required".to_string()));
                };
                
                selectors.push(SelectorConfig { selector, attributes });
            } else if let Some(Value::Array(selector_configs)) = map.get("selectors") {
                // Multiple selector configs
                for config in selector_configs {
                    if let Value::Object(config_map) = config {
                        let selector = config_map.get("selector")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| PluginError::InvalidConfig("selector must be a string".to_string()))?
                            .to_string();
                        
                        let attributes = if let Some(attrs_val) = config_map.get("attributes") {
                            parse_attributes(attrs_val)?
                        } else {
                            return Err(PluginError::InvalidConfig("attributes parameter is required".to_string()));
                        };
                        
                        selectors.push(SelectorConfig { selector, attributes });
                    } else {
                        return Err(PluginError::InvalidConfig("selector config must be an object".to_string()));
                    }
                }
            } else {
                return Err(PluginError::InvalidConfig("either 'selector' or 'selectors' parameter is required".to_string()));
            }
        } else {
            return Err(PluginError::InvalidConfig("parameters must be an object".to_string()));
        }
        
        if selectors.is_empty() {
            return Err(PluginError::InvalidConfig("at least one selector is required".to_string()));
        }
        
        Ok(Self { selectors })
    }
}

/// Parse attributes from JSON value (can be string or array of strings)
fn parse_attributes(value: &Value) -> PluginResult<Vec<String>> {
    match value {
        Value::String(attr) => Ok(vec![attr.clone()]),
        Value::Array(attrs) => {
            let mut result = Vec::new();
            for attr in attrs {
                if let Value::String(s) = attr {
                    result.push(s.clone());
                } else {
                    return Err(PluginError::InvalidConfig("attributes must be strings".to_string()));
                }
            }
            Ok(result)
        }
        _ => Err(PluginError::InvalidConfig("attributes must be a string or array of strings".to_string())),
    }
}

/// Element wrapper for selector matching
#[derive(Debug, Clone)]
struct ElementWrapper<'a> {
    element: &'a Element,
}

impl<'a> selectors::Element for ElementWrapper<'a> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> selectors::OpaqueElement {
        selectors::OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        None // We don't need parent traversal for this plugin
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_part(&self, _name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        false
    }

    fn imported_part(
        &self,
        _: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.element.name == other.element.name
    }

    fn is_root(&self) -> bool {
        self.element.name == "svg"
    }

    fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName) -> bool {
        self.element.name == local_name
    }

    fn has_namespace(&self, ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        // SVG elements don't have namespaces in our AST structure
        ns.is_empty()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.element.attributes.get("id").map_or(false, |v| v == &id.0)
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        _case_sensitivity: CaseSensitivity,
    ) -> bool {
        if let Some(class_attr) = self.element.attributes.get("class") {
            class_attr.split_whitespace().any(|class| class == &name.0)
        } else {
            false
        }
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as selectors::SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as selectors::SelectorImpl>::AttrValue>,
    ) -> bool {
        // We only support no namespace for now
        if !matches!(ns, NamespaceConstraint::Specific(&ref url) if url.0.is_empty()) && !matches!(ns, NamespaceConstraint::Any) {
            return false;
        }

        if let Some(attr_value) = self.element.attributes.get(&local_name.0) {
            match operation {
                AttrSelectorOperation::Exists => true,
                AttrSelectorOperation::WithValue {
                    operator,
                    case_sensitivity: _,
                    value,
                } => {
                    use selectors::attr::AttrSelectorOperator::*;
                    match operator {
                        Equal => attr_value == value.0.as_str(),
                        Includes => attr_value.split_whitespace().any(|v| v == value.0.as_str()),
                        DashMatch => {
                            attr_value == value.0.as_str() || attr_value.starts_with(&format!("{}-", value.0))
                        }
                        Prefix => attr_value.starts_with(&value.0),
                        Substring => attr_value.contains(&value.0),
                        Suffix => attr_value.ends_with(&value.0),
                    }
                }
            }
        } else {
            false
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        None
    }

    fn first_element_child(&self) -> Option<Self> {
        None
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {
        // No flags to apply
    }
}

/// Selector implementation
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectorImpl;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttrValue(String);

impl<'a> From<&'a str> for AttrValue {
    fn from(s: &'a str) -> Self {
        AttrValue(s.to_string())
    }
}

impl ToCss for AttrValue {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

impl AsRef<str> for AttrValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Identifier(String);

impl<'a> From<&'a str> for Identifier {
    fn from(s: &'a str) -> Self {
        Identifier(s.to_string())
    }
}

impl ToCss for Identifier {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalName(String);

impl<'a> From<&'a str> for LocalName {
    fn from(s: &'a str) -> Self {
        LocalName(s.to_string())
    }
}

impl std::borrow::Borrow<str> for LocalName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ToCss for LocalName {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct NamespaceUrl(String);

impl std::borrow::Borrow<str> for NamespaceUrl {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ToCss for NamespaceUrl {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct NamespacePrefix(String);

impl<'a> From<&'a str> for NamespacePrefix {
    fn from(s: &'a str) -> Self {
        NamespacePrefix(s.to_string())
    }
}

impl ToCss for NamespacePrefix {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

impl selectors::SelectorImpl for SelectorImpl {
    type ExtraMatchingData<'a> = ();
    type AttrValue = AttrValue;
    type Identifier = Identifier;
    type LocalName = LocalName;
    type NamespaceUrl = NamespaceUrl;
    type NamespacePrefix = NamespacePrefix;
    type BorrowedLocalName = str;
    type BorrowedNamespaceUrl = str;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NonTSPseudoClass {}

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SelectorImpl;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, _dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        match *self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PseudoElement {}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        match *self {}
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = SelectorImpl;
}

/// Collect matching elements without mutable borrows
fn collect_matching_paths(
    node: &Node,
    selector_list: &SelectorList<SelectorImpl>,
    current_path: Vec<usize>,
    matching_paths: &mut Vec<Vec<usize>>,
) {
    if let Node::Element(element) = node {
        let wrapper = ElementWrapper { element };
        let mut nth_index_cache = NthIndexCache::default();
        let mut context = MatchingContext::new(
            MatchingMode::Normal,
            None,
            &mut nth_index_cache,
            selectors::context::QuirksMode::NoQuirks,
            selectors::matching::NeedsSelectorFlags::No,
            selectors::matching::IgnoreNthChildForInvalidation::No,
        );
        
        if matches_selector_list(selector_list, &wrapper, &mut context) {
            matching_paths.push(current_path.clone());
        }
        
        // Recursively search children
        for (i, child) in element.children.iter().enumerate() {
            let mut child_path = current_path.clone();
            child_path.push(i);
            collect_matching_paths(child, selector_list, child_path, matching_paths);
        }
    }
}

/// Get mutable element by path
fn get_element_by_path_mut<'a>(node: &'a mut Node, path: &[usize]) -> Option<&'a mut Element> {
    if path.is_empty() {
        if let Node::Element(element) = node {
            return Some(element);
        }
        return None;
    }
    
    if let Node::Element(element) = node {
        if let Some(&index) = path.first() {
            if let Some(child) = element.children.get_mut(index) {
                return get_element_by_path_mut(child, &path[1..]);
            }
        }
    }
    None
}

impl Plugin for RemoveAttributesBySelectorPlugin {
    fn name(&self) -> &'static str {
        "removeAttributesBySelector"
    }
    
    fn description(&self) -> &'static str {
        "removes attributes of elements that match a css selector"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = RemoveAttributesBySelectorParams::from_value(params)?;
        
        // Process each selector configuration
        for config in &params.selectors {
            // Parse the CSS selector
            let mut parser_input = cssparser::ParserInput::new(&config.selector);
            let mut parser = cssparser::Parser::new(&mut parser_input);
            let selector_list = match SelectorList::<SelectorImpl>::parse(&selectors::parser::Parser, &mut parser, selectors::parser::ParseRelative::No) {
                Ok(list) => list,
                Err(_) => {
                    return Err(PluginError::InvalidConfig(format!(
                        "Invalid CSS selector: {}",
                        config.selector
                    )));
                }
            };
            
            // Collect paths to matching elements
            let mut matching_paths = Vec::new();
            let root_node = Node::Element(document.root.clone());
            collect_matching_paths(&root_node, &selector_list, vec![], &mut matching_paths);
            
            // Remove specified attributes from matching elements
            let mut root_node_mut = Node::Element(document.root.clone());
            for path in matching_paths {
                if let Some(element) = get_element_by_path_mut(&mut root_node_mut, &path) {
                    for attr_name in &config.attributes {
                        element.attributes.shift_remove(attr_name);
                    }
                }
            }
            
            // Update the document root
            if let Node::Element(updated_root) = root_node_mut {
                document.root = updated_root;
            }
        }
        
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        // Try to parse parameters to validate them
        RemoveAttributesBySelectorParams::from_value(params)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};
    use indexmap::IndexMap;
    use serde_json::json;

    fn create_test_document() -> Document {
        let mut doc = Document::default();
        
        // Create a simple SVG structure
        let mut svg = Element {
            name: "svg".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        
        // Add rect with fill="#00ff00"
        let mut rect = Element {
            name: "rect".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        rect.attributes.insert("x".to_string(), "0".to_string());
        rect.attributes.insert("y".to_string(), "0".to_string());
        rect.attributes.insert("width".to_string(), "100".to_string());
        rect.attributes.insert("height".to_string(), "100".to_string());
        rect.attributes.insert("fill".to_string(), "#00ff00".to_string());
        rect.attributes.insert("stroke".to_string(), "#00ff00".to_string());
        
        svg.children.push(Node::Element(rect));
        doc.root = svg;
        doc
    }

    #[test]
    fn test_single_attribute_removal() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[fill='#00ff00']",
            "attributes": "fill"
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that fill was removed but stroke remains
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), Some(&"#00ff00".to_string()));
        }
    }

    #[test]
    fn test_multiple_attributes_removal() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[fill='#00ff00']",
            "attributes": ["fill", "stroke"]
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that both fill and stroke were removed
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), None);
            // Other attributes should remain
            assert_eq!(rect.attributes.get("width"), Some(&"100".to_string()));
        }
    }

    #[test]
    fn test_multiple_selectors() {
        let mut doc = create_test_document();
        
        // Add an element with id="remove"
        let mut circle = Element {
            name: "circle".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
            circle.attributes.insert("id".to_string(), "remove".to_string());
            circle.attributes.insert("cx".to_string(), "50".to_string());
            circle.attributes.insert("cy".to_string(), "50".to_string());
            circle.attributes.insert("r".to_string(), "25".to_string());
            circle.attributes.insert("stroke".to_string(), "black".to_string());
            
        doc.root.children.push(Node::Element(circle));
        
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selectors": [
                {
                    "selector": "[fill='#00ff00']",
                    "attributes": "fill"
                },
                {
                    "selector": "#remove",
                    "attributes": ["stroke", "id"]
                }
            ]
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check results
        // Check rect
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
            assert_eq!(rect.attributes.get("stroke"), Some(&"#00ff00".to_string()));
        }
        
        // Check circle
        if let Some(Node::Element(ref circle)) = doc.root.children.get(1) {
            assert_eq!(circle.attributes.get("id"), None);
            assert_eq!(circle.attributes.get("stroke"), None);
            assert_eq!(circle.attributes.get("cx"), Some(&"50".to_string()));
        }
    }

    #[test]
    fn test_element_name_selector() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "rect",
            "attributes": "fill"
        });
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Check that fill was removed from rect
        if let Some(Node::Element(ref rect)) = doc.root.children.first() {
            assert_eq!(rect.attributes.get("fill"), None);
        }
    }

    #[test]
    fn test_invalid_selector() {
        let mut doc = create_test_document();
        let mut plugin = RemoveAttributesBySelectorPlugin;
        let plugin_info = PluginInfo::default();
        
        let params = json!({
            "selector": "[invalid selector",
            "attributes": "fill"
        });
        
        let result = plugin.apply(&mut doc, &plugin_info, Some(&params));
        assert!(result.is_err());
    }
}