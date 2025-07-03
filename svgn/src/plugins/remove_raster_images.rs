// this_file: svgn/src/plugins/remove_raster_images.rs

//! Plugin to remove raster image references
//!
//! This plugin removes `<image>` elements that reference raster images (JPEG, PNG, GIF).
//! This is useful when you want a pure vector SVG without embedded or linked bitmaps.

use crate::ast::{Document, Element, Node};
use crate::plugin::{Plugin, PluginInfo, PluginResult};
use serde_json::Value;
use regex::Regex;
use once_cell::sync::Lazy;

/// Regex to detect raster image references
static RASTER_IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\.|image/)(jpe?g|png|gif)").unwrap()
});

/// Plugin to remove raster image references (disabled by default)
pub struct RemoveRasterImagesPlugin;

impl Plugin for RemoveRasterImagesPlugin {
    fn name(&self) -> &'static str {
        "removeRasterImages"
    }

    fn description(&self) -> &'static str {
        "removes raster images (disabled by default)"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, _params: Option<&Value>) -> PluginResult<()> {
        self.process_element(&mut document.root);
        Ok(())
    }
}

impl RemoveRasterImagesPlugin {
    fn process_element(&self, element: &mut Element) {
        // Filter out raster image elements
        element.children.retain(|child| {
            if let Node::Element(ref elem) = child {
                !self.is_raster_image(elem)
            } else {
                true
            }
        });

        // Process remaining children recursively
        for child in &mut element.children {
            if let Node::Element(ref mut elem) = child {
                self.process_element(elem);
            }
        }
    }

    fn is_raster_image(&self, element: &Element) -> bool {
        if element.name != "image" {
            return false;
        }

        // Check both xlink:href and href attributes
        for attr_name in ["xlink:href", "href"] {
            if let Some(href_value) = element.attributes.get(attr_name) {
                if RASTER_IMAGE_REGEX.is_match(href_value) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
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

    fn create_image_element(href_attr: &str, href_value: &str) -> Element {
        let mut attributes = IndexMap::new();
        attributes.insert(href_attr.to_string(), href_value.to_string());
        attributes.insert("width".to_string(), "100".to_string());
        attributes.insert("height".to_string(), "100".to_string());
        
        Element {
            name: "image".to_string(),
            attributes,
            namespaces: HashMap::new(),
            children: vec![],
        }
    }

    #[test]
    fn test_plugin_name_and_description() {
        let plugin = RemoveRasterImagesPlugin;
        assert_eq!(plugin.name(), "removeRasterImages");
        assert_eq!(plugin.description(), "removes raster images (disabled by default)");
    }

    #[test]
    fn test_remove_jpeg_image() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "photo.jpg")),
            Node::Element(create_image_element("xlink:href", "image.jpeg")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Both JPEG images should be removed
        assert_eq!(document.root.children.len(), 0);
    }

    #[test]
    fn test_remove_png_image() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "icon.png")),
            Node::Element(create_image_element("href", "logo.png")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Both PNG images should be removed
        assert_eq!(document.root.children.len(), 0);
    }

    #[test]
    fn test_remove_gif_image() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "animation.gif")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // GIF image should be removed
        assert_eq!(document.root.children.len(), 0);
    }

    #[test]
    fn test_remove_data_uri_images() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "data:image/jpeg;base64,/9j/4AAQ...")),
            Node::Element(create_image_element("href", "data:image/png;base64,iVBORw0KGgo...")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Data URI raster images should be removed
        assert_eq!(document.root.children.len(), 0);
    }

    #[test]
    fn test_preserve_svg_images() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "icon.svg")),
            Node::Element(create_image_element("href", "#symbol1")),
            Node::Element(create_image_element("xlink:href", "drawing.svgz")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // SVG images should be preserved
        assert_eq!(document.root.children.len(), 3);
    }

    #[test]
    fn test_preserve_non_image_elements() {
        let mut document = create_test_document();
        
        let mut rect_attrs = IndexMap::new();
        rect_attrs.insert("xlink:href".to_string(), "photo.jpg".to_string());
        
        let rect_element = Element {
            name: "rect".to_string(),
            attributes: rect_attrs,
            namespaces: HashMap::new(),
            children: vec![],
        };

        document.root.children = vec![
            Node::Element(rect_element),
            Node::Element(create_image_element("xlink:href", "icon.svg")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Non-image elements should be preserved even with raster hrefs
        assert_eq!(document.root.children.len(), 2);
    }

    #[test]
    fn test_case_sensitivity() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "photo.JPG")),
            Node::Element(create_image_element("xlink:href", "image.JPEG")),
            Node::Element(create_image_element("xlink:href", "icon.PNG")),
            Node::Element(create_image_element("xlink:href", "anim.GIF")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // The regex is case-sensitive, so uppercase extensions are preserved
        assert_eq!(document.root.children.len(), 4);
    }

    #[test]
    fn test_nested_images() {
        let mut document = create_test_document();
        
        let g_element = Element {
            name: "g".to_string(),
            attributes: IndexMap::new(),
            namespaces: HashMap::new(),
            children: vec![
                Node::Element(create_image_element("xlink:href", "photo1.jpg")),
                Node::Element(create_image_element("href", "icon.svg")),
                Node::Element(create_image_element("xlink:href", "photo2.png")),
            ],
        };

        document.root.children = vec![Node::Element(g_element)];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // Only SVG image should remain in the group
        if let Node::Element(ref g) = document.root.children[0] {
            assert_eq!(g.children.len(), 1);
            if let Node::Element(ref img) = g.children[0] {
                assert_eq!(img.attributes.get("href"), Some(&"icon.svg".to_string()));
            } else {
                panic!("Expected image element");
            }
        } else {
            panic!("Expected g element");
        }
    }

    #[test]
    fn test_url_with_path() {
        let mut document = create_test_document();
        
        document.root.children = vec![
            Node::Element(create_image_element("xlink:href", "/images/photo.jpg")),
            Node::Element(create_image_element("href", "https://example.com/image.png")),
            Node::Element(create_image_element("xlink:href", "../assets/banner.gif")),
        ];

        let mut plugin = RemoveRasterImagesPlugin;
        let result = plugin.apply(&mut document, &PluginInfo::default(), Some(&Value::Null));
        assert!(result.is_ok());

        // All raster images with paths should be removed
        assert_eq!(document.root.children.len(), 0);
    }
}