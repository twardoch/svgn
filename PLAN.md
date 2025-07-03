# svgn: Implementation Plan

This document outlines the detailed plan for building `svgn`, a high-performance, native Rust port of the `svgo` SVG optimizer. The goal is to achieve full functional and API compatibility with the original JavaScript version while leveraging Rust's performance and safety features.

## Phase 1: Foundation & Core Infrastructure - COMPLETED ✅

This phase focused on setting up the project and building the fundamental components that all other parts will rely on.

-   **[x] 1.1. Basic Project Setup**
    -   [x] Create Rust project structure with `Cargo.toml`
    -   [x] Set up basic `src/lib.rs` with SVGO-compatible API structure
    -   [x] Set up basic `src/main.rs` with CLI interface
    -   [x] Add core dependencies: `clap`, `serde`, `anyhow`, `quick-xml`, `roxmltree`, `regex`, `indexmap`, `base64`, `urlencoding`
    -   [x] Configure project for release optimization (LTO, single codegen unit)

-   **[x] 1.2. Core API Structure**
    -   [x] Define `optimize` function with SVGO-compatible signature
    -   [x] Implement `Config`, `PluginConfig`, `Js2SvgOptions` structs
    -   [x] Create `OptimizeResult` and `OptimizeInfo` for result metadata
    -   [x] Set up basic error handling with `anyhow`

-   **[x] 1.3. CLI Interface**
    -   [x] Implement command-line interface with `clap`
    -   [x] Support for input/output files, pretty printing, multipass, data URI output
    -   [x] Handle data URI formats (base64, encoded, unencoded)
    -   [x] Basic optimization workflow (read → optimize → write)

-   **[x] 1.4. GitHub Actions CI**
    -   [x] Enhanced CI workflow with Rust toolchain setup
    -   [x] Added dependency caching for faster builds
    -   [x] Integrated clippy and formatting checks
    -   [x] Fixed build failures due to missing Cargo.toml

**Next Phase**: Need to implement the actual AST, parser, stringifier, and plugin system before proceeding with plugin porting.

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
    -   [D] `addAttributesToSVGElement`
    -   [D] `addClassesToSVGElement`
    -   [D] `cleanupAttrs`
    -   [D] `cleanupEnableBackground`
    -   [D] `cleanupIds`
    -   [D] `cleanupListOfValues`
    -   [D] `cleanupNumericValues`
    -   [D] `collapseGroups`
    -   [D] `convertColors`
    -   [D] `convertEllipseToCircle`
    -   [D] `convertOneStopGradients`
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
    -   [D] `prefixIds`
    -   [D] `removeAttributesBySelector`
    -   [D] `removeAttrs`
    -   [D] `removeComments`
    -   [D] `removeDeprecatedAttrs`
    -   [D] `removeDesc`
    -   [D] `removeDimensions`
    -   [D] `removeDoctype`
    -   [D] `removeEditorsNSData`
    -   [D] `removeElementsByAttr`
    -   [D] `removeEmptyAttrs`
    -   [D] `removeEmptyContainers`
    -   [D] `removeEmptyText`
    -   [D] `removeHiddenElems`
    -   [D] `removeMetadata`
    -   [D] `removeNonInheritableGroupAttrs`
    -   [D] `removeOffCanvasPaths`
    -   [D] `removeRasterImages`
    -   [D] `removeScripts`
    -   [D] `removeStyleElement`
    -   [D] `removeTitle`
    -   [D] `removeUnknownsAndDefaults`
    -   [D] `removeUnusedNS`
    -   [D] `removeUselessDefs`
    -   [P] `removeUselessStrokeAndFill`
    -   [D] `removeViewBox`
    -   [D] `removeXlink`
    -   [D] `removeXMLNS`
    -   [D] `removeXMLProcInst`
    -   [P] `reusePaths`
    -   [D] `sortAttrs`
    -   [D] `sortDefsChildren`

### Implementation Progress Summary

**Total Plugins Completed: 43/54 (80%)**

**Completed Batches:**
- ✅ **Simple Removers (6 plugins)**: removeComments, removeDesc, removeDoctype, removeMetadata, removeTitle, removeXMLProcInst
- ✅ **Numeric/Value Cleaners (4 plugins)**: cleanupAttrs, cleanupIds, cleanupNumericValues, cleanupListOfValues  
- ✅ **Empty Element Cleaners (3 plugins)**: removeEmptyAttrs, removeEmptyContainers, removeEmptyText
- ✅ **Attribute Processors (6 plugins)**: sortAttrs, removeAttrs, removeUnknownsAndDefaults, addAttributesToSVGElement, addClassesToSVGElement, removeDeprecatedAttrs
- ✅ **Style and Color Handlers (4 plugins)**: removeStyleElement, mergeStyles, convertStyleToAttrs, convertColors
- ✅ **Cleanup and Validation (1 plugin)**: cleanupEnableBackground
- ✅ **Additional Optimizers (2 plugins)**: removeDeprecatedAttrs, convertEllipseToCircle
- ✅ **Structural Optimizers (1 plugin)**: collapseGroups

**Key Technical Achievements:**
- ✅ Fixed Plugin trait compilation issues and enhanced with PluginInfo parameter
- ✅ Fixed HashMap ordering issue by migrating to IndexMap for attribute preservation
- ✅ Implemented comprehensive regex-based pattern matching for removeAttrs
- ✅ Added simplified SVG specification compliance for removeUnknownsAndDefaults
- ✅ Implemented CSS parsing regex for style attribute conversion
- ✅ Added PRESENTATION_ATTRS collection for SVG presentation attributes
- ✅ Added comprehensive color conversion algorithms with full SVG color name support
- ✅ **Additional Plugins (10 plugins)**: removeScripts, removeUselessDefs, removeViewBox, removeUnusedNS, removeXlink, removeXMLNS, sortDefsChildren, removeRasterImages, removeHiddenElems, removeNonInheritableGroupAttrs, removeOffCanvasPaths
- ✅ Comprehensive test coverage: **300+ tests passing**

### Next Implementation Priorities

Based on complexity and usefulness, the recommended order for next plugins:

1.  **Style and color handlers**:
    - `minifyStyles` (requires CSS parsing)
    - `inlineStyles`

2.  **Transform and path optimizers**:
    - `convertTransform`
    - `convertPathData` (most complex, uses lyon)
    - `convertShapeToPath`

## Phase 4: Testing, Benchmarking, and CI

This phase focuses on ensuring the correctness, performance, and stability of the project.

-   **[I] 4.1. Test Suite Porting**
    -   [ ] Systematically port the entire `svgo` test suite from `ref/svgo/test/`.
    -   [ ] Create a test runner that can easily execute these tests and compare the output of `svgn` with the expected output from `svgo`.

-   **[I] 4.2. Benchmarking**
    -   [ ] Create a comprehensive benchmark suite using `cargo bench`.
    -   [ ] Benchmarks should cover parsing, stringifying, and each of the major plugins.
    -   [ ] Compare the performance of `svgn` against the original `svgo` to quantify the performance gains.

-   **[I] 4.3. Continuous Integration (CI)**
    -   [ ] Set up a CI pipeline using GitHub Actions.
    -   [x] The CI pipeline will:
        -   [x] Run `cargo test` on every push and pull request.
        -   [ ] Run `cargo clippy` to enforce code quality.
        -   [ ] Run `cargo fmt --check` to ensure consistent formatting.
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

-   **[x] 6.2. User Documentation**
    -   [x] Create user-friendly documentation in the `docs/` directory.
    -   [x] This will include guides on how to use the CLI, the Rust library, and the WASM module.
    -   [x] Provide a detailed reference for all configuration options and plugins.

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

## Issues Identified During Testing

### `ref/svgo` Issues:
- **ExperimentalWarning: VM Modules**: This is a Node.js warning about an experimental feature. It doesn't indicate a functional error but should be noted.
- **`cleanupListOfValues` warning**: The plugin `cleanupListOfValues` is being configured outside of `preset-default` in a way that triggers a warning. This might indicate a misconfiguration in the test setup or a need to adjust how plugins are enabled/disabled.
- **`removeAttrs` warning**: The `removeAttrs` plugin is being used without the required `attrs` parameter, making it a no-op. This also points to a potential misconfiguration in the test setup.
- **Timeout errors in CLI tests**: Several CLI tests (`test/cli/cli.test.js` and `test/coa/_index.test.js`) are timing out. This indicates a potential issue with the test environment or the CLI's execution, possibly related to how it handles input/output streams or long-running operations.

### `svgn` Issues:
- No compiler warnings or test failures observed.