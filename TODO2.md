# SVGN TODO List 2.0

## Phase 1: Critical Missing Plugins (4-6 weeks)

### Priority 1A: Default Preset Plugins (4 weeks)
- [ ] Implement `removeUselessStrokeAndFill` plugin (2 weeks - MEDIUM complexity)
  - [ ] Create plugin file: `svgn/src/plugins/remove_useless_stroke_and_fill.rs`
  - [ ] Implement stroke inheritance analysis logic
  - [ ] Implement fill inheritance analysis logic  
  - [ ] Handle `currentColor` correctly
  - [ ] Remove redundant stroke="none" and fill="none" attributes
  - [ ] Add comprehensive test suite
- [ ] Implement `convertTransform` plugin (2 weeks - HIGH complexity)
  - [ ] Add nalgebra dependency for matrix operations
  - [ ] Create plugin file: `svgn/src/plugins/convert_transform.rs`
  - [ ] Implement transform string parser
  - [ ] Implement matrix multiplication and optimization
  - [ ] Implement matrix decomposition (matrix to translate/scale/rotate)
  - [ ] Support all 12 SVGO parameters
  - [ ] Add comprehensive test suite

### Priority 1B: Transform and Path Plugins (2 weeks)
- [ ] Implement `mergePaths` plugin (1 week - MEDIUM complexity)
  - [ ] Create plugin file: `svgn/src/plugins/merge_paths.rs`
  - [ ] Group paths by identical style attributes
  - [ ] Check DOM adjacency for mergeable paths
  - [ ] Concatenate path data strings correctly
  - [ ] Support force, floatPrecision, noSpaceAfterFlags parameters
  - [ ] Add test suite
- [ ] Implement `applyTransforms` plugin (1 week - HIGH complexity)
  - [ ] Create plugin file: `svgn/src/plugins/apply_transforms.rs`
  - [ ] Parse transform matrices from elements
  - [ ] Apply transforms to path coordinates
  - [ ] Transform basic shape coordinates (rect, circle, ellipse)
  - [ ] Handle nested transforms correctly
  - [ ] Remove transform attribute after application
  - [ ] Support transformPrecision and applyTransformsStroked parameters

### Priority 1C: Style and Structure Plugins (2 weeks)
- [ ] Implement `inlineStyles` plugin (1 week - HIGH complexity)
  - [ ] Add lightningcss or css crate dependency
  - [ ] Create plugin file: `svgn/src/plugins/inline_styles.rs`
  - [ ] Parse CSS from `<style>` elements
  - [ ] Implement CSS specificity calculation
  - [ ] Match CSS selectors to SVG elements
  - [ ] Apply cascade rules and inheritance
  - [ ] Convert CSS properties to SVG attributes
  - [ ] Handle media queries and pseudo-classes
  - [ ] Support all 4 SVGO parameters
- [ ] Implement `moveElemsAttrsToGroup` plugin (0.5 weeks - MEDIUM complexity)
  - [ ] Create plugin file: `svgn/src/plugins/move_elems_attrs_to_group.rs`
  - [ ] Analyze attributes across sibling elements
  - [ ] Identify inheritable SVG attributes
  - [ ] Calculate size reduction benefit
  - [ ] Create `<g>` wrapper when beneficial
  - [ ] Move attributes and remove from children
- [ ] Implement `moveGroupAttrsToElems` plugin (0.5 weeks - MEDIUM complexity)
  - [ ] Create plugin file: `svgn/src/plugins/move_group_attrs_to_elems.rs`
  - [ ] Identify groups that only provide attributes
  - [ ] Distribute inheritable attributes to children
  - [ ] Handle attribute conflicts (child overrides)
  - [ ] Remove empty groups after distribution

## Phase 2: Fix Issues and Infrastructure (2-3 weeks)

### Priority 2A: Fix Disabled Plugin (0.5 weeks)
- [ ] Fix `removeAttributesBySelector` CSS parsing issue
  - [ ] Add selectors crate dependency
  - [ ] Fix CSS selector parsing in `svgn/src/plugins/remove_attributes_by_selector.rs`
  - [ ] Re-enable plugin in registry (`svgn/src/plugin.rs`)
  - [ ] Add test cases
  - [ ] Uncomment in mod.rs

### Priority 2B: Parser Enhancements (1 week)
- [ ] Fix XML entity expansion (Issue #201)
  - [ ] Parse `<!ENTITY>` declarations in DOCTYPE
  - [ ] Build entity table during parsing
  - [ ] Expand `&entity;` references throughout document
  - [ ] Update parser in `svgn/src/parser.rs`
- [ ] Fix selective whitespace preservation (Issue #202)
  - [ ] Preserve whitespace in `<text>`, `<tspan>`, `<pre>`, `<script>`, `<style>`
  - [ ] Add context-aware whitespace handling
  - [ ] Update parser whitespace logic
- [ ] Fix enhanced error reporting (Issue #203)
  - [ ] Track line/column positions during parsing
  - [ ] Provide context snippets in error messages
  - [ ] Enhance error types and messages
- [ ] Fix namespace handling consistency (Issue #204)
  - [ ] Unify namespace handling in single location
  - [ ] Remove redundancy between `namespaces` and `attributes` maps
  - [ ] Update AST structure in `svgn/src/ast.rs`
- [ ] Fix document metadata usage (Issue #205)
  - [ ] Ensure `path`, `encoding`, `version` are properly populated
  - [ ] Use metadata throughout optimization pipeline
  - [ ] Update DocumentMetadata usage

### Priority 2C: Stringifier Enhancements (0.5 weeks)
- [ ] Fix XML declaration support (Issue #206)
  - [ ] Add XML declaration output based on DocumentMetadata
  - [ ] Update stringifier in `svgn/src/stringifier.rs`
- [ ] Fix DOCTYPE preservation (Issue #207)
  - [ ] Store DOCTYPE declarations during parsing
  - [ ] Output DOCTYPE declarations with entities
  - [ ] Handle public/system identifiers

### Priority 2D: Architecture Improvements (1 week)
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
  - [ ] Update config system
- [ ] Add dynamic plugin loading support (Issue #216)
  - [ ] Plugin discovery mechanism
  - [ ] Runtime loading API
  - [ ] External plugin interface

### Priority 2E: Plugin-Specific Issues (0.5 weeks)
- [ ] Fix cleanupEnableBackground style handling (Issue #225)
  - [ ] Parse enable-background from style attributes
  - [ ] Merge logic with attribute handling
  - [ ] Update `svgn/src/plugins/cleanup_enable_background.rs`
- [ ] Fix cleanupIds URL encoding (Issue #227)
  - [ ] Match SVGO's encodeURI/decodeURI behavior exactly
  - [ ] Update `svgn/src/plugins/cleanup_ids.rs`
- [ ] Fix cleanupIds optimization skip (Issue #228)
  - [ ] Skip ID minification for SVGs with only `<defs>`
  - [ ] Detect SVGs containing only defs
  - [ ] Update optimization logic

## Phase 3: Polish and Testing (1-2 weeks)

### Priority 3A: Code Quality (0.5 weeks)
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

### Priority 3B: Testing (1 week)
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

### Priority 3C: Documentation Updates (0.5 weeks)
- [ ] Update docs/plugins.md
  - [ ] Add new plugin documentation
  - [ ] Update implementation status
  - [ ] Add parameter documentation
- [ ] Update docs/comparison.md
  - [ ] Update plugin count (53/53)
  - [ ] Update compatibility metrics
  - [ ] Document performance characteristics
- [ ] Update README.md
  - [ ] Update implementation status
  - [ ] Update feature list
  - [ ] Add migration guide links

## Phase 4: Default Preset Alignment

### Update Default Preset to Match SVGO
- [ ] Add missing plugins to default preset configuration
  - [ ] Add `removeDeprecatedAttrs`
  - [ ] Add `mergeStyles` 
  - [ ] Add `inlineStyles` (when implemented)
  - [ ] Add `cleanupNumericValues`
  - [ ] Add `removeNonInheritableGroupAttrs`
  - [ ] Add `removeUselessStrokeAndFill` (when implemented)
  - [ ] Add `cleanupEnableBackground`
  - [ ] Add `removeHiddenElems`
  - [ ] Add `convertShapeToPath`
  - [ ] Add `convertEllipseToCircle`
  - [ ] Add `moveElemsAttrsToGroup` (when implemented)
  - [ ] Add `moveGroupAttrsToElems` (when implemented)
  - [ ] Add `convertTransform` (when implemented)
  - [ ] Add `mergePaths` (when implemented)
  - [ ] Add `sortDefsChildren`
- [ ] Update plugin registry order to match SVGO preset order
- [ ] Test default preset compatibility

## Clean-up Tasks

### Remove Obsolete Issues
- [ ] Remove or update obsolete issue files in issues/ directory
  - [ ] Remove issue 403.txt (just build output)
  - [ ] Update issue 404.txt (convertPathData is now implemented)
  - [ ] Review and consolidate remaining issues

### Build and Distribution
- [ ] Complete cross-platform build scripts (Issue #410)
  - [ ] Fix macOS universal binary build (Issue #412)
  - [ ] Create Linux packaging (.deb, .rpm, .AppImage)
  - [ ] Create Windows installer (.msi)
  - [ ] Update GitHub Actions workflow
- [ ] Implement version management
  - [ ] Git tag-based versioning
  - [ ] Automatic version injection at build time
  - [ ] Update set-cargo-version.sh script

## Success Metrics

### Plugin Parity
- [ ] Achieve 53/53 plugins implemented (currently 46/53)
- [ ] Fix 1 disabled plugin (removeAttributesBySelector)
- [ ] Implement 7 missing plugins

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