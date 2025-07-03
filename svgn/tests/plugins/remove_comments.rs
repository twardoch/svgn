// this_file: svgn/tests/plugins/remove_comments.rs

//! Tests for the removeComments plugin
//! Ported from SVGO test fixtures

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;
use serde_json::json;

fn test_plugin(input: &str, expected: &str, params: Option<serde_json::Value>) {
    let mut config = Config {
        plugins: vec![PluginConfig {
            name: "removeComments".to_string(),
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
        "\nPlugin: removeComments\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_remove_comments_01() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <!--- test -->
    <g>
        <!--- test -->
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_comments_02_preserve_legal() {
    // Legal comments (starting with <!--!) should be preserved by default
    let input = r#"<!--!Icon Font v1 by @iconfont - Copyright 2023 Icon Font CIC.-->
<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>"#;

    let expected = r#"<!--!Icon Font v1 by @iconfont - Copyright 2023 Icon Font CIC.-->
<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_comments_03_no_preserve() {
    // With preservePatterns: false, even legal comments are removed
    let input = r#"<!--!Icon Font v1 by @iconfont - Copyright 2023 Icon Font CIC.-->
<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>"#;

    let params = json!({
        "preservePatterns": false
    });

    test_plugin(input, expected, Some(params));
}

#[test]
fn test_remove_comments_04_mixed_comments() {
    let input = r#"<!-- Regular comment -->
<!--!Legal comment-->
<svg xmlns="http://www.w3.org/2000/svg">
    <!-- Another regular comment -->
    <g>
        <!--!Another legal comment-->
        <rect/>
    </g>
    <!-- Final comment -->
</svg>"#;

    let expected = r#"<!--!Legal comment-->
<svg xmlns="http://www.w3.org/2000/svg">
    <g>
        <!--!Another legal comment-->
        <rect/>
    </g>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_remove_comments_05_custom_preserve_pattern() {
    let input = r#"<!-- Copyright 2023 -->
<!--! Legal comment -->
<!-- LICENSE: MIT -->
<svg xmlns="http://www.w3.org/2000/svg">
    <!-- Some other comment -->
    <g/>
</svg>"#;

    let expected = r#"<!-- Copyright 2023 -->
<!--! Legal comment -->
<!-- LICENSE: MIT -->
<svg xmlns="http://www.w3.org/2000/svg">
    <g/>
</svg>"#;

    let params = json!({
        "preservePatterns": ["^\\s*!|^\\s*Copyright|^\\s*LICENSE"]
    });

    test_plugin(input, expected, Some(params));
}

#[test]
fn test_remove_comments_06_nested_deep() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <!-- Top level comment -->
    <defs>
        <!-- Definition comment -->
        <linearGradient>
            <!-- Gradient comment -->
            <stop/>
        </linearGradient>
    </defs>
    <g>
        <!-- Group comment -->
        <rect>
            <!-- Should this even be valid? -->
        </rect>
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient>
            <stop/>
        </linearGradient>
    </defs>
    <g>
        <rect/>
    </g>
</svg>"#;

    test_plugin(input, expected, None);
}
