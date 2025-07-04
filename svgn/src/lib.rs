// this_file: svgn/src/lib.rs

//! SVGN - A high-performance Rust port of SVGO
//!
//! This library provides SVG optimization capabilities that are API-compatible
//! with the original SVGO JavaScript library while offering significant
//! performance improvements.

pub mod ast;
pub mod collections;
pub mod config;
pub mod optimizer;
pub mod parser;
pub mod plugin;
pub mod plugins;
pub mod stringifier;

// Re-export main types
pub use ast::{Document, Element, Node};
pub use config::Config;
pub use optimizer::{optimize, optimize_default, optimize_with_config, OptimizationResult, OptimizeOptions};
pub use plugin::{Plugin, PluginConfig, PluginRegistry};

/// Library version (from git tag or Cargo.toml)
pub const VERSION: &str = env!("SVGN_VERSION");
