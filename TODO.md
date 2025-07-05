# SVGN Development TODO List - Organized by Dependency Threads

## Thread A: Critical Build Fixes (HARD) - BLOCKING ALL DEVELOPMENT
**Dependencies:** None - must be completed before any other work can proceed
**Complexity:** Hard - requires deep understanding of Rust crate versioning and CSS selector APIs

- [ ] A1. Fix CSS dependency version conflicts between lightningcss and selectors crates
- [ ] A2. Align cssparser versions (0.31.2 vs 0.33.0) across dependency tree
- [ ] A3. Fix ToCss trait not implemented for String types in selector implementations
- [ ] A4. Update MatchingContext::new() function calls to match current API (6 args vs 4)
- [ ] A5. Implement Parser trait for SvgSelectorImpl in SelectorList::parse() calls
- [ ] A6. Fix method resolution failures - unescape() method not found on BytesText
- [ ] A7. Fix private field access - SelectorList.0.iter() accessing private field
- [ ] A8. Resolve all 22 compilation errors blocking development

## Thread B: Remaining Core Plugins (HARD) - depends on Thread A
**Dependencies:** Thread A must be completed first
**Complexity:** Hard - requires sophisticated CSS processing and SVG manipulation

- [ ] B1. Complete inlineStyles plugin CSS specificity-based cascade resolution engine
- [ ] B2. Build media query and pseudo-class filtering logic for inlineStyles
- [ ] B3. Convert matched CSS properties to SVG attributes in inlineStyles
- [ ] B4. Clean up unused selectors and class/ID attributes in inlineStyles
- [ ] B5. Implement mergePaths plugin - path concatenation with style matching
- [ ] B6. Implement moveElemsAttrsToGroup plugin - attribute inheritance optimization
- [ ] B7. Implement moveGroupAttrsToElems plugin - reverse attribute distribution

## Thread C: Standalone Plugins (MEDIUM) - depends on Thread A
**Dependencies:** Thread A must be completed first (independent of Thread B)
**Complexity:** Medium - leverages existing infrastructure

- [ ] C1. Implement applyTransforms plugin - applies transform matrices to coordinates
- [ ] C2. Implement reusePaths plugin - replace duplicate paths with use elements

## Thread D: WASM Build Infrastructure (EASY) - independent
**Dependencies:** None - can be developed in parallel with other threads
**Complexity:** Easy - standard Rust/WASM toolchain setup

- [ ] D1. Install and configure wasm-pack for building WASM packages
- [ ] D2. Create dedicated WASM build target in Cargo.toml
- [ ] D3. Add WASM-specific feature flags for optional functionality
- [ ] D4. Configure bundle size optimization settings
- [ ] D5. Set up TypeScript definition generation
- [ ] D6. Create svgn/src/wasm.rs as the WASM entry point
- [ ] D7. Implement wasm_bindgen exports for core functionality
- [ ] D8. Design JavaScript-friendly API surface
- [ ] D9. Handle memory management across JS/WASM boundary
- [ ] D10. Implement proper error handling with JS Error objects

## Thread E: WASM Optimization (MEDIUM) - depends on Thread D
**Dependencies:** Thread D must be completed first
**Complexity:** Medium - performance optimization and bundling

- [ ] E1. Create feature flags for plugin groups (default, extended, experimental)
- [ ] E2. Implement conditional compilation for large plugins
- [ ] E3. Add size-optimized build profile
- [ ] E4. Configure dead code elimination
- [ ] E5. Implement plugin registry filtering
- [ ] E6. Use wee_alloc for smaller memory footprint
- [ ] E7. Implement streaming processing for large files
- [ ] E8. Add progress callbacks for long-running operations
- [ ] E9. Optimize critical path performance
- [ ] E10. Add benchmarking for WASM vs native performance

## Thread F: JavaScript API Layer (MEDIUM) - depends on Thread E
**Dependencies:** Thread E must be completed first
**Complexity:** Medium - API design and error handling

- [ ] F1. Implement comprehensive error handling for WASM API
- [ ] F2. Add debug logging capabilities
- [ ] F3. Create user-friendly error messages
- [ ] F4. Handle WASM loading failures gracefully
- [ ] F5. Add fallback mechanisms for unsupported browsers
- [ ] F6. Implement preset system (default, aggressive, safe)
- [ ] F7. Add plugin configuration builder
- [ ] F8. Support for custom plugin parameters
- [ ] F9. Configuration validation and sanitization
- [ ] F10. Import/export configuration profiles

## Thread G: Web UI Foundation (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - standard web development with existing frameworks

- [ ] G1. Add DaisyUI and Tailwind CSS to docs folder
- [ ] G2. Create responsive layout structure
- [ ] G3. Implement dark/light theme support
- [ ] G4. Add custom CSS for SVG preview components
- [ ] G5. Configure build process for CSS optimization
- [ ] G6. Add Jekyll PostCSS integration
- [ ] G7. Create tools collection for Jekyll
- [ ] G8. Create base HTML structure for optimizer tool

## Thread H: Core Web Functionality (MEDIUM) - depends on Threads F and G
**Dependencies:** Threads F and G must be completed first
**Complexity:** Medium - integration of WASM API with web UI

- [ ] H1. Implement file upload with drag-and-drop functionality
- [ ] H2. Create file input handling and validation
- [ ] H3. Build real-time SVG preview component
- [ ] H4. Implement before/after comparison display
- [ ] H5. Add file size statistics and optimization metrics
- [ ] H6. Create configuration panel with plugin toggles
- [ ] H7. Implement preset selection (default, aggressive, safe, custom)
- [ ] H8. Add pretty print and output formatting options
- [ ] H9. Create download functionality for optimized files

## Thread I: Advanced Web Features (MEDIUM) - depends on Thread H
**Dependencies:** Thread H must be completed first
**Complexity:** Medium - advanced web functionality

- [ ] I1. Add multiple file upload support
- [ ] I2. Implement batch optimization with progress indication
- [ ] I3. Create ZIP file output for batch operations
- [ ] I4. Build processing queue management
- [ ] I5. Add cancel/pause functionality
- [ ] I6. Implement URL import for remote SVG files
- [ ] I7. Add export to various formats (inline, base64, etc.)
- [ ] I8. Create code generation for HTML/CSS embedding
- [ ] I9. Add plugin marketplace/registry browser
- [ ] I10. Implement sharing optimized SVGs via URLs
- [ ] I11. Create before/after comparison view
- [ ] I12. Add plugin performance profiling
- [ ] I13. Build optimization step-by-step visualization
- [ ] I14. Create custom plugin development interface
- [ ] I15. Add API documentation with live examples

## Thread J: Build and Deployment (EASY) - depends on Threads F and I
**Dependencies:** Threads F and I must be completed first
**Complexity:** Easy - standard CI/CD setup

- [ ] J1. Create build-wasm.sh script for automated WASM builds
- [ ] J2. Set up wasm-opt for bundle optimization
- [ ] J3. Configure GitHub Actions for web tool deployment
- [ ] J4. Add Node.js and CSS build pipeline
- [ ] J5. Implement Jekyll site building
- [ ] J6. Set up GitHub Pages deployment
- [ ] J7. Add bundle size tracking
- [ ] J8. Implement performance regression detection
- [ ] J9. Set up privacy-focused user analytics
- [ ] J10. Add error reporting and monitoring
- [ ] J11. Implement A/B testing for UI improvements

## Thread K: Documentation (EASY) - depends on all functional threads
**Dependencies:** Threads B, C, F, H, I must be substantially complete
**Complexity:** Easy - documentation writing

- [ ] K1. Write complete JavaScript API reference
- [ ] K2. Create WASM integration guide
- [ ] K3. Document configuration options
- [ ] K4. Add plugin parameter reference
- [ ] K5. Write performance optimization guide
- [ ] K6. Create basic optimization examples
- [ ] K7. Add advanced configuration scenarios
- [ ] K8. Document integration with popular frameworks
- [ ] K9. Add Node.js server-side usage examples
- [ ] K10. Create browser extension development guide
- [ ] K11. Write getting started guide
- [ ] K12. Create migrating from SVGO tutorial
- [ ] K13. Add custom plugin development guide
- [ ] K14. Document performance optimization techniques
- [ ] K15. Create troubleshooting common issues guide

## Thread L: Testing and QA (MEDIUM) - depends on all functional threads
**Dependencies:** Threads B, C, F, H, I must be substantially complete
**Complexity:** Medium - comprehensive testing across platforms

- [ ] L1. Write unit tests for WASM API
- [ ] L2. Add cross-browser compatibility testing
- [ ] L3. Implement performance benchmarking
- [ ] L4. Add file size regression testing
- [ ] L5. Create UI interaction testing
- [ ] L6. Ensure accessibility compliance testing
- [ ] L7. Add complete user workflow testing
- [ ] L8. Test file upload/download functionality
- [ ] L9. Verify configuration persistence
- [ ] L10. Test error handling scenarios
- [ ] L11. Add mobile device testing

## Thread M: Infrastructure Improvements (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - incremental improvements to existing code

- [ ] M1. Fix XML entity expansion (Issue #201)
- [ ] M2. Parse <!ENTITY> declarations in DOCTYPE
- [ ] M3. Build entity table during parsing
- [ ] M4. Expand &entity; references throughout document
- [ ] M5. Update parser in svgn/src/parser.rs
- [ ] M6. Fix selective whitespace preservation (Issue #202)
- [ ] M7. Preserve whitespace in <text>, <tspan>, <pre>, <script>, <style>
- [ ] M8. Add context-aware whitespace handling
- [ ] M9. Fix enhanced error reporting (Issue #203)
- [ ] M10. Track line/column positions during parsing
- [ ] M11. Provide context snippets in error messages
- [ ] M12. Fix namespace handling consistency (Issue #204)
- [ ] M13. Unify namespace handling in single location
- [ ] M14. Remove redundancy between namespaces and attributes maps
- [ ] M15. Fix document metadata usage (Issue #205)
- [ ] M16. Ensure path, encoding, version are properly populated
- [ ] M17. Use metadata throughout optimization pipeline
- [ ] M18. Fix XML declaration support (Issue #206)
- [ ] M19. Add XML declaration output based on DocumentMetadata
- [ ] M20. Update stringifier in svgn/src/stringifier.rs
- [ ] M21. Fix DOCTYPE preservation (Issue #207)
- [ ] M22. Store DOCTYPE declarations during parsing
- [ ] M23. Output DOCTYPE declarations with entities
- [ ] M24. Handle public/system identifiers

## Thread N: Architecture Improvements (MEDIUM) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Medium - architectural changes

- [ ] N1. Implement visitor pattern (Issue #213)
- [ ] N2. Create Visitor trait with enter/exit methods
- [ ] N3. Support for different node types
- [ ] N4. Enable fine-grained traversal control
- [ ] N5. Update plugin system architecture
- [ ] N6. Implement preset system (Issue #215)
- [ ] N7. Create Preset trait
- [ ] N8. Implement preset-default
- [ ] N9. Support preset inheritance
- [ ] N10. Allow custom presets
- [ ] N11. Add dynamic plugin loading support (Issue #216)
- [ ] N12. Plugin discovery mechanism
- [ ] N13. Runtime loading API
- [ ] N14. External plugin interface

## Thread O: Plugin-Specific Fixes (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - minor bug fixes

- [ ] O1. Fix cleanupEnableBackground style handling (Issue #225)
- [ ] O2. Parse enable-background from style attributes
- [ ] O3. Merge logic with attribute handling
- [ ] O4. Fix cleanupIds URL encoding (Issue #227)

## Thread P: Default Preset Alignment (EASY) - depends on Threads B and C
**Dependencies:** Threads B and C must be completed first
**Complexity:** Easy - configuration updates

- [ ] P1. Add missing plugins to default preset configuration
- [ ] P2. Add removeDeprecatedAttrs
- [ ] P3. Add mergeStyles
- [ ] P4. Add cleanupNumericValues
- [ ] P5. Add removeNonInheritableGroupAttrs
- [ ] P6. Add cleanupEnableBackground
- [ ] P7. Add removeHiddenElems
- [ ] P8. Add convertShapeToPath
- [ ] P9. Add convertEllipseToCircle
- [ ] P10. Add sortDefsChildren
- [ ] P11. Update plugin registry order to match SVGO preset order
- [ ] P12. Test default preset compatibility

## Thread Q: Code Quality (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - code cleanup

- [ ] Q1. Fix all Clippy warnings (27 warnings)
- [ ] Q2. Fix collapsible if statements (2)
- [ ] Q3. Fix needless borrows (2)
- [ ] Q4. Replace manual clamp with clamp function (1)
- [ ] Q5. Add #[derive(Default)] for 3 structs
- [ ] Q6. Implement Default for 17 structs with new()
- [ ] Q7. Fix recursive parameter warnings (3)
- [ ] Q8. Fix length comparison (1)
- [ ] Q9. Fix collapsible match (1)
- [ ] Q10. Remove needless return (1)
- [ ] Q11. Fix invalid regex with backreference in prefix_ids.rs
- [x] Q12. Fix minor formatting issues in benches/optimization.rs - COMPLETED: Applied rustfmt across entire codebase

## Thread R: Testing Infrastructure (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - test setup and porting

- [ ] R1. Port remaining SVGO test fixtures
- [ ] R2. Port all missing plugin test files from /ref/svgo/test/plugins/
- [ ] R3. Implement Rust test cases for new plugins
- [ ] R4. Add parameterized tests for plugin configurations
- [ ] R5. Achieve 100% SVGO test compatibility
- [ ] R6. Fix any output differences
- [ ] R7. Ensure identical optimization results
- [ ] R8. Target 100% test pass rate (currently 93.75%)
- [ ] R9. Add fuzz testing for parser
- [ ] R10. Create fuzzing harness
- [ ] R11. Test parser robustness
- [ ] R12. Add property-based tests

## Thread S: CLI Completion (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - feature additions

- [ ] S1. Add support for .js config files (currently only .json and .toml)
- [x] S2. Implement base64 encoding for datauri output (currently placeholder) - COMPLETED: Implemented proper base64 and URL encoding using base64 and urlencoding crates

## Thread T: Build & Distribution (EASY) - independent
**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - build system improvements

- [ ] T1. Fix macOS universal binary build (Issue #412)
- [ ] T2. Create Linux packaging (.deb, .rpm, .AppImage)
- [ ] T3. Create Windows installer (.msi)
- [ ] T4. Update GitHub Actions workflow
- [ ] T5. Implement version management
- [ ] T6. Git tag-based versioning
- [ ] T7. Automatic version injection at build time
- [ ] T8. Update set-cargo-version.sh script

## Thread U: Documentation Updates (EASY) - depends on Threads B and C
**Dependencies:** Threads B and C must be completed first
**Complexity:** Easy - documentation updates

- [ ] U1. Update docs/plugins.md
- [ ] U2. Add new plugin documentation
- [ ] U3. Update implementation status
- [ ] U4. Add parameter documentation
- [ ] U5. Update docs/comparison.md
- [ ] U6. Update plugin count (54/54)
- [ ] U7. Update compatibility metrics
- [ ] U8. Document performance characteristics
- [ ] U9. Update README.md
- [ ] U10. Update implementation status
- [ ] U11. Update feature list
- [ ] U12. Add migration guide links

---

## Thread Summary

**Critical Path (must be done in order):**
Thread A → Thread B → Thread P → Thread U

**WebAssembly Path (can be done in parallel):**
Thread D → Thread E → Thread F → Thread H → Thread I → Thread J

**Independent Threads (can be done anytime):**
Threads C, G, M, N, O, Q, R, S, T

**Documentation/Testing (done after features):**
Threads K, L (depend on feature completion)

**Complexity Levels:**
- **HARD:** Threads A, B
- **MEDIUM:** Threads C, E, F, H, I, L, N  
- **EASY:** Threads D, G, J, K, M, O, P, Q, R, S, T, U