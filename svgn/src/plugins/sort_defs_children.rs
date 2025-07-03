// this_file: svgn/src/plugins/sort_defs_children.rs

//! Plugin to sort children of `<defs>` elements to improve compression
//!
//! This plugin sorts children of defs in order to improve compression. Elements are
//! sorted first by frequency (most frequent first), then by element name length 
//! (longer names first), then alphabetically by element name.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use std::collections::HashMap;
use std::cmp::Ordering;

/// Plugin to sort defs children for better compression
pub struct SortDefsChildrenPlugin;

impl Plugin for SortDefsChildrenPlugin {
    fn name(&self) -> &'static str {
        "sortDefsChildren"
    }

    fn description(&self) -> &'static str {
        "Sorts children of <defs> to improve compression"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        self.process_element(&mut document.root);
        Ok(())
    }
}

impl SortDefsChildrenPlugin {
    fn process_element(&self, element: &mut Element) {
        // Sort children if this is a defs element
        if element.name == "defs" {
            self.sort_defs_children(element);
        }

        // Process children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem);
            }
        }
    }

    fn sort_defs_children(&self, defs: &mut Element) {
        // Count frequencies of element names
        let mut frequencies: HashMap<String, usize> = HashMap::new();
        
        for child in &defs.children {
            if let Node::Element(ref elem) = child {
                *frequencies.entry(elem.name.clone()).or_insert(0) += 1;
            }
        }

        // Sort children based on frequency, name length, and name
        defs.children.sort_by(|a, b| {
            match (a, b) {
                (Node::Element(elem_a), Node::Element(elem_b)) => {
                    // First, sort by frequency (descending)
                    let freq_a = frequencies.get(&elem_a.name).unwrap_or(&0);
                    let freq_b = frequencies.get(&elem_b.name).unwrap_or(&0);
                    
                    match freq_b.cmp(freq_a) {
                        Ordering::Equal => {
                            // Then by name length (descending)
                            match elem_b.name.len().cmp(&elem_a.name.len()) {
                                Ordering::Equal => {
                                    // Finally by name (descending/reverse alphabetical)
                                    elem_b.name.cmp(&elem_a.name)
                                }
                                other => other
                            }
                        }
                        other => other
                    }
                }
                // Non-element nodes maintain their relative order
                _ => Ordering::Equal
            }
        });
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element, Node};
    use std::collections::HashMap;
    use indexmap::IndexMap;

    fn create_test_document() -> Document {
        Document {
            root: Element {
                name: "svg".to_string(),
                attributes: IndexMap::new(),
                namespaces: HashMap::new(),
                children: vec![],
            },
            ..Default::default()
        }
    }

    fn create_element(name: &str) -> Element {
        Element {
            name: name.to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = SortDefsChildrenPlugin;
        assert_eq!(plugin.name(), "sortDefsChildren");
        assert_eq!(plugin.description(), "Sorts children of <defs> to improve compression");
    }

    #[test]
    fn test_sort_by_frequency() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("pattern")),
                Node::Element(create_element("linearGradient")),
                Node::Element(create_element("pattern")),
                Node::Element(create_element("mask")),
                Node::Element(create_element("pattern")),
                Node::Element(create_element("linearGradient")),
            ],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should be sorted by frequency: pattern (3), linearGradient (2), mask (1)
        if let Node::Element(ref defs) = document.root.children[0] {
            let names: Vec<&str> = defs.children.iter()
                .filter_map(|child| {
                    if let Node::Element(elem) = child {
                        Some(elem.name.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(names, vec!["pattern", "pattern", "pattern", "linearGradient", "linearGradient", "mask"]);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_sort_by_name_length() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("g")),
                Node::Element(create_element("use")),
                Node::Element(create_element("path")),
                Node::Element(create_element("rect")),
            ],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should be sorted by name length (all have frequency 1)
        if let Node::Element(ref defs) = document.root.children[0] {
            let names: Vec<&str> = defs.children.iter()
                .filter_map(|child| {
                    if let Node::Element(elem) = child {
                        Some(elem.name.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Length 4 elements come first, then sorted alphabetically in reverse
            assert_eq!(names, vec!["rect", "path", "use", "g"]);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_sort_alphabetically() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("abc")),
                Node::Element(create_element("xyz")),
                Node::Element(create_element("def")),
                Node::Element(create_element("mno")),
            ],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should be sorted reverse alphabetically (all same length and frequency)
        if let Node::Element(ref defs) = document.root.children[0] {
            let names: Vec<&str> = defs.children.iter()
                .filter_map(|child| {
                    if let Node::Element(elem) = child {
                        Some(elem.name.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(names, vec!["xyz", "mno", "def", "abc"]);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_preserve_non_element_nodes() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Comment(" Start ".to_string()),
                Node::Element(create_element("pattern")),
                Node::Text(" whitespace ".to_string()),
                Node::Element(create_element("mask")),
                Node::Comment(" End ".to_string()),
            ],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Non-element nodes should maintain relative position
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.children.len(), 5);
            // Comments and text nodes are preserved
            assert!(matches!(defs.children[0], Node::Comment(_)));
            assert!(matches!(defs.children[2], Node::Text(_)));
            assert!(matches!(defs.children[4], Node::Comment(_)));
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_nested_defs() {
        let mut document = create_test_document();
        
        // Create nested defs structure
        let inner_defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("rect")),
                Node::Element(create_element("circle")),
            ],
        };

        let outer_defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_element("path")),
                Node::Element(inner_defs),
                Node::Element(create_element("ellipse")),
            ],
        };

        document.root.children = vec![Node::Element(outer_defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Both outer and inner defs should be sorted
        if let Node::Element(ref outer_defs) = document.root.children[0] {
            // Check outer defs sorting
            let outer_names: Vec<&str> = outer_defs.children.iter()
                .filter_map(|child| {
                    if let Node::Element(elem) = child {
                        Some(elem.name.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Should be sorted by length: ellipse(7), path(4), defs(4)
            assert_eq!(outer_names, vec!["ellipse", "path", "defs"]);
            
            // Check inner defs sorting
            if let Node::Element(ref inner_defs) = outer_defs.children[2] {
                let inner_names: Vec<&str> = inner_defs.children.iter()
                    .filter_map(|child| {
                        if let Node::Element(elem) = child {
                            Some(elem.name.as_str())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                // Should be sorted by length: circle(6), rect(4)
                assert_eq!(inner_names, vec!["circle", "rect"]);
            }
        } else {
            panic!("Expected outer defs element");
        }
    }

    #[test]
    fn test_empty_defs() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Should handle empty defs gracefully
        if let Node::Element(ref defs) = document.root.children[0] {
            assert_eq!(defs.children.len(), 0);
        } else {
            panic!("Expected defs element");
        }
    }

    #[test]
    fn test_complex_sorting() {
        let mut document = create_test_document();
        
        let defs = Element {
            name: "defs".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                // Mix of frequencies and lengths
                Node::Element(create_element("a")),     // freq=1, len=1
                Node::Element(create_element("bb")),    // freq=2, len=2
                Node::Element(create_element("ccc")),   // freq=1, len=3
                Node::Element(create_element("a")),     // freq=2 now
                Node::Element(create_element("bb")),    // freq=2, len=2
                Node::Element(create_element("dddd")),  // freq=1, len=4
            ],
        };

        document.root.children = vec![Node::Element(defs)];

        let mut plugin = SortDefsChildrenPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        if let Node::Element(ref defs) = document.root.children[0] {
            let names: Vec<&str> = defs.children.iter()
                .filter_map(|child| {
                    if let Node::Element(elem) = child {
                        Some(elem.name.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Expected order:
            // - First by frequency: a(2), bb(2), then ccc(1), dddd(1)
            // - Within same frequency, by length: bb(2) before a(1), dddd(4) before ccc(3)
            assert_eq!(names, vec!["bb", "bb", "a", "a", "dddd", "ccc"]);
        } else {
            panic!("Expected defs element");
        }
    }
}