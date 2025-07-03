// this_file: src/lib.rs

//! SVGN - A native Rust port of SVGO (SVG Optimizer)
//!
//! This library provides SVG optimization functionality that is API-compatible
//! with SVGO v4.0.0. It includes a plugin-based architecture for applying
//! various optimizations to SVG files.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main optimization result returned by the optimize function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResult {
    /// The optimized SVG string
    pub data: String,
    /// Information about the optimization process
    pub info: OptimizeInfo,
    /// Error message if optimization failed
    pub error: Option<String>,
    /// Whether the modern parser was used
    pub modern: bool,
}

/// Information about the optimization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeInfo {
    /// Original file size in bytes
    pub original_size: usize,
    /// Optimized file size in bytes
    pub optimized_size: usize,
    /// Compression ratio (0.0 to 1.0)
    pub compression_ratio: f64,
}

/// Configuration for the optimization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the file being optimized
    pub path: Option<String>,
    /// Array of plugin configurations
    pub plugins: Vec<PluginConfig>,
    /// Whether to run optimizations multiple times
    pub multipass: bool,
    /// Options for JS to SVG conversion
    pub js2svg: Js2SvgOptions,
    /// Output as data URI
    pub datauri: Option<String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginConfig {
    /// Simple plugin name
    Name(String),
    /// Plugin with parameters
    WithParams {
        name: String,
        params: HashMap<String, serde_json::Value>,
    },
}

/// Options for JS to SVG conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Js2SvgOptions {
    /// Whether to pretty-print the output
    pub pretty: bool,
    /// Indentation level for pretty-printing
    pub indent: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            plugins: vec![PluginConfig::Name("preset-default".to_string())],
            multipass: false,
            js2svg: Js2SvgOptions::default(),
            datauri: None,
        }
    }
}

impl Default for Js2SvgOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: 2,
        }
    }
}

/// The main optimization function
///
/// This function takes an SVG string and a configuration object, and returns
/// an optimized SVG with metadata about the optimization process.
///
/// # Arguments
///
/// * `input` - The SVG string to optimize
/// * `config` - Optional configuration for the optimization process
///
/// # Returns
///
/// An `OptimizeResult` containing the optimized SVG and metadata
///
/// # Example
///
/// ```rust
/// use svgn::{optimize, Config};
///
/// let svg = r#"<svg width="100" height="100"><rect x="10" y="10" width="80" height="80" fill="red"/></svg>"#;
/// let config = Config::default();
/// let result = optimize(svg, Some(config)).unwrap();
/// println!("Optimized SVG: {}", result.data);
/// ```
pub fn optimize(input: &str, config: Option<Config>) -> Result<OptimizeResult> {
    let _config = config.unwrap_or_default();
    let original_size = input.len();
    
    // TODO: Implement actual optimization logic
    // For now, just return the input unchanged
    let optimized_data = input.to_string();
    let optimized_size = optimized_data.len();
    
    let compression_ratio = if original_size > 0 {
        1.0 - (optimized_size as f64 / original_size as f64)
    } else {
        0.0
    };
    
    Ok(OptimizeResult {
        data: optimized_data,
        info: OptimizeInfo {
            original_size,
            optimized_size,
            compression_ratio,
        },
        error: None,
        modern: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_basic() {
        let svg = r#"<svg width="100" height="100"><rect x="10" y="10" width="80" height="80" fill="red"/></svg>"#;
        let result = optimize(svg, None).unwrap();
        assert_eq!(result.data, svg);
        assert_eq!(result.info.original_size, svg.len());
        assert!(result.error.is_none());
        assert!(result.modern);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.multipass);
        assert!(!config.js2svg.pretty);
        assert_eq!(config.js2svg.indent, 2);
        assert_eq!(config.plugins.len(), 1);
    }
}