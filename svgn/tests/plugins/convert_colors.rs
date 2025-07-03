use crate::ast::Document;
use crate::config::Config;
use crate::optimizer::{optimize_with_config, OptimizeOptions};
use crate::plugin::PluginConfig;
use serde_json::json;

fn test_plugin_with_config(svg_input: &str, plugin_config: PluginConfig, expected_svg: &str) {
    let mut config = Config::new();
    config.plugins.push(plugin_config);
    let result = optimize_with_config(svg_input, OptimizeOptions::new(config)).unwrap();
    assert_eq!(result.data, expected_svg);
}

#[test]
fn convert_colors_names2hex() {
    let svg_input = r#"<svg fill="red"></svg>"#;
    let expected_svg = r#"<svg fill="#ff0000"></svg>"#;
    let plugin_config = PluginConfig::new("convertColors".to_string());
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_rgb2hex() {
    let svg_input = r#"<svg fill="rgb(255, 0, 255)"></svg>"#;
    let expected_svg = r#"<svg fill="#ff00ff"></svg>"#;
    let plugin_config = PluginConfig::new("convertColors".to_string());
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_rgb2hex_percentages() {
    let svg_input = r#"<svg fill="rgb(50%, 100, 100%)"></svg>"#;
    let expected_svg = r#"<svg fill="#7f64ff"></svg>"#;
    let plugin_config = PluginConfig::new("convertColors".to_string());
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_shorthex() {
    let svg_input = r#"<svg fill="#aabbcc"></svg>"#;
    let expected_svg = r#"<svg fill="#abc"></svg>"#;
    let plugin_config = PluginConfig::new("convertColors".to_string());
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_shortname() {
    let svg_input = r#"<svg fill="#000080"></svg>"#;
    let expected_svg = r#"<svg fill="navy"></svg>"#;
    let plugin_config = PluginConfig::new("convertColors".to_string());
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_current_color_bool() {
    let svg_input = r#"<svg fill="red"></svg>"#;
    let expected_svg = r#"<svg fill="currentColor"></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "currentColor": true,
            "names2hex": false, // Disable other conversions for this test
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_current_color_string() {
    let svg_input = r#"<svg fill="blue"></svg>"#;
    let expected_svg = r#"<svg fill="currentColor"></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "currentColor": "blue",
            "names2hex": false,
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_current_color_regex() {
    let svg_input = r#"<svg fill="#123456"></svg>"#;
    let expected_svg = r#"<svg fill="currentColor"></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "currentColor": "^#",
            "names2hex": false,
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_convert_case_upper() {
    let svg_input = r#"<svg fill="#abcdef"></svg>"#;
    let expected_svg = r#"<svg fill="#ABCDEF"></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "convertCase": "upper",
            "names2hex": false,
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_convert_case_lower() {
    let svg_input = r#"<svg fill="#ABCDEF"></svg>"#;
    let expected_svg = r#"<svg fill="#abcdef"></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "convertCase": "lower",
            "names2hex": false,
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}

#[test]
fn convert_colors_mask_counter_prevents_current_color() {
    let svg_input = r#"<svg><mask fill="red"></mask></svg>"#;
    let expected_svg = r#"<svg><mask fill="#ff0000"></mask></svg>"#;
    let plugin_config = PluginConfig::with_params(
        "convertColors".to_string(),
        json!({
            "currentColor": true,
            "names2hex": true,
            "rgb2hex": false,
            "shorthex": false,
            "shortname": false
        }),
    );
    test_plugin_with_config(svg_input, plugin_config, expected_svg);
}
