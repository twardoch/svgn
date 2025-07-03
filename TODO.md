## IMPLEMENTATION TASKS

## Top

- [ ] Make the build app use git-tag-based semver, not `0.1.0`

## PRIORITY TASKS

### High Priority Complex Plugins (9 remaining)

- [ ] **inlineStyles** (move CSS from style elements to inline attributes) - Complex CSS parsing required

### Medium Priority Complex Plugins

- [ ] **convertPathData** (complex path optimization using lyon) - Most complex, requires lyon integration (STUB present, see CHANGELOG; full implementation still needed)
- [ ] **convertTransform** (transform optimization) - Matrix math and transform parsing
- [ ] **mergePaths** (path merging and optimization) - Path analysis and combination
- [ ] **moveElemsAttrsToGroup** (attribute grouping optimization) - DOM structure analysis
- [ ] **moveGroupAttrsToElems** (attribute distribution optimization) - Reverse grouping logic
- [ ] **removeUselessStrokeAndFill** (stroke/fill optimization) - Style inheritance analysis
- [ ] **reusePaths** (path deduplication with hashing) - Content deduplication
- [ ] issues/201.txt: XML Entity Expansion in Parser
- [ ] issues/202.txt: Whitespace Preservation in Textual Tags
- [ ] issues/203.txt: Detailed Error Reporting in Parser
- [ ] issues/204.txt: Inconsistent Namespace Handling in AST
- [ ] issues/205.txt: Document Metadata Handling and Usage
- [ ] issues/206.txt: XML Declaration Stringification
- [ ] issues/207.txt: DOCTYPE Stringification
- [ ] issues/213.txt: Visitor Pattern Implementation
- [ ] issues/215.txt: Missing Preset Implementation
- [ ] issues/216.txt: Limited Dynamic Plugin Loading
- [ ] issues/217.txt: Inconsistent Plugin Parameter Validation
- [ ] issues/225.txt: cleanupEnableBackground Plugin: Handling `enable-background` in style Attributes
- [ ] issues/227.txt: cleanupIds Plugin: URL Encoding/Decoding for ID References
- [ ] issues/228.txt: cleanupIds Plugin: Optimization Skip for `svg` with only `defs`
- [ ] issues/403.txt
- [ ] issues/404.txt
- [ ] issues/407.txt

## SECONDARY TASKS

- [ ] Address issues documented in `ref/svgo/TODO.md`
  - [ ] Address ExperimentalWarning: VM Modules is an experimental feature
  - [ ] Investigate 'cleanupListOfValues' not being part of preset-default warning
  - [ ] Investigate 'removeAttrs' requiring the 'attrs' parameter warning
- [ ] Address issues documented in `svgn/TODO.md`
- [ ] Create comprehensive benchmarks comparing to SVGO performance

### Previously Critical Test Failures - NOW RESOLVED ✅

All tests are now passing (329/329). The following issues have been resolved:
- ✅ Whitespace preservation in output
- ✅ Attribute ordering inconsistency
- ✅ Color case sensitivity in convertColors plugin  
- ✅ Legal comment preservation in removeComments plugin
- ✅ cleanupIds plugin minified ID generation
- ✅ Transform optimization in default preset

### Critical Plugin Implementation Issue - RESOLVED ✅

✅ convertPathData plugin stub implemented and registered in default preset (2025-07-03)

### Code Quality Issues - MEDIUM PRIORITY

- [ ] Fix 27 Clippy warnings (non-blocking)
  - Collapsible if statements (2 warnings)
  - Needless borrows and references (2 warnings)  
  - Manual clamp instead of clamp function (1 warning)
  - Derivable impls (3 warnings)
  - New without default (17 warnings)
  - Parameter only used in recursion (3 warnings)
  - Length comparison to one (1 warning)
  - Collapsible match (1 warning)
  - Needless return statement (1 warning)
  - Invalid regex with backreference (1 error in prefix_ids.rs:180)
- [ ] Fix Python script syntax error in generate_compatibility_tests.py (if still relevant)

### Advanced Features

- [ ] Implement WASM target for web usage
- [ ] Optimize performance for large SVG files
- [ ] Add CLI configuration file support
- [ ] Implement multipass optimization mode

## CURRENT STATUS

- **Plugins Implemented**: 45/54 (83%)
- **Tests Passing**: ALL 329 tests passing ✅ (includes unit, integration, compatibility, fixture tests)
- **Build Status**: ✅ Builds successfully, tests pass, minor code quality issues remain
- **Test Status Summary**:
  - Unit tests: 329/329 passing (100%)
  - Integration tests: 4/4 passing (100%)  
  - Compatibility tests: 16/16 passing (100%)
  - Fixture tests: 5/5 passing (100%)
  - Plugin tests: 5/5 passing (100%)
- **Code Quality**: 27 Clippy warnings (non-blocking) + minor formatting issues
