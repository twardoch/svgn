// this_file: svgn/tests/plugins/convert_path_data.rs
use svgn::optimize_from_str;

#[test]
fn stub_convert_path_data_plugin_errors() {
    let input = r#"<svg xmlns='http://www.w3.org/2000/svg'><path d='M0,0h10'/></svg>"#;
    let config = serde_json::json!({
        "plugins": [
            {"name": "convertPathData"}
        ]
    });
    let result = optimize_from_str(input, Some(&config.to_string()));
    assert!(result.is_err(), "convertPathData should return error until implemented");
    let msg = format!("{}", result.err().unwrap());
    assert!(msg.contains("not yet implemented"), "Error message should indicate unimplemented status, got: {msg}");
}