# SVGN TODO List

## Completed - CLI Compatibility (SVGO Feature Parity) ✅

### Core I/O Features ✅
- [✓] Implement STDIN support (`-i -` or no input args)
  - [✓] Detect when no input file is specified
  - [✓] Read SVG content from stdin
  - [✓] Handle piped input correctly
- [✓] Implement STDOUT support (`-o -` or default with stdin)
  - [✓] Detect `-o -` argument
  - [✓] Default to stdout when input is stdin and no output specified
  - [✓] Ensure no status messages pollute stdout
- [✓] Add string input support (`-s, --string <STRING>`)
  - [✓] Add new CLI argument for direct SVG string
  - [✓] Process string without file I/O
  - [✓] Handle proper escaping
- [✓] Support positional arguments for input files
  - [✓] Allow `svgn file.svg` instead of requiring `-i`
  - [✓] Support multiple input files
  - [✓] Match SVGO's argument parsing behavior

### Essential Features ✅
- [✓] Add precision control (`-p, --precision <INTEGER>`)
  - [✓] Add CLI argument for decimal precision
  - [✓] Override all plugin precision settings
  - [✓] Apply to all numeric value optimizations
- [✓] Implement plugin listing (`--show-plugins`)
  - [✓] List all available plugins
  - [✓] Show plugin descriptions
  - [✓] Exit after displaying
- [✓] Add output formatting options
  - [✓] `--indent <INTEGER>`: Control indentation spaces
  - [✓] `--eol <lf|crlf>`: Line ending control
  - [✓] `--final-newline`: Ensure trailing newline

### Folder Processing ✅
- [✓] Add recursive folder processing (`-r, --recursive`)
  - [✓] Walk directory tree recursively
  - [✓] Process all SVG files in subdirectories
  - [✓] Maintain relative path structure
- [✓] Add exclusion patterns (`--exclude <PATTERN...>`)
  - [✓] Support regex patterns for file exclusion
  - [✓] Allow multiple exclude patterns
  - [✓] Apply to folder processing mode

### Status and Output Control ✅
- [✓] Add color control (`--no-color`)
  - [✓] Disable ANSI color codes in output
  - [✓] Detect terminal capabilities
  - [✓] Respect NO_COLOR environment variable
- [✓] Improve quiet mode to match SVGO behavior
  - [✓] Suppress all non-error output
  - [✓] Ensure compatibility with unix pipes

### CLI Architecture Refactor ✅
- [✓] Refactor argument parsing for mutual exclusivity
  - [✓] Enforce string vs file vs folder input modes
  - [✓] Validate argument combinations
  - [✓] Provide clear error messages
- [✓] Implement proper default behavior
  - [✓] No args = read stdin, write stdout
  - [✓] Match SVGO's implicit behaviors
  - [✓] Document all defaults clearly

### Remaining CLI Work
- [ ] Update all writeln! calls in stringifier to use write_newline method
- [ ] Add support for .js config files (currently only .json and .toml)
- [ ] Implement base64 encoding for datauri output (currently placeholder)

## Top Priority - Version Management
- [ ] Make the build app use git-tag-based semver, not `0.1.0`

## Critical - Remaining Plugin Implementations (9 plugins)

### Path Optimization Plugins
- [ ] Implement full **convertPathData** plugin with lyon geometry library (stub exists)
  - [ ] Path simplification algorithms
  - [ ] Precision reduction
  - [ ] Relative/absolute conversion
  - [ ] Arc optimization
- [ ] Implement **mergePaths** plugin
  - [ ] Identify paths with identical styles
  - [ ] Combine path data
  - [ ] Handle transforms correctly
- [ ] Implement **reusePaths** plugin
  - [ ] Content hashing for path deduplication
  - [ ] Create <defs> and <use> elements
  - [ ] Reference management

### Transform Optimization
- [ ] Implement **convertTransform** plugin
  - [ ] Matrix multiplication and optimization
  - [ ] Convert matrices to shorter transform functions
  - [ ] Combine multiple transforms
- [ ] Complete **removeUselessTransforms** plugin implementation
  - [ ] Detect identity transforms (translate(0,0), scale(1,1), etc.)
  - [ ] Remove from elements
  - [ ] Update tests

### Style Processing
- [ ] Implement **inlineStyles** plugin
  - [ ] Parse CSS from <style> elements
  - [ ] Calculate CSS specificity
  - [ ] Apply styles as inline attributes
  - [ ] Remove empty <style> elements

### Structural Optimization
- [ ] Implement **moveElemsAttrsToGroup** plugin
  - [ ] Analyze common attributes across elements
  - [ ] Move to parent group when beneficial
  - [ ] Calculate size reduction
- [ ] Implement **moveGroupAttrsToElems** plugin
  - [ ] Distribute group attributes to children
  - [ ] Remove unnecessary groups
  - [ ] Handle inheritance correctly
- [ ] Implement **removeUselessStrokeAndFill** plugin
  - [ ] Understand style cascade and inheritance
  - [ ] Remove redundant stroke/fill attributes
  - [ ] Handle currentColor correctly

## High Priority - Infrastructure Enhancements

### Parser Improvements
- [ ] Implement XML entity expansion (Issue #201)
  - [ ] Parse <!ENTITY> declarations in DOCTYPE
  - [ ] Build entity table
  - [ ] Expand &entity; references
- [ ] Implement selective whitespace preservation (Issue #202)
  - [ ] Preserve in <text>, <tspan>, <pre>, <script>, <style>
  - [ ] Trim in other elements
  - [ ] Add configuration option
- [ ] Add enhanced error reporting (Issue #203)
  - [ ] Track line/column during parsing
  - [ ] Provide context snippets
  - [ ] Improve error messages

### Stringifier Enhancements
- [ ] Add XML declaration output support (Issue #206)
  - [ ] Output <?xml version="1.0" encoding="UTF-8"?>
  - [ ] Make encoding configurable
  - [ ] Conditional output based on config
- [ ] Add DOCTYPE output support (Issue #207)
  - [ ] Preserve DOCTYPE from input
  - [ ] Output entity definitions
  - [ ] Handle public/system identifiers

### Architecture Improvements
- [ ] Implement visitor pattern (Issue #213)
  - [ ] Create Visitor trait with enter/exit methods
  - [ ] Support for different node types
  - [ ] Enable fine-grained traversal control
- [ ] Implement preset system (Issue #215)
  - [ ] Create Preset trait
  - [ ] Implement preset-default
  - [ ] Support preset inheritance
  - [ ] Allow custom presets
- [ ] Add dynamic plugin loading support (Issue #216)
  - [ ] Plugin discovery mechanism
  - [ ] Runtime loading API
  - [ ] External plugin interface

## Medium Priority - Plugin Enhancements

- [ ] Fix cleanupEnableBackground style handling (Issue #225)
  - [ ] Parse enable-background from style attributes
  - [ ] Merge with attribute handling
- [ ] Fix cleanupIds URL encoding (Issue #227)
  - [ ] Match SVGO's encodeURI behavior
  - [ ] Handle special characters correctly
- [ ] Add cleanupIds optimization skip (Issue #228)
  - [ ] Detect SVGs with only defs
  - [ ] Skip ID minification for such files

## Medium Priority - Build and Distribution

- [ ] Complete cross-platform build scripts (Issue #410)
  - [ ] Fix macOS universal binary build (Issue #412)
  - [ ] Create Linux packaging (.deb, .rpm, .AppImage)
  - [ ] Create Windows installer (.msi)
  - [ ] Update GitHub Actions workflow
- [ ] Implement version management
  - [ ] Git tag-based versioning
  - [ ] Automatic version injection at build time
  - [ ] Update set-cargo-version.sh script

## Low Priority - Code Quality

- [ ] Fix 27 Clippy warnings
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

## Low Priority - Performance and Testing

- [ ] Create comprehensive benchmarks
  - [ ] Compare performance with SVGO
  - [ ] Benchmark individual plugins
  - [ ] Memory usage profiling
- [ ] Expand test coverage
  - [ ] Port remaining SVGO test fixtures
  - [ ] Add fuzz testing for parser
  - [ ] Performance regression tests
- [ ] Optimize performance
  - [ ] Profile and optimize hot paths
  - [ ] Consider parallel processing
  - [ ] Optimize memory allocations

## Future Enhancements

- [ ] WASM compilation support
- [ ] Node.js bindings
- [ ] Python bindings
- [ ] Plugin marketplace
- [ ] Visual optimization preview
- [ ] GPU-accelerated path optimization
- [ ] Machine learning optimization hints

## Documentation

- [ ] Complete API documentation
- [ ] Write migration guide from SVGO
- [ ] Create plugin development guide
- [ ] Add more examples
- [ ] Document performance characteristics

## Issues to Remove/Close

- [ ] Review and close resolved issue files:
  - [x] 411.txt (already removed - convertPathData stub implementation)
  - [ ] Verify other issue files are still relevant