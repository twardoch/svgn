// this_file: svgn/src/plugins/remove_xml_proc_inst.rs

//! Remove XML processing instructions plugin
//!
//! This plugin removes XML processing instructions from SVG documents,
//! particularly the XML declaration (<?xml version="1.0" encoding="utf-8"?>).
//! Ported from ref/svgo/plugins/removeXMLProcInst.js

use crate::ast::{Document, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// Plugin that removes XML processing instructions
pub struct RemoveXMLProcInstPlugin;

impl Plugin for RemoveXMLProcInstPlugin {
    fn name(&self) -> &'static str {
        "removeXMLProcInst"
    }

    fn description(&self) -> &'static str {
        "Remove XML processing instructions"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        // Remove the XML declaration by clearing metadata
        // In SVGO, the XML declaration is treated as a processing instruction,
        // but in our parser it's stored as metadata
        document.metadata.version = None;
        document.metadata.encoding = None;

        // Also remove any actual XML processing instructions from prologue
        document.prologue.retain(|node| {
            match node {
                Node::ProcessingInstruction { target, .. } => {
                    // Remove if it's an XML processing instruction
                    target != "xml"
                }
                _ => true,
            }
        });

        // Remove XML processing instructions from epilogue (shouldn't be there, but check anyway)
        document.epilogue.retain(|node| match node {
            Node::ProcessingInstruction { target, .. } => target != "xml",
            _ => true,
        });

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_remove_xml_declaration() {
        let svg = r#"<?xml version="1.0" encoding="utf-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <rect x="10" y="10" width="80" height="80"/>
</svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        // Verify XML declaration metadata is present
        assert!(document.metadata.version.is_some());
        assert!(document.metadata.encoding.is_some());

        let mut plugin = RemoveXMLProcInstPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Verify XML declaration metadata is removed
        assert!(document.metadata.version.is_none());
        assert!(document.metadata.encoding.is_none());
    }

    #[test]
    fn test_preserve_other_pi() {
        let svg = r#"<?xml version="1.0" encoding="utf-8"?>
<?xml-stylesheet type="text/css" href="style.css"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <rect x="10" y="10" width="80" height="80"/>
</svg>"#;

        let parser = Parser::new();
        let mut document = parser.parse(svg).unwrap();

        let mut plugin = RemoveXMLProcInstPlugin;
        plugin
            .apply(&mut document, &crate::plugin::PluginInfo::default(), None)
            .unwrap();

        // Verify XML declaration is removed but other PIs are kept
        let has_xml_pi = document
            .prologue
            .iter()
            .any(|n| matches!(n, Node::ProcessingInstruction { target, .. } if target == "xml"));
        assert!(!has_xml_pi);

        let has_stylesheet_pi = document.prologue.iter().any(|n| {
            matches!(n, Node::ProcessingInstruction { target, .. } if target == "xml-stylesheet")
        });
        assert!(has_stylesheet_pi);
    }
}
