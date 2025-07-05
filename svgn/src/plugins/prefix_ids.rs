// this_file: svgn/src/plugins/prefix_ids.rs

use crate::ast::{Document, Element, Node};
use crate::collections::REFERENCES_PROPS;
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PrefixIdsConfig {
    /// The prefix to use. Can be a string or auto-generated from file path
    pub prefix: Option<String>,
    /// Delimiter between prefix and original ID
    pub delim: String,
    /// Whether to prefix IDs
    pub prefix_ids: bool,
    /// Whether to prefix class names
    pub prefix_class_names: bool,
}

impl Default for PrefixIdsConfig {
    fn default() -> Self {
        Self {
            prefix: None,
            delim: "__".to_string(),
            prefix_ids: true,
            prefix_class_names: true,
        }
    }
}

#[derive(Default)]
pub struct PrefixIdsPlugin;


impl PrefixIdsPlugin {
    pub fn new() -> Self {
        Self
    }

    fn parse_config(&self, params: Option<&Value>) -> PrefixIdsConfig {
        let mut config = PrefixIdsConfig::default();

        if let Some(Value::Object(obj)) = params {
            if let Some(Value::String(prefix)) = obj.get("prefix") {
                config.prefix = Some(prefix.clone());
            }

            if let Some(Value::String(delim)) = obj.get("delim") {
                config.delim = delim.clone();
            }

            if let Some(Value::Bool(prefix_ids)) = obj.get("prefixIds") {
                config.prefix_ids = *prefix_ids;
            }

            if let Some(Value::Bool(prefix_class_names)) = obj.get("prefixClassNames") {
                config.prefix_class_names = *prefix_class_names;
            }
        }

        config
    }

    fn get_basename(path: &str) -> String {
        // Extract everything after latest slash or backslash
        if let Some(captures) = Regex::new(r"[/\\]?([^/\\]+)$").unwrap().captures(path) {
            if let Some(matched) = captures.get(1) {
                return matched.as_str().to_string();
            }
        }
        String::new()
    }

    fn escape_identifier_name(s: &str) -> String {
        s.replace(['.', ' '], "_")
    }

    fn generate_prefix(&self, config: &PrefixIdsConfig, info: &PluginInfo) -> String {
        if let Some(prefix) = &config.prefix {
            return format!("{}{}", prefix, config.delim);
        }

        if let Some(path) = &info.path {
            let basename = Self::get_basename(path);
            if !basename.is_empty() {
                return format!(
                    "{}{}",
                    Self::escape_identifier_name(&basename),
                    config.delim
                );
            }
        }

        format!("prefix{}", config.delim)
    }

    fn prefix_id(&self, prefix: &str, id: &str) -> String {
        if id.starts_with(prefix) {
            id.to_string()
        } else {
            format!("{}{}", prefix, id)
        }
    }

    fn prefix_reference(&self, prefix: &str, reference: &str) -> Option<String> {
        reference
            .strip_prefix('#')
            .map(|id| format!("#{}", self.prefix_id(prefix, id)))
    }

    fn process_element(&self, element: &mut Element, prefix: &str, config: &PrefixIdsConfig) {
        // Prefix ID attribute
        if config.prefix_ids {
            if let Some(id) = element.attributes.get_mut("id") {
                if !id.is_empty() {
                    *id = self.prefix_id(prefix, id);
                }
            }
        }

        // Prefix class attribute
        if config.prefix_class_names {
            if let Some(class) = element.attributes.get_mut("class") {
                if !class.is_empty() {
                    let classes: Vec<String> = class
                        .split_whitespace()
                        .map(|name| self.prefix_id(prefix, name))
                        .collect();
                    *class = classes.join(" ");
                }
            }
        }

        // Prefix href and xlink:href attributes
        for attr_name in ["href", "xlink:href"] {
            if let Some(href) = element.attributes.get_mut(attr_name) {
                if !href.is_empty() {
                    if let Some(prefixed) = self.prefix_reference(prefix, href) {
                        *href = prefixed;
                    }
                }
            }
        }

        // Prefix URL references in specific attributes
        for attr_name in REFERENCES_PROPS.iter() {
            if let Some(attr_value) = element.attributes.get_mut(*attr_name) {
                if !attr_value.is_empty() {
                    *attr_value = self.process_url_references(attr_value, prefix);
                }
            }
        }

        // Prefix begin/end attributes (for animation)
        for attr_name in ["begin", "end"] {
            if let Some(attr_value) = element.attributes.get_mut(attr_name) {
                if !attr_value.is_empty() {
                    *attr_value = self.process_animation_references(attr_value, prefix);
                }
            }
        }

        // Process style elements (simplified - no CSS parsing for now)
        if element.name == "style" {
            for child in &mut element.children {
                if let Node::Text(ref mut text) = child {
                    *text = self.process_style_content(text, prefix, config);
                }
            }
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut child_elem) = child {
                self.process_element(child_elem, prefix, config);
            }
        }
    }

    fn process_url_references(&self, value: &str, prefix: &str) -> String {
        // Match url() with double quotes, single quotes, or no quotes
        let url_double_quote = Regex::new(r#"\burl\("(#[^"]+)"\)"#).unwrap();
        let url_single_quote = Regex::new(r#"\burl\('(#[^']+)'\)"#).unwrap();
        let url_no_quote = Regex::new(r#"\burl\((#[^)]+)\)"#).unwrap();

        let mut result = value.to_string();

        // Process double-quoted URLs
        result = url_double_quote
            .replace_all(&result, |caps: &regex::Captures| {
                let url = caps.get(1).unwrap().as_str();
                if let Some(prefixed) = self.prefix_reference(prefix, url) {
                    format!(r#"url("{}")"#, prefixed)
                } else {
                    caps.get(0).unwrap().as_str().to_string()
                }
            })
            .to_string();

        // Process single-quoted URLs
        result = url_single_quote
            .replace_all(&result, |caps: &regex::Captures| {
                let url = caps.get(1).unwrap().as_str();
                if let Some(prefixed) = self.prefix_reference(prefix, url) {
                    format!(r#"url('{}')"#, prefixed)
                } else {
                    caps.get(0).unwrap().as_str().to_string()
                }
            })
            .to_string();

        // Process unquoted URLs
        result = url_no_quote
            .replace_all(&result, |caps: &regex::Captures| {
                let url = caps.get(1).unwrap().as_str();
                if let Some(prefixed) = self.prefix_reference(prefix, url) {
                    format!("url({})", prefixed)
                } else {
                    caps.get(0).unwrap().as_str().to_string()
                }
            })
            .to_string();

        result
    }

    fn process_animation_references(&self, value: &str, prefix: &str) -> String {
        let parts: Vec<String> = value
            .split(';')
            .map(|part| {
                let trimmed = part.trim();
                if trimmed.ends_with(".end") || trimmed.ends_with(".start") {
                    let mut split_parts = trimmed.split('.');
                    if let Some(id) = split_parts.next() {
                        let postfix = split_parts.collect::<Vec<_>>().join(".");
                        format!("{}.{}", self.prefix_id(prefix, id), postfix)
                    } else {
                        trimmed.to_string()
                    }
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        parts.join("; ")
    }

    fn process_style_content(
        &self,
        content: &str,
        prefix: &str,
        config: &PrefixIdsConfig,
    ) -> String {
        let mut result = content.to_string();

        // Simple patterns for ID and class selectors (without full CSS parsing)
        if config.prefix_ids {
            // Match #id selectors
            let id_regex = Regex::new(r"#([a-zA-Z][\w-]*)").unwrap();
            result = id_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let id = caps.get(1).unwrap().as_str();
                    format!("#{}", self.prefix_id(prefix, id))
                })
                .to_string();
        }

        if config.prefix_class_names {
            // Match .class selectors
            let class_regex = Regex::new(r"\.([a-zA-Z][\w-]*)").unwrap();
            result = class_regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let class = caps.get(1).unwrap().as_str();
                    format!(".{}", self.prefix_id(prefix, class))
                })
                .to_string();
        }

        // Also handle url() references in CSS
        result = self.process_url_references(&result, prefix);

        result
    }
}

impl Plugin for PrefixIdsPlugin {
    fn name(&self) -> &'static str {
        "prefixIds"
    }

    fn description(&self) -> &'static str {
        "prefix IDs"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        info: &PluginInfo,
        params: Option<&Value>,
    ) -> PluginResult<()> {
        let config = self.parse_config(params);
        let prefix = self.generate_prefix(&config, info);

        self.process_element(&mut document.root, &prefix, &config);

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::PluginInfo;
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
            prologue: vec![],
            epilogue: vec![],
            metadata: crate::ast::DocumentMetadata {
                path: None,
                encoding: None,
                version: None,
            },
        }
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = PrefixIdsPlugin::new();
        assert_eq!(plugin.name(), "prefixIds");
        assert_eq!(plugin.description(), "prefix IDs");
    }

    #[test]
    fn test_get_basename() {
        assert_eq!(
            PrefixIdsPlugin::get_basename("/path/to/file.svg"),
            "file.svg"
        );
        assert_eq!(
            PrefixIdsPlugin::get_basename("C:\\path\\to\\file.svg"),
            "file.svg"
        );
        assert_eq!(PrefixIdsPlugin::get_basename("file.svg"), "file.svg");
        assert_eq!(PrefixIdsPlugin::get_basename(""), "");
    }

    #[test]
    fn test_escape_identifier_name() {
        assert_eq!(
            PrefixIdsPlugin::escape_identifier_name("my file.svg"),
            "my_file_svg"
        );
        assert_eq!(PrefixIdsPlugin::escape_identifier_name("normal"), "normal");
    }

    #[test]
    fn test_generate_prefix() {
        let plugin = PrefixIdsPlugin::new();

        // Test with custom prefix
        let config = PrefixIdsConfig {
            prefix: Some("custom".to_string()),
            delim: "__".to_string(),
            prefix_ids: true,
            prefix_class_names: true,
        };
        let info = PluginInfo::default();
        assert_eq!(plugin.generate_prefix(&config, &info), "custom__");

        // Test with file path
        let config = PrefixIdsConfig::default();
        let info = PluginInfo {
            path: Some("/path/to/test.svg".to_string()),
            ..Default::default()
        };
        assert_eq!(plugin.generate_prefix(&config, &info), "test_svg__");

        // Test default
        let config = PrefixIdsConfig::default();
        let info = PluginInfo::default();
        assert_eq!(plugin.generate_prefix(&config, &info), "prefix__");
    }

    #[test]
    fn test_prefix_id() {
        let plugin = PrefixIdsPlugin::new();

        // Test normal prefixing
        assert_eq!(plugin.prefix_id("test__", "myid"), "test__myid");

        // Test when already prefixed
        assert_eq!(plugin.prefix_id("test__", "test__myid"), "test__myid");
    }

    #[test]
    fn test_prefix_reference() {
        let plugin = PrefixIdsPlugin::new();

        // Test valid reference
        assert_eq!(
            plugin.prefix_reference("test__", "#myid"),
            Some("#test__myid".to_string())
        );

        // Test invalid reference
        assert_eq!(plugin.prefix_reference("test__", "myid"), None);
    }

    #[test]
    fn test_apply_with_ids() {
        let mut plugin = PrefixIdsPlugin::new();
        let mut doc = create_test_document();

        // Add element with ID
        let mut attrs = IndexMap::new();
        attrs.insert("id".to_string(), "myId".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());

        // Check that ID was prefixed
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.attributes.get("id"), Some(&"prefix__myId".to_string()));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_apply_with_href() {
        let mut plugin = PrefixIdsPlugin::new();
        let mut doc = create_test_document();

        // Add element with href
        let mut attrs = IndexMap::new();
        attrs.insert("href".to_string(), "#myTarget".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "use".to_string(),
            attributes: attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, None);
        assert!(result.is_ok());

        // Check that href was prefixed
        if let Node::Element(use_elem) = &doc.root.children[0] {
            assert_eq!(
                use_elem.attributes.get("href"),
                Some(&"#prefix__myTarget".to_string())
            );
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_apply_with_custom_config() {
        let mut plugin = PrefixIdsPlugin::new();
        let mut doc = create_test_document();

        // Add element with ID
        let mut attrs = IndexMap::new();
        attrs.insert("id".to_string(), "myId".to_string());
        doc.root.children.push(Node::Element(Element {
            name: "rect".to_string(),
            attributes: attrs,
            namespaces: HashMap::new(),
            children: vec![],
        }));

        let config = serde_json::json!({
            "prefix": "custom",
            "delim": "_"
        });

        let info = PluginInfo::default();
        let result = plugin.apply(&mut doc, &info, Some(&config));
        assert!(result.is_ok());

        // Check that ID was prefixed with custom config
        if let Node::Element(rect) = &doc.root.children[0] {
            assert_eq!(rect.attributes.get("id"), Some(&"custom_myId".to_string()));
        } else {
            panic!("Expected element");
        }
    }
}
