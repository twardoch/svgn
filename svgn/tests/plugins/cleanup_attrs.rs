// this_file: svgn/tests/plugins/cleanup_attrs.rs

//! Tests for the cleanupAttrs plugin
//! Ported from SVGO test fixtures

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;

fn test_plugin(input: &str, expected: &str, params: Option<serde_json::Value>) {
    let mut config = Config {
        plugins: vec![PluginConfig {
            name: "cleanupAttrs".to_string(),
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
        "\nPlugin: cleanupAttrs\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_cleanup_attrs_01() {
    let input = r#"<svg xmlns="  http://www.w3.org/2000/svg
  " attr="a      b" attr2="a
b">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" attr="a b" attr2="a b">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_attrs_02_mixed_whitespace() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" 
     class="  class1   class2  "
     id=" myid "
     style="  color: red;  display: block;  ">
    <g fill="   #ff0000   "/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" class="class1 class2" id="myid" style="color: red; display: block;">
    <g fill="#ff0000"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_attrs_03_preserve_needed_spaces() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <text>   Some text with   spaces   </text>
    <rect data-value="a   b   c"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <text>Some text with spaces</text>
    <rect data-value="a b c"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_attrs_04_newlines_and_tabs() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg"
     viewBox="0\t0\n100\t100">
    <rect x="10\n20" y="\t30\t40\n"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect x="10 20" y="30 40"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_attrs_05_multiple_consecutive_spaces() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g transform="translate(10,     20)   scale(2      2)">
        <rect class="     foo     bar     baz     "/>
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g transform="translate(10, 20) scale(2 2)">
        <rect class="foo bar baz"/>
    </g>
</svg>"#;

    test_plugin(input, expected, None);
}
