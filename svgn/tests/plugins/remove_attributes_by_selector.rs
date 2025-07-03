// this_file: svgn/tests/plugins/remove_attributes_by_selector.rs

//! Integration tests for the removeAttributesBySelector plugin

use svgn::ast::Document;
use svgn::parser::Parser;
use svgn::plugins::RemoveAttributesBySelectorPlugin;
use svgn::plugin::{Plugin, PluginInfo};
use svgn::stringifier::stringify;
use serde_json::json;

fn test_plugin(input: &str, params: serde_json::Value, expected: &str) {
    let parser = Parser::new();
    let mut document = parser.parse(input).expect("Failed to parse input");
    
    let mut plugin = RemoveAttributesBySelectorPlugin;
    let plugin_info = PluginInfo::default();
    plugin.apply(&mut document, &plugin_info, Some(&params))
        .expect("Plugin failed to apply");
    
    let output = stringify(&document, &Default::default())
        .expect("Failed to stringify");
    
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn test_single_attribute_removal() {
    let input = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="0" y="0" width="100" height="100" fill="#00ff00" stroke="#00ff00"/>
</svg>"#;
    
    let expected = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="0" y="0" width="100" height="100" stroke="#00ff00"/>
</svg>"#;
    
    let params = json!({
        "selector": "[fill='#00ff00']",
        "attributes": "fill"
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_multiple_attributes_removal() {
    let input = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="0" y="0" width="100" height="100" fill="#00ff00" stroke="#00ff00"/>
</svg>"#;
    
    let expected = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="0" y="0" width="100" height="100"/>
</svg>"#;
    
    let params = json!({
        "selector": "[fill='#00ff00']",
        "attributes": ["fill", "stroke"]
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_multiple_selectors() {
    let input = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect id="remove" x="0" y="0" width="100" height="100" fill="#00ff00" stroke="#00ff00"/>
</svg>"#;
    
    let expected = r#"<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="0" y="0" width="100" height="100"/>
</svg>"#;
    
    let params = json!({
        "selectors": [
            { "selector": "[fill='#00ff00']", "attributes": "fill" },
            { "selector": "#remove", "attributes": ["stroke", "id"] }
        ]
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_class_selector() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect class="remove-me" x="0" y="0" width="100" height="100" fill="red"/>
    <rect class="keep-me" x="0" y="0" width="100" height="100" fill="blue"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect class="remove-me" x="0" y="0" width="100" height="100"/>
    <rect class="keep-me" x="0" y="0" width="100" height="100" fill="blue"/>
</svg>"#;
    
    let params = json!({
        "selector": ".remove-me",
        "attributes": "fill"
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_element_selector() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="0" y="0" width="100" height="100" fill="red"/>
    <circle cx="50" cy="50" r="25" fill="blue"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="0" y="0" width="100" height="100"/>
    <circle cx="50" cy="50" r="25" fill="blue"/>
</svg>"#;
    
    let params = json!({
        "selector": "rect",
        "attributes": "fill"
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_attribute_contains_selector() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="0" y="0" width="100" height="100" fill="red" class="shape primary"/>
    <rect x="0" y="0" width="100" height="100" fill="blue" class="shape secondary"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="0" y="0" width="100" height="100" class="shape primary"/>
    <rect x="0" y="0" width="100" height="100" fill="blue" class="shape secondary"/>
</svg>"#;
    
    let params = json!({
        "selector": "[class~='primary']",
        "attributes": "fill"
    });
    
    test_plugin(input, params, expected);
}

#[test]
fn test_nested_elements() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g>
        <rect id="target" x="0" y="0" width="100" height="100" fill="red" stroke="black"/>
    </g>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g>
        <rect id="target" x="0" y="0" width="100" height="100"/>
    </g>
</svg>"#;
    
    let params = json!({
        "selector": "#target",
        "attributes": ["fill", "stroke"]
    });
    
    test_plugin(input, params, expected);
}