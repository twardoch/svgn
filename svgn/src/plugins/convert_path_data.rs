// this_file: svgn/src/plugins/convert_path_data.rs

//! Plugin stub for convertPathData (SVGO parity)
//!
//! This plugin is responsible for advanced path data normalization and simplification.
//! It is currently only implemented as a stub. Full functionality to come.

use crate::ast::Document;
use crate::plugin::{Plugin, PluginInfo, PluginResult, PluginError};
use serde_json::Value;

/// Plugin for `convertPathData` (not yet implemented)
pub struct ConvertPathDataPlugin;

impl Plugin for ConvertPathDataPlugin {
    fn name(&self) -> &'static str {
        "convertPathData"
    }

    fn description(&self) -> &'static str {
        "(stub) Converts path data to relative or absolute, optimizes segments, simplifies curves – Not yet implemented"
    }

    fn apply(&mut self, _document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        Err(PluginError::ProcessingError(
            "convertPathData plugin not yet implemented in svgn. You may disable it to proceed, or watch for updates.".to_string()
        ))
    }
}