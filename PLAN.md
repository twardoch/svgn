# SVGN Development Plan: Organized by Dependency Threads

## 1. Executive Summary

SVGN is a high-performance Rust port of SVGO that has achieved 90.5% plugin implementation. This plan organizes all development tasks into dependency-based threads that can be worked on independently or in parallel, enabling efficient resource allocation and parallel development.

**Current Status (2025-07-05):**
- ✅ **48/53 plugins** implemented (90.5% complete - 5 plugins remaining)
- ❌ **PROJECT CANNOT COMPILE** - 22 compilation errors blocking all development
- ❌ **5 plugins** remaining for 100% parity: mergePaths, moveElemsAttrsToGroup, moveGroupAttrsToElems, applyTransforms, reusePaths

**🚨 CRITICAL PRIORITY: Thread A (Build Fixes) must be completed before any plugin development can continue.**

## 2. Thread-Based Development Strategy

Tasks are organized into 21 independent threads (A-U) based on their dependencies. Each thread is marked as **EASY**, **MEDIUM**, or **HARD** complexity, allowing developers to choose appropriate tasks based on their expertise and available time.

### 2.1. Critical Path Threads (Must be completed in order)

**Thread A → Thread B → Thread P → Thread U**

These threads form the minimum viable path to 100% SVGO plugin parity.

### 2.2. WebAssembly Path (Can be developed in parallel)

**Thread D → Thread E → Thread F → Thread H → Thread I → Thread J**

Complete WebAssembly implementation with online tool, independent of core plugin development.

### 2.3. Independent Threads (Can be developed anytime)

**Threads C, G, M, N, O, Q, R, S, T**

These have no dependencies and can be worked on by any team member at any time.

## 3. Thread A: Critical Build Fixes (HARD) ⚡ BLOCKING

**Dependencies:** None - must be completed before any other work can proceed
**Complexity:** Hard - requires deep understanding of Rust crate versioning and CSS selector APIs
**Estimated Duration:** 1-2 weeks
**Required Skills:** Expert Rust, CSS parser internals, dependency management

### 3.1. Current Build Failure Analysis

❌ **22 compilation errors** completely blocking development:
- **cssparser version conflicts** between lightningcss v0.33.0 and selectors crate expecting v0.31.2
- **ToCss trait missing** for String types in SelectorImpl implementations  
- **PrecomputedHash trait missing** for String types used as Identifier/LocalName
- **MatchingContext API mismatch** - function expects 6 parameters, code provides 4
- **Parser trait missing** for SvgSelectorImpl in SelectorList::parse() calls
- **Method resolution failures** - unescape() method not found on BytesText
- **Private field access** - SelectorList.0.iter() accessing private field

### 3.2. Resolution Strategy

1. **Dependency Alignment:** Resolve cssparser version conflicts between crates
2. **Trait Implementation:** Add missing trait implementations for String types
3. **API Updates:** Update function calls to match current selectors crate API
4. **Access Pattern Fixes:** Replace private field access with public APIs

### 3.3. Success Criteria

- [ ] Project compiles without errors
- [ ] All tests pass
- [ ] CSS selector functionality works correctly
- [ ] inlineStyles plugin foundation is usable

## 4. Thread B: Remaining Core Plugins (HARD)

**Dependencies:** Thread A must be completed first
**Complexity:** Hard - requires sophisticated CSS processing and SVG manipulation
**Estimated Duration:** 3-4 weeks
**Required Skills:** CSS processing, SVG specification, path manipulation

### 4.1. inlineStyles Plugin Completion

**Status:** Foundation complete, CSS parsing working, Element trait implemented
**Remaining Work:**
- CSS specificity-based cascade resolution engine
- Media query and pseudo-class filtering logic
- CSS property to SVG attribute conversion
- Unused selector cleanup

### 4.2. Path Manipulation Plugins

**mergePaths:** Concatenate adjacent paths with identical styling
**moveElemsAttrsToGroup:** Move common attributes to parent groups
**moveGroupAttrsToElems:** Distribute group attributes to children

### 4.3. Implementation Advantages

- **CSS Processing:** `lightningcss`, `cssparser`, and `selectors` crates already configured
- **Path Processing:** Robust path parsing and manipulation from `convertPathData`
- **DOM Manipulation:** Mature element traversal and attribute handling
- **Plugin Architecture:** Well-tested plugin system with 50 working implementations

## 5. Thread C: Standalone Plugins (MEDIUM)

**Dependencies:** Thread A must be completed first (independent of Thread B)
**Complexity:** Medium - leverages existing infrastructure
**Estimated Duration:** 2 weeks
**Required Skills:** Matrix mathematics, SVG geometry, path processing

### 5.1. applyTransforms Plugin
- Apply transform matrices directly to coordinate data
- Handle different SVG shape types (paths, circles, rects, etc.)
- Resolve transform hierarchies correctly

### 5.2. reusePaths Plugin
- Create content-based hashes for path deduplication
- Generate `<defs>` and `<use>` element structure
- Calculate byte savings vs. overhead analysis

## 6. WebAssembly Implementation Threads

### 6.1. Thread D: WASM Build Infrastructure (EASY)

**Dependencies:** None - can be developed in parallel with other threads
**Complexity:** Easy - standard Rust/WASM toolchain setup
**Estimated Duration:** 1 week
**Required Skills:** Rust toolchain, WebAssembly basics

**Key Tasks:**
- Install and configure wasm-pack
- Create WASM build target in Cargo.toml
- Implement wasm_bindgen exports
- Design JavaScript-friendly API surface

### 6.2. Thread E: WASM Optimization (MEDIUM)

**Dependencies:** Thread D must be completed first
**Complexity:** Medium - performance optimization and bundling
**Estimated Duration:** 1 week
**Required Skills:** WebAssembly optimization, bundle analysis

**Key Tasks:**
- Create feature flags for plugin groups
- Implement conditional compilation
- Configure dead code elimination
- Add benchmarking for WASM vs native performance

### 6.3. Thread F: JavaScript API Layer (MEDIUM)

**Dependencies:** Thread E must be completed first
**Complexity:** Medium - API design and error handling
**Estimated Duration:** 1 week
**Required Skills:** JavaScript/TypeScript, API design

**Key Tasks:**
- Implement comprehensive error handling
- Add debug logging capabilities
- Create preset system (default, aggressive, safe)
- Support configuration import/export

## 7. Online Tool Implementation Threads

### 7.1. Thread G: Web UI Foundation (EASY)

**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - standard web development with existing frameworks
**Estimated Duration:** 1 week
**Required Skills:** HTML/CSS, DaisyUI, Jekyll

**Key Tasks:**
- Add DaisyUI and Tailwind CSS to docs folder
- Create responsive layout structure
- Implement dark/light theme support
- Configure Jekyll integration

### 7.2. Thread H: Core Web Functionality (MEDIUM)

**Dependencies:** Threads F and G must be completed first
**Complexity:** Medium - integration of WASM API with web UI
**Estimated Duration:** 1 week
**Required Skills:** JavaScript, DOM manipulation, file handling

**Key Tasks:**
- Implement drag-and-drop file upload
- Build real-time SVG preview component
- Create configuration panel with plugin toggles
- Add download functionality

### 7.3. Thread I: Advanced Web Features (MEDIUM)

**Dependencies:** Thread H must be completed first
**Complexity:** Medium - advanced web functionality
**Estimated Duration:** 1-2 weeks
**Required Skills:** Advanced JavaScript, batch processing, UI/UX

**Key Tasks:**
- Multiple file upload support
- Batch optimization with progress indication
- ZIP file output for batch operations
- Plugin performance profiling

### 7.4. Thread J: Build and Deployment (EASY)

**Dependencies:** Threads F and I must be completed first
**Complexity:** Easy - standard CI/CD setup
**Estimated Duration:** 1 week
**Required Skills:** GitHub Actions, build automation

**Key Tasks:**
- Create automated WASM build scripts
- Configure GitHub Actions deployment
- Set up GitHub Pages
- Add performance monitoring

## 8. Independent Support Threads

### 8.1. Thread M: Infrastructure Improvements (EASY)

**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - incremental improvements to existing code
**Estimated Duration:** 2 weeks (can be spread over time)

**Key Areas:**
- XML entity expansion and DOCTYPE handling
- Whitespace preservation improvements
- Enhanced error reporting with context
- Namespace handling consistency

### 8.2. Thread N: Architecture Improvements (MEDIUM)

**Dependencies:** None - can be developed in parallel
**Complexity:** Medium - architectural changes
**Estimated Duration:** 2 weeks

**Key Features:**
- Visitor pattern implementation
- Preset system with inheritance
- Dynamic plugin loading support
- External plugin interface

### 8.3. Thread Q: Code Quality (EASY)

**Dependencies:** None - can be developed in parallel
**Complexity:** Easy - code cleanup
**Estimated Duration:** 1 week

**Focus Areas:**
- Fix all Clippy warnings (27 warnings)
- Add #[derive(Default)] implementations
- Fix minor formatting and style issues

## 9. Documentation and Testing Threads

### 9.1. Thread K: Documentation (EASY)

**Dependencies:** Threads B, C, F, H, I must be substantially complete
**Complexity:** Easy - documentation writing
**Estimated Duration:** 2 weeks

**Deliverables:**
- Complete JavaScript API reference
- WASM integration guide
- Configuration options documentation
- Migration guides and tutorials

### 9.2. Thread L: Testing and QA (MEDIUM)

**Dependencies:** Threads B, C, F, H, I must be substantially complete
**Complexity:** Medium - comprehensive testing across platforms
**Estimated Duration:** 2 weeks

**Coverage Areas:**
- Unit tests for WASM API
- Cross-browser compatibility testing
- Performance benchmarking
- Accessibility compliance testing

## 10. Resource Allocation Strategy

### 10.1. Team Size Recommendations

**Single Developer:**
1. Start with Thread A (critical path)
2. Continue with Thread B (plugin completion)
3. Parallelize with independent threads (M, Q, R) during breaks

**Small Team (2-3 developers):**
1. **Developer 1:** Thread A → Thread B (critical path)
2. **Developer 2:** Thread D → Thread E → Thread F (WebAssembly)
3. **Developer 3:** Independent threads (M, N, Q, R)

**Larger Team (4+ developers):**
- **Critical Path Team:** Threads A, B, P, U
- **WebAssembly Team:** Threads D, E, F, G, H, I, J
- **Infrastructure Team:** Threads M, N, O, Q, R, S, T
- **QA Team:** Threads K, L

### 10.2. Complexity-Based Assignment

**HARD Threads (Senior developers):**
- Thread A: Critical Build Fixes
- Thread B: Remaining Core Plugins

**MEDIUM Threads (Mid-level developers):**
- Threads C, E, F, H, I, L, N

**EASY Threads (Junior developers or parallel work):**
- Threads D, G, J, K, M, O, P, Q, R, S, T, U

## 11. Success Metrics and Timeline

### 11.1. Milestone Targets

**Week 1-2:** Thread A completion (Build fixes)
**Week 3-6:** Thread B completion (Core plugins)
**Week 7:** Thread P completion (Default preset alignment)
**Week 8:** Thread U completion (Documentation updates)

**Parallel Track:**
**Week 1-9:** WebAssembly threads (D → E → F → H → I → J)

### 11.2. Quality Gates

- [ ] **100% SVGO Output Compatibility:** Bit-for-bit identical output for test cases
- [ ] **Performance Benchmark:** Maintain 2-3x speed advantage over SVGO
- [ ] **Test Coverage:** 367+ tests passing with new plugin integration
- [ ] **CLI Compatibility:** All SVGO parameters and options supported

### 11.3. WebAssembly Success Metrics

- [ ] **Bundle Size:** WASM + JS < 2MB compressed
- [ ] **Load Time:** Initial load < 3 seconds on 3G
- [ ] **Browser Support:** All modern browsers (Chrome, Firefox, Safari, Edge)
- [ ] **API Compatibility:** Match SVGO JavaScript API

## 12. Risk Mitigation

### 12.1. Critical Path Risks

**Thread A Failure:** If build fixes prove too complex, consider:
- Downgrading CSS dependencies to compatible versions
- Implementing alternative CSS processing approach
- Seeking community support for dependency resolution

**Thread B Complexity:** If plugin implementation exceeds estimates:
- Implement MVP versions first
- Focus on default preset plugins only
- Consider community contributions for advanced features

### 12.2. WebAssembly Risks

**Bundle Size Concerns:**
- Implement aggressive feature flags
- Use tree-shaking and dead code elimination
- Consider streaming/lazy loading for large plugins

**Browser Compatibility:**
- Maintain JavaScript fallback options
- Implement progressive enhancement
- Test across target browser matrix

## 13. Long-term Vision

This thread-based approach enables:

1. **Parallel Development:** Multiple teams can work simultaneously
2. **Flexible Resource Allocation:** Match developer skills to thread complexity
3. **Risk Distribution:** Failure in one thread doesn't block others
4. **Community Contributions:** Clear, independent tasks for external contributors
5. **Incremental Delivery:** Each completed thread provides immediate value

The goal is not just 100% SVGO parity, but a sustainable, extensible platform that can evolve beyond the original SVGO feature set while maintaining compatibility and performance advantages.