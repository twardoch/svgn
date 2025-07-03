// this_file: svgn/tests/plugins/remove_deprecated_attrs.rs

//! Integration tests for the removeDeprecatedAttrs plugin

use svgn::ast::Document;
use svgn::parser::Parser;
use svgn::plugins::RemoveDeprecatedAttrsPlugin;
use svgn::plugin::{Plugin, PluginInfo};
use svgn::stringifier::stringify;
use serde_json::json;

fn test_plugin(input: &str, params: Option<serde_json::Value>, expected: &str) {
    let parser = Parser::new();
    let mut document = parser.parse(input).expect("Failed to parse input");
    
    let mut plugin = RemoveDeprecatedAttrsPlugin;
    let plugin_info = PluginInfo::default();
    plugin.apply(&mut document, &plugin_info, params.as_ref())
        .expect("Plugin failed to apply");
    
    let output = stringify(&document, &Default::default())
        .expect("Failed to stringify");
    
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn test_remove_xml_lang_with_lang() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xml:lang="en" lang="en">
    <rect width="100" height="100"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" lang="en">
    <rect width="100" height="100"/>
</svg>"#;
    
    test_plugin(input, None, expected);
}

#[test]
fn test_keep_xml_lang_without_lang() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xml:lang="en">
    <rect width="100" height="100"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" xml:lang="en">
    <rect width="100" height="100"/>
</svg>"#;
    
    test_plugin(input, None, expected);
}

#[test]
fn test_remove_unsafe_core_attributes() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xml:space="preserve" xml:base="/path">
    <rect width="100" height="100"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="100"/>
</svg>"#;
    
    let params = json!({
        "removeUnsafe": true
    });
    
    test_plugin(input, Some(params), expected);
}

#[test]
fn test_remove_deprecated_presentation_attributes() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="100" enable-background="new" clip="rect(0 0 100 100)" kerning="auto"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="100"/>
</svg>"#;
    
    let params = json!({
        "removeUnsafe": true
    });
    
    test_plugin(input, Some(params), expected);
}

#[test]
fn test_keep_unsafe_attributes_by_default() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="100" enable-background="new" clip="rect(0 0 100 100)"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect width="100" height="100" enable-background="new" clip="rect(0 0 100 100)"/>
</svg>"#;
    
    test_plugin(input, None, expected);
}

#[test]
fn test_remove_attribute_type_from_animate() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <animate attributeType="XML" attributeName="x" dur="5s" values="0;100;0"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <animate attributeName="x" dur="5s" values="0;100;0"/>
</svg>"#;
    
    let params = json!({
        "removeUnsafe": true
    });
    
    test_plugin(input, Some(params), expected);
}

#[test]
fn test_remove_required_features() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <switch>
        <rect requiredFeatures="http://www.w3.org/TR/SVG11/feature#Gradient" width="100" height="100"/>
        <rect width="50" height="50"/>
    </switch>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <switch>
        <rect width="100" height="100"/>
        <rect width="50" height="50"/>
    </switch>
</svg>"#;
    
    let params = json!({
        "removeUnsafe": true
    });
    
    test_plugin(input, Some(params), expected);
}

#[test]
fn test_nested_elements() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xml:space="preserve">
    <g enable-background="new">
        <rect width="100" height="100" clip="rect(0 0 100 100)"/>
        <text kerning="auto">Hello</text>
    </g>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g>
        <rect width="100" height="100"/>
        <text>Hello</text>
    </g>
</svg>"#;
    
    let params = json!({
        "removeUnsafe": true
    });
    
    test_plugin(input, Some(params), expected);
}