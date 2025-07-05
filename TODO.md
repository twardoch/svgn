# SVGN TODO List

## CRITICAL: CI/CD Pipeline Fixes ⚠️ IMMEDIATE PRIORITY (2025-07-05)

### GitHub Actions Failures - Issue #506
The CI/CD pipeline is currently failing and blocks development progress. These must be fixed immediately:

#### Selector Trait API Mismatch (URGENT - Blocking CI)
- [ ] **Fix selector trait in remove_attributes_by_selector.rs:503**
  - Change `impl<'i> selectors::Parser<'i> for DummyParser` 
  - To: `impl<'i> selectors::parser::SelectorParser<'i> for DummyParser`
  - Location: `svgn/src/plugins/remove_attributes_by_selector.rs` line 503
  - Estimated time: 15 minutes

#### GitHub Actions Security Permissions (HIGH PRIORITY)  
- [ ] **Add permissions block to .github/workflows/rust.yml**
  - Add `permissions: contents: read, security-events: write` at top of file
  - Fixes "Resource not accessible by integration" error in security audit
  - Estimated time: 10 minutes

#### Deprecated Actions Update (MEDIUM PRIORITY)
- [ ] **Upgrade actions/upload-artifact from v3 to v4**
  - Update `.github/workflows/release.yml:154`
  - Change `actions/upload-artifact@v3` to `actions/upload-artifact@v4`
  - Estimated time: 10 minutes

#### Verification Tasks
- [ ] **Test CI/CD pipeline after fixes**
  - Commit all changes and push to trigger workflows
  - Verify all GitHub Actions pass successfully
  - Ensure security audit completes without errors
  - Estimated time: 30 minutes

## Immediate Build Fixes ✅ COMPLETED (2025-07-04)

- [x] Fixed unresolved import `crate::test_utils` in remove_useless_stroke_and_fill.rs
- [x] Fixed function call with wrong arguments in convert_path_data.rs test
- [x] Removed unused import `Node` in convert_transform.rs
- [x] Fixed Document Display trait issues in remove_useless_stroke_and_fill.rs tests
- [x] Suppressed dead code warning for `transform_precision` field

## Phase 0 – Clippy Compliance Sprint ✅ MAJOR PROGRESS (2025-07-04)

### Core Library Fixes Completed ✅
- [x] Fixed redundant closures in optimizer.rs and inline_styles files
- [x] Replaced `map_or` chains with `is_none_or`/`is_some_and` (multiple files)
- [x] Fixed manual strip usage in inline_styles.rs (2 fixes)
- [x] Removed needless borrows and op_ref issues (multiple files) 
- [x] Fixed len_zero issues (`len() >= 1` → `!is_empty()`)
- [x] Added missing `optimize_default` export to lib.rs
- [x] Fixed CLI argument conflicts needless borrows

### Remaining Test/Bench Fixes (Non-blocking)
- [ ] C01 Remove illegal `#[cfg(disabled)]` attribute (remove_useless_stroke_and_fill.rs:319)
- [ ] C02 Collapse nested `if` in remove_useless_stroke_and_fill.rs:115-118  
- [ ] C04 Fix `only_used_in_recursion` warnings or allow (remove_useless_stroke_and_fill.rs:68,240)
- [ ] C05 Prefix unused param `_plugin_info` (convert_path_data.rs:37)
- [ ] C06 Replace `while let Some(ch)` loop with `for` (convert_path_data.rs:208)
- [ ] C08 Refactor `optimize_path_data` signature to fewer args (<8) (convert_path_data.rs:357)
- [ ] C09 Replace `.get(0)` with `.first()` in convert_transform.rs:149-209  
- [ ] C10 Use `*=` instead of manual `=` `*` (convert_transform.rs:260)
- [ ] C11 Replace `len() >= 1` in convert_transform.rs:341-346
- [ ] Additional test-specific clippy fixes identified
- [ ] Ensure `cargo clippy -- -D warnings` runs clean

### Status
- ✅ **Core development unblocked** - 347 tests passing, library functionality working
- ⚠️ **CI still requires full clippy compliance** but development can proceed


## Phase 1A: Critical Default Preset Plugins (Weeks 1-3)

### 1.1 inlineStyles ✅ MVP COMPLETED (2025-07-05)

#### Phase 1A.1: Foundation Setup ✅ COMPLETED
- [x] Verify lightningcss dependency (already configured in workspace)
- [x] Create plugin file: `svgn/src/plugins/inline_styles.rs` 
- [x] Set up basic plugin structure with SVGO parameter parsing
- [x] Define `InlineStylesParams` struct with 4 SVGO parameters:
  - [x] `onlyMatchedOnce: bool` (default: true)
  - [x] `removeMatchedSelectors: bool` (default: true) 
  - [x] `useMqs: bool` (default: true)
  - [x] `usePseudos: bool` (default: true)

#### Phase 1A.2: CSS Processing Engine ✅ COMPLETED
- [x] Implement CSS parsing using lightningcss StyleSheet
- [x] Create CSS rule extraction from `<style>` elements
- [x] Add CSS rule validation and error handling
- [x] CSS property to SVG attribute conversion implemented

#### Phase 1A.3: SVG DOM Integration ✅ COMPLETED
- [x] Implement custom Element trait for selectors crate (inline_styles_selector.rs)
- [x] Complete basic selector matching logic for class selectors (`.st0 {}`)
- [x] Apply matched CSS rules as inline SVG attributes 
- [x] Add support for ID selectors (`#foo`) and element selectors (`rect`)
- [x] Verified with real SVG testing

#### Phase 1A.4: CSS-to-SVG Conversion ✅ COMPLETED
- [x] Create CSS property to SVG attribute mapping (inline_styles_converter.rs)
- [x] Build attribute value conversion (colors, units, etc.)
- [x] Implement specificity-based rule application
- [x] Add conflict resolution for existing attributes

#### Phase 1A.5: Basic Testing and Integration ✅ COMPLETED
- [x] Add basic functionality tests for plugin interface
- [x] Implement functional testing with real SVG files
- [x] Register plugin in mod.rs and default preset configuration
- [x] Enable plugin in config.rs alongside convertTransform

#### Plugin Status: **FUNCTIONAL** 
- ✅ Successfully processes real SVG files
- ✅ Class, ID, and element selectors working
- ✅ Color conversion (named → RGB) functional
- ✅ Integrated into build system and enabled by default
- ✅ No compilation errors or test failures

#### Future Enhancements (Optional)
- [ ] Media query processing (for `useMqs` parameter)
- [ ] Pseudo-class filtering (for `usePseudos` parameter)
- [ ] Advanced selector removal optimization
- [ ] Performance optimization for complex CSS

#### ⚠️ Fallback Strategy (If inlineStyles complexity exceeds estimates)
- [ ] **Incremental MVP:** Implement basic CSS rule inlining without full specificity
- [ ] **Fallback Approach:** Use regex-based CSS parsing for complex selectors
- [ ] **Milestone Gates:** Define 80% functionality checkpoint before full implementation
- [ ] **Alternative Timeline:** Extend to 2.5 weeks if full CSS specification support needed

### 1.2 mergePaths (1 week)

#### Phase 1A.2: Implementation Steps
- [ ] Create plugin file: `svgn/src/plugins/merge_paths.rs`
- [ ] Implement path grouping by style attribute fingerprinting
- [ ] Build DOM adjacency detection for mergeable path elements
- [ ] Create path data concatenation with proper moveTo insertion
- [ ] Add SVGO parameter support: `force`, `floatPrecision`, `noSpaceAfterFlags`
- [ ] Implement size optimization analysis (merge vs. separate paths)
- [ ] Handle edge cases: transforms, markers, animations
- [ ] Add comprehensive test suite with SVGO compatibility validation

### 1.3 moveElemsAttrsToGroup (0.5 weeks)

#### Phase 1A.3: Implementation Steps
- [ ] Create plugin file: `svgn/src/plugins/move_elems_attrs_to_group.rs`
- [ ] Implement SVG presentation attribute inheritance analysis
- [ ] Build sibling element grouping detection algorithm
- [ ] Create size optimization calculator (group overhead vs. savings)
- [ ] Implement `<g>` wrapper creation with attribute consolidation
- [ ] Add proper handling of transforms, styles, and presentation attributes
- [ ] Create test suite with inheritance rule validation

### 1.4 moveGroupAttrsToElems (0.5 weeks)

#### Phase 1A.4: Implementation Steps  
- [ ] Create plugin file: `svgn/src/plugins/move_group_attrs_to_elems.rs`
- [ ] Implement group analysis for attribute-only containers
- [ ] Build attribute distribution logic with conflict resolution
- [ ] Add size benefit analysis for group removal optimization
- [ ] Handle edge cases: nested groups, mixed content, transforms
- [ ] Implement empty group cleanup after attribute distribution
- [ ] Create comprehensive test suite with SVGO compatibility checks

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

## Success Metrics & Definition of Done

### **Plugin Parity (Primary Goal)**
- [ ] Achieve 54/54 plugins implemented (currently 51/54 - 94.4% complete)
- [x] Fix 1 disabled plugin (removeAttributesBySelector) ✅ COMPLETED
- [x] Implement convertTransform plugin ✅ COMPLETED (2025-07-04)
- [x] Implement inlineStyles plugin MVP ✅ COMPLETED (2025-07-05)
- [ ] Implement 3 remaining missing plugins:
  - [ ] **mergePaths** - Path concatenation with style matching (NEXT PRIORITY)
  - [ ] **moveElemsAttrsToGroup** - Attribute inheritance optimization
  - [ ] **moveGroupAttrsToElems** - Reverse attribute distribution

### **Quality Gates**
- [ ] **100% SVGO Output Compatibility:** Bit-for-bit identical output for test cases
- [ ] **Performance Benchmark:** Maintain 2-3x speed advantage over SVGO
- [ ] **Test Coverage:** 367+ tests passing with new plugin integration
- [ ] **CLI Compatibility:** All SVGO parameters and options supported

### **Acceptance Criteria**
- [ ] **Default Preset Complete:** All 35 SVGO default preset plugins implemented
- [ ] **Parameter Compatibility:** All plugin parameters match SVGO specifications
- [ ] **Edge Case Handling:** Complex CSS, nested transforms, and mixed content scenarios
- [ ] **Documentation Complete:** Plugin documentation and usage examples

### **Release Readiness**
- [ ] **Code Quality:** All clippy warnings resolved, comprehensive error handling
- [ ] **Integration Testing:** Multi-plugin interaction validation
- [ ] **Community Validation:** Beta testing feedback incorporated
- [ ] **Version Management:** Git tags and release automation configured
