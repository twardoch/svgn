# SVGN Development Plan 2.0: Complete SVGO Compatibility Specification

## 1. Executive Summary

SVGN is a high-performance Rust port of SVGO v4.0.0 that has achieved significant progress but requires focused effort to reach 100% feature parity and API compatibility. This comprehensive plan provides detailed specifications for completing the remaining work to achieve full SVGO compatibility.

**Current Status (As of 2025-01-04):**
- ✅ **47 plugin files** implemented in Rust
- ✅ **46 plugins** actively registered and functional 
- ✅ **convertPathData** fully implemented (not a stub!)
- ⚠️ **1 plugin** disabled due to CSS parsing issues (removeAttributesBySelector)
- ❌ **7 SVGO plugins** completely missing
- ✅ **Core infrastructure** operational (parser, AST, stringifier, plugin system)
- ✅ **Full CLI compatibility** achieved with SVGO
- ✅ **359 tests passing** (100% success rate for implemented features)

## 2. Critical Plugin Analysis: SVGO vs SVGN

### 2.1. 1.1 SVGO Plugin Inventory (53 Total Plugins)

Based on comprehensive analysis of `/ref/svgo/plugins/`, SVGO has **53 plugins** organized as:
- **35 plugins** in default preset (core optimization pipeline)
- **18 plugins** available but not in default preset

### 2.2. 1.2 SVGN Plugin Status Breakdown

#### 2.2.1. ✅ Fully Implemented and Compatible (46 plugins)

**Document Structure:**
1. `addAttributesToSVGElement` - Adds attributes to root `<svg>` element
2. `addClassesToSVGElement` - Adds class names to root `<svg>` element
3. `removeComments` - Removes comments (preserves legal comments)
4. `removeDoctype` - Removes doctype declarations
5. `removeDesc` - Removes `<desc>` elements
6. `removeMetadata` - Removes `<metadata>` elements
7. `removeTitle` - Removes `<title>` elements
8. `removeXMLProcInst` - Removes XML processing instructions

**Attribute Management:**
9. `cleanupAttrs` - Cleans up attribute whitespace and formatting
10. `cleanupEnableBackground` - Cleans up enable-background attribute
11. `cleanupIds` - Minifies and removes unused IDs
12. `cleanupListOfValues` - Rounds numeric values in list attributes
13. `cleanupNumericValues` - Rounds numeric values to fixed precision
14. `removeAttrs` - Removes attributes by pattern/name
15. `removeAttributesBySelector` - ⚠️ **DISABLED** (CSS parsing issues)
16. `removeDeprecatedAttrs` - Removes deprecated SVG attributes
17. `removeDimensions` - Removes width/height (preserves viewBox)
18. `removeEditorsNSData` - Removes editor namespaces and metadata
19. `removeElementsByAttr` - Removes elements by ID or class
20. `removeEmptyAttrs` - Removes empty attributes
21. `removeUnknownsAndDefaults` - Removes unknown elements and defaults
22. `removeUnusedNS` - Removes unused namespace declarations
23. `removeViewBox` - Removes viewBox when possible
24. `removeXlink` - Removes deprecated xlink attributes
25. `removeXMLNS` - Removes xmlns attribute from root element
26. `sortAttrs` - Sorts element attributes for compression

**Style Processing:**
27. `convertColors` - Converts colors to hex format
28. `convertStyleToAttrs` - Converts style attributes to presentation attributes
29. `mergeStyles` - Merges multiple `<style>` elements
30. `minifyStyles` - Basic CSS minification
31. `removeStyleElement` - Removes `<style>` elements (SVGN-exclusive)

**Shape and Path Optimization:**
32. `convertEllipseToCircle` - Converts `<ellipse>` to `<circle>` when possible
33. `convertOneStopGradients` - Converts single-stop gradients to solid colors
34. `convertPathData` - ✅ **FULLY IMPLEMENTED** (not a stub!)
35. `convertShapeToPath` - Converts basic shapes to `<path>` elements

**Structure Optimization:**
36. `collapseGroups` - Collapses useless groups
37. `removeEmptyContainers` - Removes empty container elements
38. `removeEmptyText` - Removes empty text elements
39. `removeHiddenElems` - Removes hidden elements
40. `removeNonInheritableGroupAttrs` - Removes non-inheritable group attributes
41. `removeOffCanvasPaths` - Removes elements outside viewBox
42. `removeUselessDefs` - Removes `<defs>` without IDs
43. `removeUselessTransforms` - Removes identity transforms (SVGN-exclusive)
44. `sortDefsChildren` - Sorts `<defs>` children for compression

**Security and Content:**
45. `prefixIds` - Adds prefix to IDs
46. `removeRasterImages` - Removes raster images
47. `removeScripts` - Removes `<script>` elements

#### 2.2.2. ❌ Missing SVGO Plugins (7 plugins)

These are the **critical missing plugins** needed for 100% SVGO compatibility:

### 2.3. 1.3 Missing Plugin Specifications

#### 2.3.1. 1.3.1 applyTransforms (Not in SVGN)

**Purpose:** Applies transform attributes directly to path coordinates and shapes
**SVGO Default Preset:** No (standalone plugin)
**Complexity:** HIGH - Requires matrix math and coordinate transformation

**Implementation Requirements:**
- Parse transform matrices from `transform` attribute
- Apply transforms to path coordinates using matrix multiplication
- Transform basic shape coordinates (rect, circle, ellipse)
- Handle nested transforms correctly
- Remove `transform` attribute after application

**Parameters (from SVGO):**
```rust
struct ApplyTransformsParams {
    transform_precision: u8,        // default: 5
    apply_transforms_stroked: bool, // default: true
}
```

**Implementation Steps:**
1. Create transform matrix parser
2. Implement matrix multiplication functions
3. Create coordinate transformation functions for each path command
4. Handle shape coordinate transformation
5. Add comprehensive test suite

#### 2.3.2. 1.3.2 convertTransform (Missing - Critical for Default Preset)

**Purpose:** Optimizes transform matrices, converts to shorter forms
**SVGO Default Preset:** Yes (position 28/35)
**Complexity:** HIGH - Matrix decomposition and optimization

**Implementation Requirements:**
- Parse transform strings into matrices
- Multiply consecutive transforms
- Convert matrices back to shorter transform functions
- Optimize precision and remove redundant transforms
- Decompose matrices into translate/scale/rotate when beneficial

**Parameters (from SVGO):**
```rust
struct ConvertTransformParams {
    convert_to_shorts: bool,      // default: true
    deg_precision: u8,            // default: 3
    float_precision: u8,          // default: 3
    transform_precision: u8,      // default: 5
    matrix_to_transform: bool,    // default: true
    short_translate: bool,        // default: true
    short_scale: bool,           // default: true
    short_rotate: bool,          // default: true
    remove_useless: bool,        // default: true
    collapse_into_one: bool,     // default: true
    leading_zero: bool,          // default: true
    negative_extra_space: bool,  // default: true
}
```

#### 2.3.3. 1.3.3 inlineStyles (Missing - Critical for Default Preset)

**Purpose:** Inlines styles from `<style>` elements to `style` attributes
**SVGO Default Preset:** Yes (position 10/35)
**Complexity:** HIGH - Requires full CSS engine

**Implementation Requirements:**
- Parse CSS from `<style>` elements using css-parser crate
- Implement CSS specificity calculation
- Match CSS selectors to SVG elements
- Apply cascade rules and inheritance
- Convert CSS properties to SVG attributes
- Handle media queries and pseudo-classes

**Parameters (from SVGO):**
```rust
struct InlineStylesParams {
    only_matched_once: bool,        // default: true
    remove_matched_selectors: bool, // default: true
    use_mqs: Vec<String>,          // default: ["", "screen"]
    use_pseudos: Vec<String>,      // default: [":hover"]
}
```

#### 2.3.4. 1.3.4 mergePaths (Missing - Critical for Default Preset)

**Purpose:** Merges multiple path elements with identical styles
**SVGO Default Preset:** Yes (position 31/35)
**Complexity:** MEDIUM - Path concatenation and style matching

**Implementation Requirements:**
- Group paths by identical fill, stroke, and other style attributes
- Check DOM adjacency for mergeable paths
- Concatenate path data strings correctly
- Handle edge cases with transforms and animations

**Parameters (from SVGO):**
```rust
struct MergePathsParams {
    force: bool,               // default: false
    float_precision: u8,       // default: 3
    no_space_after_flags: bool, // default: false
}
```

#### 2.3.5. 1.3.5 moveElemsAttrsToGroup (Missing - Critical for Default Preset)

**Purpose:** Moves common attributes from elements to parent group
**SVGO Default Preset:** Yes (position 24/35)
**Complexity:** MEDIUM - DOM analysis and inheritance calculation

**Implementation Requirements:**
- Analyze attributes across sibling elements
- Identify inheritable SVG attributes
- Calculate size reduction benefit
- Create `<g>` wrapper if beneficial
- Move attributes and remove from children

#### 2.3.6. 1.3.6 moveGroupAttrsToElems (Missing - Critical for Default Preset)

**Purpose:** Distributes group attributes to children when beneficial
**SVGO Default Preset:** Yes (position 25/35)
**Complexity:** MEDIUM - Reverse of moveElemsAttrsToGroup

**Implementation Requirements:**
- Identify groups that only provide attributes
- Distribute inheritable attributes to children
- Handle attribute conflicts (child overrides)
- Remove empty groups after distribution

#### 2.3.7. 1.3.7 removeUselessStrokeAndFill (Missing - Critical for Default Preset)

**Purpose:** Removes useless stroke and fill attributes
**SVGO Default Preset:** Yes (position 18/35)
**Complexity:** MEDIUM - Style inheritance analysis

**Implementation Requirements:**
- Understand SVG rendering model and inheritance
- Remove `stroke="none"` when not inherited
- Remove `fill="none"` when not inherited  
- Remove `stroke-width="0"` (equivalent to `stroke="none"`)
- Handle `currentColor` correctly
- Track inherited values through DOM tree

### 2.4. 1.4 Plugin Issues to Fix

#### 2.4.1. 1.4.1 removeAttributesBySelector (Implemented but Disabled)

**Issue:** CSS selector parsing not working, commented out in registry
**Status:** Plugin exists but disabled due to CSS parsing problems
**Solution:** Use `selectors` crate for proper CSS selector parsing

**Implementation Fix:**
```rust
// Use the selectors crate instead of manual parsing
use selectors::{parser::Parser, visitor::Visit};
```

## 3. Infrastructure Issues Analysis

### 3.1. 2.1 Parser Issues (5 issues)

#### 3.1.1. Issue #201: XML Entity Expansion
**Problem:** Rust parser doesn't handle XML entity declarations for expansion
**Impact:** SVGs with custom entities fail to parse correctly
**Solution:** 
- Parse `<!ENTITY>` declarations in DOCTYPE
- Build entity table during parsing
- Expand `&entity;` references throughout document

#### 3.1.2. Issue #202: Selective Whitespace Preservation  
**Problem:** Rust parser trims whitespace globally, SVGO preserves in text elements
**Impact:** Text content may be corrupted
**Solution:**
- Preserve whitespace in `<text>`, `<tspan>`, `<pre>`, `<script>`, `<style>`
- Add context-aware whitespace handling

#### 3.1.3. Issue #203: Enhanced Error Reporting
**Problem:** Rust parser lacks detailed context and snippets in errors
**Impact:** Poor debugging experience
**Solution:**
- Track line/column positions during parsing
- Provide context snippets in error messages

#### 3.1.4. Issue #204: Namespace Handling Consistency
**Problem:** AST stores `xmlns` in both `namespaces` and `attributes` maps
**Impact:** Potential inconsistencies and redundancy
**Solution:**
- Unify namespace handling in single location
- Ensure consistent access patterns

#### 3.1.5. Issue #205: Document Metadata Usage
**Problem:** `DocumentMetadata` fields not consistently used
**Impact:** Loss of document information during processing
**Solution:**
- Ensure `path`, `encoding`, `version` are properly populated
- Use metadata throughout optimization pipeline

### 3.2. 2.2 Stringifier Issues (2 issues)

#### 3.2.1. Issue #206: XML Declaration Support
**Problem:** Rust stringifier doesn't output XML declarations
**Solution:** Add XML declaration based on `DocumentMetadata`

#### 3.2.2. Issue #207: DOCTYPE Preservation
**Problem:** DOCTYPE declarations are lost during stringification
**Solution:** Store and output DOCTYPE declarations with entities

### 3.3. 2.3 Architecture Issues (4 issues)

#### 3.3.1. Issue #213: Visitor Pattern Implementation
**Problem:** Single `apply` method instead of granular visitor pattern
**Solution:** Implement proper visitor with `enter`/`exit` methods

#### 3.3.2. Issue #215: Missing Preset System
**Problem:** No concept of presets like SVGO's `preset-default`
**Solution:** Implement `Preset` trait and preset configurations

#### 3.3.3. Issue #216: Limited Dynamic Plugin Loading
**Problem:** Static plugin registration only
**Solution:** Add runtime plugin loading capability

#### 3.3.4. Issue #217: Inconsistent Plugin Parameter Validation
**Problem:** Parameter validation inconsistent across plugins
**Solution:** Standardize parameter validation patterns

### 3.4. 2.4 Plugin-Specific Issues (3 issues)

#### 3.4.1. Issue #225: cleanupEnableBackground Style Handling
**Problem:** Only handles attribute, not style properties
**Solution:** Parse `enable-background` from `style` attributes

#### 3.4.2. Issue #227: cleanupIds URL Encoding
**Problem:** Different encoding behavior than SVGO
**Solution:** Match SVGO's `encodeURI`/`decodeURI` behavior exactly

#### 3.4.3. Issue #228: cleanupIds Optimization Skip  
**Problem:** Missing optimization for `<defs>`-only SVGs
**Solution:** Skip ID minification for SVGs with only `<defs>`

## 4. Implementation Roadmap

### 4.1. Phase 1: Critical Missing Plugins (4-6 weeks)

**Priority 1A: Default Preset Plugins (4 weeks)**
1. `removeUselessStrokeAndFill` (2 weeks) - MEDIUM complexity
2. `convertTransform` (2 weeks) - HIGH complexity, critical for preset

**Priority 1B: Transform and Path Plugins (2 weeks)**  
3. `mergePaths` (1 week) - MEDIUM complexity, critical for preset
4. `applyTransforms` (1 week) - HIGH complexity, but not in default preset

**Priority 1C: Style and Structure Plugins (2 weeks)**
5. `inlineStyles` (1 week) - HIGH complexity, CSS parsing required
6. `moveElemsAttrsToGroup` (0.5 weeks) - MEDIUM complexity
7. `moveGroupAttrsToElems` (0.5 weeks) - MEDIUM complexity

### 4.2. Phase 2: Fix Issues and Infrastructure (2-3 weeks)

**Priority 2A: Fix Disabled Plugin (0.5 weeks)**
1. Fix `removeAttributesBySelector` CSS parsing issue

**Priority 2B: Parser Enhancements (1 week)**
2. XML entity expansion (Issue #201)
3. Selective whitespace preservation (Issue #202)
4. Enhanced error reporting (Issue #203)

**Priority 2C: Architecture Improvements (1 week)**
5. Visitor pattern implementation (Issue #213)
6. Preset system (Issue #215)

**Priority 2D: Plugin Issues (0.5 weeks)**
7. Fix cleanupEnableBackground style handling (Issue #225)
8. Fix cleanupIds URL encoding (Issue #227)
9. Fix cleanupIds optimization skip (Issue #228)

### 4.3. Phase 3: Polish and Testing (1-2 weeks)

**Priority 3A: Code Quality (0.5 weeks)**
1. Fix all Clippy warnings
2. Implement missing Default traits
3. Fix regex with backreference

**Priority 3B: Testing (1 week)**
4. Port remaining SVGO test fixtures
5. Achieve 100% SVGO test compatibility
6. Add fuzz testing for parser

**Priority 3C: Documentation (0.5 weeks)**
7. Update all documentation
8. Create migration guides
9. Document API compatibility

## 5. Detailed Plugin Implementation Specifications

### 5.1. 4.1 removeUselessStrokeAndFill Implementation

**File:** `svgn/src/plugins/remove_useless_stroke_and_fill.rs`

**Core Logic:**
```rust
pub struct RemoveUselessStrokeAndFillPlugin;

impl Plugin for RemoveUselessStrokeAndFillPlugin {
    fn name(&self) -> &'static str {
        "removeUselessStrokeAndFill"
    }

    fn description(&self) -> &'static str {
        "removes useless stroke and fill attributes"
    }

    fn apply(&mut self, document: &mut Document, _info: &PluginInfo, params: Option<&Value>) -> PluginResult<()> {
        // Parse parameters
        let remove_stroke = params
            .and_then(|v| v.get("stroke"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
            
        let remove_fill = params
            .and_then(|v| v.get("fill"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Process document
        process_element(&mut document.root, remove_stroke, remove_fill)?;
        Ok(())
    }
}

fn process_element(element: &mut Element, remove_stroke: bool, remove_fill: bool) -> PluginResult<()> {
    // Logic to determine if stroke/fill attributes are useless
    // Consider inheritance, element type, and rendering context
    // Remove attributes that don't affect rendering
}
```

### 5.2. 4.2 convertTransform Implementation

**File:** `svgn/src/plugins/convert_transform.rs`

**Dependencies:** Add `nalgebra` crate for matrix operations

**Core Logic:**
```rust
use nalgebra::{Matrix3, Vector2};

pub struct ConvertTransformPlugin;

// Transform parsing and optimization logic
fn parse_transform_list(transform_str: &str) -> Result<Vec<Transform>, String> {
    // Parse individual transform functions
}

fn optimize_transform_list(transforms: Vec<Transform>) -> String {
    // Multiply matrices, decompose when beneficial
    // Convert to shortest representation
}

// Matrix decomposition for readability
fn decompose_matrix(matrix: Matrix3<f64>) -> Option<String> {
    // Attempt to convert matrix to translate/scale/rotate
}
```

### 5.3. 4.3 inlineStyles Implementation  

**File:** `svgn/src/plugins/inline_styles.rs`

**Dependencies:** Add `lightningcss` or `css` crate for CSS parsing

**Core Logic:**
```rust
use css::parser::Parser;
use css::selector::Selector;

pub struct InlineStylesPlugin;

// CSS parsing and specificity calculation
fn parse_stylesheet(style_content: &str) -> Result<Stylesheet, String> {
    // Parse CSS rules and selectors
}

fn calculate_specificity(selector: &Selector) -> u32 {
    // Implement CSS specificity calculation
}

fn apply_styles_to_elements(element: &mut Element, rules: &[CssRule]) {
    // Match selectors and apply styles as attributes
}
```

## 6. Test Compatibility Requirements

### 6.1. 5.1 SVGO Test Suite Porting

**Objective:** Achieve 100% compatibility with SVGO test output

**Current Status:** 359 tests passing (93.75% parity achieved)

**Requirements:**
1. Port all remaining SVGO test fixtures from `/ref/svgo/test/plugins/`
2. Ensure identical output for all test cases
3. Add regression tests for each fixed plugin

### 6.2. 5.2 Test File Locations

**SVGO Test Fixtures:** `/ref/svgo/test/plugins/`
**SVGN Test Implementation:** `/svgn/tests/plugins/`

**Process:**
1. For each missing plugin, port corresponding `.svg.txt` test files
2. Implement Rust test cases that verify identical output
3. Add parameterized tests for plugin configuration options

## 7. Configuration Compatibility

### 7.1. 6.1 Default Preset Alignment

**Current SVGN Default (21 plugins):**
```rust
vec![
    "removeComments", "removeMetadata", "removeTitle", "removeDesc",
    "removeDoctype", "removeXMLProcInst", "removeEditorsNSData",
    "cleanupAttrs", "removeEmptyAttrs", "removeUnknownsAndDefaults",
    "removeUnusedNS", "removeUselessDefs", "cleanupIds", "minifyStyles",
    "convertStyleToAttrs", "convertColors", "removeEmptyText",
    "removeEmptyContainers", "collapseGroups", "removeUselessTransforms",
    "sortAttrs"
]
```

**Target SVGO Default (35 plugins):**
Must add these plugins to achieve preset compatibility:
- `removeDeprecatedAttrs`
- `mergeStyles` 
- `inlineStyles` (missing - implement)
- `cleanupNumericValues`
- `removeNonInheritableGroupAttrs`
- `removeUselessStrokeAndFill` (missing - implement)
- `cleanupEnableBackground`
- `removeHiddenElems`
- `convertShapeToPath`
- `convertEllipseToCircle`
- `moveElemsAttrsToGroup` (missing - implement)
- `moveGroupAttrsToElems` (missing - implement)
- `convertPathData` (implemented!)
- `convertTransform` (missing - implement)
- `mergePaths` (missing - implement)
- `sortDefsChildren`

### 7.2. 6.2 Parameter Compatibility

**Requirement:** All plugin parameters must match SVGO exactly

**Example - convertPathData parameters:**
```rust
// Must support all SVGO parameters
struct ConvertPathDataParams {
    apply_transforms: bool,           // default: true
    apply_transforms_stroked: bool,   // default: true
    make_arcs: Option<ArcConfig>,     // default: { threshold: 2.5, tolerance: 0.5 }
    straight_curves: bool,            // default: true
    convert_to_q: bool,              // default: true
    line_shorthands: bool,           // default: true
    convert_to_z: bool,              // default: true
    curve_smooth_shorthands: bool,   // default: true
    float_precision: Option<u8>,     // default: 3
    transform_precision: u8,         // default: 5
    smart_arc_rounding: bool,        // default: true
    remove_useless: bool,            // default: true
    collapse_repeated: bool,         // default: true
    utilize_absolute: bool,          // default: true
    leading_zero: bool,              // default: true
    negative_extra_space: bool,      // default: true
}
```

## 8. Success Metrics and Verification

### 8.1. 7.1 Plugin Parity Metrics

**Target:** 53/53 plugins implemented (100%)
**Current:** 46/53 plugins functional (87%)
**Remaining:** 7 plugins to implement

### 8.2. 7.2 Test Compatibility Metrics

**Target:** 100% SVGO test suite passing
**Current:** 359 tests passing, 93.75% parity
**Requirements:** Port and pass all remaining SVGO test cases

### 8.3. 7.3 Performance Metrics

**Target:** Maintain 2-3x performance advantage over SVGO
**Current:** Already achieved for implemented plugins
**Requirement:** New plugins must not degrade performance

### 8.4. 7.4 API Compatibility Metrics  

**Target:** 100% drop-in CLI replacement
**Current:** ✅ Already achieved
**Requirement:** Maintain CLI compatibility during plugin additions

## 9. Risk Assessment and Mitigation

### 9.1. 8.1 Technical Risks

**High Risk - CSS Engine Complexity (inlineStyles)**
- Mitigation: Use proven CSS parser crates (`lightningcss`, `css`)
- Alternative: Implement subset sufficient for SVG use cases

**Medium Risk - Matrix Math Complexity (convertTransform)**
- Mitigation: Use `nalgebra` crate for robust matrix operations
- Alternative: Port SVGO's JavaScript matrix code directly

**Low Risk - Path Optimization Complexity**
- Mitigation: `convertPathData` already implemented successfully

### 9.2. 8.2 Timeline Risks

**Risk:** Underestimating implementation complexity
- Mitigation: Break down into smaller increments, test frequently
- Contingency: Prioritize plugins in default preset first

### 9.3. 8.3 Compatibility Risks

**Risk:** Subtle differences in plugin behavior
- Mitigation: Extensive testing against SVGO output
- Validation: Port all SVGO test fixtures

## 10. Long-term Vision

### 10.1. 9.1 Beyond SVGO Parity

Once 100% parity is achieved:
- Advanced path optimization using computational geometry
- GPU-accelerated optimizations
- Machine learning-based optimization hints
- Real-time optimization APIs

### 10.2. 9.2 Ecosystem Leadership

- Contribute optimizations back to SVGO project
- Become reference implementation for SVG optimization
- Drive SVG specification improvements

## 11. Conclusion

SVGN has made exceptional progress with 87% plugin implementation and full CLI compatibility. The remaining work is well-defined and achievable within 8-10 weeks:

**Immediate Priorities (Weeks 1-4):**
1. Implement 7 missing plugins, focusing on default preset plugins first
2. Fix removeAttributesBySelector CSS parsing issue

**Infrastructure Enhancement (Weeks 5-6):**  
3. Address parser and stringifier issues
4. Implement visitor pattern and preset system

**Polish and Release (Weeks 7-8):**
5. Fix all code quality issues  
6. Achieve 100% SVGO test compatibility
7. Update documentation

This plan provides the detailed roadmap to achieve complete SVGO compatibility while maintaining SVGN's performance advantages and positioning it as the definitive SVG optimization solution for the Rust ecosystem.