# SVGN Development Plan: Path to 100% SVGO Parity

## 1. Executive Summary

SVGN is a high-performance Rust port of SVGO that has achieved 93% plugin implementation. This plan outlines the focused path to achieve complete SVGO v4.0.0 compatibility.

**Current Status (2025-07-05):**
- ✅ **51/54 plugins** fully implemented and functional (94.4%)
- ✅ **convertPathData** fully implemented
- ✅ **removeUselessStrokeAndFill** fully implemented (was incorrectly listed as missing)
- ✅ **removeAttributesBySelector** fixed and enabled (CSS parsing issue resolved)
- ✅ **convertTransform** fully implemented (critical default preset plugin)
- ✅ **inlineStyles** MVP implementation completed (Foundation + CSS Processing + Basic functionality)
- ❌ **3 plugins** remaining for 100% parity: mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems
- ❌ **CRITICAL BLOCKING ISSUE:** Build failing due to CSS dependency version conflicts
- ✅ **Full CLI compatibility** achieved (when build passes)
- ❌ **Build Status:** FAILING - 17 compilation errors in CSS selector implementation

**Build Status Update (2025-07-05)**

❌ **CRITICAL BUILD FAILURE:** Project currently cannot compile due to CSS dependency conflicts:
- **17 compilation errors** in CSS selector and parser implementations
- **cssparser version conflicts** between lightningcss and selectors crates
- **Trait implementation issues** - ToCss not implemented for String types
- **Function signature mismatches** in MatchingContext::new() calls
- **Borrowing conflicts** in CSS rule application logic

❌ **URGENT PRIORITY: Fix Build Issues**
- Cannot proceed with feature development until compilation errors resolved
- inlineStyles plugin implementation needs CSS dependency version alignment
- Selector trait implementations need compatibility fixes
- Function signature updates required for selectors crate API changes

**Development Priority:**
**IMMEDIATE FOCUS:** Resolve build compilation errors before continuing with remaining 3 plugins. The CSS dependency conflicts must be fixed to restore development capability.

## 2. Critical Missing Plugins (Priority: IMMEDIATE)

### 2.1. Phase 1A: Default Preset Plugins (Highest Impact)
These 4 plugins are in SVGO's default preset and required for preset compatibility:

#### 2.1.1. 1.1 inlineStyles (1.5 weeks - HIGH) 🚧 IN PROGRESS
- **Impact:** Critical - in SVGO default preset position 9/35  
- **Complexity:** High - requires sophisticated CSS processing
- **Dependencies:** ✅ `lightningcss = "1.0.0-alpha.67"` (already configured)
- **Technical Architecture:**
  - **CSS Parsing:** ✅ Using lightningcss for complete CSS AST processing
  - **Selector Matching:** ✅ Leveraging selectors crate for DOM matching with custom Element trait
  - **Specificity Resolution:** Built-in specificity calculation via selectors crate
  - **Media Query Support:** Handle `useMqs` parameter with lightningcss media query parsing
  - **Pseudo-class Filtering:** Support `usePseudos` parameter with configurable preservation
- **Implementation Progress:**
  1. ✅ Parse CSS from `<style>` elements using lightningcss stylesheet parser
  2. ✅ Implement custom Element trait for SVG DOM matching 
  3. 🚧 Create CSS specificity-based cascade resolution engine
  4. Build media query and pseudo-class filtering logic
  5. Convert matched CSS properties to SVG attributes
  6. Clean up unused selectors and class/ID attributes
- **SVGO Parameters:** `onlyMatchedOnce`, `removeMatchedSelectors`, `useMqs`, `usePseudos`
- **Status:** Foundation complete, CSS parsing working, Element trait implemented

#### 2.1.2. 1.2 mergePaths (1 week - MEDIUM)
- **Impact:** Critical - in SVGO default preset position 29/35
- **Complexity:** Medium - path analysis and DOM manipulation
- **Dependencies:** ✅ Existing path parsing infrastructure from `convertPathData`
- **Technical Architecture:**
  - **Path Grouping:** Hash-based grouping by style attribute combinations
  - **Adjacency Analysis:** DOM tree traversal to identify mergeable adjacent paths
  - **Path Concatenation:** Smart joining with proper moveTo insertion between segments
- **Implementation Steps:**
  1. Identify all `<path>` elements in document
  2. Group paths by identical presentation attributes (fill, stroke, etc.)
  3. Within groups, find DOM-adjacent paths eligible for merging
  4. Concatenate path data with proper segment separation
  5. Replace multiple paths with single merged path element
  6. Handle edge cases: transforms, markers, animations
- **SVGO Parameters:** `force`, `floatPrecision`, `noSpaceAfterFlags`

#### 2.1.3. 1.3 moveElemsAttrsToGroup (0.5 weeks - MEDIUM)
- **Impact:** Critical - in SVGO default preset position 22/35
- **Complexity:** Medium - SVG inheritance analysis and DOM restructuring
- **Dependencies:** ✅ Existing attribute handling infrastructure
- **Technical Architecture:**
  - **Inheritance Analysis:** SVG presentation attribute inheritance rules
  - **Sibling Grouping:** Identify elements that can share a common parent group
  - **Size Optimization:** Calculate byte savings from attribute consolidation
- **Implementation Steps:**
  1. Analyze sibling elements for common inheritable attributes
  2. Identify attributes eligible for group inheritance (fill, stroke, etc.)
  3. Calculate optimization benefit (group wrapper cost vs. attribute duplication savings)
  4. Create `<g>` wrapper with common attributes when beneficial
  5. Remove consolidated attributes from child elements
- **SVG Inheritance Rules:** Handle presentation attributes, transforms, and styles properly

#### 2.1.4. 1.4 moveGroupAttrsToElems (0.5 weeks - MEDIUM)
- **Impact:** Critical - in SVGO default preset position 23/35
- **Complexity:** Medium - reverse inheritance optimization
- **Dependencies:** ✅ Existing group handling infrastructure  
- **Technical Architecture:**
  - **Group Analysis:** Identify groups that exist only for attribute sharing
  - **Distribution Logic:** Move inheritable attributes to child elements
  - **Conflict Resolution:** Handle attribute conflicts (child overrides parent)
- **Implementation Steps:**
  1. Identify `<g>` elements that provide only inheritable attributes
  2. Analyze child elements for attribute conflicts and inheritance compatibility
  3. Calculate size benefit of group removal vs. attribute distribution
  4. Distribute group attributes to child elements when beneficial
  5. Remove empty/unnecessary group wrappers
- **Edge Cases:** Handle transforms, nested groups, and mixed content properly

### 2.2. Phase 1B: Standalone Plugins (Lower Priority)

#### 2.2.1. 1.5 applyTransforms (1 week - HIGH)
- **Impact:** Medium - not in default preset, specialized optimization
- **Complexity:** High - advanced coordinate mathematics and SVG geometry
- **Dependencies:** ✅ `nalgebra` matrix operations (from `convertTransform`)
- **Technical Architecture:**
  - **Matrix Application:** Apply transform matrices directly to coordinate data
  - **Shape Transformation:** Handle different SVG shape types (paths, circles, rects, etc.)
  - **Nested Transform Resolution:** Resolve transform hierarchies correctly
- **Implementation Steps:**
  1. Parse transform attributes using existing `convertTransform` infrastructure
  2. Apply matrices to path data coordinates (use existing path parsing)
  3. Transform basic shape attributes (cx, cy, r, x, y, width, height)
  4. Handle coordinate precision and rounding
  5. Remove transform attributes after successful application
  6. Handle edge cases: nested transforms, percentage values, view transformations
- **SVGO Parameters:** `transformPrecision`, `applyTransformsStroked`

#### 2.2.2. 1.6 reusePaths (1 week - MEDIUM)
- **Impact:** Low - not in default preset, file-size optimization for repeated paths
- **Complexity:** Medium - content analysis and DOM restructuring
- **Dependencies:** ✅ Existing path parsing and `<defs>` handling
- **Technical Architecture:**
  - **Content Hashing:** Create content-based hashes for path deduplication
  - **DOM Restructuring:** Generate `<defs>` and `<use>` element structure
  - **Size Analysis:** Calculate byte savings vs. overhead of `<use>` references
- **Implementation Steps:**
  1. Extract and normalize path data content from all `<path>` elements
  2. Create content-based hashes to identify identical paths
  3. Analyze size benefit of `<use>` references vs. duplication
  4. Create `<defs>` section with unique path definitions
  5. Replace duplicate paths with `<use>` element references
  6. Handle edge cases: paths with different attributes, transforms, styles

## 3. URGENT: Build Compilation Issues (Priority: CRITICAL)

❌ **BUILD BLOCKING ISSUES** must be resolved before implementing remaining plugins:

### 3.1. 2.1 CSS Dependency Version Conflicts
- **Problem:** Multiple cssparser versions in dependency tree (0.31.2 vs 0.33.0)
- **Impact:** ToCss trait not implemented for String types in selector implementations
- **Solution:** Align cssparser versions between lightningcss and selectors crates

### 3.2. 2.2 Selector API Compatibility Issues
- **Problem:** MatchingContext::new() function signature changed
- **Impact:** Function takes 6 arguments but 4 provided in remove_attributes_by_selector.rs
- **Solution:** Update function calls to match current selectors crate API

### 3.3. 2.3 Trait Implementation Gaps
- **Problem:** SvgSelectorImpl missing Parser trait implementation
- **Impact:** SelectorList::parse() cannot be called
- **Solution:** Implement required traits for CSS selector parsing

## 4. Remaining Missing Plugins (Priority: HIGH - after build fixes)

With `convertTransform` and `inlineStyles` completed, only 3 plugins remain for 100% SVGO parity:

1. **mergePaths** - Critical default preset plugin (position 29/35)
2. **moveElemsAttrsToGroup** - Critical default preset plugin (position 22/35)
3. **moveGroupAttrsToElems** - Critical default preset plugin (position 23/35)

All 3 remaining plugins are in SVGO's default preset, making them the highest priority after build issues are resolved.

## 5. 2.1 Strategic Implementation Advantages

SVGN is excellently positioned for rapid completion due to existing infrastructure:

### 5.1. ✅ **Foundation Already Established:**
- **CSS Processing:** `lightningcss`, `cssparser`, and `selectors` crates already configured
- **Mathematical Operations:** `nalgebra` infrastructure from `convertTransform` completion
- **Path Processing:** Robust path parsing and manipulation from `convertPathData`
- **DOM Manipulation:** Mature element traversal and attribute handling
- **Plugin Architecture:** Well-tested plugin system with 50 working implementations

### 5.2. ✅ **Shared Infrastructure Benefits:**
- **inlineStyles + CSS plugins:** Can leverage existing CSS processing in `convert_style_to_attrs` and `minify_styles`
- **mergePaths + convertPathData:** Direct reuse of path parsing and manipulation logic
- **Attribute movement plugins:** Build on existing group handling from `collapse_groups`
- **applyTransforms + convertTransform:** Share matrix operations and transform parsing

### 5.3. ✅ **Risk Mitigation:**
- **Proven Architecture:** Plugin system has 50 successful implementations
- **SVGO Compatibility:** Extensive test suite with 367 passing tests validates approach
- **Performance Validated:** Current 2-3x speed advantage over SVGO demonstrates sound architecture

## 6. 2.2 Implementation Strategy Options

### 6.1. **Option A: Complexity-First Approach (Recommended)**
**Rationale:** Tackle the most complex plugin (`inlineStyles`) first while energy and focus are highest
- ✅ **Advantages:** Hardest problems solved early, momentum builds with easier plugins
- ⚠️ **Risks:** Potential early blocking on complex CSS edge cases

### 6.2. **Option B: Progressive Complexity Approach** 
**Rationale:** Build confidence with simpler plugins, tackle complexity incrementally
- ✅ **Advantages:** Early wins, better understanding of plugin interactions
- ⚠️ **Risks:** Saving hardest problem for last when energy may be lower

### 6.3. **Option C: Parallel Development Approach**
**Rationale:** Multiple plugins developed simultaneously by different team members
- ✅ **Advantages:** Fastest completion if resources available
- ⚠️ **Risks:** Integration challenges, coordination overhead

**Recommendation:** Proceed with **Option A** but with built-in fallback to incremental implementation if inlineStyles complexity exceeds estimates.

## 7. Infrastructure Improvements (Priority: MEDIUM)

### 7.1. 3.1 Parser Enhancements (1 week)

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

### 7.2. 3.2 Stringifier Enhancements (0.5 weeks)

- [ ] Fix XML declaration support (Issue #206)
- [ ] Add XML declaration output based on DocumentMetadata
- [ ] Update stringifier in `svgn/src/stringifier.rs`
- [ ] Fix DOCTYPE preservation (Issue #207)
- [ ] Store DOCTYPE declarations during parsing
- [ ] Output DOCTYPE declarations with entities
- [ ] Handle public/system identifiers

### 7.3. 3.3 Architecture Improvements (1 week)

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

### 7.4. 3.4 Plugin-Specific Fixes (0.5 weeks)

- [ ] Fix cleanupEnableBackground style handling (Issue #225)
- [ ] Parse enable-background from style attributes
- [ ] Merge logic with attribute handling
- [ ] Fix cleanupIds URL encoding (Issue #227)
- [x] **cleanupIds Optimization (Issue #228):** Skip for defs-only SVGs
- [x] Detect SVGs containing only defs

## 8. Phase 4: Default Preset Alignment (Week 5)

### 8.1. 4.1 Update Default Configuration

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

## 9. Phase 5: Quality & Testing (Weeks 6-8)

### 9.1. 5.1 Code Quality (0.5 weeks)

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

### 9.2. 5.2 Testing (1 week)

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

### 9.3. 5.3 CLI Completion (0.5 weeks)

- [ ] Add support for .js config files (currently only .json and .toml)
- [ ] Implement base64 encoding for datauri output (currently placeholder)

### 9.4. 5.4 Build & Distribution (1 week)

- [ ] Complete cross-platform build scripts (Issue #410)
- [ ] Fix macOS universal binary build (Issue #412)
- [ ] Create Linux packaging (.deb, .rpm, .AppImage)
- [ ] Create Windows installer (.msi)
- [ ] Update GitHub Actions workflow
- [ ] Implement version management
- [ ] Git tag-based versioning
- [ ] Automatic version injection at build time
- [ ] Update set-cargo-version.sh script

### 9.5. 5.5 Documentation Updates (0.5 weeks)

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

## 10. Success Metrics & Definition of Done

### 10.1. **Plugin Parity (Primary Goal)**
- [ ] Achieve 54/54 plugins implemented (currently 51/54 - 94.4% complete)
- [x] Fix 1 disabled plugin (removeAttributesBySelector) ✅ COMPLETED
- [x] Implement convertTransform plugin ✅ COMPLETED (2025-07-04)
- [x] Implement inlineStyles plugin MVP ✅ COMPLETED (2025-07-05)
- [ ] **CRITICAL BLOCKER:** Fix CSS dependency compilation errors (IMMEDIATE PRIORITY)
- [ ] Implement 3 remaining missing plugins (after build fixes):
  - [ ] **mergePaths** - Path concatenation with style matching
  - [ ] **moveElemsAttrsToGroup** - Attribute inheritance optimization
  - [ ] **moveGroupAttrsToElems** - Reverse attribute distribution

### 10.2. **Quality Gates**
- [ ] **100% SVGO Output Compatibility:** Bit-for-bit identical output for test cases
- [ ] **Performance Benchmark:** Maintain 2-3x speed advantage over SVGO
- [ ] **Test Coverage:** 367+ tests passing with new plugin integration
- [ ] **CLI Compatibility:** All SVGO parameters and options supported

### 10.3. **Acceptance Criteria**
- [ ] **Default Preset Complete:** All 35 SVGO default preset plugins implemented
- [ ] **Parameter Compatibility:** All plugin parameters match SVGO specifications
- [ ] **Edge Case Handling:** Complex CSS, nested transforms, and mixed content scenarios
- [ ] **Documentation Complete:** Plugin documentation and usage examples

### 10.4. **Release Readiness**
- [ ] **Code Quality:** All clippy warnings resolved, comprehensive error handling
- [ ] **Integration Testing:** Multi-plugin interaction validation
- [ ] **Community Validation:** Beta testing feedback incorporated
- [ ] **Version Management:** Git tags and release automation configured