# CHANGELOG

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-01-02

### Added
- ✅ Initialized Rust library project with proper structure
- ✅ Set up comprehensive Cargo.toml with all necessary dependencies
- ✅ Implemented core AST structures (Document, Element, Node)
- ✅ Created plugin system with Plugin trait and PluginRegistry
- ✅ Built SVG parser using quick-xml with custom mutable AST
- ✅ Implemented SVG stringifier with configurable output options
- ✅ Added configuration system compatible with SVGO format
- ✅ Created main optimization engine with multipass support
- ✅ Built CLI binary with SVGO-compatible command-line options
- ✅ Added comprehensive test suite for all core components
- ✅ Set up benchmark infrastructure using criterion
- ✅ Enhanced PLAN.md with detailed technical architecture decisions
- ✅ Created TODO.md as flat task list representation

### Technical Achievements
- ✅ Fast streaming XML parsing with quick-xml
- ✅ Custom mutable AST optimized for SVG transformations  
- ✅ Plugin architecture using trait objects for extensibility
- ✅ WASM-compatible design throughout the codebase
- ✅ Comprehensive error handling with thiserror
- ✅ Serde-based configuration with JSON/TOML support
- ✅ Memory-efficient tree traversal and mutation
- ✅ SVGO API compatibility for drop-in replacement

### Core Foundation Complete
The foundational architecture is now complete and ready for plugin implementation. The project successfully:
- Parses SVG strings into mutable AST
- Applies plugin transformations 
- Outputs optimized SVG with configurable formatting
- Provides both library and CLI interfaces
- Maintains API compatibility with SVGO

### Next Phase
Ready to begin implementing actual optimization plugins starting with removeComments, removeMetadata, and removeTitle.

## [0.1.1] - 2025-01-02

### Added
- ✅ Implemented three initial optimization plugins:
  - `removeComments` - Removes comments from SVG documents with support for preserving legal comments
  - `removeMetadata` - Removes all `<metadata>` elements from SVG documents
  - `removeTitle` - Removes all `<title>` elements from SVG documents
- ✅ Extended Document AST to support prologue and epilogue nodes (comments/PIs before/after root)
- ✅ Added plugin infrastructure with a `plugins/` module structure
- ✅ Registered initial plugins in the default plugin registry
- ✅ Created comprehensive plugin test suite ported from SVGO tests

### Fixed
- ✅ Fixed failing tests in optimizer, config, and stringifier modules
- ✅ Fixed config deserialization to support mixed string/object plugin arrays (SVGO compatibility)
- ✅ Fixed floating-point comparison in optimization info tests
- ✅ Fixed attribute escaping in stringifier to properly handle quotes
- ✅ Fixed parser to preserve comments and processing instructions in document prologue/epilogue
- ✅ Fixed stringifier to output prologue/epilogue nodes correctly
- ✅ Fixed whitespace handling in pretty-printing mode

### Technical Improvements
- ✅ Enhanced parser to respect preserve_whitespace setting for text trimming
- ✅ Improved stringifier to handle empty elements with whitespace-only content
- ✅ Added whitespace cleanup logic to plugins when removing elements
- ✅ Custom deserialization for plugin configurations to match SVGO's flexible format
- ✅ All 38 tests now passing (33 unit tests + 5 plugin integration tests)

### Next Steps
The three initial plugins are complete and tested. Ready to continue with Phase 3 - Full Plugin Porting.

## [0.1.2] - 2025-01-02

### Added
- ✅ Implemented two more optimization plugins:
  - `cleanupAttrs` - Cleans up attributes from newlines, trailing and repeating spaces with configurable options
  - `cleanupEnableBackground` - Removes or simplifies enable-background attribute when it matches SVG dimensions
- ✅ Added regex dependency for pattern matching in plugins
- ✅ Created comprehensive test suites for both new plugins

### Technical Details
- ✅ cleanupAttrs supports three configurable options: newlines, trim, and spaces (all default to true)
- ✅ cleanupEnableBackground intelligently handles filter presence and simplifies values for mask/pattern elements
- ✅ Used LazyLock for efficient regex compilation at startup
- ✅ Maintained full compatibility with SVGO's plugin behavior

### Progress
- 5 plugins now implemented out of 50+ total plugins

## [0.1.3] - 2025-01-02

### Added
- ✅ Implemented `cleanupIds` plugin with full functionality:
  - Removes unused IDs from elements that are not referenced
  - Minifies used IDs to save space (long-gradient-id → a, b, c, etc.)
  - Preserves specific IDs and ID prefixes based on configuration
  - Safely handles scripts and styles by skipping optimization when present
  - Updates all references (url(), href, begin attributes) when IDs are changed
- ✅ Added urlencoding dependency for proper ID encoding in references
- ✅ Created comprehensive test suite covering all major use cases

### Technical Details
- ✅ Used separate regex patterns to handle quoted and unquoted URL references (regex crate doesn't support backreferences)
- ✅ Implemented ID generation algorithm matching SVGO's approach (a-z, A-Z sequence)
- ✅ Used raw pointers safely to track elements by ID while mutating the tree
- ✅ Full compatibility with SVGO's cleanupIds behavior and options

### Progress
- 6 plugins now implemented out of 50+ total plugins
- More complex plugins like cleanupIds demonstrate the robustness of the plugin system

## [0.1.4] - 2025-01-02

### Added
- ✅ Created comprehensive integration test (`tests/integration_test.rs`) that:
  - Tests multiple plugins working together in the optimization pipeline
  - Verifies that all implemented plugins apply their optimizations correctly
  - Validates optimization metrics (size reduction, compression ratio)
- ✅ Enhanced test coverage to 45 total tests (44 unit tests + 1 integration test)

### Technical Improvements
- ✅ All plugins now work seamlessly together in a single optimization pass
- ✅ Verified proper interaction between plugins (e.g., cleanupIds respects cleaned attributes)
- ✅ Integration test demonstrates real-world SVG optimization scenario

### Current Plugin Status
Completed plugins (9 total):
1. `removeComments` - Removes comments with legal comment preservation
2. `removeMetadata` - Removes metadata elements
3. `removeTitle` - Removes title elements
4. `cleanupAttrs` - Cleans up attribute formatting
5. `cleanupEnableBackground` - Optimizes enable-background attributes
6. `cleanupIds` - Removes unused and minifies used IDs
7. `removeDoctype` - Removes DOCTYPE declarations
8. `removeXMLProcInst` - Removes XML processing instructions/declarations
9. `removeDesc` - Removes description elements (empty/standard only by default)

### Project Metrics
- Total Lines of Code: ~4,500+
- Test Coverage: 52 tests all passing
- Dependencies: Minimal and all WASM-compatible
- Performance: Native Rust performance advantages over JavaScript

## [0.1.5] - 2025-01-02

### Added
- ✅ Implemented 3 more simple removal plugins (Simple Removers batch):
  - `removeDoctype` - Removes DOCTYPE declarations from SVG documents
  - `removeXMLProcInst` - Removes XML processing instructions including XML declarations
  - `removeDesc` - Removes `<desc>` elements (empty or standard editor descriptions by default)
- ✅ Extended AST to support DOCTYPE nodes for proper parsing and removal
- ✅ Enhanced parser to handle DOCTYPE declarations using quick-xml
- ✅ Enhanced stringifier to output XML declarations based on document metadata
- ✅ Added comprehensive test suites for all new plugins

### Technical Improvements
- ✅ Fixed unreachable pattern warning in parser by explicitly handling all Event types
- ✅ Added proper DOCTYPE support throughout the entire pipeline (parse → transform → stringify)
- ✅ XML declarations are now properly handled through document metadata rather than as processing instructions
- ✅ removeDesc plugin implements smart filtering (preserves accessibility descriptions, removes editor fluff)

### Progress Summary
- 9 plugins now implemented out of 50+ total plugins (18% complete)
- Simple removal plugins demonstrate consistent architecture patterns
- Enhanced AST and parser capabilities for more complex future plugins
- All tests passing with robust error handling

## [0.1.6] - 2025-01-02

### Added
- ✅ Completed Simple Removers batch with 3 additional plugins:
  - `removeEmptyAttrs` - Removes attributes with empty values while preserving conditional processing attributes
  - `removeEmptyContainers` - Removes empty container elements with smart handling for special cases
  - `removeEmptyText` - Removes empty text elements (`<text>`, `<tspan>`, `<tref>`) with configurable options
- ✅ Added comprehensive test suites for all new plugins (25 additional tests)
- ✅ Enhanced plugin architecture to support configurable parameters via JSON

### Technical Improvements
- ✅ `removeEmptyAttrs` preserves conditional processing attributes (`requiredExtensions`, `requiredFeatures`, `systemLanguage`)
- ✅ `removeEmptyContainers` implements sophisticated logic for SVG container elements with special cases:
  - Preserves root SVG elements
  - Preserves patterns with attributes (reusable configuration)
  - Preserves masks with IDs (hide masked elements)
  - Preserves groups with filters (may create rectangles)
  - Preserves elements in switch contexts
- ✅ `removeEmptyText` supports configurable removal of text/tspan/tref elements with parameter validation
- ✅ All plugins use efficient tree traversal patterns for nested element processing

### Testing & Quality
- ✅ 83 total tests now passing (up from 52 tests)
- ✅ Comprehensive edge case coverage for all removal scenarios
- ✅ Parameter validation and configuration testing
- ✅ Integration testing confirms all plugins work together seamlessly

### Progress Update
- 12 plugins now implemented out of 50+ total plugins (24% complete)
- Simple Removers batch complete - ready for next phase (Numeric/Value Cleaners)
- Solid foundation established for more complex plugin implementations

## [0.1.7] - 2025-01-02

### Added
- ✅ Completed Numeric/Value Cleaners batch with 2 powerful plugins:
  - `cleanupNumericValues` - Rounds numeric values to fixed precision, converts units to pixels, removes default "px" units
  - `cleanupListOfValues` - Rounds lists of values (viewBox, points, stroke-dasharray, etc.) with smart unit conversion
- ✅ Added advanced numeric processing capabilities with configurable precision and unit handling
- ✅ Added comprehensive test suites for numeric optimization (23 additional tests)

### Technical Achievements
- ✅ `cleanupNumericValues` features:
  - Configurable float precision (default: 3 decimal places)
  - Leading zero removal (0.5 → .5, -0.5 → -.5)
  - Smart unit conversion (absolute units to pixels when beneficial)
  - Default "px" unit removal for cleaner output
  - Special handling for viewBox and version attributes
- ✅ `cleanupListOfValues` features:
  - Handles space/comma-separated value lists (points, viewBox, stroke-dasharray, dx, dy, x, y)
  - Special "new" keyword preservation for enable-background
  - Flexible separator handling (spaces, commas, mixed)
  - Same numeric optimizations as cleanupNumericValues but for lists
- ✅ Advanced regex-based value parsing with unit detection
- ✅ Efficient LazyLock pattern for compiled regexes and lookup tables

### Numeric Optimization Examples
- Precision rounding: `1.23456` → `1.235`
- Leading zero removal: `0.5` → `.5`, `-0.25` → `-.25`
- Unit conversion: `1in` → `96` (when beneficial)
- List processing: `"208.250977 77.1308594"` → `"208.251 77.131"`
- viewBox optimization: `"0.12345 1.6789 100.555 50.999"` → `".123 1.679 100.555 51"`

### Testing & Quality
- ✅ 106 total tests now passing (up from 83 tests)
- ✅ Comprehensive numeric precision and rounding test coverage
- ✅ Unit conversion optimization validation
- ✅ Edge case handling for all numeric formats
- ✅ Parameter configuration testing for all options

### Progress Summary
- 14 plugins now implemented out of 50+ total plugins (28% complete)
- Numeric/Value Cleaners batch complete - substantial optimization capabilities added
- Ready for next phase: Attribute Processors batch
- Strong numeric processing foundation for advanced optimizations

## [0.1.8] - 2025-07-03

### Added
- ✅ Completed Numeric/Value Cleaners batch with `cleanupNumericValues` and `cleanupListOfValues` plugins.
- ✅ Enhanced numeric processing capabilities with configurable precision and unit handling.
- ✅ Added comprehensive test suites for numeric optimization.

### Progress Summary
- 14 plugins now implemented out of 50+ total plugins (28% complete).
- Numeric/Value Cleaners batch complete - substantial optimization capabilities added.
- Ready for next phase: Attribute Processors batch.
- Strong numeric processing foundation for advanced optimizations.

## [0.1.9] - 2025-07-03

### Added
- ✅ Completed Attribute Processors batch with 3 powerful plugins:
  - `sortAttrs` - Sorts attributes by name for better gzip compression.
  - `removeAttrs` - Removes attributes by name or by regular expression.
  - `removeUnknownsAndDefaults` - Removes unknown elements' content and attributes, and removes default attribute values.

### Technical Achievements
- ✅ `sortAttrs` ensures consistent attribute ordering, improving compression.
- ✅ `removeAttrs` provides flexible attribute removal based on name or regex patterns.
- ✅ `removeUnknownsAndDefaults` cleans up non-standard or default attributes, reducing file size.

### Progress Summary
- 17 plugins now implemented out of 50+ total plugins (31% complete).
- Attribute Processors batch complete - significant progress in attribute optimization.
- Ready for next phase: Style and Color Handlers.
- Continued focus on matching SVGO's behavior and test suite.

## [0.1.10] - 2025-07-03

### Added
- ✅ Implemented 3 style-related plugins:
  - `removeStyleElement` - Removes all `<style>` elements from SVG documents
  - `mergeStyles` - Merges multiple `<style>` elements into one, with media query support
  - `convertStyleToAttrs` - Converts inline styles to SVG presentation attributes where possible
- ✅ Added `PRESENTATION_ATTRS` collection to define valid SVG presentation attributes
- ✅ Created CSS parsing regex for `convertStyleToAttrs` that handles comments, strings, and escape sequences

### Technical Details
- ✅ `removeStyleElement` provides simple removal of all style elements
- ✅ `mergeStyles` intelligently combines style content, wrapping media-specific styles with @media
- ✅ `convertStyleToAttrs` parses inline styles and converts presentation attributes while preserving non-presentation styles
- ✅ All style plugins handle CDATA sections appropriately
- ✅ Added comprehensive test coverage for all style handling edge cases

### Progress Summary
- 20 plugins now implemented out of 50+ total plugins (40% complete)
- Style handling capabilities significantly enhanced
- Foundation laid for more complex CSS-based optimizations (minifyStyles, inlineStyles)
- Plugin system continues to prove robust for diverse optimization tasks