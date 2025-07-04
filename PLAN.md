# SVGN Development Plan: Path to 100% SVGO Parity

## Executive Summary

SVGN is a high-performance Rust port of SVGO (SVG Optimizer) that has achieved 83% plugin implementation and 100% test coverage for implemented features. This document outlines the comprehensive plan to achieve complete feature parity and API compatibility with SVGO v4.0.0.

**Current Status:**
- ✅ 43/54 plugins fully functional (80% coverage)
- ⚠️ 1 plugin implemented as stub (convertPathData - will error when used)
- ⚠️ 1 plugin implemented but disabled (removeAttributesBySelector)
- ❌ 10 SVGO plugins completely missing
- ✅ 2 SVGN-exclusive plugins added (removeUselessTransforms, removeStyleElement)
- ✅ 359 tests passing (100% success rate for implemented features)
- ✅ Core infrastructure operational (parser, AST, stringifier, plugin system)
- ⚠️ Several architectural enhancements needed

## 0. CLI Compatibility Enhancement (Priority: IMMEDIATE) ✅ COMPLETED

### 0.1 SVGO CLI Feature Parity ✅

The CLI is the primary interface for most users and must achieve full compatibility with SVGO's command-line interface. This is a prerequisite for adoption and should be prioritized above other features.

#### 0.1.1 Current CLI Gaps (ALL RESOLVED ✅)

**Previously Missing Features (Now Implemented):**
1. **STDIN/STDOUT Support** (`-i -` / `-o -`)
   - Default behavior: No args → read from STDIN, write to STDOUT
   - Explicit: `-i -` for STDIN, `-o -` for STDOUT
   - Essential for pipeline integration

2. **String Input** (`-s, --string`)
   - Direct SVG string optimization without files
   - Useful for build tool integrations

3. **Precision Control** (`-p, --precision`)
   - Override decimal precision for all plugins
   - Critical for file size optimization

4. **Plugin Discovery** (`--show-plugins`)
   - List all available plugins with descriptions
   - Help users understand optimization options

5. **Output Formatting**
   - `--indent <INTEGER>`: Custom indentation (not just on/off)
   - `--eol <lf|crlf>`: Line ending control
   - `--final-newline`: Ensure trailing newline

6. **Folder Processing**
   - `-r, --recursive`: Process subdirectories
   - `--exclude <PATTERN>`: Regex exclusion patterns

7. **Status Control**
   - `--no-color`: Disable colored output
   - Positional arguments support

#### 0.1.2 CLI Architecture Specification

```rust
// Enhanced CLI argument structure
struct CliArgs {
    // Input sources (mutually exclusive)
    input_files: Vec<String>,      // Positional args or -i
    input_string: Option<String>,  // -s, --string
    input_folder: Option<String>,  // -f, --folder
    use_stdin: bool,              // Default or -i -
    
    // Output targets
    output: Option<String>,        // -o (file/folder/-)
    use_stdout: bool,             // Default with stdin or -o -
    
    // Processing options
    precision: Option<u8>,         // -p, --precision
    config: Option<PathBuf>,       // --config
    multipass: bool,              // --multipass
    
    // Plugin control
    enable: Vec<String>,          // --enable (multiple)
    disable: Vec<String>,         // --disable (multiple)
    show_plugins: bool,           // --show-plugins
    
    // Output formatting
    pretty: bool,                 // --pretty
    indent: Option<usize>,        // --indent
    eol: Option<LineEnding>,      // --eol
    final_newline: bool,          // --final-newline
    datauri: Option<DataUriFormat>, // --datauri
    
    // Folder options
    recursive: bool,              // -r, --recursive
    exclude: Vec<String>,         // --exclude patterns
    
    // Status options
    quiet: bool,                  // -q, --quiet
    no_color: bool,              // --no-color
}
```

#### 0.1.3 Input/Output Logic Flow

```
1. Parse arguments
2. Determine input source:
   - If --string → use string
   - If --folder → use folder mode
   - If positional args or --input → use files
   - If no input specified → use STDIN

3. Determine output target:
   - If --output specified:
     - "-" → STDOUT
     - File path → file
     - Directory → directory mode
   - If input is STDIN and no --output → STDOUT
   - Otherwise → overwrite input files

4. Apply precision override:
   - If --precision specified, inject into all numeric plugins
   - Override plugin-specific precision settings
```

#### 0.1.4 Implementation Strategy

**Phase 1: Core I/O ✅ COMPLETED**
- Implement STDIN/STDOUT handling
- Add string input support
- Fix default behavior (no args → stdin/stdout)

**Phase 2: Feature Completion ✅ COMPLETED**
- Add precision control
- Implement --show-plugins
- Add formatting options (indent, eol, final-newline)

**Phase 3: Advanced Features ✅ COMPLETED**
- Recursive folder processing
- Exclusion patterns
- Positional argument support
- Color control

## 1. Critical Path to Feature Parity

### 1.0 Plugin Compatibility Summary

**Comprehensive Plugin Analysis Results:**
- **Total SVGO Plugins**: 54
- **Fully Functional in SVGN**: 46 (85%) ✅
- **Disabled Implementation**: 1 (removeAttributesBySelector - CSS selector issues)
- **Missing Implementations**: 7
- **SVGN-Exclusive Additions**: 2 (removeUselessTransforms, removeStyleElement)

**Missing Plugins by Category:**
1. **Path Operations**: mergePaths, reusePaths, applyTransforms
2. **Transform Operations**: convertTransform
3. **Style Operations**: inlineStyles
4. **Structural Operations**: moveElemsAttrsToGroup, moveGroupAttrsToElems
5. **Selector Operations**: removeAttributesBySelector (disabled)

**SVGN-Exclusive Enhancements:**
1. **removeUselessTransforms**: Removes identity transforms (translate(0), scale(1), rotate(0), etc.)
2. **removeStyleElement**: Removes <style> elements entirely
3. **More Conservative Defaults**: 21 plugins in default preset vs SVGO's 33

### 1.1 Remaining Plugin Implementations (Priority: CRITICAL)

Based on comprehensive analysis, SVGN needs to implement/fix 8 plugins to achieve full SVGO compatibility:

**Critical Issues:**
1. **removeAttributesBySelector** - Implemented but disabled due to CSS selector issues

**Recently Fixed:**
- **convertPathData** - Was a stub, now has basic implementation with:
  - Path command parsing and optimization
  - Absolute/relative coordinate conversion  
  - Redundant command removal
  - Number precision control
  - Leading zero removal option
  - May benefit from lyon geometry integration for advanced features like arc optimization

**Recently Implemented:**
- **convertOneStopGradients** ✅ - Converts single-stop gradients to solid colors
- **removeUselessStrokeAndFill** ✅ - Removes redundant stroke/fill attributes

**Missing Plugins (7 total):**

#### 1.1.1 Path Optimization Plugins

1. **mergePaths**
   - Combines multiple path elements with identical styles
   - Features:
     - Merge adjacent <path> elements with identical attributes
     - Combine path data strings
     - Preserve transform and style attributes
     - Skip if paths have different stroke/fill/opacity
   - Parameters:
     - `force`: Force merging even with different attributes (default: false)
     - `floatPrecision`: Coordinate precision (default: 3)
     - `noSpaceAfterFlags`: Remove space after arc flags (default: false)
   - Implementation approach:
     - Group paths by identical attributes
     - Check adjacency in DOM
     - Concatenate path data with proper spacing
   - Complexity: MEDIUM

2. **reusePaths**
   - Identifies duplicate paths and replaces with <use> elements
   - Features:
     - Hash path data + attributes to find duplicates
     - Create <defs> section if needed
     - Move first occurrence to <defs> with ID
     - Replace duplicates with <use> references
     - Calculate size reduction benefit
   - Implementation approach:
     - Create content hash from path data + relevant attributes
     - Build HashMap of path signatures
     - Only deduplicate if size reduction > threshold
     - Handle transforms correctly
   - Complexity: MEDIUM

3. **applyTransforms** (MISSING)
   - Not implemented in SVGN
   - Applies transform attribute to path coordinates
   - Features:
     - Parse transform matrices from elements
     - Apply transforms directly to path coordinates
     - Apply transforms to other shape coordinates
     - Remove transform attribute after application
     - Handle nested transforms correctly
   - Parameters:
     - `applyToPath`: Apply to path data (default: true)
     - `applyToShapes`: Apply to basic shapes (default: true)
     - `floatPrecision`: Coordinate precision (default: 3)
   - Implementation approach:
     - Parse transforms into matrices
     - Transform each path command's coordinates
     - Transform shape attributes (x, y, width, height, etc.)
     - Remove transform attribute when done
   - Complexity: HIGH

#### 1.1.2 Transform Optimization
4. **convertTransform**
   - Optimizes transform matrices, converts to shorter forms
   - Features:
     - Multiply consecutive transforms into single matrix
     - Convert matrix to shorter translate/scale/rotate when possible
     - Remove identity transforms
     - Optimize transform precision
     - Collapse transform sequences
   - Parameters:
     - `convertToShorts`: Convert to short transforms (default: true)
     - `floatPrecision`: Number precision (default: 3)
     - `transformPrecision`: Transform precision (default: 5)
     - `matrixToTransform`: Convert matrices to transforms (default: true)
     - `shortTranslate`: Use short translate syntax (default: true)
     - `shortScale`: Use short scale syntax (default: true)
     - `shortRotate`: Use short rotate syntax (default: true)
     - `removeUseless`: Remove identity transforms (default: true)
     - `collapseIntoOne`: Merge into one transform (default: true)
     - `leadingZero`: Keep leading zeros (default: true)
     - `negativeExtraSpace`: Use negative values efficiently (default: true)
   - Implementation approach:
     - Parse transform string into transform list
     - Implement matrix multiplication
     - Implement matrix decomposition algorithms
     - Optimize output format
   - Complexity: HIGH

#### 1.1.3 Style Processing
5. **inlineStyles**
   - Moves styles from <style> elements to inline attributes
   - Features:
     - Parse CSS from <style> elements
     - Calculate CSS specificity for each rule
     - Match selectors to elements
     - Apply styles as inline attributes
     - Handle cascade and inheritance
     - Remove empty <style> elements
   - Parameters:
     - `onlyMatchedOnce`: Only inline if selector matches once (default: true)
     - `removeMatchedSelectors`: Remove inlined selectors (default: true)
     - `useMqs`: Process media queries (default: ['', 'screen'])
     - `usePseudos`: Process pseudo classes (default: [':hover'])
   - Implementation approach:
     - Use CSS parser (css_parser crate or similar)
     - Implement specificity calculation
     - Build style cascade for each element
     - Convert CSS properties to SVG attributes
   - Complexity: HIGH - Requires full CSS engine

#### 1.1.4 Structural Optimization
6. **moveElemsAttrsToGroup**
   - Moves common attributes from elements to parent group
   - Features:
     - Analyze attributes across sibling elements
     - Find common inheritable attributes
     - Move to parent <g> when size reduction occurs
     - Handle attribute inheritance correctly
     - Create <g> wrapper if needed
   - Implementation approach:
     - Build attribute frequency map for siblings
     - Check which attributes are inheritable
     - Calculate size reduction (common attrs * elements > overhead)
     - Move attributes and remove from children
   - Inheritable attributes: fill, stroke, opacity, font-*, etc.
   - Complexity: MEDIUM

7. **moveGroupAttrsToElems**
   - Distributes group attributes to children when beneficial
   - Features:
     - Distribute group attributes to all children
     - Remove empty groups after distribution
     - Only distribute if group has no other purpose
     - Calculate if distribution reduces size
   - Implementation approach:
     - Check if group only provides attributes (no transform, etc.)
     - Copy inheritable attributes to each child
     - Handle attribute conflicts (child overrides)
     - Remove group if empty after distribution
   - Complexity: MEDIUM

#### 1.1.5 Other Missing Plugins

8. **removeAttributesBySelector** (IMPLEMENTED BUT DISABLED)
   - Currently commented out in plugin registry
   - Issue: CSS selector parsing not working
   - Features:
     - Remove attributes matching CSS selectors
     - Support complex selectors (class, id, element, attribute)
     - Multiple selector support
   - Parameters:
     - `selectors`: Array of objects with selector and attributes
   - Fix needed:
     - Implement proper CSS selector parsing (use selectors crate)
     - Re-enable in plugin registry
   - Complexity: MEDIUM

7. **applyTransforms** (MISSING)
   - Not implemented in SVGN
   - Applies transform attribute to path coordinates
   - Features:
     - Parse transform matrices from elements
     - Apply transforms directly to path coordinates
     - Apply transforms to other shape coordinates
     - Remove transform attribute after application
     - Handle nested transforms correctly
   - Parameters:
     - `applyToPath`: Apply to path data (default: true)
     - `applyToShapes`: Apply to basic shapes (default: true)
     - `floatPrecision`: Coordinate precision (default: 3)
   - Implementation approach:
     - Parse transforms into matrices
     - Transform each path command's coordinates
     - Transform shape attributes (x, y, width, height, etc.)
     - Remove transform attribute when done
   - Complexity: HIGH

### 1.2 Infrastructure Enhancements (Priority: HIGH)

#### 1.2.1 Parser Improvements
- **XML Entity Expansion** (Issue #201)
  - Support custom entities in DOCTYPE declarations
  - Parse and expand entity references
  - Implementation: Enhance parser with entity table

- **Selective Whitespace Preservation** (Issue #202)
  - Preserve whitespace in <text>, <tspan>, <pre>, <script>, <style>
  - Trim whitespace in other elements
  - Implementation: Context-aware whitespace handling

- **Enhanced Error Reporting** (Issue #203)
  - Add line/column information to errors
  - Provide context snippets
  - Implementation: Track positions during parsing

#### 1.2.2 Stringifier Enhancements
- **XML Declaration Output** (Issue #206)
  - Support <?xml version="1.0" encoding="UTF-8"?>
  - Configurable encoding
  - Implementation: Add to stringifier preamble

- **DOCTYPE Output** (Issue #207)
  - Preserve and output DOCTYPE declarations
  - Handle entity definitions
  - Implementation: Store DOCTYPE in AST

#### 1.2.3 Architecture Improvements
- **Visitor Pattern** (Issue #213)
  - Implement proper visitor pattern for granular traversal
  - Support enter/exit callbacks
  - Enable better plugin control flow

- **Preset System** (Issue #215)
  - Implement preset-default, preset-next concepts
  - Allow custom preset definitions
  - Configuration inheritance

- **Dynamic Plugin Loading** (Issue #216)
  - Support runtime plugin loading
  - Plugin discovery mechanism
  - External plugin API

### 1.3 Plugin-Specific Enhancements (Priority: MEDIUM)

#### 1.3.1 Partial Compatibility Issues

- **cleanupEnableBackground Style Handling** (Issue #225)
  - Current: Only handles enable-background as attribute
  - Missing: Parse enable-background from style attributes
  - Implementation needed:
    - CSS parser to extract enable-background from style
    - Merge logic with attribute handling
  - Example case not handled: `style="enable-background: new 0 0 100 50"`

- **cleanupIds Full Compatibility**
  - Current implementation appears fully compatible
  - Includes URL encoding, reference tracking, minification
  - No known compatibility issues

- **convertPathData Full Optimization** (Recently implemented)
  - Basic implementation now exists (no longer a stub)
  - May need lyon geometry integration for advanced features:
    - Arc to curve conversion
    - Advanced path simplification
    - Geometric shape detection
  - Current features: command optimization, relative/absolute conversion

### 1.4 Build and Distribution (Priority: MEDIUM)

- **Cross-Platform Build Scripts** (Issue #410)
  - Complete macOS universal binary build
  - Linux packaging (.deb, .rpm, .AppImage)
  - Windows installer (.msi)
  - Automated release pipeline

- **Version Management**
  - Git tag-based versioning
  - Automatic version injection
  - Changelog automation

### 1.5 Quality and Performance (Priority: LOW)

- **Code Quality**
  - Fix 27 Clippy warnings
  - Implement missing Default traits
  - Fix regex with backreference issue
  - Clean up recursive parameters

- **Performance Optimization**
  - Benchmark against SVGO
  - Optimize hot paths
  - Parallel processing support

- **Test Coverage**
  - Port remaining SVGO test fixtures
  - Fuzz testing for parser
  - Performance regression tests

## 2. Implementation Strategy

### Phase 1: Complete Plugin Parity (4-6 weeks)
1. Complete transform optimization plugins (convertTransform, applyTransforms)
2. Implement style processing plugins (inlineStyles, removeUselessStrokeAndFill, convertOneStopGradients)
3. Complete structural optimization plugins (moveElemsAttrsToGroup, moveGroupAttrsToElems)
4. Implement path plugins (mergePaths, reusePaths)
5. Fix removeAttributesBySelector CSS parsing

### Phase 2: Infrastructure Enhancement (2-3 weeks)
1. Parser improvements (entities, whitespace, errors)
2. Stringifier enhancements (XML decl, DOCTYPE)
3. Visitor pattern implementation
4. Preset system

### Phase 3: Polish and Release (2-3 weeks)
1. Fix all code quality issues
2. Complete build/distribution scripts
3. Performance optimization
4. Documentation and examples

### Phase 4: Advanced Features (4-6 weeks)
1. Dynamic plugin loading
2. WASM compilation
3. Advanced error recovery
4. Plugin marketplace

## 3. Technical Specifications

### 3.1 Plugin API Enhancements

```rust
// Enhanced visitor pattern
trait Visitor {
    fn enter_element(&mut self, element: &mut Element, ancestors: &[&Element]) -> VisitResult;
    fn exit_element(&mut self, element: &mut Element, ancestors: &[&Element]) -> VisitResult;
    fn visit_text(&mut self, text: &mut Text, ancestors: &[&Element]) -> VisitResult;
    // ... other node types
}

enum VisitResult {
    Continue,
    Skip,
    Remove,
    Replace(Node),
}
```

### 3.2 Preset System

```rust
trait Preset {
    fn name(&self) -> &'static str;
    fn plugins(&self) -> Vec<PluginConfig>;
    fn extends(&self) -> Option<&'static str>;
}

struct PresetDefault;
impl Preset for PresetDefault {
    fn name(&self) -> &'static str { "preset-default" }
    fn plugins(&self) -> Vec<PluginConfig> {
        // Return default plugin list
    }
}
```

### 3.3 Entity Expansion

```rust
struct EntityTable {
    entities: HashMap<String, String>,
}

impl Parser {
    fn parse_doctype(&mut self) -> Result<EntityTable> {
        // Parse <!ENTITY> declarations
    }
    
    fn expand_entity(&self, name: &str) -> Option<&str> {
        self.entities.get(name)
    }
}
```

## 4. Success Metrics

### 4.1 Functional Parity
- [ ] All 54 SVGO plugins implemented (currently 46/54 = 85% fully functional)
- [x] Fix convertPathData stub implementation ✅ (basic implementation complete)
- [x] Implement convertOneStopGradients ✅ (completed)
- [x] Implement removeUselessStrokeAndFill ✅ (completed)
- [ ] Fix and re-enable removeAttributesBySelector
- [ ] Implement 7 missing plugins
- [ ] Enhance convertPathData with lyon for advanced features
- [ ] 100% SVGO test suite passing
- [ ] Identical optimization results for test corpus

### 4.2 Performance
- [ ] 2-10x faster than SVGO on benchmark suite
- [ ] Linear scaling with file size
- [ ] Sub-100ms for typical web SVGs

### 4.3 Compatibility
- [x] Drop-in replacement for SVGO CLI ✅
- [ ] Config file compatibility (partial - .json/.toml supported, .js pending)
- [ ] API compatibility for Node.js bindings

### 4.4 Quality
- [ ] Zero Clippy warnings
- [ ] 95%+ test coverage
- [ ] Comprehensive documentation

## 5. Risk Mitigation

### 5.1 Technical Risks
- **Path optimization complexity**: Leverage lyon's proven algorithms
- **CSS parsing complexity**: Use existing CSS parser crates
- **SVGO compatibility**: Extensive testing against SVGO output

### 5.2 Resource Risks
- **Development time**: Prioritize high-impact plugins
- **Maintenance burden**: Automated testing and CI/CD
- **Community adoption**: Clear migration guides

## 6. Long-Term Vision

### 6.1 Beyond SVGO Parity
- GPU-accelerated path optimization
- Machine learning-based optimization hints
- Real-time optimization API
- Visual optimization preview

### 6.2 Ecosystem Integration
- Native IDE plugins
- Build tool integrations
- CDN optimization services
- Design tool plugins

### 6.3 Standards Leadership
- Contribute optimizations back to SVGO
- Propose SVG specification improvements
- Reference implementation for new features

## 7. Conclusion

SVGN has made excellent progress with 85% fully functional plugin coverage (46/54 plugins working correctly) and perfect test results for implemented features. The remaining work is well-defined and achievable:
- ✅ convertPathData has been fixed (basic implementation complete)
- ✅ convertOneStopGradients has been implemented (completed)
- ✅ removeUselessStrokeAndFill has been implemented (completed)
- Fix 1 disabled plugin (removeAttributesBySelector)
- Implement 7 missing plugins
- Enhance convertPathData with lyon for advanced geometric optimizations

By following this plan, we will deliver a faster, more reliable SVG optimizer that maintains full compatibility with SVGO while offering significant performance improvements and a foundation for future innovation.

The path to 100% parity is clear, with concrete specifications for each remaining feature. With focused execution on the critical path items, SVGN will become the definitive SVG optimization solution for the Rust ecosystem and beyond.