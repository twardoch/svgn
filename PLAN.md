# svgn: Implementation Plan

This document outlines the detailed plan for building `svgn`, a high-performance, native Rust port of the `svgo` SVG optimizer. The goal is to achieve full functional and API compatibility with the original JavaScript version while leveraging Rust's performance and safety features.

## Phase 1: Foundation & Core Infrastructure

This phase focuses on setting up the project and building the fundamental components that all other parts will rely on.

-   [x] 1.1. Project Setup
    -   [x] Initialize a new Rust library project: `cargo new svgn`
    -   [x] Set up the `Cargo.toml` file with project metadata (name, version, authors, license).
    -   [x] Create the initial directory structure:
        -   `src/`: Main library code.
        -   `src/bin/`: For the CLI binary.
        -   `tests/`: Integration tests.
        -   `benches/`: Benchmarks.
        -   `docs/`: User documentation.
    -   [x] Add initial dependencies to `Cargo.toml`:
        -   XML Parser: `quick-xml` (for fast streaming parsing, then build custom mutable AST).
        -   CSS Parser: `cssparser` (Mozilla's battle-tested CSS parser).
        -   CLI Argument Parser: `clap` v4 with derive features.
        -   Configuration: `serde`, `serde_json`, and `toml` (for `svgo.config.js` compatibility and native config).
        -   Path Processing: `lyon` (for advanced path data optimization).
        -   Optional: `usvg` utilities for SVG-specific operations.

-   **[x] 1.2. Abstract Syntax Tree (AST)**
    -   [x] Define the core Rust structs and enums for the SVG AST (e.g., `Document`, `Element`, `Attribute`, `Text`, `Comment`).
    -   [x] The AST should be designed for efficient traversal and mutation.

-   **[x] 1.3. SVG Parser**
    -   [x] Implement the parser using `quick-xml` for fast streaming XML parsing.
    -   [x] Build custom mutable AST optimized for SVG transformations (not using XML library's tree directly).
    -   [x] Ensure the parser correctly handles SVG-specific features, namespaces, and edge cases.
    -   [x] Consider leveraging `usvg` utilities for SVG-specific parsing challenges.
    -   [x] Port initial parser tests from `ref/svgo/test/svg2js/`.

-   **[x] 1.4. SVG Stringifier**
    -   [x] Implement the logic to traverse the AST and convert it back into an optimized SVG string.
    -   [x] Support both compact and pretty-printed output, controlled by configuration.

-   **[x] 1.5. Plugin Engine**
    -   [x] Define a `Plugin` trait that all optimization plugins will implement.
    -   [x] The trait will have a primary method, e.g., `apply(&self, ast: &mut Document, params: &Option<serde_json::Value>)`.
    -   [x] Create the core optimization pipeline that takes a list of plugins, iterates through them, and applies them to the AST.

-   **[x] 1.6. Configuration Handling**
    -   [x] Create a `Config` struct using `serde` to represent the `svgo` configuration.
    -   [x] Implement logic to load configuration from a `svgo.config.js` (by shelling out to Node.js to get the JSON) or a future `svgn.toml`.

## Phase 2: CLI and Initial Optimization

This phase focuses on creating a usable command-line tool and proving the core pipeline with a few initial plugins.

-   **[x] 2.1. Command-Line Interface (CLI)**
    -   [x] Use `clap` to build the CLI, matching the options of the original `svgo` CLI (`-i`, `-o`, `-f`, `--config`, etc.).
    -   [x] The CLI will parse arguments, load the configuration, read the input SVG, call the core `optimize` function, and write the output.

-   **[x] 2.2. `optimize` Function**
    -   [x] Implement the main `optimize` function that orchestrates the entire process: `parse -> apply plugins -> stringify`.
    -   [x] This function will be the primary entry point for both the library and the CLI.

-   **[x] 2.3. Port Initial Plugins**
    -   [x] Port a few simple, representative plugins to test the entire pipeline end-to-end.
        -   [x] `removeComments`
        -   [x] `removeMetadata`
        -   [x] `removeTitle`
    -   [x] Port the corresponding tests for these plugins to validate their correctness.

## Phase 3: Full Plugin Porting

This is the most extensive phase, involving the porting of all `svgo` plugins to Rust.

### Plugin Implementation Strategy

For efficient plugin porting, we'll categorize plugins by complexity and dependencies:

**Simple Plugins** (attribute/element removal):
- These plugins simply remove or modify specific elements/attributes
- Estimated time: 30-60 minutes each
- Examples: removeDoctype, removeXMLProcInst, removeDesc

**Medium Complexity Plugins** (pattern matching and transformation):
- These require regex patterns and more complex logic
- Estimated time: 1-2 hours each
- Examples: cleanupNumericValues, removeEmptyAttrs, sortAttrs

**Complex Plugins** (deep transformations):
- These require parsing CSS, transforms, or path data
- Estimated time: 2-4 hours each
- Examples: convertPathData, minifyStyles, convertTransform

**Plugin Dependencies to Add**:
- CSS parsing: Already have `cssparser`
- Color manipulation: May need `palette` or custom implementation
- Transform matrix operations: Custom implementation or `nalgebra`
- Path parsing/manipulation: Already have `lyon`

-   **[ ] 3.1. Plugin Porting Checklist**
    -   A checklist will be maintained here to track the status of each plugin.
    -   *Status: (P)lanned, (I)n-Progress, (D)one*
    -   [P] `addAttributesToSVGElement`
    -   [P] `addClassesToSVGElement`
    -   [D] `cleanupAttrs`
    -   [D] `cleanupEnableBackground`
    -   [D] `cleanupIds`
    -   [D] `cleanupListOfValues`
    -   [D] `cleanupNumericValues`
    -   [P] `collapseGroups`
    -   [D] `convertColors`
    -   [P] `convertEllipseToCircle`
    -   [P] `convertOneStopGradients`
    -   [P] `convertPathData`
    -   [P] `convertShapeToPath`
    -   [D] `convertStyleToAttrs`
    -   [P] `convertTransform`
    -   [P] `inlineStyles`
    -   [P] `mergePaths`
    -   [D] `mergeStyles`
    -   [P] `minifyStyles`
    -   [P] `moveElemsAttrsToGroup`
    -   [P] `moveGroupAttrsToElems`
    -   [P] `prefixIds`
    -   [P] `removeAttributesBySelector`
    -   [D] `removeAttrs`
    -   [D] `removeComments`
    -   [P] `removeDeprecatedAttrs`
    -   [D] `removeDesc`
    -   [P] `removeDimensions`
    -   [D] `removeDoctype`
    -   [P] `removeEditorsNSData`
    -   [P] `removeElementsByAttr`
    -   [D] `removeEmptyAttrs`
    -   [D] `removeEmptyContainers`
    -   [D] `removeEmptyText`
    -   [P] `removeHiddenElems`
    -   [D] `removeMetadata`
    -   [P] `removeNonInheritableGroupAttrs`
    -   [P] `removeOffCanvasPaths`
    -   [P] `removeRasterImages`
    -   [P] `removeScripts`
    -   [D] `removeStyleElement`
    -   [D] `removeTitle`
    -   [D] `removeUnknownsAndDefaults`
    -   [P] `removeUnusedNS`
    -   [P] `removeUselessDefs`
    -   [P] `removeUselessStrokeAndFill`
    -   [P] `removeViewBox`
    -   [P] `removeXlink`
    -   [P] `removeXMLNS`
    -   [D] `removeXMLProcInst`
    -   [P] `reusePaths`
    -   [D] `sortAttrs`
    -   [P] `sortDefsChildren`

### Implementation Progress Summary

**Total Plugins Completed: 20/54 (37%)**

**Completed Batches:**
- ✅ **Simple Removers (6 plugins)**: removeComments, removeDesc, removeDoctype, removeMetadata, removeTitle, removeXMLProcInst
- ✅ **Numeric/Value Cleaners (4 plugins)**: cleanupAttrs, cleanupIds, cleanupNumericValues, cleanupListOfValues  
- ✅ **Empty Element Cleaners (3 plugins)**: removeEmptyAttrs, removeEmptyContainers, removeEmptyText
- ✅ **Attribute Processors (3 plugins)**: sortAttrs, removeAttrs, removeUnknownsAndDefaults
- ✅ **Style Handlers (3 plugins)**: removeStyleElement, mergeStyles, convertStyleToAttrs

**Key Technical Achievements:**
- ✅ Fixed HashMap ordering issue by migrating to IndexMap for attribute preservation
- ✅ Implemented comprehensive regex-based pattern matching for removeAttrs
- ✅ Added simplified SVG specification compliance for removeUnknownsAndDefaults
- ✅ Implemented CSS parsing regex for style attribute conversion
- ✅ Added PRESENTATION_ATTRS collection for SVG presentation attributes
- ✅ Comprehensive test coverage: **129+ tests passing**

### Next Implementation Priorities

Based on complexity and usefulness, the recommended order for next plugins:

1. **✅ Attribute processors** *(COMPLETED)*:
   - ✅ `sortAttrs`
   - ✅ `removeAttrs`
   - ✅ `removeUnknownsAndDefaults`

2. **Style and color handlers**:
   - `convertColors`
   - `minifyStyles` (requires CSS parsing)
   - `inlineStyles`

3. **Transform and path optimizers**:
   - `convertTransform`
   - `convertPathData` (most complex, uses lyon)
   - `convertShapeToPath`

## Phase 4: Testing, Benchmarking, and CI

This phase focuses on ensuring the correctness, performance, and stability of the project.

-   **[ ] 4.1. Test Suite Porting**
    -   [ ] Systematically port the entire `svgo` test suite from `ref/svgo/test/`.
    -   [ ] Create a test runner that can easily execute these tests and compare the output of `svgn` with the expected output from `svgo`.

-   **[ ] 4.2. Benchmarking**
    -   [ ] Create a comprehensive benchmark suite using `cargo bench`.
    -   [ ] Benchmarks should cover parsing, stringifying, and each of the major plugins.
    -   [ ] Compare the performance of `svgn` against the original `svgo` to quantify the performance gains.

-   **[ ] 4.3. Continuous Integration (CI)**
    -   [ ] Set up a CI pipeline using GitHub Actions.
    -   [ ] The CI pipeline will:
        -   Run `cargo test` on every push and pull request.
        -   Run `cargo clippy` to enforce code quality.
        -   Run `cargo fmt --check` to ensure consistent formatting.
        -   (Optional) Run benchmarks and report on performance changes.

## Phase 5: WebAssembly (WASM) Target

This phase focuses on making `svgn` available in web environments.

-   **[ ] 5.1. WASM Build Setup**
    -   [ ] Integrate `wasm-pack` into the build process.
    -   [ ] Ensure the codebase is compatible with the WASM target.

-   **[ ] 5.2. WASM Bindings**
    -   [ ] Create a WASM-compatible `optimize` function using `wasm-bindgen`.
    -   [ ] This function will accept a string and a `JsValue` for configuration, and return the optimized string.

-   **[ ] 5.3. Web Demo**
    -   [ ] Create a simple web page that uses the WASM-compiled `svgn` to optimize SVGs in the browser.
    -   [ ] This will serve as a proof-of-concept and a demonstration of the WASM capabilities.

## Phase 6: Documentation and Release

This final phase focuses on preparing the project for public use.

-   **[ ] 6.1. Code Documentation**
    -   [ ] Write comprehensive `rustdoc` comments for all public APIs, structs, and functions.

-   **[ ] 6.2. User Documentation**
    -   [ ] Create user-friendly documentation in the `docs/` directory.
    -   [ ] This will include guides on how to use the CLI, the Rust library, and the WASM module.
    -   [ ] Provide a detailed reference for all configuration options and plugins.

-   **[ ] 6.3. Release**
    -   [ ] Publish the `svgn` crate to `crates.io`.
    -   [ ] Publish the WASM package to `npm`.
    -   [ ] Create a GitHub release with release notes.

## Technical Architecture Decisions

### XML Parsing Strategy
- Use `quick-xml` for initial parsing (streaming, fast, WASM-compatible)
- Build custom mutable AST rather than using XML library's tree
- This provides optimal performance for the read → transform → write pipeline

### Path Processing
- Leverage `lyon` crate for complex path data transformations
- This will be crucial for the `convertPathData` plugin which is one of the most sophisticated

### Plugin Architecture  
- Use trait objects for dynamic plugin loading
- Each plugin implements a common `Plugin` trait
- Support both built-in plugins and potential future extensibility

### WASM Considerations
- All chosen libraries are WASM-compatible
- Avoid file I/O in core library (use readers/writers)
- Use `wasm-bindgen` for JavaScript interop

### Performance Priorities
1. Memory efficiency for large SVG files
2. Fast parsing and stringification
3. Efficient tree traversal and mutation
4. Minimize allocations during optimization passes