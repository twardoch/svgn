# SVGN TODO List

## Phase 1A: Critical Default Preset Plugins (Weeks 1-3)

### 1.1 inlineStyles (1.5 weeks) - CURRENT PRIORITY

- [ ] Add lightningcss or css crate dependency
- [ ] Create plugin file: `svgn/src/plugins/inline_styles.rs`
- [ ] Parse CSS from `<style>` elements
- [ ] Implement CSS specificity calculation
- [ ] Match CSS selectors to SVG elements
- [ ] Apply cascade rules and inheritance
- [ ] Convert CSS properties to SVG attributes
- [ ] Handle media queries and pseudo-classes
- [ ] Support all 4 SVGO parameters
- [ ] Add test suite

### 1.2 mergePaths (1 week)

- [ ] Create plugin file: `svgn/src/plugins/merge_paths.rs`
- [ ] Group paths by identical style attributes
- [ ] Check DOM adjacency for mergeable paths
- [ ] Concatenate path data strings correctly
- [ ] Support force, floatPrecision, noSpaceAfterFlags parameters
- [ ] Add test suite

### 1.3 moveElemsAttrsToGroup (0.5 weeks)

- [ ] Create plugin file: `svgn/src/plugins/move_elems_attrs_to_group.rs`
- [ ] Analyze attributes across sibling elements
- [ ] Identify inheritable SVG attributes
- [ ] Calculate size reduction benefit
- [ ] Create `<g>` wrapper when beneficial
- [ ] Move attributes and remove from children
- [ ] Add test suite

### 1.4 moveGroupAttrsToElems (0.5 weeks)

- [ ] Create plugin file: `svgn/src/plugins/move_group_attrs_to_elems.rs`
- [ ] Identify groups that only provide attributes
- [ ] Distribute inheritable attributes to children
- [ ] Handle attribute conflicts (child overrides)
- [ ] Remove empty groups after distribution
- [ ] Add test suite

## Phase 1B: Standalone Plugins (Weeks 4-5)

### 1.5 applyTransforms (1 week)

- [ ] Create plugin file: `svgn/src/plugins/apply_transforms.rs`
- [ ] Parse transform matrices from elements
- [ ] Apply transform matrices to path coordinates
- [ ] Transform basic shape coordinates (rect, circle, ellipse)
- [ ] Handle nested transforms correctly
- [ ] Remove transform attributes after application
- [ ] Support transformPrecision and applyTransformsStroked parameters
- [ ] Add test suite

### 1.6 reusePaths (1 week)

- [ ] Create plugin file: `svgn/src/plugins/reuse_paths.rs`
- [ ] Hash path content for duplicate detection
- [ ] Create `<defs>` and `<use>` elements
- [ ] Replace duplicates with references
- [ ] Calculate size reduction benefits
- [ ] Add test suite

## Phase 2: Completed Tasks ✅

## Phase 3: Infrastructure (Weeks 6-7)

### 3.1 Parser Enhancements (1 week)

- [ ] Fix XML entity expansion (Issue #201)
- [ ] Parse `<!ENTITY>` declarations in DOCTYPE
- [ ] Build entity table during parsing
- [ ] Expand `&entity;` references throughout document
- [ ] Update parser in `svgn/src/parser.rs`
- [ ] Fix selective whitespace preservation (Issue #202)
- [ ] Preserve whitespace in `<text>`, `<tspan>`, `<pre>`, `<script>`, `<style>`
- [ ] Add context-aware whitespace handling
- [ ] Fix enhanced error reporting (Issue #203)
- [ ] Track line/column positions during parsing
- [ ] Provide context snippets in error messages
- [ ] Fix namespace handling consistency (Issue #204)
- [ ] Unify namespace handling in single location
- [ ] Remove redundancy between `namespaces` and `attributes` maps
- [ ] Fix document metadata usage (Issue #205)
- [ ] Ensure `path`, `encoding`, `version` are properly populated
- [ ] Use metadata throughout optimization pipeline

### 3.2 Stringifier Enhancements (0.5 weeks)

- [ ] Fix XML declaration support (Issue #206)
- [ ] Add XML declaration output based on DocumentMetadata
- [ ] Update stringifier in `svgn/src/stringifier.rs`
- [ ] Fix DOCTYPE preservation (Issue #207)
- [ ] Store DOCTYPE declarations during parsing
- [ ] Output DOCTYPE declarations with entities
- [ ] Handle public/system identifiers

### 3.3 Architecture Improvements (1 week)

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

### 3.4 Plugin-Specific Fixes (0.5 weeks)

- [ ] Fix cleanupEnableBackground style handling (Issue #225)
- [ ] Parse enable-background from style attributes
- [ ] Merge logic with attribute handling
- [ ] Fix cleanupIds URL encoding (Issue #227)
- [ ] Match SVGO's encodeURI/decodeURI behavior exactly
- [ ] Fix cleanupIds optimization skip (Issue #228)
- [ ] Skip ID minification for SVGs with only `<defs>`
- [ ] Detect SVGs containing only defs

## Phase 4: Default Preset Alignment (Week 5)

### 4.1 Update Default Configuration

- [ ] Add missing plugins to default preset configuration
- [ ] Add `removeDeprecatedAttrs`
- [ ] Add `mergeStyles`
- [ ] Add `cleanupNumericValues`
- [ ] Add `removeNonInheritableGroupAttrs`
- [ ] Add `cleanupEnableBackground`
- [ ] Add `removeHiddenElems`
- [ ] Add `convertShapeToPath`
- [ ] Add `convertEllipseToCircle`
- [ ] Add `sortDefsChildren`
- [ ] Add the 6 missing plugins when implemented
- [ ] Update plugin registry order to match SVGO preset order
- [ ] Test default preset compatibility

## Phase 5: Quality & Testing (Weeks 6-8)

### 5.1 Code Quality (0.5 weeks)

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

### 5.2 Testing (1 week)

- [ ] Port remaining SVGO test fixtures
- [ ] Port all missing plugin test files from `/ref/svgo/test/plugins/`
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

### 5.3 CLI Completion (0.5 weeks)

- [ ] Add support for .js config files (currently only .json and .toml)
- [ ] Implement base64 encoding for datauri output (currently placeholder)

### 5.4 Build & Distribution (1 week)

- [ ] Complete cross-platform build scripts (Issue #410)
- [ ] Fix macOS universal binary build (Issue #412)
- [ ] Create Linux packaging (.deb, .rpm, .AppImage)
- [ ] Create Windows installer (.msi)
- [ ] Update GitHub Actions workflow
- [ ] Implement version management
- [ ] Git tag-based versioning
- [ ] Automatic version injection at build time
- [ ] Update set-cargo-version.sh script

### 5.5 Documentation Updates (0.5 weeks)

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

## Success Metrics

### Plugin Parity

- [ ] Achieve 54/54 plugins implemented (currently 50/54 - 93% complete)
- [ ] Implement 4 remaining missing plugins (inlineStyles, mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems)

### Test Compatibility

- [ ] Achieve 100% SVGO test suite passing (currently 93.75%)
- [ ] Port all remaining test fixtures
- [ ] Verify identical optimization results

### Performance

- [ ] Maintain 2-3x performance advantage over SVGO
- [ ] Ensure new plugins don't degrade performance
- [ ] Complete benchmark suite

### API Compatibility

- [ ] Maintain 100% CLI drop-in replacement capability
- [ ] Support all SVGO configuration options
- [ ] Match parameter names and behaviors exactly
