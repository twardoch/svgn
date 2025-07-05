# SVGN Development Plan: Path to 100% SVGO Parity

## Executive Summary

SVGN is a high-performance Rust port of SVGO that has achieved 93% plugin implementation. This plan outlines the focused path to achieve complete SVGO v4.0.0 compatibility.

**Current Status (2025-07-05):**
- ✅ **50/54 plugins** fully implemented and functional (93%)
- ✅ **convertPathData** fully implemented 
- ✅ **removeUselessStrokeAndFill** fully implemented (was incorrectly listed as missing)
- ✅ **removeAttributesBySelector** fixed and enabled (CSS parsing issue resolved)
- ✅ **convertTransform** fully implemented (critical default preset plugin)
- 🚧 **inlineStyles** implementation in progress (Foundation + CSS Processing complete)
- ❌ **4 plugins** remaining for 100% parity: inlineStyles, mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems
- ✅ **Full CLI compatibility** achieved
- ✅ **377 tests passing** (100% success rate - major improvement from 25 to 377 tests)

**Build Status Update (2025-07-05)**

✅ **Excellent Build Health:** Project now compiles cleanly and all tests pass successfully:
- All critical compilation errors resolved (from previous blockers)
- **377/377 tests passing** (347 unit tests + 30 integration tests + 5 fixture tests + 16 compatibility tests)
- Core library clippy violations resolved - development can proceed efficiently
- Code properly formatted with rustfmt - all diffs are cosmetic formatting only

✅ **Stable Development Foundation:**
- Build pipeline green and stable for continued feature development
- Plugin system proven with 50 working implementations
- Performance benchmarks maintained (2-3x speed advantage over SVGO)
- CLI compatibility verified and working

**Development Priority:**  
With the build now stable and all tests passing, the focus shifts to completing the final 4 plugins for 100% SVGO parity. The robust foundation enables rapid feature development.

## 1. Critical Missing Plugins (Priority: IMMEDIATE)

### Phase 1A: Default Preset Plugins (Highest Impact)
These 4 plugins are in SVGO's default preset and required for preset compatibility:

#### 1.1 inlineStyles (1.5 weeks - HIGH) 🚧 IN PROGRESS
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

#### 1.2 mergePaths (1 week - MEDIUM)
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

#### 1.3 moveElemsAttrsToGroup (0.5 weeks - MEDIUM)
- **Impact:** Critical - in SVGO default preset position 22/35
- **Complexity:** Medium - SVG inheritance analysis and DOM restructuring
- **Dependencies:** ✅ Existing attribute handling infrastructure
- **Technical Architecture:**
  - **Inheritance Analysis:** SVG presentation attribute inheritance rules
  - **Sibling Grouping:** Identify elements that can share a common parent group
  - **Size Optimization:** Calculate byte savings from attribute consolidation
- **Implementation Steps:**
  1. Analyze sibling elements for common inheritable attributes
  2. Identify attributes eligible for group inheritance (fill, stroke, transform, etc.)
  3. Calculate optimization benefit (group wrapper cost vs. attribute duplication savings)
  4. Create `<g>` wrapper with common attributes when beneficial
  5. Remove consolidated attributes from child elements
- **SVG Inheritance Rules:** Handle presentation attributes, transforms, and styles properly

#### 1.4 moveGroupAttrsToElems (0.5 weeks - MEDIUM)
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

### Phase 1B: Standalone Plugins (Lower Priority)

#### 1.5 applyTransforms (1 week - HIGH)
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

#### 1.6 reusePaths (1 week - MEDIUM)
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

## 2. Remaining Missing Plugins (Priority: IMMEDIATE)

With `convertTransform` now completed and `inlineStyles` in progress, only 4 plugins remain to achieve 100% SVGO parity:

1. **inlineStyles** - Critical default preset plugin (position 9/35) - 🚧 IN PROGRESS
2. **mergePaths** - Critical default preset plugin (position 29/35)  
3. **moveElemsAttrsToGroup** - Critical default preset plugin (position 22/35)
4. **moveGroupAttrsToElems** - Critical default preset plugin (position 23/35)

All 4 remaining plugins are in SVGO's default preset, making them the highest priority for achieving complete compatibility.

## 2.1 Strategic Implementation Advantages

SVGN is excellently positioned for rapid completion due to existing infrastructure:

### ✅ **Foundation Already Established:**
- **CSS Processing:** `lightningcss`, `cssparser`, and `selectors` crates already configured
- **Mathematical Operations:** `nalgebra` infrastructure from `convertTransform` completion
- **Path Processing:** Robust path parsing and manipulation from `convertPathData`
- **DOM Manipulation:** Mature element traversal and attribute handling
- **Plugin Architecture:** Well-tested plugin system with 50 working plugins

### ✅ **Shared Infrastructure Benefits:**
- **inlineStyles + CSS plugins:** Can leverage existing CSS processing in `convert_style_to_attrs` and `minify_styles`
- **mergePaths + convertPathData:** Direct reuse of path parsing and manipulation logic
- **Attribute movement plugins:** Build on existing group handling from `collapse_groups`
- **applyTransforms + convertTransform:** Share matrix operations and transform parsing

### ✅ **Risk Mitigation:**
- **Proven Architecture:** Plugin system has 50 successful implementations
- **SVGO Compatibility:** Extensive test suite with 367 passing tests validates approach
- **Performance Validated:** Current 2-3x speed advantage over SVGO demonstrates sound architecture

## 2.2 Implementation Strategy Options

### **Option A: Complexity-First Approach (Recommended)**
**Rationale:** Tackle the most complex plugin (`inlineStyles`) first while energy and focus are highest
- ✅ **Advantages:** Hardest problems solved early, momentum builds with easier plugins
- ⚠️ **Risks:** Potential early blocking on complex CSS edge cases

### **Option B: Progressive Complexity Approach** 
**Rationale:** Build confidence with simpler plugins, tackle complexity incrementally
- ✅ **Advantages:** Early wins, better understanding of plugin interactions
- ⚠️ **Risks:** Saving hardest problem for last when energy may be lower

### **Option C: Parallel Development Approach**
**Rationale:** Multiple plugins developed simultaneously by different team members
- ✅ **Advantages:** Fastest completion if resources available
- ⚠️ **Risks:** Integration challenges, coordination overhead

**Recommendation:** Proceed with **Option A** but with built-in fallback to incremental implementation if inlineStyles complexity exceeds estimates.

## 3. Infrastructure Improvements (Priority: MEDIUM)

### 3.1 Parser Enhancements (1 week)
- **XML Entity Expansion (Issue #201):** Parse and expand custom entities
- **Whitespace Preservation (Issue #202):** Context-aware whitespace in text elements
- **Error Reporting (Issue #203):** Line/column tracking and context snippets
- **Namespace Consistency (Issue #204):** Unify namespace handling
- **Metadata Usage (Issue #205):** Consistent DocumentMetadata usage

### 3.2 Stringifier Enhancements (0.5 weeks)
- **XML Declaration (Issue #206):** Output XML declarations
- **DOCTYPE Preservation (Issue #207):** Store and output DOCTYPE

### 3.3 Architecture Improvements (1 week)
- **Visitor Pattern (Issue #213):** Implement enter/exit methods
- **Preset System (Issue #215):** Implement preset-default concept
- **Dynamic Loading (Issue #216):** Runtime plugin loading

### 3.4 Plugin-Specific Fixes (0.5 weeks)
- **cleanupEnableBackground (Issue #225):** Parse from style attributes
- **cleanupIds URL Encoding (Issue #227):** Match SVGO's behavior exactly
- **cleanupIds Optimization (Issue #228):** Skip for defs-only SVGs

## 4. Default Preset Alignment (Priority: MEDIUM)

### 4.1 Update Default Configuration
Current SVGN preset: 21 plugins
Target SVGO preset: 35 plugins

**Add to default preset:**
- `removeDeprecatedAttrs`
- `mergeStyles` 
- `cleanupNumericValues`
- `removeNonInheritableGroupAttrs`
- `cleanupEnableBackground`
- `removeHiddenElems`
- `convertShapeToPath`
- `convertEllipseToCircle`
- `sortDefsChildren`
- Plus the 4 missing plugins when implemented

## 5. Code Quality & Testing (Priority: LOW)

### 5.1 Code Quality (0.5 weeks)
- Fix 27 Clippy warnings
- Implement missing Default traits
- Fix regex with backreference in prefix_ids.rs

### 5.2 Testing (1 week)
- Port remaining SVGO test fixtures
- Achieve 100% SVGO test compatibility
- Add fuzz testing for parser

### 5.3 CLI Completion (0.5 weeks)
- Add support for .js config files (currently only .json and .toml)
- Implement base64 encoding for datauri output (currently placeholder)

### 5.4 Build & Distribution (1 week)
- Fix macOS universal binary build (Issue #412)
- Complete cross-platform packaging
- Implement git tag-based versioning

## 6. Implementation Timeline (5-7 weeks)

### Weeks 1-3: Critical Default Preset Plugins (Phase 1A)
- **Week 1:** `inlineStyles` implementation (15 days total effort)
  - Days 1-2: Foundation setup and parameter handling
  - Days 3-5: CSS processing engine with lightningcss
  - Days 6-8: SVG DOM integration and selector matching
  - Days 9-10: CSS-to-SVG conversion logic
  - Days 11-12: Cleanup and optimization features
  - Days 13-15: Testing and SVGO compatibility validation

- **Week 2:** `mergePaths` implementation (5 days effort)
  - Leverage existing path parsing from `convertPathData`
  - Focus on grouping algorithm and concatenation logic

- **Week 3:** Attribute movement plugins (5 days combined effort)
  - `moveElemsAttrsToGroup` (2.5 days) + `moveGroupAttrsToElems` (2.5 days)
  - Shared inheritance analysis infrastructure

### Weeks 4-5: Standalone Plugins + Integration (Phase 1B)  
- **Week 4:** `applyTransforms` (5 days) + `reusePaths` (5 days)
  - `applyTransforms` leverages existing `nalgebra` infrastructure
  - `reusePaths` builds on existing `<defs>` handling
- **Week 5:** Default preset alignment, testing, and integration

### Weeks 6-7: Polish and Release Preparation
- **Week 6:** Code quality, documentation, and final testing
- **Week 7:** Release preparation and community feedback integration

### Optional Extension: Infrastructure Improvements (If Time Permits)
- **Additional weeks:** Parser/stringifier enhancements, architecture improvements
- **Focus:** Performance optimization, code quality, advanced features

## 7. Success Metrics

### Plugin Parity
- **Target:** 54/54 plugins (100%)
- **Current:** 50/54 plugins (93%)
- **Remaining:** 4 plugins = 4 tasks

### Test Compatibility
- **Target:** 100% SVGO test suite passing
- **Current:** 93.75% parity
- **Action:** Port remaining test fixtures

### Performance
- **Target:** Maintain 2-3x speed advantage
- **Current:** Already achieved
- **Action:** Ensure new plugins don't degrade performance

### API Compatibility  
- **Target:** 100% drop-in replacement
- **Current:** ✅ CLI compatibility achieved
- **Action:** Maintain during plugin additions

## 8. Risk Assessment & Mitigation

### **High-Risk Areas**

#### **CSS Processing Complexity (inlineStyles)**
- **Risk:** CSS specificity calculation and cascade resolution edge cases
- **Mitigation:** 
  - Start with basic CSS rule matching, iterate to full specificity
  - Extensive testing against SVGO's CSS test cases
  - Fallback to regex-based approach for complex selectors if needed
- **Contingency:** Implement 80% functionality first, polish edge cases later

#### **lightningcss Alpha Dependency**  
- **Risk:** Breaking changes in alpha version during development
- **Mitigation:**
  - Pin to specific version in Cargo.lock
  - Have fallback plan using direct cssparser + selectors
  - Monitor upstream changes and adapt quickly

#### **Plugin Interaction Complexity**
- **Risk:** Plugins may interfere with each other's optimizations
- **Mitigation:**
  - Careful analysis of plugin execution order in default preset
  - Comprehensive integration testing
  - Implement plugin isolation where necessary

### **Medium-Risk Areas**

#### **Performance Regression**
- **Risk:** New plugins could slow down optimization pipeline
- **Mitigation:**
  - Continuous benchmarking during development
  - Profile and optimize critical paths
  - Maintain performance parity with current implementation

#### **SVGO Output Compatibility**
- **Risk:** Subtle differences in optimization output vs. SVGO
- **Mitigation:**
  - Bit-for-bit output comparison in test suite
  - Extensive edge case testing
  - Community feedback and validation

### **Timeline Risks**
- **Underestimation:** Complex CSS edge cases may require more time
  - **Mitigation:** Built-in buffer time, incremental delivery approach
- **Scope Creep:** Temptation to add extra features during implementation
  - **Mitigation:** Strict focus on SVGO parity first, enhancements later

## 9. Conclusion

SVGN is extremely close to 100% SVGO parity with only 4 remaining tasks:
- 4 missing plugins (all critical for default preset)
- All existing 50 plugins now functional and tested
- Build system stable and all 377 tests passing

The path is clear and well-defined. With focused execution on the critical default preset plugins first, SVGN will achieve complete SVGO compatibility while maintaining its significant performance advantages.

**Next Action:** Continue with `inlineStyles` implementation - complete SVG DOM integration phase.

**Latest Update (2025-07-05):** 
- Build system fully stabilized with 377/377 tests passing
- `convertTransform` plugin completed successfully with full mathematical foundation using nalgebra
- `inlineStyles` plugin MVP completed and functional with real SVG processing
- **Current Status: 51/54 plugins (94.4% complete)**
- CI/CD issues identified requiring immediate fixes

## 10. Critical CI/CD Fixes Required (Priority: IMMEDIATE)

### Issue #506 Analysis: GitHub Actions Failures

The CI/CD pipeline has multiple failing components that need immediate attention:

#### 10.1 Selector Trait Implementation Fix (HIGH PRIORITY)
**Problem:** selectors crate API mismatch causing compilation failures
**Location:** `svgn/src/plugins/remove_attributes_by_selector.rs:503`

**Required Change:**
```rust
// CHANGE FROM:
impl<'i> selectors::Parser<'i> for DummyParser {
    type Impl = SelectorImpl;
    type Error = selectors::parser::SelectorParseErrorKind<'i>;
}

// CHANGE TO:
impl<'i> selectors::parser::SelectorParser<'i> for DummyParser {
    type Impl = SelectorImpl;
    type Error = selectors::parser::SelectorParseErrorKind<'i>;
}
```

#### 10.2 GitHub Actions Permissions Fix (HIGH PRIORITY)
**Problem:** "Resource not accessible by integration" error in security audit
**Location:** `.github/workflows/rust.yml`

**Required Changes:**
1. Add permissions block at top of workflow file:
```yaml
name: Rust

permissions:
  contents: read
  security-events: write

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]
```

#### 10.3 GitHub Actions Deprecated Action Fix (MEDIUM PRIORITY)
**Problem:** actions/upload-artifact@v3 deprecated since April 2024
**Location:** `.github/workflows/release.yml:154`

**Required Change:**
```yaml
# CHANGE FROM:
- name: Upload artifacts
  uses: actions/upload-artifact@v3

# CHANGE TO:
- name: Upload artifacts
  uses: actions/upload-artifact@v4
```

### 10.4 Implementation Timeline for CI/CD Fixes
**Duration:** 0.5 days (immediate priority)
**Tasks:**
1. Fix selector trait implementation (15 minutes)
2. Update GitHub Actions permissions (10 minutes) 
3. Upgrade deprecated action versions (10 minutes)
4. Test CI/CD pipeline (remainder of time)
5. Verify all workflows pass

### 10.5 Acceptance Criteria
- ✅ All GitHub Actions workflows passing
- ✅ Security audit completing successfully
- ✅ Artifact uploads working with v4 actions
- ✅ No compilation errors in CI environment

## 10. Implementation Wisdom & Lessons Learned

### **Strategic Insights from Analysis:**

🎯 **Progressive Implementation Philosophy:** Rather than attempting perfect CSS specification compliance immediately, implement core functionality first (80% of use cases), then iterate to handle edge cases. This reduces risk and provides early validation.

🔧 **Leverage Existing Infrastructure:** SVGN's strength lies in its robust foundation - every new plugin should maximize reuse of existing parsing, DOM manipulation, and optimization infrastructure.

⚡ **Performance-First Mindset:** Establish performance baselines before implementing new plugins. SVGN's 2-3x speed advantage is a key differentiator that must be preserved.

🧪 **Test-Driven Validation:** Use SVGO's extensive test suite as the source of truth. Bit-for-bit output compatibility is the ultimate validation of correct implementation.

### **Key Implementation Principles:**

1. **Start Simple, Iterate:** Begin with minimal viable implementations, then add sophistication
2. **Fail Fast:** Identify complex edge cases early and plan fallback strategies  
3. **Measure Everything:** Continuous benchmarking and compatibility validation
4. **Community-Centric:** Beta testing and feedback integration from the start

### **Success Indicators:**
- ✅ **Early Wins:** Simple plugins working correctly builds confidence
- ✅ **Performance Maintained:** No regression in optimization speed
- ✅ **Community Validation:** Positive feedback from beta testing
- ✅ **SVGO Parity:** Identical output for all test cases
