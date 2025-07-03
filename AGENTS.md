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

### 3.1. `optimize(input, config)`

-   **`input`** (string, required): The SVG string to be optimized.
-   **`config`** (object, optional): The configuration object to customize the optimization process.

#### 3.1.1. Configuration Object (`config`)

The configuration object can have the following properties:

-   **`path`** (string): The path to the file, used for resolving relative paths and useful for plugins.
-   **`plugins`** (array): An array of plugin configurations. This is the most important part of the configuration, allowing you to enable, disable, and configure specific plugins.
-   **`multipass`** (boolean): If `true`, runs optimizations multiple times until no further changes can be made. Defaults to `false`.
-   **`js2svg`** (object): Options for the JS to SVG conversion (output formatting).
    -   `pretty` (boolean): Whether to pretty-print the output SVG. Defaults to `false`.
    -   `indent` (number): Indentation level for pretty-printing. Defaults to `2`.
-   **`datauri`** (string): Output as a data URI string (`'base64'`, `'enc'`, or `'unenc'`).

#### 3.1.2. Return Value

The `optimize` function returns an object with the following properties:

-   **`data`** (string): The optimized SVG string.
-   **`info`** (object): Information about the optimization process, such as original and optimized sizes.
-   **`error`** (string): An error message if something went wrong.
-   **`modern`** (boolean): Indicates if the modern (and faster) parser was used.

### 3.2. Example Usage (Node.js)

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

### 4.1. Plugin API

Plugins are simple objects with a `name`, `fn` (the visitor function), and optional `params`. The `fn` function receives the root node of the AST and the plugin parameters.

### 4.2. Default Preset

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

### 4.3. Full Plugin Reference

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

### 5.1. Basic Usage

```bash
svgo input.svg -o output.svg
```

### 5.2. Options

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

# `svgn`

`svgn` is an API-compatible native Rust port of `ref/svgo`, the JS SVG optimizer, along with all its plugins. It can be integrated into desktop and CLI workflows, or transpiled to Wasm for use in web applications. It is aimed to be extremely fast, and to be functionally fully compatible with `svgo`. 

## 6. Project Overview

This is SVGN - a planned Rust port of SVGO (SVG Optimizer). The project aims to create a native, high-performance SVG optimization library that is API-compatible with SVGO v4.0.0.

## 7. Current State

The project is in initial planning phase with:
- Complete SVGO v4.0.0 JavaScript reference implementation in `ref/svgo/`
- No Rust implementation yet
- Project structure needs to be created

## 8. Development Setup

### 8.1. For JavaScript Reference (in ref/svgo/)
```bash
cd ref/svgo
yarn install        # Install dependencies
yarn build          # Build bundles and types
yarn test           # Run tests with coverage
yarn lint           # Run ESLint and Prettier
yarn qa             # Run all quality checks
```

### 8.2. For Rust Implementation (to be created)
```bash
cargo build         # Build the project
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint code
cargo run -- [args] # Run the CLI
```

## 9. Architecture Reference

The SVGO architecture to be ported consists of:

1. **Core Engine** (`lib/svgo.js`): Plugin-based optimization pipeline
2. **Parser** (`lib/parser.js`): SVG string → AST using SAX parser
3. **Plugins** (`plugins/`): 50+ individual optimization plugins
4. **Stringifier** (`lib/stringifier.js`): AST → optimized SVG string
5. **CLI** (`lib/svgo-node.js`, `bin/svgo`): Command-line interface

Key design principles:
- Plugin-based architecture with configurable pipeline
- AST-based transformations
- Streaming support for large files
- Extensive test coverage

## 10. Implementation Guidelines

When porting to Rust:
1. Start with core parser/stringifier modules
2. Implement plugin infrastructure before individual plugins
3. Maintain API compatibility with SVGO's JavaScript API
4. Use existing Rust XML/SVG libraries where appropriate
5. Prioritize performance and memory efficiency
6. Enable WASM compilation from the start

## 11. Testing Strategy

- Port SVGO's extensive test suite from `test/` directory
- Ensure output parity with JavaScript implementation
- Add Rust-specific tests for performance and memory usage
- Use property-based testing for parser robustness

## 12. Key Files to Reference

- `ref/svgo/lib/svgo.js`: Core optimization logic
- `ref/svgo/lib/parser.js`: SVG parsing implementation
- `ref/svgo/plugins/`: Plugin implementations to port
- `ref/svgo/test/`: Test cases and fixtures
- `ref/svgo/docs/`: API and plugin documentation

## 13. When you write code (in any language)

- Iterate gradually, avoiding major changes
- Minimize confirmations and checks
- Preserve existing code/structure unless necessary
- Use constants over magic numbers
- Check for existing solutions in the codebase before starting
- Check often the coherence of the code you’re writing with the rest of the code.
- Focus on minimal viable increments and ship early
- Write explanatory docstrings/comments that explain what and WHY this does, explain where and how the code is used/referred to elsewhere in the code
- Analyze code line-by-line
- Handle failures gracefully with retries, fallbacks, user guidance
- Address edge cases, validate assumptions, catch errors early
- Let the computer do the work, minimize user decisions
- Reduce cognitive load, beautify code
- Modularize repeated logic into concise, single-purpose functions
- Favor flat over nested structures
- Consistently keep, document, update and consult the holistic overview mental image of the codebase:
  - README.md (purpose and functionality)
  - CHANGELOG.md (past changes)
  - TODO.md (future goals)
  - PROGRESS.md (detailed flat task list)

## 14. Use MCP tools if you can

Before and during coding (if have access to tools), you should:

- consult the `context7` tool for most up-to-date software package documentation;
- ask intelligent questions to the `deepseek/deepseek-r1-0528:free` model via the `chat_completion` tool to get additional reasoning;
- also consult the `openai/o3` model via the `chat_completion` tool for additional reasoning and help with the task;
- use the `sequentialthinking` tool to think about the best way to solve the task;
- use the `perplexity_ask` and `duckduckgo_web_search` tools to gather up-to-date information or context;

## 15. Keep track of paths

In each source file, maintain the up-to-date `this_file` record that shows the path of the current file relative to project root. Place the `this_file` record near the top of the file, as a comment after the shebangs, or in the YAML Markdown frontmatter.


## 16. Additional guidelines

Ask before extending/refactoring existing code in a way that may add complexity or break things.

When you’re finished, print "Wait, but" to go back, think & reflect, revise & improvement what you’ve done (but don’t invent functionality freely). Repeat this. But stick to the goal of "minimal viable next version".

## 17. Virtual team work

Be creative, diligent, critical, relentless & funny! Lead two experts: "Ideot" for creative, unorthodox ideas, and "Critin" to critique flawed thinking and moderate for balanced discussions. The three of you shall illuminate knowledge with concise, beautiful responses, process methodically for clear answers, collaborate step-by-step, sharing thoughts and adapting. If errors are found, step back and focus on accuracy and progress.

NOW: Read `./CLAUDE.md`, `./TODO.md` and `./PLAN.md` if they exist. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics. Research and consult extensively, and enrich `./PLAN.md` with as many details as it’s feasible. Then make sure that `./TODO.md` is the flat simplified itemized `- [ ]`-prefixed representation of `./PLAN.md`. Once these files are good, Implement the changes, and document them by moving them from `./TODO.md` to a `./CHANGELOG.md` file, and clean up `./PLAN.md` so that both `./TODO.md` and `./PLAN.md` only contain tasks not yet done. Be vigilant, thoughtful, intelligent, efficient. Work in iteration rounds.

If you work with Python, use 'uv pip' instead of 'pip', and use 'uvx hatch test' instead of 'python -m pytest'. 

When I say "/report", you must: Read all `./TODO.md` and `./PLAN.md` files and analyze recent changes. Document all changes in `./CHANGELOG.md`. From `./TODO.md` and `./PLAN.md` remove things that are done. Make sure that `./PLAN.md` contains a detailed, clear plan that discusses specifics, while `./TODO.md` is its flat simplified itemized `- [ ]`-prefixed representation. When I say "/work", you must work in iterations like so: Read all `./TODO.md` and `./PLAN.md` files and reflect. Work on the tasks. Think, contemplate, research, reflect, refine, revise. Be careful, curious, vigilant, energetic. Verify your changes. Think aloud. Consult, research, reflect. Then update `./PLAN.md` and `./TODO.md` with tasks that will lead to improving the work you’ve just done. Then '/report', and then iterate again.
