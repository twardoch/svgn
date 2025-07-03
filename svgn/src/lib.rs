// this_file: svgn/src/lib.rs

//! SVGN - A high-performance Rust port of SVGO
//!
//! This library provides SVG optimization capabilities that are API-compatible
//! with the original SVGO JavaScript library while offering significant
//! performance improvements.

pub mod ast;
pub mod plugin;
pub mod plugins;
pub mod parser;
pub mod stringifier;
pub mod config;
pub mod optimizer;
pub mod collections;

// Re-export main types
pub use ast::{Document, Element, Node};
pub use config::Config;
pub use optimizer::{optimize, optimize_with_config, OptimizeOptions, OptimizationResult};
pub use plugin::{Plugin, PluginConfig, PluginRegistry};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
