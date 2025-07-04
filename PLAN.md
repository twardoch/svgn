# SVGN Development Plan: Path to 100% SVGO Parity

## Executive Summary

SVGN is a high-performance Rust port of SVGO that has achieved 91% plugin implementation. This plan outlines the focused path to achieve complete SVGO v4.0.0 compatibility.

**Current Status (2025-07-04):**
- ✅ **49/54 plugins** fully implemented and functional (91%)
- ✅ **convertPathData** fully implemented 
- ✅ **removeUselessStrokeAndFill** fully implemented (was incorrectly listed as missing)
- ✅ **removeAttributesBySelector** fixed and enabled (CSS parsing issue resolved)
- ❌ **5 plugins** missing for 100% parity
- ✅ **Full CLI compatibility** achieved
- ✅ **359 tests passing** (100% success rate)

**Path to 100% Parity:** Implement 5 missing plugins = 5 total tasks

## 1. Critical Missing Plugins (Priority: IMMEDIATE)

### Phase 1A: Default Preset Plugins (Highest Impact)
These 5 plugins are in SVGO's default preset and required for preset compatibility:

#### 1.1 convertTransform (2 weeks - HIGH)  
- **Impact:** Critical - in SVGO default preset position 28/35
- **Complexity:** High - matrix math and decomposition
- **Dependencies:** Add `nalgebra` crate
- **Implementation:**
  - Parse transform strings into matrices
  - Multiply consecutive transforms
  - Convert matrices to shorter translate/scale/rotate forms
  - Optimize precision and remove redundant transforms

#### 1.2 inlineStyles (1.5 weeks - HIGH)
- **Impact:** Critical - in SVGO default preset position 10/35  
- **Complexity:** High - requires CSS engine
- **Dependencies:** Add `lightningcss` or `css` crate
- **Implementation:**
  - Parse CSS from `<style>` elements
  - Implement CSS specificity calculation
  - Match selectors to SVG elements
  - Apply cascade rules and convert to attributes

#### 1.3 mergePaths (1 week - MEDIUM)
- **Impact:** Critical - in SVGO default preset position 31/35
- **Complexity:** Medium - path concatenation and style matching
- **Implementation:**
  - Group paths by identical style attributes
  - Check DOM adjacency for mergeable paths
  - Concatenate path data strings correctly

#### 1.4 moveElemsAttrsToGroup + moveGroupAttrsToElems (1 week - MEDIUM)
- **Impact:** Critical - in SVGO default preset positions 24-25/35
- **Complexity:** Medium - DOM analysis and inheritance
- **Implementation:**
  - Analyze attributes across sibling elements
  - Move common inheritable attributes to/from groups
  - Calculate size reduction benefits

### Phase 1B: Standalone Plugins (Lower Priority)

#### 1.5 applyTransforms (1 week - HIGH)
- **Impact:** Medium - not in default preset
- **Complexity:** High - coordinate transformation
- **Implementation:**
  - Apply transform matrices to path coordinates
  - Transform basic shape coordinates
  - Remove transform attributes after application

#### 1.6 reusePaths (1 week - MEDIUM)
- **Impact:** Low - not in default preset  
- **Complexity:** Medium - path deduplication
- **Implementation:**
  - Hash path content for duplicate detection
  - Create `<defs>` and `<use>` elements
  - Replace duplicates with references

## 2. Remaining Missing Plugins (Priority: IMMEDIATE)

With `removeAttributesBySelector` now fixed, only 5 plugins remain to achieve 100% SVGO parity:

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
- Plus the 5 missing plugins when implemented

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

## 6. Implementation Timeline (7-9 weeks)

### Weeks 1-4: Critical Plugins (Phase 1A)
- Week 1: `removeAttributesBySelector` fix + `convertTransform` start
- Week 2: `convertTransform` completion
- Week 3: `inlineStyles`
- Week 4: `mergePaths` + `moveElemsAttrsToGroup` + `moveGroupAttrsToElems`

### Weeks 5-6: Standalone Plugins (Phase 1B)  
- Week 5: `applyTransforms` + `reusePaths`
- Week 6: Default preset alignment + testing

### Weeks 7-8: Infrastructure & Polish
- Week 7: Parser/stringifier enhancements
- Week 8: Architecture improvements + code quality

### Week 9: Final Testing & Release
- Week 9: Complete test suite compatibility + documentation + release preparation

## 7. Success Metrics

### Plugin Parity
- **Target:** 54/54 plugins (100%)
- **Current:** 49/54 plugins (91%)
- **Remaining:** 5 plugins = 5 tasks

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

## 8. Risk Mitigation

### Technical Risks
- **CSS Engine Complexity:** Use proven `lightningcss` crate
- **Matrix Math Complexity:** Use `nalgebra` crate
- **SVGO Compatibility:** Extensive testing against SVGO output

### Timeline Risks
- **Underestimation:** Break into smaller increments, test frequently
- **Dependencies:** Prioritize default preset plugins first

## 9. Conclusion

SVGN is extremely close to 100% SVGO parity with only 5 remaining tasks:
- 5 missing plugins (5 critical for default preset)
- All existing plugins now functional

The path is clear and well-defined. With focused execution on the critical default preset plugins first, SVGN will achieve complete SVGO compatibility while maintaining its significant performance advantages.

**Next Action:** Begin with `convertTransform` implementation as the highest-priority missing plugin for default preset compatibility.