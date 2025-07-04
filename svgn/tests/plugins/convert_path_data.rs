// this_file: svgn/tests/plugins/convert_path_data.rs
use svgn::optimize;

#[test]
fn test_convert_path_data_basic() {
    let input = r#"<svg xmlns='http://www.w3.org/2000/svg'><path d='M0,0 L10,10 L10,10 L20,20'/></svg>"#;
    let config = serde_json::json!({
        "plugins": [
            {"name": "convertPathData"}
        ]
    });
    let result = optimize(input, Some(&config.to_string())).unwrap().data;
    // Should have optimized the path by removing the redundant L10,10
    assert!(!result.contains("L10,10"), "Should remove redundant LineTo command");
    assert!(result.contains("M0 0L10 10 20 20"), "Should have optimized path");
}

#[test]
fn test_convert_path_data_precision() {
    let input = r#"<svg xmlns='http://www.w3.org/2000/svg'><path d='M0.123456,0.987654 L10.111111,10.222222'/></svg>"#;
    let config = serde_json::json!({
        "plugins": [
            {
                "name": "convertPathData",
                "params": {
                    "floatPrecision": 2
                }
            }
        ]
    });
    let result = optimize(input, Some(&config.to_string())).unwrap().data;
    // Should have rounded to 2 decimal places
    assert!(result.contains("M0.12 0.99L10.11 10.22"), "Should round to specified precision");
}

#[test]
fn test_convert_path_data_remove_leading_zeros() {
    let input = r#"<svg xmlns='http://www.w3.org/2000/svg'><path d='M0.5,0.5 L0.75,0.25'/></svg>"#;
    let config = serde_json::json!({
        "plugins": [
            {
                "name": "convertPathData",
                "params": {
                    "leadingZero": false
                }
            }
        ]
    });
    let result = optimize(input, Some(&config.to_string())).unwrap().data;
    // Should have removed leading zeros
    assert!(result.contains("M.5 .5L.75 .25"), "Should remove leading zeros when configured");
}