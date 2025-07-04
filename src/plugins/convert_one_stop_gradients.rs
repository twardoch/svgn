// this_file: src/plugins/convert_one_stop_gradients.rs

use crate::ast::{Element, Node};
use crate::plugin::{Plugin, PluginContext};
use crate::visitor::{visit_elements, VisitorMut};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Converts one-stop (single color) gradients to a plain color
///
/// This plugin identifies gradients (linear or radial) that contain only one <stop> element
/// and replaces all references to that gradient with the solid color from the stop.
/// Empty gradients that reference another gradient with only one stop are also handled.
pub struct ConvertOneStopGradientsPlugin;

impl Plugin for ConvertOneStopGradientsPlugin {
    fn name(&self) -> &'static str {
        "convertOneStopGradients"
    }

    fn description(&self) -> &'static str {
        "converts one-stop (single color) gradients to a plain color"
    }

    fn apply(&self, node: &mut Node, _context: &PluginContext) -> Result<(), String> {
        // First pass: collect all gradients and their stop information
        let mut gradients: HashMap<String, GradientInfo> = HashMap::new();
        let mut defs_elements: Vec<Rc<Element>> = Vec::new();
        
        collect_gradients(node, &mut gradients, &mut defs_elements);
        
        // Resolve gradient references (handle gradients that reference other gradients)
        resolve_gradient_references(&mut gradients);
        
        // Second pass: replace gradient references with solid colors
        let gradients_to_remove = replace_gradient_references(node, &gradients);
        
        // Third pass: remove the gradients and clean up empty defs
        remove_gradients_and_clean_defs(node, &gradients_to_remove, &defs_elements);
        
        Ok(())
    }
}

#[derive(Clone)]
struct GradientInfo {
    id: String,
    stops: Vec<StopInfo>,
    href: Option<String>,
    element: Rc<Element>,
}

#[derive(Clone)]
struct StopInfo {
    color: Option<String>,
}

fn collect_gradients(node: &mut Node, gradients: &mut HashMap<String, GradientInfo>, defs_elements: &mut Vec<Rc<Element>>) {
    visit_elements(node, &mut |element: &Element| {
        if element.name == "defs" {
            defs_elements.push(element.clone());
        }
        
        if element.name == "linearGradient" || element.name == "radialGradient" {
            if let Some(id) = element.attributes.get("id") {
                let mut stops = Vec::new();
                
                // Collect stop elements
                for child in &element.children {
                    if let Node::Element(stop) = child {
                        if stop.name == "stop" {
                            // Get stop-color from either attribute or style
                            let color = get_stop_color(&stop);
                            stops.push(StopInfo { color });
                        }
                    }
                }
                
                // Get href reference
                let href = element.attributes.get("href")
                    .or_else(|| element.attributes.get("xlink:href"))
                    .cloned();
                
                gradients.insert(id.clone(), GradientInfo {
                    id: id.clone(),
                    stops,
                    href,
                    element: element.clone(),
                });
            }
        }
    });
}

fn get_stop_color(stop: &Element) -> Option<String> {
    // First check stop-color attribute
    if let Some(color) = stop.attributes.get("stop-color") {
        return Some(color.clone());
    }
    
    // Then check style attribute for stop-color
    if let Some(style) = stop.attributes.get("style") {
        for part in style.split(';') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("stop-color:") {
                return Some(value.trim().to_string());
            }
        }
    }
    
    // Default to black if no color specified
    Some("black".to_string())
}

fn resolve_gradient_references(gradients: &mut HashMap<String, GradientInfo>) {
    // Create a copy for iteration to avoid borrowing issues
    let ids: Vec<String> = gradients.keys().cloned().collect();
    
    for id in ids {
        resolve_gradient_chain(&id, gradients, &mut HashSet::new());
    }
}

fn resolve_gradient_chain(
    id: &str,
    gradients: &mut HashMap<String, GradientInfo>,
    visited: &mut HashSet<String>
) -> Vec<StopInfo> {
    if visited.contains(id) {
        return Vec::new(); // Circular reference
    }
    visited.insert(id.to_string());
    
    let gradient = match gradients.get(id) {
        Some(g) => g.clone(),
        None => return Vec::new(),
    };
    
    if !gradient.stops.is_empty() {
        return gradient.stops;
    }
    
    // If this gradient has no stops but references another gradient
    if let Some(href) = &gradient.href {
        if let Some(ref_id) = href.strip_prefix('#') {
            let resolved_stops = resolve_gradient_chain(ref_id, gradients, visited);
            // Update the gradient with resolved stops
            if let Some(g) = gradients.get_mut(id) {
                g.stops = resolved_stops.clone();
            }
            return resolved_stops;
        }
    }
    
    Vec::new()
}

fn replace_gradient_references(node: &mut Node, gradients: &HashMap<String, GradientInfo>) -> HashSet<String> {
    let mut gradients_to_remove = HashSet::new();
    
    // Identify single-stop gradients
    for (id, info) in gradients {
        if info.stops.len() == 1 {
            if let Some(stop_color) = &info.stops[0].color {
                let gradient_ref = format!("url(#{})", id);
                replace_in_tree(node, &gradient_ref, stop_color);
                gradients_to_remove.insert(id.clone());
            }
        }
    }
    
    gradients_to_remove
}

fn replace_in_tree(node: &mut Node, gradient_ref: &str, color: &str) {
    struct ReplaceVisitor<'a> {
        gradient_ref: &'a str,
        color: &'a str,
    }
    
    impl<'a> VisitorMut for ReplaceVisitor<'a> {
        fn visit_element(&mut self, element: &mut Element) {
            // Replace in fill and stroke attributes
            for attr in &["fill", "stroke"] {
                if let Some(value) = element.attributes.get_mut(attr) {
                    if value == self.gradient_ref {
                        *value = self.color.to_string();
                    }
                }
            }
            
            // Replace in style attribute
            if let Some(style) = element.attributes.get_mut("style") {
                *style = style.replace(self.gradient_ref, self.color);
            }
        }
    }
    
    let mut visitor = ReplaceVisitor { gradient_ref, color };
    visitor.visit_node(node);
}

fn remove_gradients_and_clean_defs(
    node: &mut Node,
    gradients_to_remove: &HashSet<String>,
    defs_elements: &[Rc<Element>]
) {
    struct RemoveVisitor<'a> {
        gradients_to_remove: &'a HashSet<String>,
        removed_from_defs: HashSet<String>,
    }
    
    impl<'a> VisitorMut for RemoveVisitor<'a> {
        fn visit_element(&mut self, element: &mut Element) {
            // Track which defs elements had gradients removed
            if element.name == "defs" {
                let before_count = element.children.len();
                
                element.children.retain(|child| {
                    if let Node::Element(child_elem) = child {
                        if (child_elem.name == "linearGradient" || child_elem.name == "radialGradient") {
                            if let Some(id) = child_elem.attributes.get("id") {
                                return !self.gradients_to_remove.contains(id);
                            }
                        }
                    }
                    true
                });
                
                if element.children.len() < before_count {
                    if let Some(id) = element.attributes.get("id") {
                        self.removed_from_defs.insert(id.clone());
                    }
                }
            }
        }
    }
    
    let mut visitor = RemoveVisitor {
        gradients_to_remove,
        removed_from_defs: HashSet::new(),
    };
    visitor.visit_node(node);
    
    // Remove empty defs elements
    if let Node::Element(root) = node {
        root.children.retain(|child| {
            if let Node::Element(elem) = child {
                if elem.name == "defs" && elem.children.is_empty() {
                    return false;
                }
            }
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::parse_svg;

    #[test]
    fn test_convert_single_stop_gradient() {
        let mut doc = parse_svg(r#"
            <svg>
                <defs>
                    <linearGradient id="grad1">
                        <stop stop-color="red"/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let plugin = ConvertOneStopGradientsPlugin;
        let context = PluginContext::default();
        plugin.apply(&mut doc, &context).unwrap();
        
        let output = doc.to_string();
        assert!(output.contains(r#"fill="red""#));
        assert!(!output.contains("linearGradient"));
        assert!(!output.contains("url(#grad1)"));
    }

    #[test]
    fn test_gradient_with_multiple_stops() {
        let mut doc = parse_svg(r#"
            <svg>
                <defs>
                    <linearGradient id="grad1">
                        <stop offset="0%" stop-color="red"/>
                        <stop offset="100%" stop-color="blue"/>
                    </linearGradient>
                </defs>
                <rect fill="url(#grad1)" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let plugin = ConvertOneStopGradientsPlugin;
        let context = PluginContext::default();
        plugin.apply(&mut doc, &context).unwrap();
        
        let output = doc.to_string();
        assert!(output.contains("url(#grad1)")); // Should not be converted
        assert!(output.contains("linearGradient"));
    }

    #[test]
    fn test_gradient_reference() {
        let mut doc = parse_svg(r#"
            <svg>
                <defs>
                    <linearGradient id="grad1">
                        <stop stop-color="green"/>
                    </linearGradient>
                    <linearGradient id="grad2" href="#grad1"/>
                </defs>
                <rect fill="url(#grad2)" width="100" height="100"/>
            </svg>
        "#).unwrap();
        
        let plugin = ConvertOneStopGradientsPlugin;
        let context = PluginContext::default();
        plugin.apply(&mut doc, &context).unwrap();
        
        let output = doc.to_string();
        assert!(output.contains(r#"fill="green""#));
        assert!(!output.contains("url(#grad2)"));
    }

    #[test]
    fn test_style_attribute() {
        let mut doc = parse_svg(r#"
            <svg>
                <defs>
                    <radialGradient id="grad1">
                        <stop style="stop-color: blue"/>
                    </radialGradient>
                </defs>
                <circle style="fill: url(#grad1); stroke: black" r="50"/>
            </svg>
        "#).unwrap();
        
        let plugin = ConvertOneStopGradientsPlugin;
        let context = PluginContext::default();
        plugin.apply(&mut doc, &context).unwrap();
        
        let output = doc.to_string();
        assert!(output.contains("fill: blue"));
        assert!(!output.contains("url(#grad1)"));
    }
}