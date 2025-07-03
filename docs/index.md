---
layout: default
title: SVGN Specification
---

# SVGO Specification

## 1. Introduction

`svgo` is a powerful, Node.js-based tool for optimizing SVG vector graphics files. It removes redundant and useless information, minifies the code, and applies various optimizations to reduce the file size without affecting the rendering quality. This makes SVGs lighter and faster to load, especially in web environments.

This document describes the structure, API, and plugins of the original JavaScript `svgo` package. It is based on the `svgo` reference implementation, which is the basis for `svgn`, a native Rust port of `svgo` and its plugins.

## 2. Project Structure

The `svgo` repository is organized into several key directories:

-   **/bin**: Contains the executable script for the command-line interface (CLI).
-   **/docs**: Detailed documentation in Markdown (`.mdx`) format, covering usage, plugins, and APIs.
-   **/lib**: The core library source code, including the main `svgo.js` entry point, parser, and utility functions.
-   **/plugins**: The individual optimization plugins that form the core of `svgo`'s functionality. Each file corresponds to a specific optimization rule.
-   **/test**: A comprehensive suite of tests, including regression tests, fixtures, and plugin-specific test cases.

## 3. Core API

The primary API of `svgo` is the `optimize` function, which takes an SVG string and a configuration object, and returns an object containing the optimized SVG data.

### `optimize(input, config)`

-   **`input`** (string, required): The SVG string to be optimized.
-   **`config`** (object, optional): The configuration object to customize the optimization process.

#### Configuration Object (`config`)

The configuration object can have the following properties:

-   **`path`** (string): The path to the file, used for resolving relative paths and useful for plugins.
-   **`plugins`** (array): An array of plugin configurations. This is the most important part of the configuration, allowing you to enable, disable, and configure specific plugins.
-   **`multipass`** (boolean): If `true`, runs optimizations multiple times until no further changes can be made. Defaults to `false`.
-   **`js2svg`** (object): Options for the JS to SVG conversion (output formatting).
    -   `pretty` (boolean): Whether to pretty-print the output SVG. Defaults to `false`.
    -   `indent` (number): Indentation level for pretty-printing. Defaults to `2`.
-   **`datauri`** (string): Output as a data URI string (`'base64'`, `'enc'`, or `'unenc'`).

#### Return Value

The `optimize` function returns an object with the following properties:

-   **`data`** (string): The optimized SVG string.
-   **`info`** (object): Information about the optimization process, such as original and optimized sizes.
-   **`error`** (string): An error message if something went wrong.
-   **`modern`** (boolean): Indicates if the modern (and faster) parser was used.

### Example Usage (Node.js)

```javascript
import { optimize } from 'svgo';

const svgString = '<svg width="100" height="100"><rect x="10" y="10" width="80" height="80" fill="red"/></svg>';

const result = optimize(svgString, {
  path: 'my-icon.svg',
  plugins: [
    'removeDimensions',
    {
      name: 'sortAttrs',
      params: {
        xmlnsOrder: 'alphabetical',
      },
    },
  ],
});

console.log(result.data);
```

## 4. Plugins

Plugins are the core of `svgo`. They perform the actual optimizations on the SVG's AST (Abstract Syntax Tree).

### Plugin API

Plugins are simple objects with a `name`, `fn` (the visitor function), and optional `params`. The `fn` function receives the root node of the AST and the plugin parameters.

### Default Preset

`svgo` includes a default preset of plugins that are safe to use and provide good optimization. This preset is enabled by default. The plugins in the default preset include:

-   `cleanupAttrs`
-   `cleanupEnableBackground`
-   `cleanupIds`
-   `cleanupListOfValues`
-   `cleanupNumericValues`
-   `collapseGroups`
-   `convertColors`
-   `convertPathData`
-   `convertShapeToPath`
-   `convertStyleToAttrs`
-   `convertTransform`
-   `inlineStyles`
-   `mergePaths`
-   `mergeStyles`
-   `minifyStyles`
-   `moveElemsAttrsToGroup`
-   `moveGroupAttrsToElems`
-   `removeComments`
-   `removeDesc`
-   `removeDoctype`
-   `removeEditorsNSData`
-   `removeEmptyAttrs`
-   `removeEmptyContainers`
-   `removeEmptyText`
-   `removeHiddenElems`
-   `removeMetadata`
-   `removeNonInheritableGroupAttrs`
-   `removeTitle`
-   `removeUnknownsAndDefaults`
-   `removeUnusedNS`
-   `removeUselessDefs`
-   `removeUselessStrokeAndFill`
-   `removeXMLProcInst`
-   `sortAttrs`
-   `sortDefsChildren`

### Full Plugin Reference

Below is a list of all available plugins in `svgo`.

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
-   **`convertPathData`**: Converts path data to relative or absolute, optimizes segments, and collapses sequences.
-   **`convertShapeToPath`**: Converts basic shapes (`<rect>`, `<circle>`, etc.) to `<path>` elements.
-   **`convertStyleToAttrs`**: Converts styles from `<style>` elements to attributes.
-   **`convertTransform`**: Collapses multiple transforms into one, converts matrices to shorter syntaxes, and more.
-   **`inlineStyles`**: Inlines styles from `<style>` elements to `style` attributes.
-   **`mergePaths`**: Merges multiple paths into one.
-   **`mergeStyles`**: Merges multiple `<style>` elements into one.
-   **`minifyStyles`**: Minifies CSS in `<style>` elements using `csso`.
-   **`moveElemsAttrsToGroup`**: Moves attributes from elements to their common group.
-   **`moveGroupAttrsToElems`**: Moves group attributes to the contained elements.
-   **`prefixIds`**: Adds a prefix to IDs.
-   **`preset-default`**: The default set of plugins.
-   **`removeAttributesBySelector`**: Removes attributes of elements that match a CSS selector.
-   **`removeAttrs`**: Removes attributes by name.
-   **`removeComments`**: Removes comments.
-   **`removeDeprecatedAttrs`**: Removes deprecated attributes.
-   **`removeDesc`**: Removes `<desc>` elements (enabled by default).
-   **`removeDimensions`**: Removes `width` and `height` attributes and preserves the `viewBox`.
-   **`removeDoctype`**: Removes doctype declarations.
-   **`removeEditorsNSData`**: Removes editor-specific namespaces, elements, and attributes.
-
-   **`removeElementsByAttr`**: Removes elements by ID or class name.
-   **`removeEmptyAttrs`**: Removes empty attributes.
-   **`removeEmptyContainers`**: Removes empty container elements.
-   **`removeEmptyText`**: Removes empty text elements.
-   **`removeHiddenElems`**: Removes hidden elements (`display="none"` or `visibility="hidden"`).
-   **`removeMetadata`**: Removes `<metadata>` elements.
-   **`removeNonInheritableGroupAttrs`**: Removes non-inheritable group's "presentation" attributes.
-   **`removeOffCanvasPaths`**: Removes elements that are drawn outside of the `viewBox`.
-   **`removeRasterImages`**: Removes raster images.
-   **`removeScripts`**: Removes `<script>` elements.
-   **`removeStyleElement`**: Removes `<style>` elements.
-   **`removeTitle`**: Removes `<title>` elements (enabled by default).
-   **`removeUnknownsAndDefaults`**: Removes unknown elements' content and attributes, and removes default attribute values.
-   **`removeUnusedNS`**: Removes unused namespace declarations.
-   **`removeUselessDefs`**: Removes elements in `<defs>` without an `id`.
-   **`removeUselessStrokeAndFill`**: Removes useless `stroke` and `fill` attributes.
-   **`removeViewBox`**: Removes the `viewBox` attribute when possible.
-   **`removeXlink`**: Removes `xlink` attributes.
-   **`removeXMLNS`**: Removes the `xmlns` attribute from the root `<svg>` element.
-   **`removeXMLProcInst`**: Removes XML processing instructions.
-   **`reusePaths`**: Replaces paths with `<use>` elements if they are identical.
-   **`sortAttrs`**: Sorts element attributes for better gzip compression.
-   **`sortDefsChildren`**: Sorts children of `<defs>` to improve compression.

## 5. Command-Line Interface (CLI)

`svgo` provides a CLI for optimizing files directly from the terminal.

### Basic Usage

```bash
svgo input.svg -o output.svg
```

### Options

-   `-i, --input`: Input file or directory.
-   `-o, --output`: Output file or directory.
-   `-f, --folder`: Input folder, optimize and rewrite all `*.svg` files.
-   `-p, --pretty`: Make SVG pretty printed.
-   `--config`: Custom config file.
-   `--disable`: Disable a plugin by name.
-   `--enable`: Enable a plugin by name.
-   `--datauri`: Output as Data URI string.
-   `--multipass`: Optimize SVG multiple times.
-   `--quiet`: Only output error messages.
-   `-v, --version`: Show version.
-   `-h, --help`: Show help.
