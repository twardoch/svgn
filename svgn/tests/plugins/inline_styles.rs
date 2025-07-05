// this_file: svgn/tests/plugins/inline_styles.rs
//! Minimal test for inlineStyles plugin (class selector matches)

use svgn::{optimize_with_config, Config, PluginConfig};

fn run_inline_styles_test(input: &str, expected: &str) {
    let mut config = Config::new();
    config.js2svg.pretty = true;
    config.js2svg.indent = 4;
    let mut plugin_config = PluginConfig::new("inlineStyles".to_string());
    config.plugins.push(plugin_config);
    let result = optimize_with_config(input, config).expect("Optimization should succeed");
    let output = result.data.trim();
    let expected = expected.trim();
    assert_eq!(output, expected, "\nPlugin: inlineStyles\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", input, expected, output);
}

#[test]
fn class_selector_basic_blue_fill() {
    let input = r#"
<svg id='test' xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
    <style>
        .st0{fill:blue;}
    </style>
    <rect width='100' height='100' class='st0'/>
</svg>
"#;
    let expected = r#"
<svg id="test" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <rect width="100" height="100" style="fill:blue"/>
</svg>
"#;
    run_inline_styles_test(input, expected);
}
