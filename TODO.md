# SVGN Development TODO List

## CRITICAL BUILD FIXES (IMMEDIATE PRIORITY)
- [ ] Fix CSS dependency version conflicts between lightningcss and selectors crates
- [ ] Align cssparser versions (0.31.2 vs 0.33.0) across dependency tree
- [ ] Fix ToCss trait not implemented for String types in selector implementations
- [ ] Update MatchingContext::new() function calls to match current API (6 args vs 4)
- [ ] Implement Parser trait for SvgSelectorImpl in SelectorList::parse() calls
- [ ] Fix method resolution failures - unescape() method not found on BytesText
- [ ] Fix private field access - SelectorList.0.iter() accessing private field
- [ ] Resolve all 22 compilation errors blocking development

## REMAINING PLUGINS (HIGH PRIORITY - after build fixes)
- [ ] Complete inlineStyles plugin CSS specificity-based cascade resolution engine
- [ ] Build media query and pseudo-class filtering logic for inlineStyles
- [ ] Convert matched CSS properties to SVG attributes in inlineStyles
- [ ] Clean up unused selectors and class/ID attributes in inlineStyles
- [ ] Implement mergePaths plugin - path concatenation with style matching
- [ ] Implement moveElemsAttrsToGroup plugin - attribute inheritance optimization
- [ ] Implement moveGroupAttrsToElems plugin - reverse attribute distribution

## WEBASSEMBLY BUILD IMPLEMENTATION

### Phase 1: WASM Build Infrastructure
- [ ] Install and configure wasm-pack for building WASM packages
- [ ] Create dedicated WASM build target in Cargo.toml
- [ ] Add WASM-specific feature flags for optional functionality
- [ ] Configure bundle size optimization settings
- [ ] Set up TypeScript definition generation
- [ ] Create svgn/src/wasm.rs as the WASM entry point
- [ ] Implement wasm_bindgen exports for core functionality
- [ ] Design JavaScript-friendly API surface
- [ ] Handle memory management across JS/WASM boundary
- [ ] Implement proper error handling with JS Error objects

### Phase 2: Bundle Size Optimization
- [ ] Create feature flags for plugin groups (default, extended, experimental)
- [ ] Implement conditional compilation for large plugins
- [ ] Add size-optimized build profile
- [ ] Configure dead code elimination
- [ ] Implement plugin registry filtering
- [ ] Use wee_alloc for smaller memory footprint
- [ ] Implement streaming processing for large files
- [ ] Add progress callbacks for long-running operations
- [ ] Optimize critical path performance
- [ ] Add benchmarking for WASM vs native performance

### Phase 3: JavaScript API Layer
- [ ] Implement comprehensive error handling for WASM API
- [ ] Add debug logging capabilities
- [ ] Create user-friendly error messages
- [ ] Handle WASM loading failures gracefully
- [ ] Add fallback mechanisms for unsupported browsers
- [ ] Implement preset system (default, aggressive, safe)
- [ ] Add plugin configuration builder
- [ ] Support for custom plugin parameters
- [ ] Configuration validation and sanitization
- [ ] Import/export configuration profiles

## ONLINE TOOL IMPLEMENTATION

### Phase 4: Web UI Foundation
- [ ] Add DaisyUI and Tailwind CSS to docs folder
- [ ] Create responsive layout structure
- [ ] Implement dark/light theme support
- [ ] Add custom CSS for SVG preview components
- [ ] Configure build process for CSS optimization
- [ ] Add Jekyll PostCSS integration
- [ ] Create tools collection for Jekyll
- [ ] Create base HTML structure for optimizer tool

### Phase 5: Core Functionality
- [ ] Implement file upload with drag-and-drop functionality
- [ ] Create file input handling and validation
- [ ] Build real-time SVG preview component
- [ ] Implement before/after comparison display
- [ ] Add file size statistics and optimization metrics
- [ ] Create configuration panel with plugin toggles
- [ ] Implement preset selection (default, aggressive, safe, custom)
- [ ] Add pretty print and output formatting options
- [ ] Create download functionality for optimized files

### Phase 6: Advanced Features
- [ ] Add multiple file upload support
- [ ] Implement batch optimization with progress indication
- [ ] Create ZIP file output for batch operations
- [ ] Build processing queue management
- [ ] Add cancel/pause functionality
- [ ] Implement URL import for remote SVG files
- [ ] Add export to various formats (inline, base64, etc.)
- [ ] Create code generation for HTML/CSS embedding
- [ ] Add plugin marketplace/registry browser
- [ ] Implement sharing optimized SVGs via URLs
- [ ] Create before/after comparison view
- [ ] Add plugin performance profiling
- [ ] Build optimization step-by-step visualization
- [ ] Create custom plugin development interface
- [ ] Add API documentation with live examples

### Phase 7: Build and Deployment
- [ ] Create build-wasm.sh script for automated WASM builds
- [ ] Set up wasm-opt for bundle optimization
- [ ] Configure GitHub Actions for web tool deployment
- [ ] Add Node.js and CSS build pipeline
- [ ] Implement Jekyll site building
- [ ] Set up GitHub Pages deployment
- [ ] Add bundle size tracking
- [ ] Implement performance regression detection
- [ ] Set up privacy-focused user analytics
- [ ] Add error reporting and monitoring
- [ ] Implement A/B testing for UI improvements

### Phase 8: Documentation
- [ ] Write complete JavaScript API reference
- [ ] Create WASM integration guide
- [ ] Document configuration options
- [ ] Add plugin parameter reference
- [ ] Write performance optimization guide
- [ ] Create basic optimization examples
- [ ] Add advanced configuration scenarios
- [ ] Document integration with popular frameworks
- [ ] Add Node.js server-side usage examples
- [ ] Create browser extension development guide
- [ ] Write getting started guide
- [ ] Create migrating from SVGO tutorial
- [ ] Add custom plugin development guide
- [ ] Document performance optimization techniques
- [ ] Create troubleshooting common issues guide

### Phase 9: Testing and QA
- [ ] Write unit tests for WASM API
- [ ] Add cross-browser compatibility testing
- [ ] Implement performance benchmarking
- [ ] Add file size regression testing
- [ ] Create UI interaction testing
- [ ] Ensure accessibility compliance testing
- [ ] Add complete user workflow testing
- [ ] Test file upload/download functionality
- [ ] Verify configuration persistence
- [ ] Test error handling scenarios
- [ ] Add mobile device testing

## STANDALONE PLUGINS (LOWER PRIORITY)
- [ ] Implement applyTransforms plugin - applies transform matrices to coordinates
- [ ] Implement reusePaths plugin - replace duplicate paths with use elements

## INFRASTRUCTURE IMPROVEMENTS
- [ ] Fix XML entity expansion (Issue #201)
- [ ] Parse <!ENTITY> declarations in DOCTYPE
- [ ] Build entity table during parsing
- [ ] Expand &entity; references throughout document
- [ ] Update parser in svgn/src/parser.rs
- [ ] Fix selective whitespace preservation (Issue #202)
- [ ] Preserve whitespace in <text>, <tspan>, <pre>, <script>, <style>
- [ ] Add context-aware whitespace handling
- [ ] Fix enhanced error reporting (Issue #203)
- [ ] Track line/column positions during parsing
- [ ] Provide context snippets in error messages
- [ ] Fix namespace handling consistency (Issue #204)
- [ ] Unify namespace handling in single location
- [ ] Remove redundancy between namespaces and attributes maps
- [ ] Fix document metadata usage (Issue #205)
- [ ] Ensure path, encoding, version are properly populated
- [ ] Use metadata throughout optimization pipeline
- [ ] Fix XML declaration support (Issue #206)
- [ ] Add XML declaration output based on DocumentMetadata
- [ ] Update stringifier in svgn/src/stringifier.rs
- [ ] Fix DOCTYPE preservation (Issue #207)
- [ ] Store DOCTYPE declarations during parsing
- [ ] Output DOCTYPE declarations with entities
- [ ] Handle public/system identifiers
- [ ] Implement visitor pattern (Issue #213)
- [ ] Create Visitor trait with enter/exit methods
- [ ] Support for different node types
- [ ] Enable fine-grained traversal control
- [ ] Update plugin system architecture
- [ ] Implement preset system (Issue #215)
- [ ] Create Preset trait
- [ ] Implement preset-default
- [ ] Support preset inheritance
- [ ] Allow custom presets
- [ ] Add dynamic plugin loading support (Issue #216)
- [ ] Plugin discovery mechanism
- [ ] Runtime loading API
- [ ] External plugin interface

## PLUGIN-SPECIFIC FIXES
- [ ] Fix cleanupEnableBackground style handling (Issue #225)
- [ ] Parse enable-background from style attributes
- [ ] Merge logic with attribute handling
- [ ] Fix cleanupIds URL encoding (Issue #227)

## DEFAULT PRESET ALIGNMENT
- [ ] Add missing plugins to default preset configuration
- [ ] Add removeDeprecatedAttrs
- [ ] Add mergeStyles
- [ ] Add cleanupNumericValues
- [ ] Add removeNonInheritableGroupAttrs
- [ ] Add cleanupEnableBackground
- [ ] Add removeHiddenElems
- [ ] Add convertShapeToPath
- [ ] Add convertEllipseToCircle
- [ ] Add sortDefsChildren
- [ ] Update plugin registry order to match SVGO preset order
- [ ] Test default preset compatibility

## CODE QUALITY
- [ ] Fix all Clippy warnings (27 warnings)
- [ ] Fix collapsible if statements (2)
- [ ] Fix needless borrows (2)
- [ ] Replace manual clamp with clamp function (1)
- [ ] Add #[derive(Default)] for 3 structs
- [ ] Implement Default for 17 structs with new()
- [ ] Fix recursive parameter warnings (3)
- [ ] Fix length comparison (1)
- [ ] Fix collapsible match (1)
- [ ] Remove needless return (1)
- [ ] Fix invalid regex with backreference in prefix_ids.rs
- [ ] Fix minor formatting issues in benches/optimization.rs

## TESTING
- [ ] Port remaining SVGO test fixtures
- [ ] Port all missing plugin test files from /ref/svgo/test/plugins/
- [ ] Implement Rust test cases for new plugins
- [ ] Add parameterized tests for plugin configurations
- [ ] Achieve 100% SVGO test compatibility
- [ ] Fix any output differences
- [ ] Ensure identical optimization results
- [ ] Target 100% test pass rate (currently 93.75%)
- [ ] Add fuzz testing for parser
- [ ] Create fuzzing harness
- [ ] Test parser robustness
- [ ] Add property-based tests

## CLI COMPLETION
- [ ] Add support for .js config files (currently only .json and .toml)
- [ ] Implement base64 encoding for datauri output (currently placeholder)

## BUILD & DISTRIBUTION
- [ ] Fix macOS universal binary build (Issue #412)
- [ ] Create Linux packaging (.deb, .rpm, .AppImage)
- [ ] Create Windows installer (.msi)
- [ ] Update GitHub Actions workflow
- [ ] Implement version management
- [ ] Git tag-based versioning
- [ ] Automatic version injection at build time
- [ ] Update set-cargo-version.sh script

## DOCUMENTATION UPDATES
- [ ] Update docs/plugins.md
- [ ] Add new plugin documentation
- [ ] Update implementation status
- [ ] Add parameter documentation
- [ ] Update docs/comparison.md
- [ ] Update plugin count (54/54)
- [ ] Update compatibility metrics
- [ ] Document performance characteristics
- [ ] Update README.md
- [ ] Update implementation status
- [ ] Update feature list
- [ ] Add migration guide links