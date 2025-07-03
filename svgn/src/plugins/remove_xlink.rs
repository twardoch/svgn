// this_file: svgn/src/plugins/remove_xlink.rs

//! Plugin to remove xlink namespace and replace attributes with SVG 2 equivalents
//!
//! This plugin removes the deprecated XLink namespace and converts XLink attributes
//! to their SVG 2 equivalents where applicable. XLink was deprecated in SVG 2.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashMap;

/// XLink namespace URI
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

/// Elements that use xlink:href but were deprecated in SVG 2
const LEGACY_ELEMENTS: &[&str] = &["cursor", "filter", "font-face-uri", "glyphRef", "tref"];

/// Configuration for the removeXlink plugin
#[derive(Debug, Clone, Default)]
pub struct RemoveXlinkConfig {
    /// Include legacy elements that don't support SVG 2 href
    pub include_legacy: bool,
}

/// Plugin to remove xlink namespace and convert to SVG 2 equivalents
pub struct RemoveXlinkPlugin;

impl Plugin for RemoveXlinkPlugin {
    fn name(&self) -> &'static str {
        "removeXlink"
    }

    fn description(&self) -> &'static str {
        "remove xlink namespace and replaces attributes with the SVG 2 equivalent where applicable"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = self.parse_config(params)?;
        let mut context = XlinkContext::new(config);

        self.process_element(&mut document.root, &mut context);
        Ok(())
    }
}

struct XlinkContext {
    config: RemoveXlinkConfig,
    xlink_prefixes: Vec<String>,
    used_in_legacy: Vec<String>,
}

impl XlinkContext {
    fn new(config: RemoveXlinkConfig) -> Self {
        Self {
            config,
            xlink_prefixes: Vec::new(),
            used_in_legacy: Vec::new(),
        }
    }
}

impl RemoveXlinkPlugin {
    fn parse_config(&self, params: Option<&Value>) -> PluginResult<RemoveXlinkConfig> {
        let mut config = RemoveXlinkConfig::default();

        if let Some(params_obj) = params {
            if let Some(include_legacy) = params_obj.get("includeLegacy") {
                if let Some(val) = include_legacy.as_bool() {
                    config.include_legacy = val;
                }
            }
        }

        Ok(config)
    }

    fn process_element(&self, element: &mut Element, context: &mut XlinkContext) {
        // Collect xlink namespace prefixes
        let mut current_xlink_prefixes = Vec::new();

        for (key, value) in &element.attributes {
            if key.starts_with("xmlns:") && value == XLINK_NAMESPACE {
                let prefix = key.strip_prefix("xmlns:").unwrap();
                current_xlink_prefixes.push(prefix.to_string());
                context.xlink_prefixes.push(prefix.to_string());
            }
        }

        // Handle xlink:href conversion
        let is_legacy = LEGACY_ELEMENTS.contains(&element.name.as_str());

        if is_legacy && !context.config.include_legacy {
            // Mark prefixes as used in legacy elements - check for any xlink attributes
            let has_xlink_attrs = element.attributes.keys().any(|key| {
                context
                    .xlink_prefixes
                    .iter()
                    .any(|prefix| key.starts_with(&format!("{}:", prefix)))
            });

            if has_xlink_attrs {
                for prefix in &context.xlink_prefixes {
                    if !context.used_in_legacy.contains(prefix) {
                        context.used_in_legacy.push(prefix.clone());
                    }
                }
            }
        } else {
            // Convert xlink:href to href if no href exists
            self.convert_href_attributes(element, &context.xlink_prefixes);
        }

        // Only convert other xlink attributes if not a legacy element or include_legacy is true
        if !is_legacy || context.config.include_legacy {
            // Handle xlink:show conversion to target
            self.convert_show_attributes(element, &context.xlink_prefixes);

            // Handle xlink:title conversion to <title> element
            self.convert_title_attributes(element, &context.xlink_prefixes);

            // Remove unused xlink attributes
            self.remove_unused_xlink_attributes(
                element,
                &context.xlink_prefixes,
                &context.used_in_legacy,
            );
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem, context);
            }
        }

        // Remove xlink namespace declarations if not used in legacy elements
        for prefix in &current_xlink_prefixes {
            if !context.used_in_legacy.contains(prefix) {
                let xmlns_key = format!("xmlns:{}", prefix);
                element.attributes.shift_remove(&xmlns_key);
            }
        }

        // Remove processed prefixes from context
        for prefix in &current_xlink_prefixes {
            context.xlink_prefixes.retain(|p| p != prefix);
        }
    }

    fn convert_href_attributes(&self, element: &mut Element, xlink_prefixes: &[String]) {
        // Find xlink:href attributes
        let href_attrs: Vec<String> = element
            .attributes
            .keys()
            .filter(|key| {
                xlink_prefixes
                    .iter()
                    .any(|prefix| *key == &format!("{}:href", prefix))
            })
            .cloned()
            .collect();

        for href_attr in href_attrs {
            if let Some(href_value) = element.attributes.get(&href_attr).cloned() {
                // Only convert if no href attribute exists
                if !element.attributes.contains_key("href") {
                    element.attributes.insert("href".to_string(), href_value);
                }
                element.attributes.shift_remove(&href_attr);
            }
        }
    }

    fn convert_show_attributes(&self, element: &mut Element, xlink_prefixes: &[String]) {
        // Find xlink:show attributes
        let show_attrs: Vec<String> = element
            .attributes
            .keys()
            .filter(|key| {
                xlink_prefixes
                    .iter()
                    .any(|prefix| *key == &format!("{}:show", prefix))
            })
            .cloned()
            .collect();

        for show_attr in show_attrs {
            if let Some(show_value) = element.attributes.get(&show_attr).cloned() {
                // Convert to target attribute if no target exists
                if !element.attributes.contains_key("target") {
                    let target_value = match show_value.as_str() {
                        "new" => "_blank",
                        "replace" => "_self",
                        _ => {
                            // Remove unknown values
                            element.attributes.shift_remove(&show_attr);
                            continue;
                        }
                    };
                    element
                        .attributes
                        .insert("target".to_string(), target_value.to_string());
                }
                element.attributes.shift_remove(&show_attr);
            }
        }
    }

    fn convert_title_attributes(&self, element: &mut Element, xlink_prefixes: &[String]) {
        // Find xlink:title attributes
        let title_attrs: Vec<String> = element
            .attributes
            .keys()
            .filter(|key| {
                xlink_prefixes
                    .iter()
                    .any(|prefix| *key == &format!("{}:title", prefix))
            })
            .cloned()
            .collect();

        for title_attr in title_attrs {
            if let Some(title_value) = element.attributes.get(&title_attr).cloned() {
                // Check if element already has a title child
                let has_title_child = element
                    .children
                    .iter()
                    .any(|child| matches!(child, Node::Element(elem) if elem.name == "title"));

                if !has_title_child {
                    // Create title element
                    let title_element = Element {
                        name: "title".to_string(),
                        attributes: indexmap::IndexMap::new(),
                        namespaces: HashMap::new(),
                        children: vec![Node::Text(title_value)],
                    };
                    element.children.insert(0, Node::Element(title_element));
                }
                element.attributes.shift_remove(&title_attr);
            }
        }
    }

    fn remove_unused_xlink_attributes(
        &self,
        element: &mut Element,
        xlink_prefixes: &[String],
        used_in_legacy: &[String],
    ) {
        // Remove any remaining xlink attributes that weren't converted
        let attrs_to_remove: Vec<String> = element
            .attributes
            .keys()
            .filter(|key| {
                if let Some(colon_pos) = key.find(':') {
                    let prefix = &key[..colon_pos];
                    xlink_prefixes.contains(&prefix.to_string())
                        && !used_in_legacy.contains(&prefix.to_string())
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        for attr in attrs_to_remove {
            element.attributes.shift_remove(&attr);
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_test_document() -> Document {
        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = RemoveXlinkPlugin;
        assert_eq!(plugin.name(), "removeXlink");
        assert_eq!(plugin.description(), "remove xlink namespace and replaces attributes with the SVG 2 equivalent where applicable");
    }

    #[test]
    fn test_convert_xlink_href_to_href() {
        let mut document = create_test_document();

        // Add xlink namespace
        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        // Add element with xlink:href
        let mut use_attrs = IndexMap::new();
        use_attrs.insert("xlink:href".to_string(), "#symbol1".to_string());

        let use_element = Element {
            name: "use".to_string(),
            attributes: use_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(use_element)];

        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xlink:href should be converted to href
        if let Node::Element(ref use_elem) = document.root.children[0] {
            assert!(!use_elem.attributes.contains_key("xlink:href"));
            assert_eq!(
                use_elem.attributes.get("href"),
                Some(&"#symbol1".to_string())
            );
        } else {
            panic!("Expected use element");
        }

        // xlink namespace should be removed
        assert!(!document.root.attributes.contains_key("xmlns:xlink"));
    }

    #[test]
    fn test_preserve_existing_href() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        let mut element_attrs = IndexMap::new();
        element_attrs.insert("href".to_string(), "#existing".to_string());
        element_attrs.insert("xlink:href".to_string(), "#xlink".to_string());

        let element = Element {
            name: "a".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should preserve existing href and remove xlink:href
        if let Node::Element(ref elem) = document.root.children[0] {
            assert!(!elem.attributes.contains_key("xlink:href"));
            assert_eq!(elem.attributes.get("href"), Some(&"#existing".to_string()));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_convert_xlink_show_to_target() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        let mut element_attrs = IndexMap::new();
        element_attrs.insert("xlink:show".to_string(), "new".to_string());

        let element = Element {
            name: "a".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xlink:show="new" should become target="_blank"
        if let Node::Element(ref elem) = document.root.children[0] {
            assert!(!elem.attributes.contains_key("xlink:show"));
            assert_eq!(elem.attributes.get("target"), Some(&"_blank".to_string()));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_convert_xlink_title_to_title_element() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        let mut element_attrs = IndexMap::new();
        element_attrs.insert("xlink:title".to_string(), "Element title".to_string());

        let element = Element {
            name: "rect".to_string(),
            attributes: element_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(element)];

        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // xlink:title should be converted to title element
        if let Node::Element(ref elem) = document.root.children[0] {
            assert!(!elem.attributes.contains_key("xlink:title"));
            assert_eq!(elem.children.len(), 1);

            if let Node::Element(ref title) = elem.children[0] {
                assert_eq!(title.name, "title");
                assert_eq!(title.children.len(), 1);
                if let Node::Text(ref text) = title.children[0] {
                    assert_eq!(text, "Element title");
                } else {
                    panic!("Expected text in title");
                }
            } else {
                panic!("Expected title element");
            }
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_preserve_legacy_elements() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        let mut filter_attrs = IndexMap::new();
        filter_attrs.insert("xlink:href".to_string(), "#filter1".to_string());

        let filter_element = Element {
            name: "filter".to_string(),
            attributes: filter_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(filter_element)];

        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Legacy elements should preserve xlink attributes by default
        if let Node::Element(ref filter) = document.root.children[0] {
            assert!(filter.attributes.contains_key("xlink:href"));
            assert!(!filter.attributes.contains_key("href"));
        } else {
            panic!("Expected filter element");
        }

        // xlink namespace should be preserved for legacy usage
        assert!(document.root.attributes.contains_key("xmlns:xlink"));
    }

    #[test]
    fn test_include_legacy_option() {
        let mut document = create_test_document();

        document
            .root
            .attributes
            .insert("xmlns:xlink".to_string(), XLINK_NAMESPACE.to_string());

        let mut filter_attrs = IndexMap::new();
        filter_attrs.insert("xlink:href".to_string(), "#filter1".to_string());

        let filter_element = Element {
            name: "filter".to_string(),
            attributes: filter_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(filter_element)];

        let params = serde_json::json!({"includeLegacy": true});
        let mut plugin = RemoveXlinkPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&params));
        assert!(result.is_ok());

        // With includeLegacy=true, should convert even legacy elements
        if let Node::Element(ref filter) = document.root.children[0] {
            assert!(!filter.attributes.contains_key("xlink:href"));
            assert_eq!(filter.attributes.get("href"), Some(&"#filter1".to_string()));
        } else {
            panic!("Expected filter element");
        }
    }
}
