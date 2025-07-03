// this_file: svgn/src/plugin.rs

//! Plugin system for SVG optimization
//!
//! This module defines the plugin trait and infrastructure for applying
//! optimization transformations to SVG documents.

use crate::ast::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin operations
#[derive(Debug)]
pub enum PluginError {
    /// Invalid configuration parameter
    InvalidConfig(String),
    /// Processing error
    ProcessingError(String),
    /// I/O error
    IoError(std::io::Error),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            PluginError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            PluginError::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::IoError(err)
    }
}

/// Information passed to plugins during optimization
#[derive(Default)]
pub struct PluginInfo {
    /// Path to the current SVG file (if available)
    pub path: Option<String>,
    /// Current multipass count (0-indexed)
    pub multipass_count: usize,
}

/// Plugin trait that all optimization plugins must implement
pub trait Plugin: Send + Sync {
    /// Plugin name (must be unique)
    fn name(&self) -> &'static str;
    
    /// Plugin description
    fn description(&self) -> &'static str;
    
    /// Apply the plugin transformation to the document
    fn apply(&mut self, document: &mut Document, plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()>;
    
    /// Check if the plugin should be applied based on the document
    /// Default implementation always returns true
    fn should_apply(&self, _document: &Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> bool {
        true
    }
    
    /// Validate plugin parameters
    /// Default implementation accepts any parameters
    fn validate_params(&self, _params: Option<&Value>) -> PluginResult<()> {
        Ok(())
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Plugin parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Whether the plugin is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl PluginConfig {
    /// Create a new plugin configuration
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: None,
            enabled: true,
        }
    }

    /// Create a new plugin configuration with parameters
    pub fn with_params(name: String, params: Value) -> Self {
        Self {
            name,
            params: Some(params),
            enabled: true,
        }
    }

    /// Disable this plugin
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Plugin registry for managing available plugins
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Get a plugin by name (mutable)
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        for plugin in &mut self.plugins {
            if plugin.name() == name {
                return Some(plugin.as_mut());
            }
        }
        None
    }

    /// Get all registered plugin names
    pub fn plugin_names(&self) -> Vec<&'static str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Apply a list of plugin configurations to a document
    pub fn apply_plugins(
        &mut self,
        document: &mut Document,
        configs: &[PluginConfig],
        plugin_info: &PluginInfo,
    ) -> PluginResult<()> {
        for config in configs {
            if !config.enabled {
                continue;
            }

            let plugin = self.get_mut(&config.name).ok_or_else(|| {
                PluginError::InvalidConfig(format!("Unknown plugin: {}", config.name))
            })?;

            // Validate parameters
            plugin.validate_params(config.params.as_ref())?;

            // Check if plugin should be applied
            if !plugin.should_apply(document, plugin_info, config.params.as_ref()) {
                continue;
            }

            // Apply the plugin
            plugin.apply(document, plugin_info, config.params.as_ref())?;
        }

        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the default plugin registry with all built-in plugins
pub fn create_default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    
    // Register built-in plugins
    registry.register(crate::plugins::CleanupAttrsPlugin);
    registry.register(crate::plugins::CleanupEnableBackgroundPlugin);
    registry.register(crate::plugins::CleanupIdsPlugin);
    registry.register(crate::plugins::CleanupListOfValuesPlugin);
    registry.register(crate::plugins::CleanupNumericValuesPlugin);
    registry.register(crate::plugins::RemoveCommentsPlugin);
    registry.register(crate::plugins::RemoveDescPlugin);
    registry.register(crate::plugins::RemoveDoctypePlugin);
    registry.register(crate::plugins::RemoveEmptyAttrsPlugin);
    registry.register(crate::plugins::RemoveEmptyContainersPlugin);
    registry.register(crate::plugins::RemoveEmptyTextPlugin);
    registry.register(crate::plugins::RemoveAttrsPlugin);
    registry.register(crate::plugins::RemoveMetadataPlugin);
    registry.register(crate::plugins::RemoveTitlePlugin);
    registry.register(crate::plugins::RemoveUnknownsAndDefaultsPlugin);
    registry.register(crate::plugins::RemoveXMLProcInstPlugin);
    registry.register(crate::plugins::SortAttrsPlugin);
    registry.register(crate::plugins::RemoveStyleElement);
    registry.register(crate::plugins::MergeStylesPlugin);
    registry.register(crate::plugins::ConvertStyleToAttrsPlugin);
    registry.register(crate::plugins::ConvertColorsPlugin);
    registry.register(crate::plugins::AddAttributesToSVGElementPlugin);
    registry.register(crate::plugins::AddClassesToSVGElementPlugin);
    // registry.register(crate::plugins::RemoveAttributesBySelectorPlugin); // TODO: Fix CSS selector parsing
    registry.register(crate::plugins::RemoveDeprecatedAttrsPlugin);
    registry.register(crate::plugins::ConvertEllipseToCirclePlugin);
    registry.register(crate::plugins::CollapseGroupsPlugin);
    registry.register(crate::plugins::ConvertOneStopGradientsPlugin);
    registry.register(crate::plugins::PrefixIdsPlugin);
    registry.register(crate::plugins::RemoveEditorsNSDataPlugin);
    registry.register(crate::plugins::RemoveElementsByAttrPlugin);
    registry.register(crate::plugins::RemoveDimensionsPlugin);
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Test plugin for unit tests
    struct TestPlugin {
        name: &'static str,
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }

        fn description(&self) -> &'static str {
            "Test plugin"
        }

        fn apply(&mut self, _document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.register(TestPlugin { name: "test" });

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
        assert_eq!(registry.plugin_names(), vec!["test"]);
    }

    #[test]
    fn test_plugin_config() {
        let config = PluginConfig::new("test".to_string());
        assert_eq!(config.name, "test");
        assert!(config.enabled);
        assert!(config.params.is_none());

        let config_with_params = PluginConfig::with_params(
            "test".to_string(),
            json!({"option": "value"})
        );
        assert!(config_with_params.params.is_some());

        let disabled_config = PluginConfig::new("test".to_string()).disabled();
        assert!(!disabled_config.enabled);
    }

    #[test]
    fn test_apply_plugins() {
        let mut registry = PluginRegistry::new();
        registry.register(TestPlugin { name: "test" });

        let mut document = Document::new();
        let configs = vec![PluginConfig::new("test".to_string())];

        let plugin_info = PluginInfo {
            path: None,
            multipass_count: 0,
        };
        let result = registry.apply_plugins(&mut document, &configs, &plugin_info);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_unknown_plugin() {
        let mut registry = PluginRegistry::new();
        let mut document = Document::new();
        let configs = vec![PluginConfig::new("unknown".to_string())];

        let plugin_info = PluginInfo {
            path: None,
            multipass_count: 0,
        };
        let result = registry.apply_plugins(&mut document, &configs, &plugin_info);
        assert!(result.is_err());
    }
}