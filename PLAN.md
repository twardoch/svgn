# SVGN Development Plan: Path to 100% SVGO Parity

## Executive Summary

SVGN is a high-performance Rust port of SVGO (SVG Optimizer) that has achieved 83% plugin implementation and 100% test coverage for implemented features. This document outlines the comprehensive plan to achieve complete feature parity and API compatibility with SVGO v4.0.0.

**Current Status:**
- ✅ 46/53 plugins implemented (87% coverage)
- ✅ 359 tests passing (100% success rate)
- ✅ Core infrastructure operational (parser, AST, stringifier, plugin system)
- ⚠️ 7 complex plugins remaining
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

### 1.1 Remaining Plugin Implementations (Priority: CRITICAL)

These 7 plugins represent the most complex optimizations and are essential for SVGO compatibility:

#### 1.1.1 Path Optimization Plugins
1. **convertPathData** (STUB EXISTS)
   - Current: Stub implementation returns "not implemented" error
   - Required: Full path optimization using lyon geometry library
   - Features: 
     - Path simplification (remove redundant commands)
     - Precision reduction (round coordinates to specified decimals)
     - Relative/absolute conversion based on size optimization
     - Arc optimization (convert arcs to curves when smaller)
     - Line optimization (horizontal/vertical detection)
     - Curve optimization (smooth curve detection)
   - Parameters:
     - `floatPrecision`: Number of decimal places (default: 3)
     - `transformPrecision`: Transform matrix precision (default: 5)
     - `removeUseless`: Remove useless path segments (default: true)
     - `collapseRepeated`: Collapse repeated commands (default: true)
     - `utilizeAbsolute`: Use absolute commands when smaller (default: true)
     - `leadingZero`: Keep leading zeros (default: true)
     - `negativeExtraSpace`: Use negative values to reduce size (default: true)
   - Implementation approach:
     - Use lyon_path for path parsing and building
     - Use lyon_geom for geometric operations
     - Implement command optimization algorithms
   - Complexity: HIGH - Requires geometric algorithms

2. **mergePaths**
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

3. **reusePaths**
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

5. **removeUselessTransforms** (PARTIAL IMPLEMENTATION)
   - Plugin exists but needs completion
   - Features to add:
     - Remove translate(0) or translate(0,0)
     - Remove scale(1) or scale(1,1)
     - Remove rotate(0)
     - Remove skewX(0) and skewY(0)
     - Remove identity matrices
   - Current: Basic structure exists
   - Required: Pattern matching for all identity transforms
   - Complexity: LOW

#### 1.1.3 Style Processing
6. **inlineStyles**
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
7. **moveElemsAttrsToGroup**
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

8. **moveGroupAttrsToElems**
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

### 1.1.5 Missing SVGO Default Preset Plugin
**removeUselessStrokeAndFill**
   - Not implemented in svgn (missing from both plugin list and registry)
   - Part of SVGO's default preset (position 16)
   - Features:
     - Remove stroke="none" from elements that don't inherit stroke
     - Remove fill="none" from elements that don't inherit fill
     - Remove stroke-width="0" (equivalent to stroke="none")
     - Remove stroke when fill is sufficient
     - Remove fill="black" when it's the default
     - Handle currentColor correctly
   - Parameters:
     - `stroke`: Remove useless stroke (default: true)
     - `fill`: Remove useless fill (default: true)
     - `removeNone`: Remove "none" values (default: true)
   - Implementation approach:
     - Understand SVG rendering model
     - Track inherited values through DOM tree
     - Identify redundant/default values
   - Complexity: MEDIUM

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

- **cleanupEnableBackground Style Handling** (Issue #225)
  - Parse enable-background from style attributes
  - Unify with attribute handling

- **cleanupIds URL Encoding** (Issue #227)
  - Match SVGO's encodeURI behavior
  - Handle special characters in IDs

- **cleanupIds Optimization Skip** (Issue #228)
  - Skip optimization for SVGs with only defs
  - Performance optimization

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
1. Implement convertPathData with lyon
2. Complete transform optimization plugins
3. Implement style processing plugins
4. Complete structural optimization plugins

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
- [ ] All 53 SVGO plugins implemented (currently 46/53 = 87%)
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

SVGN has made excellent progress with 87% plugin coverage (46/53 plugins) and perfect test results. The remaining work is well-defined and achievable. By following this plan, we will deliver a faster, more reliable SVG optimizer that maintains full compatibility with SVGO while offering significant performance improvements and a foundation for future innovation.

The path to 100% parity is clear, with concrete specifications for each remaining feature. With focused execution on the critical path items, SVGN will become the definitive SVG optimization solution for the Rust ecosystem and beyond.