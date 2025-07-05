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

`svgn`, like `svgo`, includes a default preset of plugins that are generally safe and provide good optimization results. This preset is applied by default when no custom plugin configuration is provided. 

The default preset currently includes most implemented plugins but is more conservative than SVGO's default preset. SVGN's default preset is being actively aligned with SVGO's. Currently, several complex plugins from SVGO's default preset (mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems) are not yet implemented and are excluded from the default configuration.

### SVGO v4 Default Preset Order

The SVGO v4 default preset runs plugins in this specific order:
1. removeDoctype
2. removeXMLProcInst
3. removeComments
4. removeMetadata
5. removeEditorsNSData
6. cleanupAttrs
7. mergeStyles
8. inlineStyles (not yet implemented)
9. minifyStyles
10. cleanupIds
11. removeUselessDefs
12. cleanupNumericValues
13. convertColors
14. removeUnknownsAndDefaults
15. removeNonInheritableGroupAttrs
16. removeUselessStrokeAndFill (not implemented - see note)
17. cleanupEnableBackground
18. removeHiddenElems
19. removeEmptyText
20. convertShapeToPath
21. convertEllipseToCircle
22. moveElemsAttrsToGroup (not yet implemented)
23. moveGroupAttrsToElems (not yet implemented)
24. collapseGroups
25. convertPathData (✅ fully implemented)
26. convertTransform (not yet implemented)
27. removeEmptyAttrs
28. removeEmptyContainers
29. removeUnusedNS
30. mergePaths (not yet implemented)
31. sortAttrs
32. sortDefsChildren
33. removeDesc



## Implementation Status

### Fully Implemented Plugins (55/53)

The following `svgo` plugins have been successfully ported to `svgn`:

#### Basic Optimization Plugins
-   **`cleanupAttrs`**: Cleans up attributes from newlines, trailing, and repeating spaces
-   **`cleanupEnableBackground`**: Removes or cleans up the `enable-background` attribute
-   **`cleanupIds`**: Minifies and removes unused IDs
-   **`cleanupListOfValues`**: Rounds numeric values in attributes that have a list of numbers
-   **`cleanupNumericValues`**: Rounds numeric values to a fixed precision
-   **`removeComments`**: Removes comments (preserves legal comments starting with `!`)
-   **`removeDesc`**: Removes `<desc>` elements
-   **`removeDoctype`**: Removes doctype declarations
-   **`removeEmptyAttrs`**: Removes empty attributes
-   **`removeEmptyContainers`**: Removes empty container elements
-   **`removeEmptyText`**: Removes empty text elements
-   **`removeMetadata`**: Removes `<metadata>` elements
-   **`removeTitle`**: Removes `<title>` elements
-   **`removeXMLProcInst`**: Removes XML processing instructions
-   **`sortAttrs`**: Sorts element attributes for better gzip compression
-   **`sortDefsChildren`**: Sorts children of `<defs>` to improve compression

#### Style and Color Plugins
-   **`convertColors`**: Converts colors to hex format (rgb→#rrggbb, names→hex)
-   **`convertStyleToAttrs`**: Converts styles from style attributes to presentation attributes
-   **`mergeStyles`**: Merges multiple `<style>` elements into one
-   **`minifyStyles`**: Basic CSS minification (removes comments, normalizes whitespace)
-   **`removeStyleElement`**: Removes `<style>` elements

#### Structure Optimization Plugins
-   **`collapseGroups`**: Collapses useless groups (`<g>`)
-   **`convertEllipseToCircle`**: Converts `<ellipse>` to `<circle>` when possible
-   **`convertOneStopGradients`**: Converts single-stop gradients to solid colors
-   **`convertPathData`**: Optimizes path data, converts coordinates, removes redundant commands
-   **`convertShapeToPath`**: Converts basic shapes to `<path>` elements
-   **`removeHiddenElems`**: Removes hidden elements (display:none, visibility:hidden)
-   **`removeNonInheritableGroupAttrs`**: Removes non-inheritable group attributes
-   **`removeOffCanvasPaths`**: Removes elements outside the viewBox
-   **`removeUselessDefs`**: Removes `<defs>` elements without IDs
-   **`removeUselessStrokeAndFill`**: Removes unnecessary stroke and fill attributes
-   **`removeUselessTransforms`**: Removes identity transforms

#### Attribute Management Plugins
-   **`addAttributesToSVGElement`**: Adds attributes to the root `<svg>` element
-   **`addClassesToSVGElement`**: Adds class names to the root `<svg>` element
-   **`prefixIds`**: Adds a prefix to IDs
-   **`removeAttrs`**: Removes attributes by pattern/name
-   **`removeAttributesBySelector`**: Removes attributes matching CSS selectors
-   **`removeDeprecatedAttrs`**: Removes deprecated SVG attributes
-   **`removeDimensions`**: Removes width/height attributes (preserves viewBox)
-   **`removeEditorsNSData`**: Removes editor namespaces and metadata
-   **`removeElementsByAttr`**: Removes elements by ID or class
-   **`removeUnknownsAndDefaults`**: Removes unknown elements and default values
-   **`removeUnusedNS`**: Removes unused namespace declarations
-   **`removeViewBox`**: Removes viewBox when possible
-   **`removeXlink`**: Removes deprecated xlink attributes
-   **`removeXMLNS`**: Removes xmlns attribute from root element

### Not Yet Implemented (5/58)

These complex plugins require additional work:

-   **`applyTransforms`**: Applies transform matrices to path coordinates and shapes
-   **`mergePaths`**: Merge multiple paths into one
-   **`moveElemsAttrsToGroup`**: Move common attributes to parent group
-   **`moveGroupAttrsToElems`**: Move group attributes to child elements
-   **`reusePaths`**: Replace duplicate paths with `<use>` elements

## Plugin Configuration

Configuring plugins in `svgn` is similar to `svgo`. You can enable or disable plugins, and for some, provide specific parameters to control their behavior. This is done through the `SvgnConfig` structure, as shown in the [Usage documentation](./usage.md).

### Example: Disabling a Plugin

To disable a plugin from the command line:

```bash
svgn input.svg -o output.svg --disable removeComments
```

Or in Rust code, omit it from your `plugins` list in the `SvgnConfig`.

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

### Viewing Available Plugins

To see all available plugins with their descriptions:

```bash
svgn --show-plugins
```

This will list all 45 implemented plugins, making it easy to understand what optimizations are available.
