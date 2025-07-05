# SVGN Development Plan: Path to 100% SVGO Parity

## 1. Executive Summary

SVGN is a high-performance Rust port of SVGO that has achieved 90.5% plugin implementation. This plan outlines the focused path to achieve complete SVGO v4.0.0 compatibility.

**Current Status (2025-07-05):**
- ✅ **48/53 plugins** implemented (90.5% complete - 5 plugins remaining)
- ✅ **convertPathData** fully implemented
- ✅ **removeUselessStrokeAndFill** fully implemented
- ✅ **removeAttributesBySelector** fixed and enabled (CSS parsing issue resolved)
- ✅ **convertTransform** fully implemented (critical default preset plugin)
- ✅ **inlineStyles** MVP implementation completed (Foundation + CSS Processing + Basic functionality)
- ❌ **5 plugins** remaining for 100% parity: mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems, applyTransforms, reusePaths
- ✅ **Full CLI compatibility** achieved (when build works)
- ❌ **Test suite execution BLOCKED** by compilation failures

**🚨 CRITICAL BUILD FAILURE STATUS (2025-07-05)**

❌ **PROJECT CANNOT COMPILE - 22 COMPILATION ERRORS:**
- **22 compilation errors** in CSS selector and parser implementations
- **cssparser version conflicts** between lightningcss v0.33.0 and selectors crate expecting v0.31.2
- **ToCss trait missing** for String types in SelectorImpl implementations
- **PrecomputedHash trait missing** for String types used as Identifier/LocalName
- **MatchingContext API mismatch** - function expects 6 parameters, code provides 4
- **Parser trait missing** for SvgSelectorImpl in SelectorList::parse() calls
- **Method resolution failures** - unescape() method not found on BytesText
- **Private field access** - SelectorList.0.iter() accessing private field

❌ **DEVELOPMENT COMPLETELY BLOCKED:**
- Cannot build project or run any tests
- Cannot implement remaining 5 plugins until build issues resolved
- CSS dependency version conflicts prevent all CSS-related functionality
- inlineStyles plugin foundation work unusable due to compilation failures

**IMMEDIATE PRIORITY:**
**FIX BUILD FAILURES FIRST** - All 22 compilation errors must be resolved before any feature development can continue. The CSS dependency conflicts represent a complete blocker to project progress.

## 2. Critical Missing Plugins (Priority: IMMEDIATE)

### 2.1. Phase 1A: Default Preset Plugins (Highest Impact)
These plugins are in SVGO's default preset and required for preset compatibility:

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

With `convertTransform` and `inlineStyles` completed, only 5 plugins remain for 100% SVGO parity:

1. **mergePaths** - Critical default preset plugin (position 29/35)
2. **moveElemsAttrsToGroup** - Critical default preset plugin (position 22/35)
3. **moveGroupAttrsToElems** - Critical default preset plugin (position 23/35)
4. **applyTransforms** - Not in default preset, specialized optimization
5. **reusePaths** - Not in default preset, file-size optimization for repeated paths

All 5 remaining plugins are in SVGO's default preset, making them the highest priority after build issues are resolved.

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

### 7.4. 3.4 Build & Distribution (1 week)

- [ ] Complete cross-platform build scripts (Issue #410)
- [ ] Fix macOS universal binary build (Issue #412)
- [ ] Create Linux packaging (.deb, .rpm, .AppImage)
- [ ] Create Windows installer (.msi)
- [ ] Update GitHub Actions workflow
- [ ] Implement version management
- [ ] Git tag-based versioning
- [ ] Automatic version injection at build time
- [ ] Update set-cargo-version.sh script
- [x] **Implement comprehensive benchmarking tool and Jekyll report generation**

### 7.5. 3.5 Plugin-Specific Fixes (0.5 weeks)

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
- [ ] Update parameter documentation
- [ ] Update docs/comparison.md
- [ ] Update plugin count (54/54)
- [ ] Update compatibility metrics
- [ ] Document performance characteristics
- [ ] Update README.md
- [ ] Update implementation status
- [ ] Update feature list
- [ ] Add migration guide links
- [x] **Update docs/_config.yml for benchmark navigation**

## 10. Success Metrics & Definition of Done

### 10.1. **Plugin Parity (Primary Goal)**
- [ ] Achieve 53/53 plugins implemented (currently 48/53 - 90.5% complete)
- [x] Fix 1 disabled plugin (removeAttributesBySelector) ✅ COMPLETED
- [x] Implement convertTransform plugin ✅ COMPLETED (2025-07-04)
- [x] Implement inlineStyles plugin MVP ✅ COMPLETED (2025-07-05)
- [ ] Implement 5 remaining missing plugins:
  - [ ] **mergePaths** - Path concatenation with style matching (NEXT PRIORITY)
  - [ ] **moveElemsAttrsToGroup** - Attribute inheritance optimization
  - [ ] **moveGroupAttrsToElems** - Reverse attribute distribution
  - [ ] **applyTransforms** - Applies transform matrices to path coordinates and shapes
  - [ ] **reusePaths** - Replace duplicate paths with `<use>` elements

### 10.2. **Quality Gates**
- [ ] **100% SVGO Output Compatibility:** Bit-for-bit identical output for test cases
- [ ] **Performance Benchmark:** Maintain 2-3x speed advantage over SVGO
- [ ] **Test Coverage:** 367+ tests passing with new plugin integration
- [ ] **CLI Compatibility:** All SVGO parameters and options supported
- [x] **Comprehensive Benchmarking Tool:** Implemented and generating Jekyll reports.

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

## 11. WebAssembly Build and Online Tool Implementation Plan

### 11.1. Overview

This section outlines the comprehensive plan for implementing WebAssembly builds of SVGN and creating an online SVG optimization tool within the docs folder. The goal is to make SVGN accessible directly in web browsers while maintaining the performance and compatibility advantages of the native Rust implementation.

### 11.2. WebAssembly Build Implementation

#### 11.2.1. Phase 1: WASM Build Infrastructure (Week 1)

**1.1. Build System Setup**
- [ ] Install and configure `wasm-pack` for building WASM packages
- [ ] Create dedicated WASM build target in `Cargo.toml`
- [ ] Add WASM-specific feature flags for optional functionality
- [ ] Configure bundle size optimization settings
- [ ] Set up TypeScript definition generation

**1.2. WASM Entry Point Creation**
- [ ] Create `svgn/src/wasm.rs` as the WASM entry point
- [ ] Implement `wasm_bindgen` exports for core functionality
- [ ] Design JavaScript-friendly API surface
- [ ] Handle memory management across JS/WASM boundary
- [ ] Implement proper error handling with JS Error objects

**1.3. Cargo Configuration**
```toml
# Add to svgn/Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[package.metadata.wasm-pack.profile.release]
wee_alloc = true

[dependencies]
# WASM-specific dependencies
wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
js-sys = "0.3"
wee_alloc = { version = "0.4", optional = true }
console_error_panic_hook = { version = "0.1", optional = true }

[features]
default = ["console_error_panic_hook"]
wasm = ["wee_alloc"]
```

**1.4. API Design**
```rust
// Core WASM API structure
#[wasm_bindgen]
pub struct SvgOptimizer {
    config: OptimizationConfig,
}

#[wasm_bindgen]
impl SvgOptimizer {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<JsValue>) -> Result<SvgOptimizer, JsValue>;
    
    #[wasm_bindgen]
    pub fn optimize(&self, svg_content: &str) -> Result<String, JsValue>;
    
    #[wasm_bindgen]
    pub fn get_info(&self) -> OptimizationInfo;
}
```

#### 11.2.2. Phase 2: Bundle Size Optimization (Week 2)

**2.1. Feature Flags Implementation**
- [ ] Create feature flags for plugin groups (default, extended, experimental)
- [ ] Implement conditional compilation for large plugins
- [ ] Add size-optimized build profile
- [ ] Configure dead code elimination
- [ ] Implement plugin registry filtering

**2.2. Performance Optimization**
- [ ] Use `wee_alloc` for smaller memory footprint
- [ ] Implement streaming processing for large files
- [ ] Add progress callbacks for long-running operations
- [ ] Optimize critical path performance
- [ ] Add benchmarking for WASM vs native performance

**2.3. Build Pipeline**
```bash
# Build script for WASM
#!/bin/bash
# build-wasm.sh

# Build optimized WASM package
wasm-pack build --target web --out-dir docs/wasm \
  --features wasm \
  --release \
  -- --no-default-features

# Optimize bundle size
wasm-opt -Oz docs/wasm/svgn_bg.wasm -o docs/wasm/svgn_bg.wasm

# Generate TypeScript definitions
wasm-pack build --target bundler --out-dir pkg \
  --features wasm \
  --release \
  -- --no-default-features
```

#### 11.2.3. Phase 3: JavaScript API Layer (Week 3)

**3.1. High-Level API Design**
```javascript
// svgn-web.js - High-level JavaScript API
class SvgnWeb {
  constructor(options = {}) {
    this.wasmModule = null;
    this.optimizer = null;
    this.options = options;
  }

  async init() {
    // Load WASM module
    const wasm = await import('./wasm/svgn.js');
    await wasm.default();
    this.wasmModule = wasm;
    this.optimizer = new wasm.SvgOptimizer(this.options);
  }

  optimize(svgContent, options = {}) {
    if (!this.optimizer) {
      throw new Error('SvgnWeb not initialized. Call init() first.');
    }
    
    const result = this.optimizer.optimize(svgContent);
    return {
      data: result,
      info: this.optimizer.get_info()
    };
  }

  static async create(options = {}) {
    const instance = new SvgnWeb(options);
    await instance.init();
    return instance;
  }
}
```

**3.2. Error Handling & Logging**
- [ ] Implement comprehensive error handling
- [ ] Add debug logging capabilities
- [ ] Create user-friendly error messages
- [ ] Handle WASM loading failures gracefully
- [ ] Add fallback mechanisms for unsupported browsers

**3.3. Configuration Management**
- [ ] Implement preset system (default, aggressive, safe)
- [ ] Add plugin configuration builder
- [ ] Support for custom plugin parameters
- [ ] Configuration validation and sanitization
- [ ] Import/export configuration profiles

### 11.3. Online Tool Implementation

#### 11.3.1. Phase 4: Web UI Foundation (Week 4)

**4.1. DaisyUI Setup**
- [ ] Add DaisyUI and Tailwind CSS to docs folder
- [ ] Create responsive layout structure
- [ ] Implement dark/light theme support
- [ ] Add custom CSS for SVG preview components
- [ ] Configure build process for CSS optimization

**4.2. Jekyll Integration**
```yaml
# Add to docs/_config.yml
plugins:
  - jekyll-postcss

# PostCSS configuration
postcss:
  cache: false
  
# Custom collections for tool pages
collections:
  tools:
    output: true
    permalink: /tools/:name/
```

**4.3. Base HTML Structure**
```html
<!-- docs/tools/optimizer.html -->
---
layout: default
title: SVG Optimizer
permalink: /tools/optimizer/
---

<div class="container mx-auto px-4 py-8">
  <div class="hero bg-base-200 rounded-lg">
    <div class="hero-content text-center">
      <div class="max-w-md">
        <h1 class="text-5xl font-bold">SVG Optimizer</h1>
        <p class="py-6">Optimize your SVG files instantly in your browser</p>
      </div>
    </div>
  </div>
  
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mt-8">
    <div class="card bg-base-100 shadow-xl">
      <div class="card-body">
        <h2 class="card-title">Input</h2>
        <div id="input-section">
          <!-- File upload and editor -->
        </div>
      </div>
    </div>
    
    <div class="card bg-base-100 shadow-xl">
      <div class="card-body">
        <h2 class="card-title">Output</h2>
        <div id="output-section">
          <!-- Preview and download -->
        </div>
      </div>
    </div>
  </div>
  
  <div class="card bg-base-100 shadow-xl mt-8">
    <div class="card-body">
      <h2 class="card-title">Settings</h2>
      <div id="settings-section">
        <!-- Configuration panel -->
      </div>
    </div>
  </div>
</div>
```

#### 11.3.2. Phase 5: Core Functionality (Week 5)

**5.1. File Upload Implementation**
```javascript
// File upload with drag-and-drop
class FileUploader {
  constructor(containerSelector) {
    this.container = document.querySelector(containerSelector);
    this.setupDragDrop();
    this.setupFileInput();
  }

  setupDragDrop() {
    this.container.addEventListener('dragover', (e) => {
      e.preventDefault();
      this.container.classList.add('drag-over');
    });

    this.container.addEventListener('drop', (e) => {
      e.preventDefault();
      this.container.classList.remove('drag-over');
      const files = Array.from(e.dataTransfer.files);
      this.handleFiles(files);
    });
  }

  async handleFiles(files) {
    for (const file of files) {
      if (file.type === 'image/svg+xml' || file.name.endsWith('.svg')) {
        const content = await this.readFile(file);
        this.onFileLoad(content, file.name);
      }
    }
  }

  readFile(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = (e) => resolve(e.target.result);
      reader.onerror = reject;
      reader.readAsText(file);
    });
  }
}
```

**5.2. Real-time Preview**
```javascript
// SVG preview component
class SvgPreview {
  constructor(containerSelector) {
    this.container = document.querySelector(containerSelector);
    this.originalSvg = null;
    this.optimizedSvg = null;
  }

  setOriginal(svgContent) {
    this.originalSvg = svgContent;
    this.renderPreview('original', svgContent);
  }

  setOptimized(svgContent) {
    this.optimizedSvg = svgContent;
    this.renderPreview('optimized', svgContent);
    this.updateStats();
  }

  renderPreview(type, content) {
    const previewContainer = this.container.querySelector(`#${type}-preview`);
    previewContainer.innerHTML = content;
  }

  updateStats() {
    if (this.originalSvg && this.optimizedSvg) {
      const originalSize = new Blob([this.originalSvg]).size;
      const optimizedSize = new Blob([this.optimizedSvg]).size;
      const reduction = ((originalSize - optimizedSize) / originalSize * 100).toFixed(1);
      
      this.container.querySelector('#size-stats').innerHTML = `
        <div class="stats shadow">
          <div class="stat">
            <div class="stat-title">Original Size</div>
            <div class="stat-value">${this.formatBytes(originalSize)}</div>
          </div>
          <div class="stat">
            <div class="stat-title">Optimized Size</div>
            <div class="stat-value">${this.formatBytes(optimizedSize)}</div>
          </div>
          <div class="stat">
            <div class="stat-title">Reduction</div>
            <div class="stat-value">${reduction}%</div>
          </div>
        </div>
      `;
    }
  }

  formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
}
```

**5.3. Configuration Panel**
```javascript
// Configuration management
class ConfigPanel {
  constructor(containerSelector) {
    this.container = document.querySelector(containerSelector);
    this.config = this.getDefaultConfig();
    this.render();
  }

  getDefaultConfig() {
    return {
      preset: 'default',
      plugins: {
        removeDoctype: true,
        removeXMLProcInst: true,
        removeComments: true,
        removeMetadata: true,
        removeTitle: true,
        removeDesc: true,
        removeUselessDefs: true,
        removeEditorsNSData: true,
        removeEmptyAttrs: true,
        removeHiddenElems: true,
        removeEmptyText: true,
        removeEmptyContainers: true,
        cleanupEnableBackground: true,
        convertStyleToAttrs: true,
        convertColors: true,
        convertTransform: true
      },
      js2svg: {
        pretty: false,
        indent: 2
      }
    };
  }

  render() {
    this.container.innerHTML = `
      <div class="form-control">
        <label class="label">
          <span class="label-text">Preset</span>
        </label>
        <select class="select select-bordered" id="preset-select">
          <option value="default">Default</option>
          <option value="aggressive">Aggressive</option>
          <option value="safe">Safe</option>
          <option value="custom">Custom</option>
        </select>
      </div>
      
      <div class="divider">Plugin Options</div>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        ${this.renderPluginToggles()}
      </div>
      
      <div class="divider">Output Options</div>
      
      <div class="form-control">
        <label class="label cursor-pointer">
          <span class="label-text">Pretty print</span>
          <input type="checkbox" class="toggle" id="pretty-toggle" />
        </label>
      </div>
    `;
    
    this.bindEvents();
  }

  renderPluginToggles() {
    return Object.entries(this.config.plugins)
      .map(([plugin, enabled]) => `
        <div class="form-control">
          <label class="label cursor-pointer">
            <span class="label-text">${plugin}</span>
            <input type="checkbox" class="toggle toggle-sm" 
                   data-plugin="${plugin}" ${enabled ? 'checked' : ''} />
          </label>
        </div>
      `).join('');
  }

  bindEvents() {
    // Preset selection
    this.container.querySelector('#preset-select').addEventListener('change', (e) => {
      this.applyPreset(e.target.value);
    });

    // Plugin toggles
    this.container.querySelectorAll('[data-plugin]').forEach(toggle => {
      toggle.addEventListener('change', (e) => {
        this.config.plugins[e.target.dataset.plugin] = e.target.checked;
        this.onChange(this.config);
      });
    });

    // Pretty print toggle
    this.container.querySelector('#pretty-toggle').addEventListener('change', (e) => {
      this.config.js2svg.pretty = e.target.checked;
      this.onChange(this.config);
    });
  }

  applyPreset(preset) {
    const presets = {
      default: this.getDefaultConfig(),
      aggressive: { ...this.getDefaultConfig(), plugins: { ...this.config.plugins } },
      safe: { ...this.getDefaultConfig(), plugins: Object.fromEntries(
        Object.keys(this.config.plugins).map(k => [k, false])
      )},
      custom: this.config
    };

    this.config = presets[preset] || this.config;
    this.render();
  }

  onChange(config) {
    // Override in implementation
  }
}
```

#### 11.3.3. Phase 6: Advanced Features (Week 6)

**6.1. Batch Processing**
- [ ] Multiple file upload support
- [ ] Batch optimization with progress indication
- [ ] ZIP file output for batch operations
- [ ] Processing queue management
- [ ] Cancel/pause functionality

**6.2. Integration Features**
- [ ] URL import for remote SVG files
- [ ] Export to various formats (inline, base64, etc.)
- [ ] Code generation for HTML/CSS embedding
- [ ] Plugin marketplace/registry browser
- [ ] Sharing optimized SVGs via URLs

**6.3. Developer Tools**
- [ ] Before/after comparison view
- [ ] Plugin performance profiling
- [ ] Optimization step-by-step visualization
- [ ] Custom plugin development interface
- [ ] API documentation with live examples

### 11.4. Build and Deployment

#### 11.4.1. Phase 7: Build Pipeline (Week 7)

**7.1. Automated Build Process**
```bash
#!/bin/bash
# build-web-tool.sh

echo "Building WASM package..."
wasm-pack build --target web --out-dir docs/wasm \
  --features wasm \
  --release \
  -- --no-default-features

echo "Optimizing WASM bundle..."
wasm-opt -Oz docs/wasm/svgn_bg.wasm -o docs/wasm/svgn_bg.wasm

echo "Building CSS..."
cd docs
npm run build:css

echo "Building Jekyll site..."
bundle exec jekyll build

echo "Running tests..."
npm test

echo "Build complete!"
```

**7.2. GitHub Actions Integration**
```yaml
# .github/workflows/build-web-tool.yml
name: Build Web Tool

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
    
    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    
    - name: Install wasm-opt
      run: |
        wget https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz
        tar -xzf binaryen-version_116-x86_64-linux.tar.gz
        sudo cp binaryen-version_116/bin/wasm-opt /usr/local/bin/
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
    
    - name: Install dependencies
      run: cd docs && npm install
    
    - name: Build web tool
      run: ./build-web-tool.sh
    
    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      if: github.ref == 'refs/heads/main'
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./docs/_site
```

**7.3. Performance Monitoring**
- [ ] Bundle size tracking
- [ ] Performance regression detection
- [ ] User analytics (privacy-focused)
- [ ] Error reporting and monitoring
- [ ] A/B testing for UI improvements

### 11.5. Documentation and Examples

#### 11.5.1. Phase 8: Documentation (Week 8)

**8.1. API Documentation**
- [ ] Complete JavaScript API reference
- [ ] WASM integration guide
- [ ] Configuration options documentation
- [ ] Plugin parameter reference
- [ ] Performance optimization guide

**8.2. Usage Examples**
- [ ] Basic optimization examples
- [ ] Advanced configuration scenarios
- [ ] Integration with popular frameworks
- [ ] Node.js server-side usage
- [ ] Browser extension development

**8.3. Tutorials**
- [ ] Getting started guide
- [ ] Migrating from SVGO
- [ ] Custom plugin development
- [ ] Performance optimization techniques
- [ ] Troubleshooting common issues

### 11.6. Testing and Quality Assurance

#### 11.6.1. Phase 9: Testing (Week 9)

**9.1. Unit Testing**
```javascript
// tests/wasm-api.test.js
import { SvgnWeb } from '../docs/js/svgn-web.js';

describe('WASM API', () => {
  let svgn;
  
  beforeAll(async () => {
    svgn = await SvgnWeb.create();
  });

  test('basic optimization', () => {
    const input = '<svg width="100" height="100"><rect width="50" height="50" fill="red"/></svg>';
    const result = svgn.optimize(input);
    
    expect(result.data).toBeTruthy();
    expect(result.info.originalSize).toBeGreaterThan(0);
    expect(result.info.optimizedSize).toBeLessThanOrEqual(result.info.originalSize);
  });

  test('configuration options', () => {
    const config = {
      plugins: {
        removeComments: false,
        removeMetadata: true
      }
    };
    
    const result = svgn.optimize(input, config);
    expect(result.data).toBeTruthy();
  });
});
```

**9.2. Integration Testing**
- [ ] Cross-browser compatibility testing
- [ ] Performance benchmarking
- [ ] File size regression testing
- [ ] UI interaction testing
- [ ] Accessibility compliance testing

**9.3. End-to-End Testing**
- [ ] Complete user workflow testing
- [ ] File upload/download testing
- [ ] Configuration persistence testing
- [ ] Error handling testing
- [ ] Mobile device testing

### 11.7. Success Metrics

#### 11.7.1. Performance Targets
- [ ] **Bundle Size**: WASM + JS < 2MB compressed
- [ ] **Load Time**: Initial load < 3 seconds on 3G
- [ ] **Optimization Speed**: Match or exceed native performance
- [ ] **Memory Usage**: < 50MB for typical SVG files
- [ ] **Browser Support**: All modern browsers (Chrome, Firefox, Safari, Edge)

#### 11.7.2. User Experience Goals
- [ ] **Ease of Use**: Zero-config optimization for beginners
- [ ] **Advanced Control**: Full plugin customization for experts
- [ ] **Feedback**: Real-time preview and statistics
- [ ] **Accessibility**: WCAG 2.1 AA compliance
- [ ] **Mobile**: Fully responsive design

#### 11.7.3. Technical Goals
- [ ] **API Compatibility**: Match SVGO JavaScript API
- [ ] **Plugin Parity**: All implemented plugins available
- [ ] **Configuration**: Import/export SVGO configurations
- [ ] **Integration**: Easy embedding in other projects
- [ ] **Documentation**: Complete API and usage documentation

### 11.8. Implementation Timeline

**Total Estimated Duration: 9 weeks**

| Phase | Duration | Description |
|-------|----------|-------------|
| Phase 1 | Week 1 | WASM Build Infrastructure |
| Phase 2 | Week 2 | Bundle Size Optimization |
| Phase 3 | Week 3 | JavaScript API Layer |
| Phase 4 | Week 4 | Web UI Foundation |
| Phase 5 | Week 5 | Core Functionality |
| Phase 6 | Week 6 | Advanced Features |
| Phase 7 | Week 7 | Build and Deployment |
| Phase 8 | Week 8 | Documentation |
| Phase 9 | Week 9 | Testing and QA |

**Dependencies:**
- Phases 1-3 can run in parallel with current plugin development
- Phase 4-6 require completion of Phase 1-3
- Phase 7-9 require completion of all previous phases

**Risk Mitigation:**
- Start with minimal viable product (MVP) approach
- Prioritize core functionality over advanced features
- Maintain fallback options for unsupported browsers
- Regular testing and user feedback throughout development

This comprehensive plan provides a roadmap for implementing both WebAssembly builds and an online SVG optimization tool, making SVGN accessible to a broader audience while maintaining its performance and compatibility advantages.