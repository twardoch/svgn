// this_file: svgn/tests/fixture_tests.rs

//! Tests that load and process SVGO-style test fixtures
//! This matches the pattern used in SVGO's test suite

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;

/// Parse a test fixture file in SVGO format:
/// ```
/// Description (optional)
/// ===
/// Input SVG
/// @@@
/// Expected output SVG
/// @@@
/// Plugin params JSON (optional)
/// ```
#[derive(Debug)]
struct TestFixture {
    description: Option<String>,
    input: String,
    expected: String,
    params: Option<serde_json::Value>,
}

impl TestFixture {
    fn parse(content: &str) -> Result<Self, String> {
        let normalized = content.trim().replace("\r\n", "\n");
        
        // Split by === to separate description from test case
        let parts: Vec<&str> = normalized.split("\n===\n").collect();
        let (description, test_content) = if parts.len() == 2 {
            (Some(parts[0].trim().to_string()), parts[1])
        } else {
            (None, normalized.as_str())
        };
        
        // Split test content by @@@
        let test_parts: Vec<&str> = test_content.split("\n@@@\n").collect();
        
        if test_parts.len() < 2 {
            return Err("Invalid fixture format: missing @@@ separator".to_string());
        }
        
        let input = test_parts[0].trim().to_string();
        let expected = test_parts[1].trim().to_string();
        
        let params = if test_parts.len() > 2 && !test_parts[2].trim().is_empty() {
            match serde_json::from_str(test_parts[2].trim()) {
                Ok(p) => Some(p),
                Err(e) => return Err(format!("Invalid JSON params: {}", e)),
            }
        } else {
            None
        };
        
        Ok(TestFixture {
            description,
            input,
            expected,
            params,
        })
    }
}

/// Run a test with a specific plugin
fn run_plugin_fixture_test(plugin_name: &str, fixture: &TestFixture) {
    let mut plugin_config = PluginConfig::new(plugin_name.to_string());
    plugin_config.params = fixture.params.clone();
    
    let config = Config {
        plugins: vec![plugin_config],
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: true,
            indent: 4,
            ..Default::default()
        },
        ..Default::default()
    };

    // Test idempotence like SVGO does (run twice)
    let mut last_result = fixture.input.clone();
    for i in 0..2 {
        let options = OptimizeOptions::new(config.clone());
        let result = optimize(&last_result, options)
            .expect("Optimization should succeed");
        
        let output = result.data.trim();
        last_result = result.data.clone();
        let expected = fixture.expected.trim();
        
        assert_eq!(output, expected, 
            "\nPlugin: {} (iteration {})\nDescription: {:?}\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
            plugin_name, i + 1, fixture.description, fixture.input, expected, output);
    }
}

#[test]
fn test_cleanup_attrs_fixture() {
    let fixture_content = r#"<svg xmlns="  http://www.w3.org/2000/svg
  " attr="a      b" attr2="a
b">
    test
</svg>

@@@

<svg xmlns="http://www.w3.org/2000/svg" attr="a b" attr2="a b">
    test
</svg>"#;
    
    let fixture = TestFixture::parse(fixture_content)
        .expect("Should parse fixture");
    
    run_plugin_fixture_test("cleanupAttrs", &fixture);
}

#[test]
fn test_convert_colors_fixture() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g color=\"black\"/>\n    <g color=\"BLACK\"/>\n    <path fill=\"rgb(64 64 64)\"/>\n    <path fill=\"rgb(64, 64, 64)\"/>\n    <path fill=\"rgb(86.27451%,86.666667%,87.058824%)\"/>\n    <path fill=\"rgb(-255,100,500)\"/>\n</svg>";
    
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g color=\"#000\"/>\n    <g color=\"#000\"/>\n    <path fill=\"#404040\"/>\n    <path fill=\"#404040\"/>\n    <path fill=\"#dcddde\"/>\n    <path fill=\"#0064ff\"/>\n</svg>";
    
    let fixture = TestFixture {
        description: None,
        input: input.to_string(),
        expected: expected.to_string(),
        params: None,
    };
    
    run_plugin_fixture_test("convertColors", &fixture);
}

#[test]
fn test_remove_empty_attrs_fixture() {
    let fixture_content = r#"Removes empty attributes

===

<svg xmlns="http://www.w3.org/2000/svg">
    <g attr1="" attr2=""/>
</svg>

@@@

<svg xmlns="http://www.w3.org/2000/svg">
    <g/>
</svg>"#;
    
    let fixture = TestFixture::parse(fixture_content)
        .expect("Should parse fixture");
    
    run_plugin_fixture_test("removeEmptyAttrs", &fixture);
}

#[test]
fn test_remove_comments_with_params_fixture() {
    let fixture_content = r#"<!--!Icon Font v1 by @iconfont - Copyright 2023 Icon Font CIC.-->
<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>

@@@

<svg xmlns="http://www.w3.org/2000/svg">
    test
</svg>

@@@

{
    "preservePatterns": false
}"#;
    
    let fixture = TestFixture::parse(fixture_content)
        .expect("Should parse fixture");
    
    run_plugin_fixture_test("removeComments", &fixture);
}

/// Test multipass optimization like SVGO does
#[test]
fn test_multipass_optimization() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <!-- Comment to remove -->
    <metadata>Metadata to remove</metadata>
    <title>Title to remove</title>
    <g transform="" class="  foo   bar  ">
        <rect x="" y="10" width="50" height="" fill="red"/>
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <g class="foo bar">
        <rect y="10" width="50" fill="red"/>
    </g>
</svg>"#;

    let config = Config {
        plugins: vec![
            PluginConfig::new("removeComments".to_string()),
            PluginConfig::new("removeMetadata".to_string()),
            PluginConfig::new("removeTitle".to_string()),
            PluginConfig::new("cleanupAttrs".to_string()),
            PluginConfig::new("removeEmptyAttrs".to_string()),
        ],
        multipass: true,
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
        "\nMultipass optimization\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}
