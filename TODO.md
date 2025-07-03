
## IMPLEMENTATION TASKS

## PRIORITY TASKS

### High Priority Complex Plugins (9 remaining)
- [ ] **inlineStyles** (move CSS from style elements to inline attributes) - Complex CSS parsing required

### Medium Priority Complex Plugins  
- [ ] **convertPathData** (complex path optimization using lyon) - Most complex, requires lyon integration
- [ ] **convertTransform** (transform optimization) - Matrix math and transform parsing
- [ ] **mergePaths** (path merging and optimization) - Path analysis and combination
- [ ] **moveElemsAttrsToGroup** (attribute grouping optimization) - DOM structure analysis
- [ ] **moveGroupAttrsToElems** (attribute distribution optimization) - Reverse grouping logic
- [ ] **removeUselessStrokeAndFill** (stroke/fill optimization) - Style inheritance analysis
- [ ] **reusePaths** (path deduplication with hashing) - Content deduplication

## SECONDARY TASKS

### Infrastructure & Quality
- [x] Enhance test coverage with SVGO compatibility tests ✅ COMPLETED (2025-07-03)
  - ✅ 13 comprehensive test files implemented
  - ✅ SVGO-style fixture testing framework
  - ✅ 16 compatibility tests covering core functionality
  - ✅ Individual plugin test modules for 8+ plugins  
  - ✅ Integration tests for multi-plugin pipelines
  - ✅ Error resilience and edge case testing
- [ ] Address issues documented in `ref/svgo/TODO.md`
  - [ ] Address ExperimentalWarning: VM Modules is an experimental feature
  - [ ] Investigate 'cleanupListOfValues' not being part of preset-default warning
  - [ ] Investigate 'removeAttrs' requiring the 'attrs' parameter warning
- [ ] Address issues documented in `svgn/TODO.md`
- [ ] Create comprehensive benchmarks comparing to SVGO performance

### Critical Test Failures - URGENT
- [ ] Fix whitespace preservation in output (11 test failures)
  - Issue: Tests expect pretty-printed output with preserved whitespace but getting minified output
  - Affects: `test_remove_comments_with_params_fixture`, `test_cleanup_attrs_fixture`, `test_pretty_print_formatting`, multiple compatibility tests
- [ ] Fix attribute ordering inconsistency 
  - Issue: Attributes appear in different order than SVGO expects
  - Affects: Multiple compatibility tests (`test_remove_dimensions_with_viewbox`, `test_multiple_plugins_pipeline`)
- [ ] Fix color case sensitivity in convertColors plugin
  - Issue: Outputs uppercase hex colors (#DCDDDE) instead of lowercase (#dcddde)
  - Affects: `test_convert_colors_fixture`
- [ ] Fix legal comment preservation in removeComments plugin
  - Issue: Legal comments (starting with !) are not preserved
  - Affects: `test_remove_comments_preserve_legal`
- [ ] Fix cleanupIds plugin minified ID generation
  - Issue: Generates 'b' instead of 'a' for first minified ID
  - Affects: `test_cleanup_ids_minification`
- [ ] Fix transform optimization in default preset
  - Issue: `transform="translate(0,0)"` not being removed
  - Affects: `test_default_preset_pipeline`

### Code Quality Issues - HIGH PRIORITY
- [ ] Fix 27 Clippy warnings
  - Collapsible if statements (2 warnings)
  - Needless borrows and references (2 warnings)
  - Manual clamp instead of clamp function (1 warning)
  - Derivable impls (3 warnings)
  - New without default (17 warnings)
  - Parameter only used in recursion (3 warnings)
  - Length comparison to one (1 warning)
  - Collapsible match (1 warning)
  - Needless return statement (1 warning)
- [ ] Fix compilation error preventing cargo clippy from completing
- [ ] Fix Python script syntax error in generate_compatibility_tests.py
  - Issue: Line 48 has invalid string escaping causing SyntaxError
  - Error: `unexpected character after line continuation character`

### Advanced Features
- [ ] Implement WASM target for web usage
- [ ] Optimize performance for large SVG files  
- [ ] Add CLI configuration file support
- [ ] Implement multipass optimization mode

## CURRENT STATUS
- **Plugins Implemented**: 45/54 (83%)
- **Tests Passing**: 325 unit tests ✅, 11 integration/compatibility tests ❌
- **Build Status**: ❌ Multiple critical test failures and code quality issues
- **Test Status Summary**:
  - Unit tests: 325/325 passing (100%)
  - Integration tests: 2/4 passing (50%)
  - Compatibility tests: 10/16 passing (62.5%)
  - Fixture tests: 1/5 passing (20%)
  - Plugin tests: 5/5 passing (100%)
- **Code Quality**: 27 Clippy warnings + 1 compilation error