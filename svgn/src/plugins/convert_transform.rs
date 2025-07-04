// this_file: svgn/src/plugins/convert_transform.rs

//! Plugin to convert and optimize transform attributes
//!
//! This plugin collapses multiple transformations, converts matrices to short aliases,
//! converts long transform notations to short ones, and removes useless transforms.
//!
//! Based on SVGO's convertTransform plugin.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use nalgebra::Matrix3;
use std::f64::consts::PI;

/// Plugin to convert and optimize transform attributes
pub struct ConvertTransformPlugin;

/// Parameters for the convertTransform plugin
#[derive(Debug, Clone)]
pub struct ConvertTransformParams {
    /// Convert matrices to shorter aliases (translate, scale, rotate)
    pub convert_to_shorts: bool,
    /// Precision for degree values
    pub deg_precision: Option<u8>,
    /// Precision for float values
    pub float_precision: u8,
    /// Precision for transform values (matrix parameters)
    pub transform_precision: u8,
    /// Convert matrices to transform functions
    pub matrix_to_transform: bool,
    /// Use short translate notation
    pub short_translate: bool,
    /// Use short scale notation
    pub short_scale: bool,
    /// Use short rotate notation
    pub short_rotate: bool,
    /// Remove useless transforms
    pub remove_useless: bool,
    /// Collapse multiple transforms into one
    pub collapse_into_one: bool,
    /// Include leading zero
    pub leading_zero: bool,
    /// Add extra space for negative values
    pub negative_extra_space: bool,
}

impl Default for ConvertTransformParams {
    fn default() -> Self {
        Self {
            convert_to_shorts: true,
            deg_precision: None,
            float_precision: 3,
            transform_precision: 5,
            matrix_to_transform: true,
            short_translate: true,
            short_scale: true,
            short_rotate: true,
            remove_useless: true,
            collapse_into_one: true,
            leading_zero: true,
            negative_extra_space: false,
        }
    }
}

impl ConvertTransformParams {
    /// Parse parameters from JSON value
    pub fn from_value(value: Option<&Value>) -> PluginResult<Self> {
        let mut params = Self::default();
        
        if let Some(Value::Object(map)) = value {
            if let Some(Value::Bool(v)) = map.get("convertToShorts") {
                params.convert_to_shorts = *v;
            }
            if let Some(Value::Number(n)) = map.get("degPrecision") {
                if let Some(val) = n.as_u64() {
                    params.deg_precision = Some(val as u8);
                }
            }
            if let Some(Value::Number(n)) = map.get("floatPrecision") {
                if let Some(val) = n.as_u64() {
                    params.float_precision = val as u8;
                }
            }
            if let Some(Value::Number(n)) = map.get("transformPrecision") {
                if let Some(val) = n.as_u64() {
                    params.transform_precision = val as u8;
                }
            }
            if let Some(Value::Bool(v)) = map.get("matrixToTransform") {
                params.matrix_to_transform = *v;
            }
            if let Some(Value::Bool(v)) = map.get("shortTranslate") {
                params.short_translate = *v;
            }
            if let Some(Value::Bool(v)) = map.get("shortScale") {
                params.short_scale = *v;
            }
            if let Some(Value::Bool(v)) = map.get("shortRotate") {
                params.short_rotate = *v;
            }
            if let Some(Value::Bool(v)) = map.get("removeUseless") {
                params.remove_useless = *v;
            }
            if let Some(Value::Bool(v)) = map.get("collapseIntoOne") {
                params.collapse_into_one = *v;
            }
            if let Some(Value::Bool(v)) = map.get("leadingZero") {
                params.leading_zero = *v;
            }
            if let Some(Value::Bool(v)) = map.get("negativeExtraSpace") {
                params.negative_extra_space = *v;
            }
        }
        
        Ok(params)
    }
}

/// Represents a single transform operation
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub name: String,
    pub data: Vec<f64>,
}

impl Transform {
    /// Create a new transform
    pub fn new(name: String, data: Vec<f64>) -> Self {
        Self { name, data }
    }
    
    /// Convert to matrix representation
    pub fn to_matrix(&self) -> Matrix3<f64> {
        match self.name.as_str() {
            "matrix" => {
                if self.data.len() >= 6 {
                    Matrix3::new(
                        self.data[0], self.data[2], self.data[4],
                        self.data[1], self.data[3], self.data[5],
                        0.0, 0.0, 1.0,
                    )
                } else {
                    Matrix3::identity()
                }
            }
            "translate" => {
                let tx = self.data.get(0).copied().unwrap_or(0.0);
                let ty = self.data.get(1).copied().unwrap_or(0.0);
                Matrix3::new(
                    1.0, 0.0, tx,
                    0.0, 1.0, ty,
                    0.0, 0.0, 1.0,
                )
            }
            "scale" => {
                let sx = self.data.get(0).copied().unwrap_or(1.0);
                let sy = self.data.get(1).copied().unwrap_or(sx);
                Matrix3::new(
                    sx, 0.0, 0.0,
                    0.0, sy, 0.0,
                    0.0, 0.0, 1.0,
                )
            }
            "rotate" => {
                let angle = self.data.get(0).copied().unwrap_or(0.0) * PI / 180.0;
                let cx = self.data.get(1).copied().unwrap_or(0.0);
                let cy = self.data.get(2).copied().unwrap_or(0.0);
                
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                
                if cx == 0.0 && cy == 0.0 {
                    Matrix3::new(
                        cos_a, -sin_a, 0.0,
                        sin_a, cos_a, 0.0,
                        0.0, 0.0, 1.0,
                    )
                } else {
                    // rotate(angle, cx, cy) = translate(cx, cy) rotate(angle) translate(-cx, -cy)
                    let translate_to = Matrix3::new(
                        1.0, 0.0, cx,
                        0.0, 1.0, cy,
                        0.0, 0.0, 1.0,
                    );
                    let rotate = Matrix3::new(
                        cos_a, -sin_a, 0.0,
                        sin_a, cos_a, 0.0,
                        0.0, 0.0, 1.0,
                    );
                    let translate_back = Matrix3::new(
                        1.0, 0.0, -cx,
                        0.0, 1.0, -cy,
                        0.0, 0.0, 1.0,
                    );
                    translate_to * rotate * translate_back
                }
            }
            "skewX" => {
                let angle = self.data.get(0).copied().unwrap_or(0.0) * PI / 180.0;
                Matrix3::new(
                    1.0, angle.tan(), 0.0,
                    0.0, 1.0, 0.0,
                    0.0, 0.0, 1.0,
                )
            }
            "skewY" => {
                let angle = self.data.get(0).copied().unwrap_or(0.0) * PI / 180.0;
                Matrix3::new(
                    1.0, 0.0, 0.0,
                    angle.tan(), 1.0, 0.0,
                    0.0, 0.0, 1.0,
                )
            }
            _ => Matrix3::identity(),
        }
    }
}

impl ConvertTransformPlugin {
    /// Parse transform string to transform operations
    pub fn parse_transform_string(&self, transform_str: &str) -> Vec<Transform> {
        let mut transforms = Vec::new();
        
        // Regex pattern to match transform functions
        let re = regex::Regex::new(r"\s*(matrix|translate|scale|rotate|skewX|skewY)\s*\(\s*([^)]*)\s*\)").unwrap();
        
        for cap in re.captures_iter(transform_str) {
            if let (Some(name_match), Some(data_match)) = (cap.get(1), cap.get(2)) {
                let name = name_match.as_str().to_string();
                let data_str = data_match.as_str();
                
                // Parse numeric values
                let data: Vec<f64> = data_str
                    .split(',')
                    .flat_map(|s| s.split_whitespace())
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                transforms.push(Transform::new(name, data));
            }
        }
        
        transforms
    }
    
    /// Convert transforms to optimized form
    pub fn optimize_transforms(&self, transforms: Vec<Transform>, params: &ConvertTransformParams) -> Vec<Transform> {
        if transforms.is_empty() {
            return transforms;
        }
        
        let mut result = transforms;
        
        // Collapse into one matrix if requested
        if params.collapse_into_one && result.len() > 1 {
            let mut combined_matrix = Matrix3::identity();
            for transform in &result {
                combined_matrix = combined_matrix * transform.to_matrix();
            }
            result = vec![self.matrix_to_transform(combined_matrix, params)];
        }
        
        // Convert to shorts if requested
        if params.convert_to_shorts {
            result = result.into_iter().map(|t| self.convert_to_short(t, params)).collect();
        }
        
        // Remove useless transforms
        if params.remove_useless {
            result = self.remove_useless_transforms(result);
        }
        
        result
    }
    
    /// Convert matrix to transform function
    fn matrix_to_transform(&self, matrix: Matrix3<f64>, params: &ConvertTransformParams) -> Transform {
        let a = matrix[(0, 0)];
        let b = matrix[(1, 0)];
        let c = matrix[(0, 1)];
        let d = matrix[(1, 1)];
        let e = matrix[(0, 2)];
        let f = matrix[(1, 2)];
        
        // Check for simple transforms first
        if params.matrix_to_transform {
            // Pure translation
            if a == 1.0 && b == 0.0 && c == 0.0 && d == 1.0 {
                return Transform::new("translate".to_string(), vec![e, f]);
            }
            
            // Pure scale
            if b == 0.0 && c == 0.0 && e == 0.0 && f == 0.0 {
                return Transform::new("scale".to_string(), vec![a, d]);
            }
            
            // Pure rotation (no translation)
            if e == 0.0 && f == 0.0 && (a * a + b * b - 1.0).abs() < 1e-10 {
                let angle = b.atan2(a) * 180.0 / PI;
                return Transform::new("rotate".to_string(), vec![angle]);
            }
        }
        
        // Fallback to matrix
        Transform::new("matrix".to_string(), vec![a, b, c, d, e, f])
    }
    
    /// Convert transform to shorter notation if possible
    fn convert_to_short(&self, transform: Transform, params: &ConvertTransformParams) -> Transform {
        match transform.name.as_str() {
            "translate" => {
                if params.short_translate && transform.data.len() >= 2 && transform.data[1] == 0.0 {
                    Transform::new("translate".to_string(), vec![transform.data[0]])
                } else {
                    transform
                }
            }
            "scale" => {
                if params.short_scale && transform.data.len() >= 2 && transform.data[0] == transform.data[1] {
                    Transform::new("scale".to_string(), vec![transform.data[0]])
                } else {
                    transform
                }
            }
            _ => transform,
        }
    }
    
    /// Remove useless transforms
    fn remove_useless_transforms(&self, transforms: Vec<Transform>) -> Vec<Transform> {
        transforms.into_iter().filter(|t| !self.is_useless_transform(t)).collect()
    }
    
    /// Check if transform is useless (identity)
    fn is_useless_transform(&self, transform: &Transform) -> bool {
        match transform.name.as_str() {
            "translate" => {
                transform.data.is_empty() || 
                (transform.data.len() >= 1 && transform.data[0] == 0.0 &&
                 (transform.data.len() == 1 || transform.data[1] == 0.0))
            }
            "scale" => {
                transform.data.is_empty() ||
                (transform.data.len() >= 1 && transform.data[0] == 1.0 &&
                 (transform.data.len() == 1 || transform.data[1] == 1.0))
            }
            "rotate" => {
                transform.data.is_empty() || transform.data[0] == 0.0
            }
            "skewX" | "skewY" => {
                transform.data.is_empty() || transform.data[0] == 0.0
            }
            "matrix" => {
                transform.data.len() >= 6 &&
                transform.data[0] == 1.0 && // a
                transform.data[1] == 0.0 && // b
                transform.data[2] == 0.0 && // c
                transform.data[3] == 1.0 && // d
                transform.data[4] == 0.0 && // e
                transform.data[5] == 0.0    // f
            }
            _ => false,
        }
    }
    
    /// Convert transforms back to string
    pub fn transforms_to_string(&self, transforms: Vec<Transform>, params: &ConvertTransformParams) -> String {
        if transforms.is_empty() {
            return String::new();
        }
        
        transforms.iter().map(|t| {
            let data_str = t.data.iter()
                .map(|&val| self.format_number(val, params))
                .collect::<Vec<_>>()
                .join(",");
            format!("{}({})", t.name, data_str)
        }).collect::<Vec<_>>().join(" ")
    }
    
    /// Format number according to precision settings
    fn format_number(&self, val: f64, params: &ConvertTransformParams) -> String {
        let precision = params.float_precision;
        
        let formatted = if precision == 0 {
            format!("{:.0}", val)
        } else {
            format!("{:.prec$}", val, prec = precision as usize)
        };
        
        // Remove trailing zeros after decimal point
        if formatted.contains('.') {
            formatted.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            formatted
        }
    }
    
    /// Process element recursively
    fn process_element(&self, element: &mut Element, params: &ConvertTransformParams) {
        // Process transform attribute
        if let Some(transform_value) = element.attributes.get("transform").cloned() {
            let transforms = self.parse_transform_string(&transform_value);
            let optimized = self.optimize_transforms(transforms, params);
            
            if optimized.is_empty() {
                element.attributes.shift_remove("transform");
            } else {
                let new_value = self.transforms_to_string(optimized, params);
                element.attributes.insert("transform".to_string(), new_value);
            }
        }
        
        // Process gradientTransform attribute
        if let Some(transform_value) = element.attributes.get("gradientTransform").cloned() {
            let transforms = self.parse_transform_string(&transform_value);
            let optimized = self.optimize_transforms(transforms, params);
            
            if optimized.is_empty() {
                element.attributes.shift_remove("gradientTransform");
            } else {
                let new_value = self.transforms_to_string(optimized, params);
                element.attributes.insert("gradientTransform".to_string(), new_value);
            }
        }
        
        // Process patternTransform attribute
        if let Some(transform_value) = element.attributes.get("patternTransform").cloned() {
            let transforms = self.parse_transform_string(&transform_value);
            let optimized = self.optimize_transforms(transforms, params);
            
            if optimized.is_empty() {
                element.attributes.shift_remove("patternTransform");
            } else {
                let new_value = self.transforms_to_string(optimized, params);
                element.attributes.insert("patternTransform".to_string(), new_value);
            }
        }
        
        // Process children
        for child in &mut element.children {
            if let Node::Element(child_element) = child {
                self.process_element(child_element, params);
            }
        }
    }
}

impl Plugin for ConvertTransformPlugin {
    fn name(&self) -> &'static str {
        "convertTransform"
    }
    
    fn description(&self) -> &'static str {
        "collapses multiple transformations and optimizes it"
    }
    
    fn apply(&mut self, document: &mut Document, _plugin_info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        let params = ConvertTransformParams::from_value(params)?;
        self.process_element(&mut document.root, &params);
        Ok(())
    }
    
    fn validate_params(&self, params: Option<&Value>) -> PluginResult<()> {
        // Try to parse parameters to validate them
        ConvertTransformParams::from_value(params)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use crate::plugin::{Plugin, PluginInfo};
    use indexmap::IndexMap;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_element_with_transform(transform: &str) -> Element {
        let mut element = Element {
            name: "rect".to_string(),
            namespaces: HashMap::new(),
            attributes: IndexMap::new(),
            children: vec![],
        };
        element.attributes.insert("transform".to_string(), transform.to_string());
        element
    }

    #[test]
    fn test_parse_transform_string() {
        let plugin = ConvertTransformPlugin;
        let transforms = plugin.parse_transform_string("translate(10,20) scale(2)");
        
        assert_eq!(transforms.len(), 2);
        assert_eq!(transforms[0].name, "translate");
        assert_eq!(transforms[0].data, vec![10.0, 20.0]);
        assert_eq!(transforms[1].name, "scale");
        assert_eq!(transforms[1].data, vec![2.0]);
    }

    #[test]
    fn test_remove_useless_transforms() {
        let plugin = ConvertTransformPlugin;
        
        // Identity transforms
        assert!(plugin.is_useless_transform(&Transform::new("translate".to_string(), vec![0.0, 0.0])));
        assert!(plugin.is_useless_transform(&Transform::new("scale".to_string(), vec![1.0, 1.0])));
        assert!(plugin.is_useless_transform(&Transform::new("rotate".to_string(), vec![0.0])));
        
        // Non-identity transforms
        assert!(!plugin.is_useless_transform(&Transform::new("translate".to_string(), vec![10.0, 0.0])));
        assert!(!plugin.is_useless_transform(&Transform::new("scale".to_string(), vec![2.0, 1.0])));
        assert!(!plugin.is_useless_transform(&Transform::new("rotate".to_string(), vec![45.0])));
    }

    #[test]
    fn test_plugin_removes_identity_transform() {
        let mut doc = Document::default();
        let element = create_test_element_with_transform("translate(0,0)");
        doc.root = element;
        
        let mut plugin = ConvertTransformPlugin;
        let plugin_info = PluginInfo::default();
        let params = json!({});
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Transform should be removed
        assert!(!doc.root.attributes.contains_key("transform"));
    }

    #[test]
    fn test_plugin_optimizes_transform() {
        let mut doc = Document::default();
        let element = create_test_element_with_transform("translate(10,0)");
        doc.root = element;
        
        let mut plugin = ConvertTransformPlugin;
        let plugin_info = PluginInfo::default();
        let params = json!({"shortTranslate": true});
        
        plugin.apply(&mut doc, &plugin_info, Some(&params)).unwrap();
        
        // Should be shortened to single parameter
        assert_eq!(doc.root.attributes.get("transform"), Some(&"translate(10)".to_string()));
    }
}