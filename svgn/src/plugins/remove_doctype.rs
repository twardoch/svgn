// this_file: svgn/src/plugins/remove_doctype.rs

//! Remove DOCTYPE plugin
//!
//! This plugin removes DOCTYPE declarations from SVG documents.
//! DOCTYPE declarations are not recommended for SVG documents as they
//! can be a source of issues and are not required.
//! Ported from ref/svgo/plugins/removeDoctype.js

use crate::ast::Document;
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that removes DOCTYPE declarations
pub struct RemoveDoctypePlugin;

impl Plugin for RemoveDoctypePlugin {
    fn name(&self) -> &'static str {
        "removeDoctype"
    }
    
    fn description(&self) -> &'static str {
        "Remove DOCTYPE declaration"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        // Remove DOCTYPE from prologue
        document.prologue.retain(|node| !node.is_doctype());
        
        // DOCTYPE shouldn't appear in epilogue, but check anyway
        document.epilogue.retain(|node| !node.is_doctype());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_remove_doctype() {
        let svg = r#"<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <rect x="10" y="10" width="80" height="80"/>
</svg>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        // Verify DOCTYPE is present
        assert!(document.prologue.iter().any(|n| n.is_doctype()));
        
        let mut plugin = RemoveDoctypePlugin;
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
        
        // Verify DOCTYPE is removed
        assert!(!document.prologue.iter().any(|n| n.is_doctype()));
    }
    
    #[test]
    fn test_empty_document() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#;
        
        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();
        
        let mut plugin = RemoveDoctypePlugin;
        // Should not fail on documents without DOCTYPE
        plugin.apply(&mut document, &crate::plugin::PluginInfo::default(), None).unwrap();
    }
}