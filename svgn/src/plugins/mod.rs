// this_file: svgn/src/plugins/mod.rs

//! Built-in SVG optimization plugins
//!
//! This module contains all the built-in plugins that can be used to optimize
//! SVG documents. Each plugin implements the `Plugin` trait.

pub mod cleanup_attrs;
pub mod cleanup_enable_background;
pub mod cleanup_ids;
pub mod cleanup_list_of_values;
pub mod cleanup_numeric_values;
pub mod remove_comments;
pub mod remove_desc;
pub mod remove_doctype;
pub mod remove_empty_attrs;
pub mod remove_empty_containers;
pub mod remove_empty_text;
pub mod remove_attrs;
pub mod remove_metadata;
pub mod remove_title;
pub mod remove_unknowns_and_defaults;
pub mod remove_xml_proc_inst;
pub mod sort_attrs;
pub mod remove_style_element;
pub mod merge_styles;
pub mod convert_style_to_attrs;
pub mod convert_colors;
pub mod add_attributes_to_svg_element;
pub mod add_classes_to_svg_element;
// pub mod remove_attributes_by_selector; // TODO: Fix CSS selector parsing
pub mod remove_deprecated_attrs;
pub mod convert_ellipse_to_circle;
pub mod collapse_groups;

// Re-export plugins
pub use cleanup_attrs::CleanupAttrsPlugin;
pub use cleanup_enable_background::CleanupEnableBackgroundPlugin;
pub use cleanup_ids::CleanupIdsPlugin;
pub use cleanup_list_of_values::CleanupListOfValuesPlugin;
pub use cleanup_numeric_values::CleanupNumericValuesPlugin;
pub use remove_comments::RemoveCommentsPlugin;
pub use remove_desc::RemoveDescPlugin;
pub use remove_doctype::RemoveDoctypePlugin;
pub use remove_empty_attrs::RemoveEmptyAttrsPlugin;
pub use remove_empty_containers::RemoveEmptyContainersPlugin;
pub use remove_empty_text::RemoveEmptyTextPlugin;
pub use remove_attrs::RemoveAttrsPlugin;
pub use remove_metadata::RemoveMetadataPlugin;
pub use remove_title::RemoveTitlePlugin;
pub use remove_unknowns_and_defaults::RemoveUnknownsAndDefaultsPlugin;
pub use remove_xml_proc_inst::RemoveXMLProcInstPlugin;
pub use sort_attrs::SortAttrsPlugin;
pub use remove_style_element::RemoveStyleElement;
pub use merge_styles::MergeStylesPlugin;
pub use convert_style_to_attrs::ConvertStyleToAttrsPlugin;
pub use convert_colors::ConvertColorsPlugin;
pub use add_attributes_to_svg_element::AddAttributesToSVGElementPlugin;
pub use add_classes_to_svg_element::AddClassesToSVGElementPlugin;
// pub use remove_attributes_by_selector::RemoveAttributesBySelectorPlugin; // TODO: Fix CSS selector parsing
pub use remove_deprecated_attrs::RemoveDeprecatedAttrsPlugin;
pub use convert_ellipse_to_circle::ConvertEllipseToCirclePlugin;
pub use collapse_groups::CollapseGroupsPlugin;