// this_file: svgn/tests/plugins/convert_one_stop_gradients.rs

use svgn::optimize;
use serde_json::json;

#[test]
fn test_convert_one_stop_gradient_basic() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="a">
      <stop stop-color="#ddc4cc"/>
    </linearGradient>
  </defs>
  <rect fill="url(#a)" width="100" height="100"/>
</svg>"#;
    
    let config = json!({
        "plugins": [
            {"name": "convertOneStopGradients"}
        ]
    });
    
    let result = optimize(input, Some(&config.to_string())).unwrap();
    
    // Should convert url(#a) to the color
    assert!(result.data.contains(r#"fill="#ddc4cc""#));
    assert!(!result.data.contains("linearGradient"));
    assert!(!result.data.contains("url(#a)"));
}

#[test]
fn test_convert_one_stop_gradient_with_style() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="b">
      <stop style="stop-color:#a8c4cc"/>
    </linearGradient>
  </defs>
  <rect style="fill:url(#b)" width="100" height="100"/>
</svg>"#;
    
    let config = json!({
        "plugins": [
            {"name": "convertOneStopGradients"}
        ]
    });
    
    let result = optimize(input, Some(&config.to_string())).unwrap();
    
    // Should convert url(#b) to the color in style
    assert!(result.data.contains("fill:#a8c4cc"));
    assert!(!result.data.contains("linearGradient"));
    assert!(!result.data.contains("url(#b)"));
}

#[test]
fn test_keep_multi_stop_gradient() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="c">
      <stop offset="0%" stop-color="red"/>
      <stop offset="100%" stop-color="blue"/>
    </linearGradient>
  </defs>
  <rect fill="url(#c)" width="100" height="100"/>
</svg>"#;
    
    let config = json!({
        "plugins": [
            {"name": "convertOneStopGradients"}
        ]
    });
    
    let result = optimize(input, Some(&config.to_string())).unwrap();
    
    // Should NOT convert multi-stop gradients
    assert!(result.data.contains("linearGradient"));
    assert!(result.data.contains("url(#c)"));
}

#[test]
fn test_remove_empty_defs() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="d">
      <stop stop-color="green"/>
    </linearGradient>
  </defs>
  <rect fill="url(#d)" width="100" height="100"/>
</svg>"#;
    
    let config = json!({
        "plugins": [
            {"name": "convertOneStopGradients"}
        ]
    });
    
    let result = optimize(input, Some(&config.to_string())).unwrap();
    
    // Should remove empty defs after converting gradient
    assert!(!result.data.contains("<defs"));
    assert!(result.data.contains(r#"fill="green""#));
}

#[test]
fn test_radial_gradient() {
    let input = r#"<svg xmlns="http://www.w3.org/2000/svg">
  <defs>
    <radialGradient id="e">
      <stop stop-color="#123456"/>
    </radialGradient>
  </defs>
  <circle fill="url(#e)" cx="50" cy="50" r="40"/>
</svg>"#;
    
    let config = json!({
        "plugins": [
            {"name": "convertOneStopGradients"}
        ]
    });
    
    let result = optimize(input, Some(&config.to_string())).unwrap();
    
    // Should convert radial gradients too
    assert!(result.data.contains(r#"fill="#123456""#));
    assert!(!result.data.contains("radialGradient"));
}