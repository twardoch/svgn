# SVGN: A Native Rust SVG Optimizer

`svgn` is a high-performance, native Rust port of `svgo` (SVG Optimizer), the popular Node.js-based tool for optimizing SVG vector graphics files. Our goal is to provide a functionally compatible, yet significantly faster and more memory-efficient SVG optimization solution for the Rust ecosystem and WebAssembly environments.

**`svgn` is 12x faster than `svgo` on `npx` and 7x than `svgo` on `bunx`**

## Current Status

- **Plugin Implementation**: 45/54 plugins (83%) implemented
- **CLI Compatibility**: Full SVGO command-line compatibility achieved
- **Test Coverage**: 359 tests passing (100% success rate)
- **SVGO Feature Parity**: 93.75% compatibility with SVGO v4.0.0

## Why SVGN?

- **Performance**: Leveraging Rust's native capabilities for superior speed.
- **Native Integration**: Seamlessly integrate into Rust applications, desktop tools, and CLIs.
- **WebAssembly (WASM)**: Designed for high-performance optimization directly in browsers or other WASM environments.
- **API Compatibility**: Strives for API compatibility with `svgo` v4.0.0 to ease migration.

## Documentation

For detailed information on `svgn`'s usage, plugins, architecture, and a comparison with `svgo`, please refer to our comprehensive documentation:

- [**Introduction**](https://twardoch.github.io/svgn/)
- [**Usage**](https://twardoch.github.io/svgn/usage.html)
- [**Plugins**](https://twardoch.github.io/svgn/plugins.html)
- [**Comparison: SVGN vs. SVGO**](https://twardoch.github.io/svgn/comparison.html)
- [**Architecture**](https://twardoch.github.io/svgn/architecture.html)

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

## Installation

### From Source

```bash
git clone https://github.com/twardoch/svgn.git
cd svgn
cargo build --release
```

The binary will be available at `./target/release/svgn`.

## CLI Usage

`svgn` now provides full SVGO CLI compatibility with enhanced features:

### Basic Usage

```bash
# Optimize a single file
svgn input.svg -o output.svg

# Use STDIN/STDOUT (default behavior)
cat input.svg | svgn > output.svg

# Optimize a string directly
svgn -s '<svg>...</svg>'

# Process multiple files
svgn file1.svg file2.svg file3.svg

# Recursive folder processing
svgn -f ./icons -r --exclude "node_modules|dist"
```

### Advanced Features

```bash
# Control numeric precision
svgn input.svg -o output.svg -p 3

# Pretty print with custom indentation
svgn input.svg -o output.svg --pretty --indent 4

# Control line endings
svgn input.svg -o output.svg --eol crlf --final-newline

# Show all available plugins
svgn --show-plugins

# Disable color output
svgn --no-color input.svg -o output.svg
```

## Examples

> `cat testdata/complex.svg`

```
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300">
  <defs>
    <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
      <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
    </linearGradient>
    <filter id="dropshadow" x="0" y="0" width="120%" height="120%">
      <feDropShadow dx="2" dy="2" stdDeviation="3"/>
    </filter>
  </defs>
  <rect x="50" y="50" width="200" height="100" fill="url(#grad1)" filter="url(#dropshadow)"/>
  <circle cx="300" cy="100" r="40" fill="blue" opacity="0.7"/>
  <text x="200" y="200" font-family="Arial" font-size="16" text-anchor="middle">Complex SVG Test</text>
  <path d="M 100 250 Q 200 200 300 250 T 350 225" stroke="green" stroke-width="3" fill="none"/>
</svg>
```

> `cat testdata/complex.svg | svgn` # No arguments needed for STDIN/STDOUT

```
<svg xmlns="http://www.w3.org/2000/svg" height="300" width="400"><defs><linearGradient id="c" x1="0%" x2="100%" y1="0%" y2="0%"><stop offset="0%" stop-color="#FF0" stop-opacity="1"/><stop offset="100%" stop-color="#F00" stop-opacity="1"/></linearGradient><filter height="120%" id="b" width="120%" x="0" y="0"/></defs><rect fill="url(#c)" filter="url(#b)" height="100" width="200" x="50" y="50"/><circle cx="300" cy="100" fill="#00f" opacity="0.7" r="40"/><text font-family="Arial" font-size="16" text-anchor="middle" x="200" y="200">Complex SVG Test</text><path d="M 100 250 Q 200 200 300 250 T 350 225" fill="none" stroke="green" stroke-width="3"/></svg>
Optimized: 862 B → 655 B (24.0% reduction)
```

> `cat testdata/complex.svg | npx svgo` # SVGO comparison

```
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300"><defs><linearGradient id="a" x1="0%" x2="100%" y1="0%" y2="0%"><stop offset="0%" style="stop-color:#ff0;stop-opacity:1"/><stop offset="100%" style="stop-color:red;stop-opacity:1"/></linearGradient><filter id="b" width="120%" height="120%" x="0" y="0"><feDropShadow dx="2" dy="2" stdDeviation="3"/></filter></defs><path fill="url(#a)" d="M50 50h200v100H50z" filter="url(#b)"/><circle cx="300" cy="100" r="40" fill="#00f" opacity=".7"/><text x="200" y="200" font-family="Arial" font-size="16" text-anchor="middle">Complex SVG Test</text><path fill="none" stroke="green" stroke-width="3" d="M100 250q100-50 200 0t50-25"/></svg>
Done in 26 ms!
0.842 KiB - 20.1% = 0.673 KiB
```
