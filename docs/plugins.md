---
layout: default
title: Plugins
nav_order: 3
description: "SVGN plugin architecture and available plugins"
---

# SVGN Plugins

Plugins are the core of `svgn`'s optimization capabilities, just as they are for `svgo`. They perform specific transformations on the SVG's Abstract Syntax Tree (AST) to reduce file size and improve rendering efficiency. `svgn` aims to port all of `svgo`'s plugins, maintaining functional parity and API compatibility where it makes sense in a Rust context.

## Plugin Architecture

Similar to `svgo`, `svgn` utilizes a plugin-based architecture. Each plugin is a distinct module responsible for a specific optimization task. This modularity allows for flexible configuration and extensibility.

In `svgn`, plugins are implemented as Rust functions or structs that operate on the SVG's AST. The core optimizer iterates through the enabled plugins, applying their transformations sequentially.

## Default Preset

`svgn`, like `svgo`, includes a default preset of plugins that are generally safe and provide good optimization results. This preset is applied by default when no custom plugin configuration is provided. The specific plugins included in the default preset and their order of execution will mirror `svgo`'s `preset-default` as closely as possible.

## Ported Plugins

The following `svgo` plugins have been ported to `svgn`:

-   **`addAttributesToSVGElement`**: Adds attributes to the root `<svg>` element.
-   **`addClassesToSVGElement`**: Adds class names to the root `<svg>` element.
-   **`cleanupAttrs`**: Cleans up attributes from newlines, trailing, and repeating spaces.
-   **`cleanupEnableBackground`**: Removes or cleans up the `enable-background` attribute.
-   **`cleanupIds`**: Minifies and removes unused IDs.
-   **`cleanupListOfValues`**: Rounds numeric values in attributes that have a list of numbers (like `viewBox` or `stroke-dasharray`).
-   **`cleanupNumericValues`**: Rounds numeric values to a fixed precision.
-   **`collapseGroups`**: Collapses useless groups (`<g>`).
-   **`convertColors`**: Converts colors from `rgb()` to `#rrggbb`, `rgba()` to `#rrggbbaa`, and color names to hex values.
-   **`convertEllipseToCircle`**: Converts `<ellipse>` elements to `<circle>` elements when possible.
-   **`convertOneStopGradients`**: Converts gradients with only one stop to a solid color.
-   **`convertStyleToAttrs`**: Converts styles from `<style>` elements to attributes.
-   **`mergeStyles`**: Merges multiple `<style>` elements into one.
-   **`removeAttributesBySelector`**: Removes attributes of elements that match a CSS selector.
-   **`removeAttrs`**: Removes attributes by name.
-   **`removeComments`**: Removes comments.
-   **`removeDeprecatedAttrs`**: Removes deprecated attributes.
-   **`removeDesc`**: Removes `<desc>` elements.
-   **`removeDoctype`**: Removes doctype declarations.
-   **`removeEmptyAttrs`**: Removes empty attributes.
-   **`removeEmptyContainers`**: Removes empty container elements.
-   **`removeEmptyText`**: Removes empty text elements.
-   **`removeMetadata`**: Removes `<metadata>` elements.
-   **`removeStyleElement`**: Removes `<style>` elements.
-   **`removeTitle`**: Removes `<title>` elements.
-   **`removeUnknownsAndDefaults`**: Removes unknown elements' content and attributes, and removes default attribute values.
-   **`removeXMLProcInst`**: Removes XML processing instructions.
-   **`sortAttrs`**: Sorts element attributes for better gzip compression.

*(This list will be updated as more plugins are ported and verified.)*

## Plugin Configuration

Configuring plugins in `svgn` is similar to `svgo`. You can enable or disable plugins, and for some, provide specific parameters to control their behavior. This is done through the `SvgnConfig` structure, as shown in the [Usage documentation](./usage.md).

### Example: Disabling a Plugin

To disable a plugin, you would omit it from your `plugins` list in the `SvgnConfig` or explicitly set it to `false` if using a preset override mechanism (which will be implemented to mirror `svgo`'s behavior).

### Example: Configuring a Plugin with Parameters

```rust
use svgn::config::{SvgnConfig, PluginConfig};
use serde_json::json;

let config = SvgnConfig {
    plugins: vec![
        PluginConfig::WithParams {
            name: "cleanupNumericValues".to_string(),
            params: json!({
                "floatPrecision": 2,
                "leadingZero": false,
            }),
        },
    ],
    ..SvgnConfig::default()
};
```

This example demonstrates how to configure the `cleanupNumericValues` plugin to round to 2 decimal places and keep leading zeros, mirroring `svgo`'s parameter structure.
