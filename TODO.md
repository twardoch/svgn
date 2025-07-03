
## 🚨 CRITICAL BUILD ISSUES - FIX FIRST

- [ ] **URGENT: Fix compilation errors in convert_shape_to_path.rs**
  - [ ] Fix import error: `use crate::plugins::{Plugin, PluginInfo};` - Plugin/PluginInfo not found in plugins module
  - [ ] Fix import error: `use crate::ast::{Node, NodeType};` - NodeType not found in ast module  
  - [ ] Fix struct usage error: `Node {` - expected struct, found enum Node
  - [ ] Fix test import error: `use crate::ast::NodeType;` in test module
  - [ ] Remove unused import warning: `use indexmap::IndexMap;`

## IMPLEMENTATION TASKS

- [ ] Port remaining 9 complex plugins to reach 100% compatibility:
  - [ ] convertPathData (complex path optimization using lyon)
  - [x] convertShapeToPath (shape to path conversion)
  - [ ] convertTransform (transform optimization)
  - [ ] inlineStyles (style inlining with CSS parsing)
  - [ ] mergePaths (path merging and optimization)
  - [x] minifyStyles (CSS minification)
  - [ ] moveElemsAttrsToGroup (attribute grouping optimization)
  - [ ] moveGroupAttrsToElems (attribute distribution optimization)
  - [ ] removeUselessStrokeAndFill (stroke/fill optimization)
  - [ ] reusePaths (path deduplication with hashing)
- [ ] Enhance test coverage with svgo compatibility tests
- [ ] Address issues documented in `ref/svgo/TODO.md`
- [ ] Address issues documented in `svgn/TODO.md`
- [ ] Implement WASM target for web usage
- [ ] Create comprehensive benchmarks comparing to svgo
- [ ] Optimize performance for large SVG files
- [ ] Add CLI configuration file support
- [ ] Implement multipass optimization mode