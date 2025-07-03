# TODO

This is a flat task list derived from PLAN.md. Tasks are organized by priority and phase.

## Current Priority: Phase 3 - Full Plugin Porting

### Recently Completed
- [x] `removeDeprecatedAttrs` - Removes deprecated SVG attributes with safe/unsafe modes
- [x] `convertEllipseToCircle` - Converts non-eccentric ellipses to circles
- [x] `removeAttributesBySelector` - Removes attributes of elements that match a CSS selector
- [x] `collapseGroups` - Collapses useless groups by removing empty groups and moving attributes to children



### Attribute Processors  
- [x] `sortAttrs` ✅
- [x] `removeAttrs` ✅ 
- [x] `removeUnknownsAndDefaults` ✅
- [x] `addAttributesToSVGElement` ✅
- [x] `addClassesToSVGElement` ✅
- [x] `removeAttributesBySelector` ✅
- [x] `removeDeprecatedAttrs` ✅

### Style and Color Handlers
- [x] `convertColors` ✅
- [ ] `minifyStyles` (requires CSS parsing)
- [ ] `inlineStyles`
- [x] `mergeStyles` ✅
- [x] `convertStyleToAttrs` ✅
- [x] `removeStyleElement` ✅

### Transform and Path Optimizers
- [ ] `convertTransform`
- [ ] `convertPathData` (complex, uses lyon for path transformations)
- [ ] `convertShapeToPath`
- [ ] `mergePaths`

### Structural Optimizers
- [x] `collapseGroups` ✅
- [ ] `moveElemsAttrsToGroup`
- [ ] `moveGroupAttrsToElems`
- [ ] `removeNonInheritableGroupAttrs`

### Other Plugins
- [x] `convertEllipseToCircle` ✅
- [ ] `convertOneStopGradients`
- [ ] `prefixIds`
- [ ] `removeEditorsNSData`
- [ ] `removeElementsByAttr`
- [ ] `removeHiddenElems`
- [ ] `removeOffCanvasPaths`
- [ ] `removeRasterImages`
- [ ] `removeScripts`
- [ ] `removeUnusedNS`
- [ ] `removeUselessDefs`
- [ ] `removeUselessStrokeAndFill`
- [ ] `removeViewBox`
- [ ] `removeXlink`
- [ ] `removeXMLNS`
- [ ] `removeDimensions`
- [ ] `reusePaths`
- [ ] `sortDefsChildren`

## Phase 4: Testing, Benchmarking, and CI

- [ ] Systematically port the entire `svgo` test suite from `ref/svgo/test/`
- [ ] Create a test runner that can execute tests and compare output of `svgn` with expected output from `svgo`
- [ ] Create a comprehensive benchmark suite using `cargo bench`
- [ ] Benchmarks should cover parsing, stringifying, and each of the major plugins
- [ ] Compare the performance of `svgn` against the original `svgo` to quantify performance gains
- [ ] Set up a CI pipeline using GitHub Actions
- [ ] CI pipeline will run `cargo test` on every push and pull request
- [ ] CI pipeline will run `cargo clippy` to enforce code quality
- [ ] CI pipeline will run `cargo fmt --check` to ensure consistent formatting
- [ ] (Optional) Run benchmarks and report on performance changes

## Phase 5: WebAssembly (WASM) Target

- [ ] Integrate `wasm-pack` into the build process
- [ ] Ensure the codebase is compatible with the WASM target
- [ ] Create a WASM-compatible `optimize` function using `wasm-bindgen`
- [ ] Function will accept a string and a `JsValue` for configuration, and return the optimized string
- [ ] Create a simple web page that uses the WASM-compiled `svgn` to optimize SVGs in the browser
- [ ] This will serve as a proof-of-concept and demonstration of the WASM capabilities

## Phase 6: Documentation and Release

- [ ] Prepare a Jekyll+Markdown structure that will serve as the documentation on Github Pages (via main branch, docs folder setup)

- [ ] Write comprehensive `rustdoc` comments for all public APIs, structs, and functions
- [x] Create user-friendly documentation in the `docs/` directory
- [x] Include guides on how to use the CLI, the Rust library, and the WASM module
- [x] Provide a detailed reference for all configuration options and plugins
- [ ] Publish the `svgn` crate to `crates.io`
- [ ] Publish the WASM package to `npm`
- [ ] Create a GitHub release with release notes

---
**Issue:** XML Entity Expansion in Parser

**Description:**
The current Rust parser (`svgn/src/parser.rs`) does not handle XML entity declarations within the DOCTYPE for expansion. The JavaScript SVGO parser (`ref/svgo/lib/parser.js`) uses `sax.ENTITIES` to expand these entities during parsing.

**Impact:**
If an SVG document uses custom entities defined in its DOCTYPE, the Rust parser will not resolve them, leading to different parsed content compared to the original SVGO. This can result in incorrect optimization or rendering.

**Example (from ref/svgo/test/svgo/entities.svg.txt):**
```xml
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd" [
<!ENTITY Viewport "<rect x='.5' y='.5' width='49' height='29'/>">
]>
<svg>
  <g>
    &Viewport;
  </g>
</svg>
```
In this example, `&Viewport;` should be replaced by the `<rect>` element. The current Rust parser will likely treat `&Viewport;` as unparsed text or an error.

**Proposed Solution:**
Modify the Rust parser to identify and expand XML entities declared in the DOCTYPE. This might involve:
1.  Extracting entity declarations from the `DocType` event.
2.  Storing these entities in a map.
3.  Replacing entity references in text or attribute values with their corresponding expanded content during parsing.

---
**Issue:** Whitespace Preservation in Textual Tags

**Description:**
The Rust parser (`svgn/src/parser.rs`) uses `reader.trim_text(!self.preserve_whitespace)` which applies whitespace trimming globally. The JavaScript SVGO parser (`ref/svgo/lib/parser.js`) uses a `textElems` set (from `_collections.js`) to specifically prevent trimming of meaningful whitespace within certain textual SVG elements (e.g., `<text>`, `<tspan>`, `<pre>`, `<title>`).

**Impact:**
Global trimming of whitespace can lead to loss of significant whitespace in elements where it is semantically important (e.g., preserving layout in `<pre>` tags or spacing in `<text>` elements). This can alter the visual appearance or meaning of the optimized SVG.

**Example (from ref/svgo/test/svgo/whitespaces.svg.txt):**
```xml
<svg width="480" height="360" xmlns="http://www.w3.org/2000/svg">
  <text x="20" y="20">
    <tspan>Another tspan</tspan>
    <tspan>Inside tspan</tspan> - outside tspan
  </text>
</svg>
```
The whitespace between `<tspan>` tags and around " - outside tspan" is important for rendering.

**Proposed Solution:**
Implement a mechanism in the Rust parser to selectively preserve whitespace based on the element's tag name. This could involve:
1.  Creating a `HashSet` of tag names (similar to JS `textElems`) where whitespace should be preserved.
2.  Modifying the `Event::Text` handling in the parser to check if the current element's name is in this set before applying trimming.

---
**Issue:** Detailed Error Reporting in Parser

**Description:**
The Rust parser (`svgn/src/parser.rs`) provides basic error reporting via `ParseError`. However, it lacks the detailed context and code snippet highlighting offered by `SvgoParserError` in the JavaScript version (`ref/svgo/lib/parser.js`). The JS error includes the file path, line number, column number, and a visual representation of the problematic line with a pointer.

**Impact:**
Less detailed error messages make debugging and identifying the exact location of parsing issues more challenging for users. This reduces the usability and helpfulness of the CLI tool.

**Example (from ref/svgo/test/svgo.test.js):**
```
SvgoParserError: test.svg:2:33: Unquoted attribute value

  1 | <svg viewBox="0 0 120 120">
> 2 |   <circle fill="#ff0000" cx=60.444444" cy="60" r="50"/>
    |                           ^
  3 | </svg>
  4 |
```

**Proposed Solution:**
Enhance the `ParseError` enum and its `Display` implementation in Rust to:
1.  Store additional context: file path (if available), line number, column number, and the relevant source code snippet.
2.  Format the error message to include this information, potentially with a visual indicator (like `^`) pointing to the exact error location.
3.  This might require passing the original input string and file path (if available) down to the parser's error handling logic.

---
**Issue:** Inconsistent Namespace Handling in AST

**Description:**
The Rust AST's `Element` struct (`svgn/src/ast.rs`) explicitly stores `namespaces` as a `HashMap<String, String>`. While the parser extracts `xmlns` attributes into this map, it also keeps them in the `attributes` `HashMap`. In contrast, the JavaScript `xast` representation typically flattens namespace prefixes into attribute names (e.g., `xmlns:xlink` is just an attribute key), and the `xmlns` attributes themselves are often removed after parsing in SVGO's pipeline.

**Impact:**
1.  **Redundancy:** Storing `xmlns` attributes in both `namespaces` and `attributes` is redundant and can lead to inconsistencies or increased memory usage.
2.  **Processing Logic:** Plugins might expect a flattened structure or a specific handling of `xmlns` attributes that is not currently aligned between the Rust AST and SVGO's typical processing. For example, SVGO often removes `xmlns` attributes from the root `<svg>` element when inlining.
3.  **API Compatibility:** Differences in namespace representation can complicate porting plugins that rely on SVGO's `xast` structure for namespace resolution or manipulation.

**Proposed Solution:**
Align the Rust AST and parser's namespace handling with SVGO's typical behavior:
1.  **Parser:** After parsing `xmlns` attributes and populating the `namespaces` map, consider removing them from the `attributes` `HashMap` of the `Element`.
2.  **AST:** Ensure that the `Element.name` and attribute keys correctly reflect namespaced elements and attributes (e.g., `xlink:href` remains `xlink:href` as an attribute key, not just `href` with a separate namespace lookup).
3.  **Stringifier:** The stringifier should correctly re-add `xmlns` declarations based on the `namespaces` map and any namespaced elements/attributes present in the tree.

---
**Issue:** Document Metadata Handling and Usage

**Description:**
The Rust `Document` struct (`svgn/src/ast.rs`) includes `metadata` fields for `path`, `encoding`, and `version`. The Rust parser populates `encoding` and `version` from the XML declaration. The JavaScript `optimize` function passes `path` in its `info` object, and the XML declaration details are handled during stringification.

**Impact:**
While having `metadata` in the Rust AST is a good structural addition, its consistent population and utilization throughout the optimization pipeline needs verification.
1.  **`path` field:** The `path` field in `DocumentMetadata` is currently populated by the CLI, but its usage by plugins (e.g., `prefixIds` in JS uses `info.path`) needs to be ensured.
2.  **`encoding` and `version` fields:** These are parsed but their impact on optimization or stringification (e.g., preserving the original XML declaration or modifying it) needs to be defined and implemented. The JS stringifier has options for XML declaration.

**Proposed Solution:**
1.  **`path` field:** Ensure that the `path` from `DocumentMetadata` is accessible to and used by plugins that require file path context (e.g., for generating unique IDs). This might involve passing `&document.metadata` to plugin `apply` methods.
2.  **`encoding` and `version` fields:**
    *   Define how these fields should influence the stringification process. For example, should the original XML declaration be preserved, or should it be regenerated based on the `metadata`?
    *   Implement logic in the Rust stringifier (`svgn/src/stringifier.rs`) to correctly output the XML declaration based on these `metadata` fields and `js2svg` options.

---
**Issue:** XML Declaration Stringification

**Description:**
The Rust stringifier (`svgn/src/stringifier.rs`) does not explicitly handle the XML declaration (`<?xml ...?>`) based on the `DocumentMetadata` from the AST. The JavaScript SVGO stringifier implicitly handles it if present in the input and allows control over its output.

**Impact:**
Without explicit handling, the XML declaration might be lost or incorrectly generated in the output SVG, leading to non-conformant or less optimized files. The `DocumentMetadata` in the Rust AST already stores `version` and `encoding`, which should be utilized.

**Proposed Solution:**
1.  Modify the `stringify` method in `svgn/src/stringifier.rs` to check `document.metadata.version` and `document.metadata.encoding`.
2.  If these fields are present, generate an XML declaration at the beginning of the output string.
3.  Consider adding options to `Js2SvgOptions` (in `svgn/src/config.rs`) to control the output of the XML declaration (e.g., `addXmlDecl`, `xmlVersion`, `xmlEncoding`), similar to SVGO's behavior.

---
**Issue:** DOCTYPE Stringification

**Description:**
The Rust stringifier (`svgn/src/stringifier.rs`) does not explicitly handle the DOCTYPE declaration. The JavaScript SVGO stringifier has `doctypeStart` and `doctypeEnd` options and processes `XastDoctype` nodes. The Rust `Document` struct stores `DocType` nodes in its `prologue` vector.

**Impact:**
If an input SVG contains a DOCTYPE declaration, it will be lost during stringification in the current Rust implementation. This can affect document validation and compatibility with certain SVG consumers.

**Proposed Solution:**
1.  Modify the `stringify` method in `svgn/src/stringifier.rs` to iterate through `document.prologue`.
2.  When a `Node::DocType` is encountered, stringify it using the appropriate syntax (e.g., `<!DOCTYPE svg PUBLIC ...>`).
3.  Consider adding options to `Js2SvgOptions` (in `svgn/src/config.rs`) to control DOCTYPE output (e.g., `removeDoctype`).

---
**Issue:** Visitor Pattern Implementation

**Description:**
The Rust plugin system (`svgn/src/plugin.rs`) currently uses a single `apply` method per plugin, which takes the entire `Document` and applies transformations. In contrast, the JavaScript SVGO uses a more granular "visitor pattern". This pattern involves `enter` and `exit` methods for different node types (e.g., `element`, `text`, `comment`, `doctype`, `instruction`, `cdata`, `root`).

**Impact:**
1.  **Efficiency:** Processing the entire document in a single `apply` method can be less efficient for plugins that only need to operate on specific node types or at specific points in the traversal.
2.  **Modularity:** The current approach makes it harder to write modular plugins that focus on specific AST nodes without needing to traverse the entire tree themselves.
3.  **Compatibility:** Porting complex SVGO plugins that heavily rely on the visitor pattern (e.g., `convertPathData`, `inlineStyles`) becomes more challenging and less idiomatic in the current Rust structure.

**Proposed Solution:**
Refactor the Rust plugin system to implement a visitor pattern:
1.  Define a `Visitor` trait with methods like `visit_element_enter`, `visit_element_exit`, `visit_text`, `visit_comment`, etc.
2.  Modify the `Plugin` trait's `apply` method to return an implementation of this `Visitor` trait.
3.  Implement a generic AST traversal function that walks the `Document` and calls the appropriate `Visitor` methods for each node.
4.  Update existing plugins to utilize this new visitor pattern.

---
**Issue:** Missing Preset Implementation

**Description:**
The Rust plugin system (`svgn/src/plugin.rs`) does not yet have a concept of "presets" similar to SVGO's `preset-default`. In SVGO, a preset is a special plugin that groups multiple other plugins and allows for easy configuration and overriding of their parameters.

**Impact:**
1.  **Configuration Complexity:** Without presets, users have to manually list and configure every single plugin, even for common optimization scenarios, leading to verbose and complex configurations.
2.  **Maintainability:** It's harder to manage and update default sets of optimizations.
3.  **Compatibility:** `preset-default` is a core part of SVGO's API and behavior. Its absence makes the Rust port less compatible and user-friendly.

**Proposed Solution:**
1.  Define a `Preset` struct or enum that can encapsulate a list of `PluginConfig`s.
2.  Modify the `Config` struct (`svgn/src/config.rs`) to allow `plugins` to include presets.
3.  Implement logic in the `PluginRegistry` or `optimizer` to expand presets into their individual plugins and apply any overrides specified in the preset's configuration.
4.  Create a `preset-default` implementation in Rust that mirrors the behavior and included plugins of SVGO's `preset-default`.

---
**Issue:** Limited Dynamic Plugin Loading

**Description:**
The JavaScript SVGO allows for dynamic loading of custom plugins by providing a `fn` property in the plugin configuration. The Rust version currently requires plugins to be statically registered in the `PluginRegistry` (`svgn/src/plugin.rs`) at compile time.

**Impact:**
This limitation restricts the extensibility of the Rust port. Users cannot define and load their own custom optimization logic at runtime without modifying and recompiling the `svgn` library itself. This is a significant departure from SVGO's flexible plugin architecture.

**Proposed Solution:**
Explore and implement mechanisms for dynamic plugin loading in Rust. This is a complex task in Rust due to its compiled nature and strong type system, but potential approaches could include:
1.  **FFI (Foreign Function Interface):** Allow plugins to be compiled as shared libraries (DLLs/SOs) and loaded at runtime. This would require defining a stable C-compatible API for plugins.
2.  **WASM (WebAssembly):** If `svgn` is intended to be used in environments that support WASM, plugins could potentially be compiled to WASM and loaded dynamically.
3.  **Plugin DSL (Domain Specific Language):** Develop a simple DSL for defining plugins that can be interpreted or compiled at runtime.

This issue might be considered a long-term goal due to its complexity.

---
**Issue:** Inconsistent Plugin Parameter Validation

**Description:**
The Rust `Plugin` trait includes a `validate_params` method, but its implementation and enforcement are inconsistent across plugins. The JavaScript SVGO often uses JSDoc types for plugin parameters, which provides a form of documentation and implicit validation.

**Impact:**
1.  **Runtime Errors:** Plugins might receive invalid or unexpected parameters, leading to runtime panics or incorrect behavior.
2.  **Poor User Experience:** Users might struggle to understand the expected parameters for each plugin without clear validation or documentation.
3.  **Maintainability:** Lack of consistent validation makes it harder to ensure the correctness and robustness of plugins.

**Proposed Solution:**
1.  **Enforce Validation:** Ensure that `validate_params` is called for every plugin before its `apply` method is invoked in the `PluginRegistry::apply_plugins` method.
2.  **Implement Validation:** For each built-in plugin, thoroughly implement the `validate_params` method to check for expected parameter types, ranges, and values.
3.  **Documentation:** Clearly document the expected parameters for each plugin in the Rust code (e.g., using doc comments) and in user-facing documentation.
4.  **Schema-based Validation (Advanced):** Consider using a schema validation library (e.g., `jsonschema` or `serde_json::Value::validate`) to define and validate plugin parameters, similar to how SVGO's configuration is often validated.

---
**Issue:** `cleanupEnableBackground` Plugin: Handling `enable-background` in `style` Attributes

**Description:**
The JavaScript `cleanupEnableBackground` plugin (`ref/svgo/plugins/cleanupEnableBackground.js`) is capable of processing and cleaning up `enable-background` declarations that appear within an element's `style` attribute. It uses the `csstree` library to parse and manipulate the CSS within these attributes.

In contrast, the Rust `CleanupEnableBackgroundPlugin` (`svgn/src/plugins/cleanup_enable_background.rs`) explicitly has a `TODO` comment indicating that this functionality is not yet implemented. It only processes `enable-background` when it's a direct attribute.

**Impact:**
SVGs that use inline styles for `enable-background` will not be optimized by the Rust plugin, leading to larger file sizes and incomplete optimization compared to the JS version. This reduces the functional compatibility of the Rust port.

**Proposed Solution:**
1.  Integrate a CSS parsing and manipulation library into the Rust project (e.g., `cssparser` and potentially `lightningcss` or a custom solution).
2.  Modify the `CleanupEnableBackgroundPlugin` to:
    *   Check for the `style` attribute on elements.
    *   Parse the CSS content of the `style` attribute.
    *   Identify and apply the `enable-background` cleanup logic to declarations within the parsed CSS.
    *   Re-serialize the modified CSS back into the `style` attribute.

---
**Issue:** `cleanupIds` Plugin: URL Encoding/Decoding for ID References

**Description:**
The JavaScript `cleanupIds` plugin (`ref/svgo/plugins/cleanupIds.js`) uses `encodeURI` and `decodeURI` for handling IDs within `url()` references (e.g., `url(#myId)`). The Rust implementation (`svgn/src/plugins/cleanup_ids.rs`) uses the `urlencoding` crate for encoding/decoding these references.

**Impact:**
While the `urlencoding` crate is generally suitable for URL components, `encodeURI` in JavaScript has specific behaviors regarding which characters it encodes (e.g., it does not encode `;`, `/`, `?`, `:`, `@`, `&`, `=`, `+`, `$`, `,`, `#` if they are part of the URI component). A generic URL encoder might encode these characters, leading to different output or broken references in the SVG.

This discrepancy can result in:
1.  **Non-byte-for-byte compatibility:** The output SVG might differ from SVGO's, even if functionally equivalent.
2.  **Broken References:** If the encoding/decoding is not perfectly aligned, references to IDs within `url()` attributes might break, causing rendering issues.

**Proposed Solution:**
1.  **Detailed Comparison:** Perform a detailed comparison of `encodeURI`/`decodeURI` behavior with `urlencoding` crate functions for all characters that can appear in SVG IDs.
2.  **Adjust Encoding:** If discrepancies are found, adjust the Rust implementation to precisely match the JS behavior. This might involve:
    *   Using a different encoding function from `urlencoding` or another crate.
    *   Implementing a custom encoding/decoding function that replicates `encodeURI`/`decodeURI`'s specific logic for SVG ID characters.
3.  **Test Cases:** Ensure comprehensive test cases (like `cleanupIds.25.svg.txt` and `cleanupIds.26.svg.txt` from SVGO's test suite) are used to validate the encoding/decoding behavior.

---
**Issue:** `cleanupIds` Plugin: Optimization Skip for `svg` with only `defs`

**Description:**
The JavaScript `cleanupIds` plugin (`ref/svgo/plugins/cleanupIds.js`) includes a specific optimization: if the root `<svg>` element contains only `<defs>` children (i.e., no visible content outside of definitions), the plugin skips further processing by returning `visitSkip`. This is a performance optimization to avoid unnecessary work when there's nothing to minify or remove in the main document body.

In contrast, the Rust `CleanupIdsPlugin` (`svgn/src/plugins/cleanup_ids.rs`) does not currently implement this specific check.

**Impact:**
While not a functional correctness issue that breaks the SVG, the absence of this optimization means the Rust plugin might perform unnecessary work (collecting IDs, processing references, attempting minification) on certain SVG structures that offer no visible content. This can lead to minor performance inefficiencies for such specific input files.

**Proposed Solution:**
1.  **Implement Check:** Add a check at the beginning of the `CleanupIdsPlugin::apply` method (or within the `element: { enter: ... }` visitor for the root `<svg>` element if the visitor pattern is adopted).
2.  **Detect `defs`-only SVG:** Determine if the `<svg>` element's children consist solely of `<defs>` elements.
3.  **Skip Processing:** If this condition is met, the plugin should return early, indicating that no further ID cleanup is necessary for this SVG.

---
**Issue:** `convertColors` Plugin: Incomplete `currentColor` Implementation

**Description:**
The Rust `convertColors` plugin (`svgn/src/plugins/convert_colors.rs`) has a simplified implementation of the `currentColor` feature. The JavaScript version allows `currentColor` to be a boolean, a string, or a RegExp, providing more flexible matching. The Rust version currently only supports a string pattern.

**Impact:**
The Rust plugin cannot replicate the full range of `currentColor` conversion behaviors available in SVGO, limiting its utility for users who rely on more advanced configurations.

**Proposed Solution:**
1.  Update the `ConvertColorsConfig` struct in Rust to support a more flexible `currentColor` type (e.g., an enum with variants for `bool`, `String`, and `String` for a regex pattern).
2.  Modify the plugin logic to handle each variant correctly, applying `currentColor` based on the configured matching strategy.
3.  Add comprehensive tests to cover all `currentColor` configuration options.
