// this_file: svgn/tests/plugins/remove_dimensions.rs

//! Tests for the removeDimensions plugin
//! Ported from SVGO test fixtures

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;

fn test_plugin(input: &str, expected: &str, params: Option<serde_json::Value>) {
    let mut config = Config {
        plugins: vec![PluginConfig {
            name: "removeDimensions".to_string(),
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
        "\nPlugin: removeDimensions\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_remove_dimensions_01() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100.5" height=".5" viewBox="0 0 100.5 .5">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100.5 .5">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_02_no_viewbox() {
    // Should not remove dimensions if there's no viewBox
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_03_only_width() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" viewBox="0 0 200 100">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_04_only_height() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" height="150" viewBox="0 0 300 150">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 150">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_05_preserve_other_attributes() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" 
         width="100" 
         height="100" 
         viewBox="0 0 100 100"
         fill="red"
         class="my-svg"
         id="svg1">
    <rect/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="red" class="my-svg" id="svg1">
    <rect/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_06_percentage_dimensions() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100%" height="50%" viewBox="0 0 400 200">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_dimensions_07_nested_svg() {
    // Should only affect root SVG element
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
    <svg width="50" height="50" viewBox="0 0 50 50">
        <rect/>
    </svg>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <svg width="50" height="50" viewBox="0 0 50 50">
        <rect/>
    </svg>
</svg>"#;

    test_plugin(input, expected, None);
}
