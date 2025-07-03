// this_file: svgn/tests/plugins/convert_ellipse_to_circle.rs

//! Integration tests for the convertEllipseToCircle plugin

use svgn::ast::Document;
use svgn::parser::Parser;
use svgn::plugins::ConvertEllipseToCirclePlugin;
use svgn::plugin::{Plugin, PluginInfo};
use svgn::stringifier::stringify;

fn test_plugin(input: &str, expected: &str) {
    let parser = Parser::new();
    let mut document = parser.parse(input).expect("Failed to parse input");
    
    let mut plugin = ConvertEllipseToCirclePlugin;
    let plugin_info = PluginInfo::default();
    plugin.apply(&mut document, &plugin_info, None)
        .expect("Plugin failed to apply");
    
    let output = stringify(&document, &Default::default())
        .expect("Failed to stringify");
    
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn test_convert_equal_radii() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="50" cy="50" rx="25" ry="25" fill="red"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <circle cx="50" cy="50" r="25" fill="red"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_keep_unequal_radii() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="50" cy="50" rx="30" ry="20" fill="blue"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="50" cy="50" rx="30" ry="20" fill="blue"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_convert_with_auto_rx() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="100" cy="100" rx="auto" ry="40" stroke="black" stroke-width="2"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <circle cx="100" cy="100" r="40" stroke="black" stroke-width="2"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_convert_with_auto_ry() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="100" cy="100" rx="35" ry="auto"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <circle cx="100" cy="100" r="35"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_multiple_ellipses() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="30" cy="30" rx="20" ry="20" fill="red"/>
    <ellipse cx="70" cy="30" rx="25" ry="15" fill="green"/>
    <ellipse cx="50" cy="70" rx="auto" ry="20" fill="blue"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <circle cx="30" cy="30" r="20" fill="red"/>
    <ellipse cx="70" cy="30" rx="25" ry="15" fill="green"/>
    <circle cx="50" cy="70" r="20" fill="blue"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_nested_groups() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g transform="translate(50,50)">
        <ellipse cx="0" cy="0" rx="10" ry="10"/>
        <g>
            <ellipse cx="20" cy="20" rx="5" ry="5" opacity="0.5"/>
        </g>
    </g>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g transform="translate(50,50)">
        <circle cx="0" cy="0" r="10"/>
        <g>
            <circle cx="20" cy="20" r="5" opacity="0.5"/>
        </g>
    </g>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_missing_attributes() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <ellipse cx="50" cy="50"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <circle cx="50" cy="50" r="0"/>
</svg>"#;
    
    test_plugin(input, expected);
}

#[test]
fn test_preserve_other_elements() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="10" y="10" width="30" height="30" fill="yellow"/>
    <ellipse cx="50" cy="50" rx="20" ry="20" fill="red"/>
    <path d="M 10 10 L 90 90" stroke="black"/>
</svg>"#;
    
    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="10" y="10" width="30" height="30" fill="yellow"/>
    <circle cx="50" cy="50" r="20" fill="red"/>
    <path d="M 10 10 L 90 90" stroke="black"/>
</svg>"#;
    
    test_plugin(input, expected);
}