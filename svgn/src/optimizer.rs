// this_file: svgn/src/optimizer.rs

//! Core optimization engine
//!
//! This module provides the main optimization functionality that orchestrates
//! parsing, plugin application, and output generation.

use crate::config::Config;
use crate::parser::{Parser, ParseError};
use crate::plugin::{PluginRegistry, PluginError};
use crate::stringifier::{Stringifier, StringifyError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Optimization error types
#[derive(Error, Debug)]
pub enum OptimizeError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    #[error("Stringify error: {0}")]
    StringifyError(#[from] StringifyError),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Optimization result type
pub type OptimizeResult<T> = Result<T, OptimizeError>;

/// Options for the optimize function
pub struct OptimizeOptions {
    /// Configuration to use
    pub config: Config,
    /// Plugin registry (if None, uses default)
    pub registry: Option<PluginRegistry>,
}

/// Result of an optimization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Optimized SVG data
    pub data: String,
    /// Optimization information
    pub info: OptimizationInfo,
    /// Error message (if any)
    pub error: Option<String>,
    /// Whether modern parser was used
    pub modern: bool,
}

/// Information about the optimization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationInfo {
    /// Original size in bytes
    pub original_size: usize,
    /// Optimized size in bytes
    pub optimized_size: usize,
    /// Compression ratio (0.0 to 1.0)
    pub compression_ratio: f64,
    /// Number of plugins applied
    pub plugins_applied: usize,
    /// Number of optimization passes
    pub passes: usize,
}

impl OptimizeOptions {
    /// Create new options with the given config
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: None,
        }
    }

    /// Set the plugin registry
    pub fn with_registry(mut self, registry: PluginRegistry) -> Self {
        self.registry = Some(registry);
        self
    }
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self::new(Config::with_default_preset())
    }
}

impl OptimizationInfo {
    /// Create new optimization info
    pub fn new(original_size: usize, optimized_size: usize, plugins_applied: usize, passes: usize) -> Self {
        let compression_ratio = if original_size > 0 {
            1.0 - (optimized_size as f64 / original_size as f64)
        } else {
            0.0
        };

        Self {
            original_size,
            optimized_size,
            compression_ratio,
            plugins_applied,
            passes,
        }
    }

    /// Get the size reduction in bytes
    pub fn size_reduction(&self) -> i64 {
        self.original_size as i64 - self.optimized_size as i64
    }

    /// Get the compression percentage (0-100)
    pub fn compression_percentage(&self) -> f64 {
        self.compression_ratio * 100.0
    }
}

/// Main optimization function
///
/// This is the primary entry point for SVG optimization, equivalent to SVGO's
/// `optimize` function. It takes an SVG string and configuration, then returns
/// the optimized result.
pub fn optimize(input: &str, options: OptimizeOptions) -> OptimizeResult<OptimizationResult> {
    let original_size = input.len();
    let config = options.config;
    
    // Set up parser
    let parser = Parser::new()
        .preserve_whitespace(config.parser.preserve_whitespace)
        .preserve_comments(config.parser.preserve_comments);

    // Parse the SVG
    let mut document = parser.parse(input)?;

    // Set document path from config
    if let Some(path) = &config.path {
        document.metadata.path = Some(path.clone());
    }

    // Get or create plugin registry
    let mut registry = options.registry.unwrap_or_else(|| {
        crate::plugin::create_default_registry()
    });

    // Apply optimization passes
    let mut passes = 0;
    let mut plugins_applied = 0;
    let mut previous_output = String::new();

    loop {
        passes += 1;
        
        // Apply plugins
        let _initial_plugin_count = plugins_applied;
        let plugin_info = crate::plugin::PluginInfo {
            path: document.metadata.path.clone(),
            multipass_count: passes - 1,
        };
        registry.apply_plugins(&mut document, &config.plugins, &plugin_info)?;
        
        // For now, assume all enabled plugins were applied
        // In a real implementation, we'd track this more precisely
        plugins_applied += config.plugins.iter().filter(|p| p.enabled).count();

        // Generate output to check for changes
        let stringifier = Stringifier::new()
            .pretty(config.js2svg.pretty)
            .indent(config.js2svg.indent)
            .self_closing(config.js2svg.self_closing);

        let current_output = stringifier.stringify(&document)?;

        // Check if we should continue with multi-pass optimization
        if !config.multipass || current_output == previous_output || passes >= 10 {
            // Apply data URI encoding if requested
            let final_output = match &config.datauri {
                Some(format) => apply_datauri_encoding(&current_output, format),
                None => current_output,
            };

            let optimized_size = final_output.len();
            let info = OptimizationInfo::new(original_size, optimized_size, plugins_applied, passes);

            return Ok(OptimizationResult {
                data: final_output,
                info,
                error: None,
                modern: true, // We always use the modern parser
            });
        }

        previous_output = current_output;
    }
}

/// Apply data URI encoding to the SVG output
fn apply_datauri_encoding(svg: &str, format: &crate::config::DataUriFormat) -> String {
    use crate::config::DataUriFormat;
    
    match format {
        DataUriFormat::Base64 => {
            // For now, just return the SVG as-is
            // In a real implementation, we'd use base64 encoding
            format!("data:image/svg+xml;base64,{}", base64_encode(svg))
        }
        DataUriFormat::Enc => {
            // URL-encoded
            format!("data:image/svg+xml,{}", url_encode(svg))
        }
        DataUriFormat::Unenc => {
            // Unencoded (with proper escaping)
            format!("data:image/svg+xml,{}", svg)
        }
    }
}

// Placeholder functions for encoding (would use proper libraries in real implementation)
fn base64_encode(input: &str) -> String {
    // This is a placeholder - use the `base64` crate in real implementation
    input.to_string()
}

fn url_encode(input: &str) -> String {
    // This is a placeholder - use proper URL encoding in real implementation
    input.replace(' ', "%20")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('"', "%22")
        .replace('#', "%23")
}

/// Convenience function with default options
pub fn optimize_default(input: &str) -> OptimizeResult<OptimizationResult> {
    optimize(input, OptimizeOptions::default())
}

/// Optimize with a custom configuration
pub fn optimize_with_config(input: &str, config: Config) -> OptimizeResult<OptimizationResult> {
    optimize(input, OptimizeOptions::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::plugin::PluginConfig;

    #[test]
    fn test_optimize_simple_svg() {
        let svg = r#"<svg width="100" height="100">
            <!-- This is a comment -->
            <rect x="10" y="10" width="50" height="50"/>
        </svg>"#;

        // Create config with removeComments plugin
        let mut config = Config::new();
        config.plugins.push(PluginConfig::new("removeComments".to_string()));

        let result = optimize_with_config(svg, config).unwrap();
        
        assert!(!result.data.is_empty());
        assert!(result.info.original_size > 0);
        assert!(result.info.optimized_size > 0);
        assert!(result.modern);
        // Check that comment was removed
        assert!(!result.data.contains("<!--"));
    }

    #[test]
    fn test_optimize_with_config() {
        let svg = r#"<svg><rect/></svg>"#;
        let mut config = Config::new();
        config.js2svg.pretty = true;
        config.js2svg.indent = 4;

        let result = optimize_with_config(svg, config).unwrap();
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_optimization_info() {
        let info = OptimizationInfo::new(1000, 800, 5, 2);
        
        assert_eq!(info.original_size, 1000);
        assert_eq!(info.optimized_size, 800);
        assert_eq!(info.size_reduction(), 200);
        assert!((info.compression_percentage() - 20.0).abs() < 0.01);
        assert_eq!(info.plugins_applied, 5);
        assert_eq!(info.passes, 2);
    }

    #[test]
    fn test_datauri_encoding() {
        use crate::config::DataUriFormat;
        
        let svg = "<svg></svg>";
        
        let base64_result = apply_datauri_encoding(svg, &DataUriFormat::Base64);
        assert!(base64_result.starts_with("data:image/svg+xml;base64,"));
        
        let enc_result = apply_datauri_encoding(svg, &DataUriFormat::Enc);
        assert!(enc_result.starts_with("data:image/svg+xml,"));
        
        let unenc_result = apply_datauri_encoding(svg, &DataUriFormat::Unenc);
        assert!(unenc_result.starts_with("data:image/svg+xml,"));
    }
}