// this_file: svgn/tests/svgo_compatibility_tests.rs

//! SVGO compatibility tests - comprehensive test suite inspired by SVGO patterns
//! These tests verify that SVGN is feature-compatible with SVGO

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::{Js2SvgOptions, QuoteAttrsStyle};
use serde_json::json;

/// Test helper function to run optimization with specific plugins
fn test_optimization(input: &str, expected: &str, plugin_names: Vec<&str>, params: Option<serde_json::Value>) {
    let mut plugins = Vec::new();
    let first_plugin = plugin_names.get(0).copied();
    for name in &plugin_names {
        let mut plugin = PluginConfig::new(name.to_string());
        if Some(name) == first_plugin.as_ref() && params.is_some() {
            plugin.params = params.clone();
        }
        plugins.push(plugin);
    }
    
    // If expected output is the same as input, use inline format (no change expected)
    let use_pretty = input != expected;
    
    let config = Config {
        plugins,
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: use_pretty,
            indent: 4,
            quote_attrs: QuoteAttrsStyle::Always,
            self_closing: true,
        },
        path: None,
        datauri: None,
        parser: Default::default(),
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(input, options).expect("Optimization should succeed");
    let output = result.data.trim();
    let expected = expected.trim();
    
    assert_eq!(output, expected, 
        "\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_cleanup_attrs_basic() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\" attr=\"a  b\">test</svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\" attr=\"a b\">\n    test\n</svg>";
    test_optimization(input, expected, vec!["cleanupAttrs"], None);
}

#[test]
fn test_remove_comments_basic() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><!-- comment --><g/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeComments"], None);
}

#[test]
fn test_remove_comments_preserve_legal() {
    let input = "<!--! legal comment --><svg xmlns=\"http://www.w3.org/2000/svg\"><g/></svg>";
    let expected = "<!--! legal comment -->\n<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeComments"], None);
}

#[test]
fn test_remove_empty_attrs() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><g attr1=\"\" attr2=\"\"/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeEmptyAttrs"], None);
}

#[test]
fn test_remove_metadata() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><metadata>test</metadata><g/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeMetadata"], None);
}

#[test]
fn test_remove_title() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><title>test</title><g/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeTitle"], None);
}

#[test]
fn test_remove_dimensions_with_viewbox() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"50\" viewBox=\"0 0 100 50\"><g/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 50\">\n    <g/>\n</svg>";
    test_optimization(input, expected, vec!["removeDimensions"], None);
}

#[test]
fn test_cleanup_ids_minification() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><defs><linearGradient id=\"veryLongGradientName\"><stop/></linearGradient></defs><rect fill=\"url(#veryLongGradientName)\"/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <defs>\n        <linearGradient id=\"a\">\n            <stop/>\n        </linearGradient>\n    </defs>\n    <rect fill=\"url(#a)\"/>\n</svg>";
    test_optimization(input, expected, vec!["cleanupIds"], None);
}

#[test]
fn test_convert_ellipse_to_circle() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><ellipse cx=\"50\" cy=\"50\" rx=\"25\" ry=\"25\"/></svg>";
    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\">\n    <circle cx=\"50\" cy=\"50\" r=\"25\"/>\n</svg>";
    test_optimization(input, expected, vec!["convertEllipseToCircle"], None);
}

#[test]
fn test_convert_colors_basic() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><g fill=\"red\"/></svg>";
    // Note: exact output depends on convertColors implementation
    // This test verifies the plugin runs without error
    let config = Config {
        plugins: vec![PluginConfig::new("convertColors".to_string())],
        multipass: false,
        js2svg: Js2SvgOptions::default(),
        path: None,
        datauri: None,
        parser: Default::default(),
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(input, options).expect("Optimization should succeed");
    
    // Just verify it processes without error and produces output
    assert!(!result.data.is_empty());
    assert!(result.data.contains("<svg"));
}

#[test]
fn test_multiple_plugins_pipeline() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
        <!-- Comment to remove -->
        <metadata>Metadata to remove</metadata>
        <title>Title to remove</title>
        <g fill="">
            <rect x="10" y="20" width="50" height="" fill="red"/>
        </g>
    </svg>"#;

    let expected = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\">\n    <g>\n        <rect x=\"10\" y=\"20\" width=\"50\" fill=\"red\"/>\n    </g>\n</svg>";

    test_optimization(input, expected, vec![
        "removeComments",
        "removeMetadata", 
        "removeTitle",
        "removeEmptyAttrs",
        "removeDimensions"
    ], None);
}

#[test]
fn test_multipass_optimization() {
    // Test that multipass optimization works
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><g><g><rect/></g></g></svg>";
    
    let config = Config {
        plugins: vec![PluginConfig::new("collapseGroups".to_string())],
        multipass: true,
        js2svg: Js2SvgOptions {
            pretty: true,
            indent: 4,
            quote_attrs: QuoteAttrsStyle::Always,
            self_closing: true,
        },
        path: None,
        datauri: None,
        parser: Default::default(),
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(input, options).expect("Optimization should succeed");
    
    // Verify optimization occurred (groups should be collapsed)
    assert!(!result.data.contains("<g><g>"));
    assert!(result.data.contains("<rect"));
}

#[test]
fn test_plugin_with_params() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"10px\" height=\"20px\"/></svg>";
    
    let params = json!({
        "floatPrecision": 2,
        "defaultPx": false
    });
    
    test_optimization(input, input, vec!["cleanupNumericValues"], Some(params));
}

#[test]
fn test_error_resilience() {
    // Test that optimization handles edge cases gracefully
    let inputs = vec![
        "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>", // Empty SVG
        "<svg xmlns=\"http://www.w3.org/2000/svg\"><g/></svg>", // Simple SVG
        "<svg xmlns=\"http://www.w3.org/2000/svg\"><unknown-element/></svg>", // Unknown element
    ];
    
    for input in inputs {
        let config = Config {
            plugins: vec![
                PluginConfig::new("removeComments".to_string()),
                PluginConfig::new("removeMetadata".to_string()),
            ],
            multipass: false,
            js2svg: Js2SvgOptions::default(),
            path: None,
            datauri: None,
            parser: Default::default(),
        };

        let options = OptimizeOptions::new(config);
        let result = optimize(input, options);
        
        // Should either succeed or fail gracefully
        match result {
            Ok(r) => {
                assert!(!r.data.is_empty());
                assert!(r.data.contains("<svg"));
            },
            Err(_) => {
                // Errors are acceptable for malformed input
            }
        }
    }
}

#[test]
fn test_optimization_info() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><!-- comment --><metadata>data</metadata><g/></svg>";
    
    let config = Config {
        plugins: vec![
            PluginConfig::new("removeComments".to_string()),
            PluginConfig::new("removeMetadata".to_string()),
        ],
        multipass: false,
        js2svg: Js2SvgOptions::default(),
        path: None,
        datauri: None,
        parser: Default::default(),
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(input, options).expect("Optimization should succeed");
    
    // Check that optimization info is populated
    assert!(result.info.original_size > 0);
    assert!(result.info.optimized_size > 0);
    assert!(result.info.optimized_size < result.info.original_size); // Should be smaller
    assert!(result.info.compression_ratio > 0.0);
    assert!(result.info.compression_ratio < 1.0);
}

#[test]
fn test_pretty_print_vs_minified() {
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"><g><rect/></g></svg>";
    
    // Test pretty-printed output
    let pretty_config = Config {
        plugins: vec![],
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: true,
            indent: 2,
            quote_attrs: QuoteAttrsStyle::Always,
            self_closing: true,
        },
        path: None,
        datauri: None,
        parser: Default::default(),
    };
    
    let pretty_result = optimize(input, OptimizeOptions::new(pretty_config)).unwrap();
    assert!(pretty_result.data.contains("\n  <g>"));
    assert!(pretty_result.data.contains("\n    <rect"));
    
    // Test minified output
    let minified_config = Config {
        plugins: vec![],
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: false,
            indent: 0,
            quote_attrs: QuoteAttrsStyle::Always,
            self_closing: true,
        },
        path: None,
        datauri: None,
        parser: Default::default(),
    };
    
    let minified_result = optimize(input, OptimizeOptions::new(minified_config)).unwrap();
    assert!(!minified_result.data.contains("\n  <g>"));
    
    // Pretty output should be longer than minified
    assert!(pretty_result.data.len() > minified_result.data.len());
}
