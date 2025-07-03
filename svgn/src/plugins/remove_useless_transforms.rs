// this_file: svgn/src/plugins/remove_useless_transforms.rs
//! Plugin to remove identity/no-op transforms from SVG elements
use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;

/// RemoveUselessTransformsPlugin
pub struct RemoveUselessTransformsPlugin;

impl Plugin for RemoveUselessTransformsPlugin {
    fn name(&self) -> &'static str {
        "removeUselessTransforms"
    }

    fn description(&self) -> &'static str {
        "Removes transform attributes that are no-op: translate(0,0), scale(1), rotate(0)"
    }

    fn apply(
        &mut self,
        document: &mut Document,
        _plugin_info: &PluginInfo,
        _params: Option<&Value>,
    ) -> PluginResult<()> {
        remove_useless_transforms_element(&mut document.root);
        Ok(())
    }
}

fn remove_useless_transforms_element(elem: &mut Element) {
    // Check and remove if transform is no-op
    if let Some(transform_str) = elem.attributes.get("transform") {
        if is_useless_transform(transform_str) {
            elem.attributes.swap_remove("transform");
        }
    }
    // Traverse children
    for child in elem.children.iter_mut() {
        if let Node::Element(ref mut e) = child {
            remove_useless_transforms_element(e);
        }
    }
}

/// Returns true if the transform string matches a no-op transform
fn is_useless_transform(s: &str) -> bool {
    let t = s.trim();

    // Check for various no-op transforms with different syntaxes
    matches!(
        t,
        "translate(0,0)"
            | "translate(0, 0)"
            | "translate(0 0)"
            | "rotate(0)"
            | "scale(1)"
            | "scale(1,1)"
            | "scale(1, 1)"
            | "scale(1 1)"
            | "skewX(0)"
            | "skewY(0)"
            | "matrix(1,0,0,1,0,0)"
            | "matrix(1, 0, 0, 1, 0, 0)"
            | "matrix(1 0 0 1 0 0)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};

    #[test]
    fn test_remove_identity_translate() {
        let mut doc = Document::new();
        let mut g = Element::new("g");
        g.attributes
            .insert("transform".to_string(), "translate(0,0)".to_string());
        doc.root.children.push(Node::Element(g));

        let mut plugin = RemoveUselessTransformsPlugin;
        let info = PluginInfo::default();
        plugin.apply(&mut doc, &info, None).unwrap();

        let g = match &doc.root.children[0] {
            Node::Element(e) => e,
            _ => panic!("Expected element"),
        };
        assert!(!g.attributes.contains_key("transform"));
    }

    #[test]
    fn test_remove_identity_scale() {
        let mut doc = Document::new();
        let mut g = Element::new("g");
        g.attributes
            .insert("transform".to_string(), "scale(1)".to_string());
        doc.root.children.push(Node::Element(g));

        let mut plugin = RemoveUselessTransformsPlugin;
        let info = PluginInfo::default();
        plugin.apply(&mut doc, &info, None).unwrap();

        let g = match &doc.root.children[0] {
            Node::Element(e) => e,
            _ => panic!("Expected element"),
        };
        assert!(!g.attributes.contains_key("transform"));
    }

    #[test]
    fn test_preserve_non_identity_transform() {
        let mut doc = Document::new();
        let mut g = Element::new("g");
        g.attributes
            .insert("transform".to_string(), "translate(10,20)".to_string());
        doc.root.children.push(Node::Element(g));

        let mut plugin = RemoveUselessTransformsPlugin;
        let info = PluginInfo::default();
        plugin.apply(&mut doc, &info, None).unwrap();

        let g = match &doc.root.children[0] {
            Node::Element(e) => e,
            _ => panic!("Expected element"),
        };
        assert_eq!(
            g.attributes.get("transform"),
            Some(&"translate(10,20)".to_string())
        );
    }

    #[test]
    fn test_is_useless_transform() {
        assert!(is_useless_transform("translate(0,0)"));
        assert!(is_useless_transform("translate(0, 0)"));
        assert!(is_useless_transform("translate(0 0)"));
        assert!(is_useless_transform("rotate(0)"));
        assert!(is_useless_transform("scale(1)"));
        assert!(is_useless_transform("scale(1,1)"));
        assert!(is_useless_transform("scale(1, 1)"));
        assert!(is_useless_transform("skewX(0)"));
        assert!(is_useless_transform("skewY(0)"));
        assert!(is_useless_transform("matrix(1 0 0 1 0 0)"));
        assert!(is_useless_transform(" translate(0,0) "));

        assert!(!is_useless_transform("translate(10,0)"));
        assert!(!is_useless_transform("translate(0,10)"));
        assert!(!is_useless_transform("rotate(45)"));
        assert!(!is_useless_transform("scale(2)"));
        assert!(!is_useless_transform("scale(1,2)"));
    }
}
