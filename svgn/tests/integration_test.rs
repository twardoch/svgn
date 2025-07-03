// this_file: tests/integration_test.rs

//! Integration tests for svgn

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::{Js2SvgOptions, QuoteAttrsStyle};

#[test]
fn test_full_optimization_pipeline() {
    let svg = r#"<?xml version="1.0"?>
    <!-- This is a comment -->
    <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" 
         enable-background="new 0 0 100 100">
        <metadata>Some metadata</metadata>
        <title>Test SVG</title>
        <defs>
            <linearGradient id="myVeryLongGradientId">
                <stop offset="0%" stop-color="red"/>
            </linearGradient>
            <linearGradient id="unused-gradient">
                <stop offset="0%" stop-color="blue"/>
            </linearGradient>
        </defs>
        <rect x="10
20" y="30  40" fill="url(#myVeryLongGradientId)" 
              width="50" height="50" class="  foo   bar  "/>
    </svg>"#;

    let config = Config {
        plugins: vec![
            PluginConfig::new("cleanupAttrs".to_string()),
            PluginConfig::new("cleanupEnableBackground".to_string()),
            PluginConfig::new("cleanupIds".to_string()),
            PluginConfig::new("removeComments".to_string()),
            PluginConfig::new("removeMetadata".to_string()),
            PluginConfig::new("removeTitle".to_string()),
        ],
        multipass: false,
        js2svg: Js2SvgOptions {
            pretty: false,
            indent: 2,
            quote_attrs: QuoteAttrsStyle::Always,
            self_closing: true,
        },
        path: None,
        datauri: None,
        parser: Default::default(),
    };

    let options = OptimizeOptions::new(config);
    let result = optimize(svg, options).unwrap();

    // Debug: print the optimized SVG
    println!("Optimized SVG:\n{}", result.data);

    // Verify optimizations were applied
    assert!(!result.data.contains("<!-- This is a comment -->"));
    assert!(!result.data.contains("<metadata>"));
    assert!(!result.data.contains("<title>"));
    assert!(!result.data.contains("enable-background"));
    assert!(!result.data.contains("unused-gradient"));
    assert!(!result.data.contains("myVeryLongGradientId")); // Should be minified
    assert!(result.data.contains("url(#b)")); // Minified ID
    assert!(result.data.contains(r#"x="10 20""#)); // Newline replaced with space
    assert!(result.data.contains(r#"y="30 40""#)); // Multiple spaces reduced
    assert!(result.data.contains(r#"class="foo bar""#)); // Trimmed and cleaned

    // Check optimization info
    assert!(result.info.original_size > 0);
    assert!(result.info.optimized_size < result.info.original_size);
    assert!(result.info.compression_ratio > 0.0);
}