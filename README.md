# SVGN: A Native Rust SVG Optimizer

`svgn` is a high-performance, native Rust port of `svgo` (SVG Optimizer), the popular Node.js-based tool for optimizing SVG vector graphics files. Our goal is to provide a functionally compatible, yet significantly faster and more memory-efficient SVG optimization solution for the Rust ecosystem and WebAssembly environments.

## Why SVGN?

-   **Performance**: Leveraging Rust's native capabilities for superior speed.
-   **Native Integration**: Seamlessly integrate into Rust applications, desktop tools, and CLIs.
-   **WebAssembly (WASM)**: Designed for high-performance optimization directly in browsers or other WASM environments.
-   **API Compatibility**: Strives for API compatibility with `svgo` v4.0.0 to ease migration.

## Documentation

For detailed information on `svgn`'s usage, plugins, architecture, and a comparison with `svgo`, please refer to our comprehensive documentation:

-   [**Introduction**](https://twardoch.github.io/svgn/)
-   [**Usage**](https://twardoch.github.io/svgn/usage.html)
-   [**Plugins**](https://twardoch.github.io/svgn/plugins.html)
-   [**Comparison: SVGN vs. SVGO**](https://twardoch.github.io/svgn/comparison.html)
-   [**Architecture**](https://twardoch.github.io/svgn/architecture.html)

## Development Setup

### For `svgn` (Rust Implementation)

```bash
cargo build         # Build the project
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint code
cargo run -- [args] # Run the CLI
```

### For `svgo` (JavaScript Reference - in `ref/svgo/`)

```bash
cd ref/svgo
yarn install        # Install dependencies
yarn build          # Build bundles and types
yarn test           # Run tests with coverage
yarn lint           # Run ESLint and Prettier
yarn qa             # Run all quality checks
```