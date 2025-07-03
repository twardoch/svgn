# TODO

This is a flat task list derived from PLAN.md. Tasks are organized by priority and phase.

## Current Priority: Phase 1 Foundation & Core Infrastructure - COMPLETED

### Recently Completed
- [x] Create basic Rust project structure with Cargo.toml
- [x] Set up src/lib.rs with SVGO-compatible API
- [x] Set up src/main.rs with CLI interface
- [x] Update GitHub Actions workflow for proper Rust CI
- [x] Add core dependencies and configuration
- [x] Create build.sh script for building and verifying the project


### Attribute Processors  
- [x] `sortAttrs` ✅
- [x] `removeAttrs` ✅ 
- [x] `removeUnknownsAndDefaults` ✅
- [x] `addAttributesToSVGElement` ✅
- [x] `addClassesToSVGElement` ✅
- [x] `removeAttributesBySelector` ✅
- [x] `removeDeprecatedAttrs` ✅

### Style and Color Handlers
- [x] `convertColors` ✅
- [ ] `minifyStyles` (requires CSS parsing)
- [ ] `inlineStyles`
- [x] `mergeStyles` ✅
- [x] `convertStyleToAttrs` ✅
- [x] `removeStyleElement` ✅

### Transform and Path Optimizers
- [ ] `convertTransform`
- [ ] `convertPathData` (complex, uses lyon for path transformations)
- [ ] `convertShapeToPath`
- [ ] `mergePaths`

### Structural Optimizers
- [x] `collapseGroups` ✅
- [ ] `moveElemsAttrsToGroup`
- [ ] `moveGroupAttrsToElems`
- [ ] `removeNonInheritableGroupAttrs`

### Other Plugins
- [x] `convertEllipseToCircle` ✅
- [ ] `convertOneStopGradients`
- [ ] `prefixIds`
- [ ] `removeEditorsNSData`
- [ ] `removeElementsByAttr`
- [ ] `removeHiddenElems`
- [ ] `removeOffCanvasPaths`
- [ ] `removeRasterImages`
- [ ] `removeScripts`
- [ ] `removeUnusedNS`
- [ ] `removeUselessDefs`
- [ ] `removeUselessStrokeAndFill`
- [ ] `removeViewBox`
- [ ] `removeXlink`
- [ ] `removeXMLNS`
- [x] `removeDimensions`
- [ ] `reusePaths`
- [ ] `sortDefsChildren`

## Phase 4: Testing, Benchmarking, and CI

- [ ] Systematically port the entire `svgo` test suite from `ref/svgo/test/`
- [ ] Create a test runner that can execute tests and compare output of `svgn` with expected output from `svgo`
- [ ] Create a comprehensive benchmark suite using `cargo bench`
- [ ] Benchmarks should cover parsing, stringifying, and each of the major plugins
- [ ] Compare the performance of `svgn` against the original `svgo` to quantify performance gains
- [ ] Set up a CI pipeline using GitHub Actions
- [ ] CI pipeline will run `cargo test` on every push and pull request
- [ ] CI pipeline will run `cargo clippy` to enforce code quality
- [ ] CI pipeline will run `cargo fmt --check` to ensure consistent formatting
- [ ] (Optional) Run benchmarks and report on performance changes

## Phase 5: WebAssembly (WASM) Target

- [ ] Integrate `wasm-pack` into the build process
- [ ] Ensure the codebase is compatible with the WASM target
- [ ] Create a WASM-compatible `optimize` function using `wasm-bindgen`
- [ ] Function will accept a string and a `JsValue` for configuration, and return the optimized string
- [ ] Create a simple web page that uses the WASM-compiled `svgn` to optimize SVGs in the browser
- [ ] This will serve as a proof-of-concept and demonstration of the WASM capabilities

## Phase 6: Documentation and Release

- [ ] Prepare a Jekyll+Markdown structure that will serve as the documentation on Github Pages (via main branch, docs folder setup)

- [ ] Write comprehensive `rustdoc` comments for all public APIs, structs, and functions
- [ ] Publish the `svgn` crate to `crates.io`
- [ ] Publish the WASM package to `npm`
- [ ] Create a GitHub release with release notes

## Identified Issues

- [ ] **XML Entity Expansion in Parser**: The Rust parser does not handle XML entity declarations within the DOCTYPE for expansion, leading to different parsed content compared to the original SVGO.
- [ ] **Whitespace Preservation in Textual Tags**: The Rust parser applies global whitespace trimming, potentially losing significant whitespace in elements where it is semantically important.
- [ ] **Detailed Error Reporting in Parser**: The Rust parser lacks detailed context and code snippet highlighting in error messages, making debugging challenging.
- [ ] **Inconsistent Namespace Handling in AST**: The Rust AST stores `xmlns` attributes redundantly in both `namespaces` and `attributes`, and its handling differs from SVGO's typical flattening.
- [ ] **Document Metadata Handling and Usage**: The `path`, `encoding`, and `version` metadata fields are not consistently populated or utilized throughout the optimization pipeline and by plugins.
- [ ] **XML Declaration Stringification**: The Rust stringifier does not explicitly handle the XML declaration based on `DocumentMetadata`, potentially leading to loss or incorrect generation.
- [ ] **DOCTYPE Stringification**: The Rust stringifier does not explicitly handle the DOCTYPE declaration, leading to its loss during stringification.
- [ ] **Visitor Pattern Implementation**: The Rust plugin system uses a single `apply` method per plugin, which is less efficient and modular than SVGO's granular visitor pattern.
- [ ] **Missing Preset Implementation**: The Rust plugin system lacks a concept of "presets" like SVGO's `preset-default`, increasing configuration complexity.
- [ ] **Limited Dynamic Plugin Loading**: The Rust version requires plugins to be statically registered, restricting extensibility compared to SVGO's dynamic loading.
- [ ] **Inconsistent Plugin Parameter Validation**: The `validate_params` method in Rust plugins is inconsistently implemented and enforced, leading to potential runtime errors.
- [ ] **`cleanupEnableBackground` Plugin: Handling `enable-background` in `style` Attributes**: The Rust plugin does not process `enable-background` within `style` attributes, unlike the JS version.
- [ ] **`cleanupIds` Plugin: URL Encoding/Decoding for ID References**: The Rust plugin's URL encoding/decoding for ID references might not perfectly match JS `encodeURI`/`decodeURI` behavior, potentially causing discrepancies or broken references.
- [ ] **`cleanupIds` Plugin: Optimization Skip for `svg` with only `defs`**: The Rust plugin does not skip processing for SVGs containing only `<defs>` children, leading to minor performance inefficiencies.
- [ ] **`convertColors` Plugin: Incomplete `currentColor` Implementation**: The Rust plugin has a simplified `currentColor` implementation, lacking the full flexibility of the JS version.