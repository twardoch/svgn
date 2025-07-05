## 2025-07-05

### Documentation Updates

- Updated plugin counts across `plugins.md`, `index.md`, `comparison.md`, and `_includes/sidebar.html` to reflect 55 implemented plugins (exceeding SVGO's 53).
- Revised "Not Yet Implemented" sections in `plugins.md` and `comparison.md` to accurately list the remaining 3 plugins.
- Removed outdated notes regarding `removeRasterImages` and `removeScripts` implementation status.

### Parser Infrastructure Improvements (Thread M)

- **M1-M5: XML Entity Expansion** (Issue #201) - Implemented complete XML entity support:
  - Added entity parsing from DOCTYPE declarations using regex patterns
  - Built entity table as HashMap during parsing for efficient lookups
  - Implemented entity expansion in both text content and attribute values
  - Added `expand_entities` flag to Parser for configurable behavior

- **M6-M8: Selective Whitespace Preservation** (Issue #202) - Fixed whitespace handling:
  - Created TEXT_ELEMENTS set containing elements requiring whitespace preservation
  - Implemented element name stack tracking during parsing
  - Added context-aware whitespace handling for text, tspan, pre, script, style elements

- **M9-M11: Enhanced Error Reporting** (Issue #203) - Improved parser error messages:
  - Created DetailedParseError struct with file path, line/column, and context
  - Implemented Display trait for formatted error output with source code snippets
  - Added calculate_line_and_column method to convert byte positions to line/column
  - Enhanced XML parsing errors with position information and visual error indicators

### Build Status

- **Build succeeds with warnings** - Project now compiles and runs tests
- **Test results:** 346 passed, 4 failed (98.8% pass rate)
- **Failed tests:** All in remove_attributes_by_selector plugin (CSS selector functionality)
- **Warnings:** 20 warnings remaining (unused imports, variables, and functions)
