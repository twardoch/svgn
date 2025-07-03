// this_file: svgn/tests/plugins/cleanup_ids.rs

//! Tests for the cleanupIds plugin
//! Ported from SVGO test fixtures

use svgn::{optimize, OptimizeOptions, Config, PluginConfig};
use svgn::config::Js2SvgOptions;
use serde_json::json;

fn test_plugin(input: &str, expected: &str, params: Option<serde_json::Value>) {
    let mut config = Config {
        plugins: vec![PluginConfig {
            name: "cleanupIds".to_string(),
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
        "\nPlugin: cleanupIds\nInput:\n{}\nExpected:\n{}\nActual:\n{}\n", 
        input, expected, output);
}

#[test]
fn test_cleanup_ids_01_minify_used_ids() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="myVeryLongGradientId">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
    </defs>
    <rect fill="url(#myVeryLongGradientId)"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="a">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
    </defs>
    <rect fill="url(#a)"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_ids_02_remove_unused_ids() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="usedGradient">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
        <linearGradient id="unusedGradient">
            <stop offset="0%" stop-color="blue"/>
        </linearGradient>
    </defs>
    <rect fill="url(#usedGradient)"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="a">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
    </defs>
    <rect fill="url(#a)"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_ids_03_preserve_force_list() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="keepThis">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
        <linearGradient id="minifyThis">
            <stop offset="0%" stop-color="blue"/>
        </linearGradient>
    </defs>
    <rect fill="url(#keepThis)"/>
    <rect fill="url(#minifyThis)"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="keepThis">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
        <linearGradient id="a">
            <stop offset="0%" stop-color="blue"/>
        </linearGradient>
    </defs>
    <rect fill="url(#keepThis)"/>
    <rect fill="url(#a)"/>
</svg>"#;

    let params = json!({
        "force": ["keepThis"]
    });

    test_plugin(input, expected, Some(params));
}

#[test]
fn test_cleanup_ids_04_complex_references() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <clipPath id="myClipPath">
            <rect x="0" y="0" width="50" height="50"/>
        </clipPath>
        <mask id="myMask">
            <rect x="0" y="0" width="100" height="100" fill="white"/>
        </mask>
        <filter id="myFilter">
            <feGaussianBlur stdDeviation="2"/>
        </filter>
    </defs>
    <g clip-path="url(#myClipPath)" mask="url(#myMask)" filter="url(#myFilter)">
        <rect width="100" height="100" fill="red"/>
    </g>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <clipPath id="a">
            <rect x="0" y="0" width="50" height="50"/>
        </clipPath>
        <mask id="b">
            <rect x="0" y="0" width="100" height="100" fill="white"/>
        </mask>
        <filter id="c">
            <feGaussianBlur stdDeviation="2"/>
        </filter>
    </defs>
    <g clip-path="url(#a)" mask="url(#b)" filter="url(#c)">
        <rect width="100" height="100" fill="red"/>
    </g>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_ids_05_href_references() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <defs>
        <g id="reusableShape">
            <rect width="10" height="10"/>
        </g>
    </defs>
    <use href="#reusableShape" x="0" y="0"/>
    <use xlink:href="#reusableShape" x="20" y="0"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <defs>
        <g id="a">
            <rect width="10" height="10"/>
        </g>
    </defs>
    <use href="#a" x="0" y="0"/>
    <use xlink:href="#a" x="20" y="0"/>
</svg>"#;

    test_plugin(input, expected, None);
}

#[test]
fn test_cleanup_ids_06_preserve_if_removal_disabled() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="usedGradient">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
        <linearGradient id="unusedGradient">
            <stop offset="0%" stop-color="blue"/>
        </linearGradient>
    </defs>
    <rect fill="url(#usedGradient)"/>
</svg>"#;

    let expected = r#"<svg xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="a">
            <stop offset="0%" stop-color="red"/>
        </linearGradient>
        <linearGradient id="b">
            <stop offset="0%" stop-color="blue"/>
        </linearGradient>
    </defs>
    <rect fill="url(#a)"/>
</svg>"#;

    let params = json!({
        "remove": false
    });

    test_plugin(input, expected, Some(params));
}
