// this_file: svgn/src/config.rs

//! Configuration handling for SVGN
//!
//! This module provides structures and functions for handling configuration
//! compatible with SVGO's configuration format.

use crate::plugin::PluginConfig;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;
use thiserror::Error;

/// Configuration error types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerError(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Configuration result type
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Path to the file being processed (for context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Plugin configurations
    #[serde(default, deserialize_with = "deserialize_plugins")]
    pub plugins: Vec<PluginConfig>,

    /// Multi-pass optimization
    #[serde(default)]
    pub multipass: bool,

    /// Output formatting options
    #[serde(default)]
    pub js2svg: Js2SvgOptions,

    /// Data URI output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datauri: Option<DataUriFormat>,

    /// Parser options
    #[serde(default)]
    pub parser: ParserOptions,
}

/// Output formatting options (equivalent to SVGO's js2svg)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Js2SvgOptions {
    /// Pretty-print the output
    #[serde(default)]
    pub pretty: bool,

    /// Indentation for pretty-printing (number of spaces)
    #[serde(default = "default_indent")]
    pub indent: usize,

    /// Use self-closing tags for empty elements
    #[serde(default = "default_true")]
    pub self_closing: bool,

    /// Quote attributes (always, never, auto)
    #[serde(default = "default_quote_attrs")]
    pub quote_attrs: QuoteAttrsStyle,
    
    /// Line ending style
    #[serde(default)]
    pub eol: LineEnding,
    
    /// Ensure final newline
    #[serde(default)]
    pub final_newline: bool,
}

/// Data URI output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataUriFormat {
    /// Base64 encoded
    Base64,
    /// URL encoded
    Enc,
    /// Unencoded
    Unenc,
}

/// Attribute quoting styles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuoteAttrsStyle {
    /// Always quote attributes
    Always,
    /// Never quote attributes (when possible)
    Never,
    /// Automatically decide based on content
    Auto,
}

/// Line ending style
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// Unix line endings (\n)
    Lf,
    /// Windows line endings (\r\n)
    Crlf,
}

impl Default for LineEnding {
    fn default() -> Self {
        #[cfg(windows)]
        return LineEnding::Crlf;
        #[cfg(not(windows))]
        return LineEnding::Lf;
    }
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

/// Parser configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserOptions {
    /// Preserve whitespace in text content
    #[serde(default)]
    pub preserve_whitespace: bool,

    /// Preserve comments
    #[serde(default)]
    pub preserve_comments: bool,
}

// Default value functions for serde
fn default_indent() -> usize {
    2
}
fn default_true() -> bool {
    true
}
fn default_quote_attrs() -> QuoteAttrsStyle {
    QuoteAttrsStyle::Auto
}

impl Default for Js2SvgOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: 2,
            self_closing: true,
            quote_attrs: QuoteAttrsStyle::Auto,
            eol: LineEnding::default(),
            final_newline: false,
        }
    }
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            preserve_whitespace: false,
            preserve_comments: true, // Must be true for removeComments plugin to work
        }
    }
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => Self::from_json(&content),
            Some("toml") => Self::from_toml(&content),
            Some("js") | Some("cjs") | Some("mjs") => {
                // For JavaScript config files, we would need to execute them
                // For now, return an error suggesting JSON or TOML
                Err(ConfigError::InvalidConfig(
                    "JavaScript config files not yet supported. Please use JSON or TOML format."
                        .to_string(),
                ))
            }
            _ => {
                // Try JSON first, then TOML
                Self::from_json(&content).or_else(|_| Self::from_toml(&content))
            }
        }
    }

    /// Load configuration from JSON string
    pub fn from_json(json: &str) -> ConfigResult<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Load configuration from TOML string
    pub fn from_toml(toml: &str) -> ConfigResult<Self> {
        Ok(toml::from_str(toml)?)
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> ConfigResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Convert to TOML string
    pub fn to_toml(&self) -> ConfigResult<String> {
        toml::to_string(self).map_err(|e| ConfigError::TomlSerError(e.to_string()))
    }

    /// Add a plugin configuration
    pub fn add_plugin(&mut self, plugin: PluginConfig) {
        self.plugins.push(plugin);
    }

    /// Remove a plugin by name
    pub fn remove_plugin(&mut self, name: &str) {
        self.plugins.retain(|p| p.name != name);
    }

    /// Get a plugin configuration by name
    pub fn get_plugin(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.iter().find(|p| p.name == name)
    }

    /// Get a mutable plugin configuration by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut PluginConfig> {
        self.plugins.iter_mut().find(|p| p.name == name)
    }

    /// Enable or disable a plugin
    pub fn set_plugin_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(plugin) = self.get_plugin_mut(name) {
            plugin.enabled = enabled;
        }
    }

    /// Create a config with the default preset
    pub fn with_default_preset() -> Self {
        let mut config = Self::new();

        // Add default plugins (excluding unimplemented complex plugins for now)
        let default_plugins = vec![
            "removeComments",
            "removeMetadata",
            "removeTitle",
            "removeDesc",
            "removeDoctype",
            "removeXMLProcInst",
            "removeEditorsNSData",
            "cleanupAttrs",
            "removeEmptyAttrs",
            "removeUnknownsAndDefaults",
            "removeUnusedNS",
            "removeUselessDefs",
            "cleanupIds",
            "minifyStyles",
            "convertStyleToAttrs",
            "convertColors",
            // TODO: Implement complex plugins:
            // "convertPathData",    // Requires lyon integration
            // "convertTransform",   // Requires matrix math
            // "mergePaths",         // Requires path analysis
            // "moveElemsAttrsToGroup", // Requires DOM analysis
            // "moveGroupAttrsToElems", // Requires DOM analysis
            // "inlineStyles",       // Requires CSS parsing
            "removeEmptyText",
            "removeEmptyContainers",
            "collapseGroups",
            "removeUselessTransforms",
            "sortAttrs",
        ];

        for plugin_name in default_plugins {
            config.add_plugin(PluginConfig::new(plugin_name.to_string()));
        }

        config
    }
}

/// Load configuration from common file names in the given directory
pub fn load_config_from_directory<P: AsRef<Path>>(dir: P) -> ConfigResult<Option<Config>> {
    let dir = dir.as_ref();

    // Common config file names (in order of preference)
    let config_names = [
        "svgn.config.toml",
        "svgn.config.json",
        "svgo.config.json",
        "svgo.config.js",
        "svgo.config.cjs",
        "svgo.config.mjs",
    ];

    for name in &config_names {
        let path = dir.join(name);
        if path.exists() {
            return Ok(Some(Config::from_file(path)?));
        }
    }

    Ok(None)
}

/// Custom deserializer for plugins that can handle both string and object formats
fn deserialize_plugins<'de, D>(deserializer: D) -> Result<Vec<PluginConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use serde_json::Value;

    struct PluginsVisitor;

    impl<'de> Visitor<'de> for PluginsVisitor {
        type Value = Vec<PluginConfig>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an array of plugin names or plugin config objects")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut plugins = Vec::new();

            while let Some(value) = seq.next_element::<Value>()? {
                let plugin = match value {
                    Value::String(name) => PluginConfig::new(name),
                    Value::Object(_) => serde_json::from_value(value).map_err(de::Error::custom)?,
                    _ => return Err(de::Error::custom("Invalid plugin format")),
                };
                plugins.push(plugin);
            }

            Ok(plugins)
        }
    }

    deserializer.deserialize_seq(PluginsVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert!(config.plugins.is_empty());
        assert!(!config.multipass);
        assert!(!config.js2svg.pretty);
        assert_eq!(config.js2svg.indent, 2);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::new();
        config.multipass = true;
        config.js2svg.pretty = true;
        config.add_plugin(PluginConfig::new("removeComments".to_string()));

        let json = config.to_json().unwrap();
        let deserialized: Config = Config::from_json(&json).unwrap();

        assert_eq!(config.multipass, deserialized.multipass);
        assert_eq!(config.js2svg.pretty, deserialized.js2svg.pretty);
        assert_eq!(config.plugins.len(), deserialized.plugins.len());
    }

    #[test]
    fn test_plugin_management() {
        let mut config = Config::new();

        config.add_plugin(PluginConfig::new("test".to_string()));
        assert_eq!(config.plugins.len(), 1);
        assert!(config.get_plugin("test").is_some());

        config.set_plugin_enabled("test", false);
        assert!(!config.get_plugin("test").unwrap().enabled);

        config.remove_plugin("test");
        assert_eq!(config.plugins.len(), 0);
    }

    #[test]
    fn test_default_preset() {
        let config = Config::with_default_preset();
        assert!(!config.plugins.is_empty());
        assert!(config.get_plugin("removeComments").is_some());
        assert!(config.get_plugin("removeMetadata").is_some());
    }

    #[test]
    fn test_json_parsing() {
        let json = json!({
            "multipass": true,
            "plugins": [
                "removeComments",
                {
                    "name": "sortAttrs",
                    "params": {
                        "xmlnsOrder": "alphabetical"
                    }
                }
            ],
            "js2svg": {
                "pretty": true,
                "indent": 4
            }
        });

        let config: Config = serde_json::from_value(json).unwrap();
        assert!(config.multipass);
        assert_eq!(config.plugins.len(), 2);
        assert!(config.js2svg.pretty);
        assert_eq!(config.js2svg.indent, 4);
    }
}
