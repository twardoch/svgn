// this_file: svgn/tests/plugins/remove_empty_attrs.rs

//! Tests for the removeEmptyAttrs plugin
//! Ported from SVGO test fixtures

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;

fn test_plugin(input: &str, expected: &str, params: Option<serde_json::Value>) {
    let mut config = Config {
        plugins: vec![PluginConfig {
            name: "removeEmptyAttrs".to_string(),
            params,
        }],
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: true,
            indent: 4,
            ..Default::default()
        },
        ..Default::default()
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(input, options).expect("Optimization should succeed");
    let output = result.data.trim();
    let expected = expected.trim();
    
    assert_eq!(output, expected, 
        "\nPlugin: removeEmptyAttrs\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_remove_empty_attrs_01() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g attr1="" attr2=""/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_empty_attrs_02_mixed_empty_and_valid() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="10" y="" width="50" height="" fill="red" stroke=""/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <rect x="10" width="50" fill="red"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_empty_attrs_03_preserve_xmlns() {
    // xmlns attributes should never be removed even if empty
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="">
    <g/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="">
    <g/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_empty_attrs_04_whitespace_only() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g class="   " id="\t\n" style="  "/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_empty_attrs_05_preserve_valid_whitespace_attributes() {
    // Some attributes can legitimately be whitespace-only
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <text xml:space="preserve"> </text>
    <tspan> </tspan>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <text xml:space="preserve"> </text>
    <tspan> </tspan>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_empty_attrs_06_nested_elements() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g transform="" opacity="">
        <rect x="" y="10" width="50" height="30"/>
        <circle cx="25" cy="" r=""/>
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g>
        <rect y="10" width="50" height="30"/>
        <circle cx="25"/>
    </g>
</svg>"#;

    test_plugin(input, expected, None);
}
