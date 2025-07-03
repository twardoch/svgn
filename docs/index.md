---
layout: default
title: Home
nav_order: 1
description: "SVGN: A Native Rust SVG Optimizer"
permalink: /
---

# SVGN: A Native Rust SVG Optimizer

## 1. Introduction

`svgn` is a high-performance, native Rust port of `svgo` (SVG Optimizer), the popular Node.js-based tool for optimizing SVG vector graphics files. While `svgo` has been instrumental in reducing SVG file sizes by removing redundant information, minifying code, and applying various optimizations, `svgn` aims to bring these benefits to a new level with the power and efficiency of Rust.

This documentation serves as a comprehensive guide to `svgn`, detailing its structure, API, and plugin system. Throughout these pages, we will draw parallels and highlight key differences with the original JavaScript `svgo` reference implementation, providing context for developers familiar with `svgo` and a clear understanding for newcomers.

### Current Status

- **Plugin Implementation**: 45/54 plugins (83%) successfully ported
- **CLI Compatibility**: Full drop-in replacement for SVGO CLI
- **Test Coverage**: 359 tests passing (100% success rate)
- **SVGO Feature Parity**: 93.75% compatibility achieved

## 2. Why SVGN?

The primary motivations behind developing `svgn` are rooted in the desire for superior performance, broader integration capabilities, and enhanced reliability for SVG optimization tasks.

-   **Unmatched Performance**: Leveraging Rust's focus on zero-cost abstractions, memory safety, and efficient concurrency, `svgn` processes SVG files significantly faster than its JavaScript counterpart. This makes it an ideal choice for:
    *   **Large-scale batch processing**: Optimizing thousands of SVG assets in build pipelines.
    *   **Real-time applications**: Where low latency SVG manipulation is critical.
    *   **Server-side rendering**: Reducing payload sizes and improving page load times.
-   **Seamless Native Integration**: As a native Rust library, `svgn` can be effortlessly integrated into a wide array of applications without the overhead of a Node.js runtime. This includes:
    *   **Desktop applications**: Building performant SVG tools.
    *   **Command-line interfaces (CLIs)**: Creating fast and efficient SVG optimization scripts.
    *   **Backend services**: Optimizing SVGs directly within Rust-based web servers or microservices.
    *   **Embedded systems**: Where resource constraints demand highly optimized code.
-   **WebAssembly (WASM) Compatibility**: `svgn` is meticulously designed with WebAssembly compilation in mind. This enables high-performance SVG optimization directly within web browsers, edge computing environments, or other WASM-compatible runtimes, unlocking new possibilities for client-side SVG processing.
-   **API Compatibility with `svgo`**: `svgn` strives for a high degree of API compatibility with `svgo` v4.0.0. This design choice significantly eases the transition for developers already familiar with `svgo`, allowing them to leverage their existing knowledge and configurations with minimal adjustments. Our goal is to ensure that if you know `svgo`, you'll feel right at home with `svgn`.

## 3. Key Features

`svgn` offers a robust set of features designed to provide comprehensive SVG optimization:

-   **Plugin-based Architecture**: A flexible and extensible system where individual optimization rules are encapsulated as plugins, allowing for fine-grained control over the optimization process.
-   **AST-based Transformations**: Utilizes an Abstract Syntax Tree (AST) for SVG manipulation, ensuring precise and reliable transformations.
-   **Comprehensive Optimization Plugins**: 45 production-ready plugins covering essential SVG optimizations, with 9 more complex plugins in development.
-   **Enhanced CLI Tool**: Full SVGO CLI compatibility plus additional features like string input, better STDIN/STDOUT handling, and precision control.
-   **Rust Library**: A powerful and efficient Rust library for programmatic integration into your projects.
-   **WASM Target**: Future-proof design with WebAssembly compilation support for browser and edge environments.
-   **Superior Performance**: Typically 2-3x faster than SVGO for common optimization tasks.

## 4. Project Structure

The `svgn` repository is organized to reflect its native Rust implementation while maintaining a clear reference to the original `svgo` structure for architectural guidance and functional parity testing:

-   **/svgn**: Contains the core Rust library and the `svgn` CLI application. This is where the primary Rust source code resides.
-   **/src**: Within the `svgn` directory, this folder holds the Rust source code for `svgn`'s core components, including the parser, optimizer, stringifier, and individual plugin implementations.
-   **/ref/svgo**: This directory contains the complete `svgo` v4.0.0 JavaScript reference implementation. It serves as a crucial benchmark for functional parity testing and provides architectural insights during the porting process.
-   **/docs**: This folder contains the project's documentation, which you are currently reading.
-   **/tests**: Comprehensive test suites for `svgn`, including integration and unit tests. Many of these tests are designed to mirror `svgo`'s test cases, ensuring that `svgn` produces identical optimization results.

## 5. Installation

To get started with `svgn`, you'll need to have Rust and Cargo (Rust's package manager) installed on your system. If you don't have them, you can install them conveniently via `rustup`, the recommended Rust toolchain installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the `rustup` installation. Once Rust and Cargo are set up, you have two primary ways to use `svgn`:

### 5.1. As a Command-Line Tool

#### From Crates.io (when published)

You can install `svgn` as a global command-line tool:

```bash
cargo install svgn
```

#### From Source

To build from the latest source:

```bash
git clone https://github.com/twardoch/svgn.git
cd svgn
cargo build --release
# The binary will be at ./target/release/svgn
```

Once installed, `svgn` can be used as a drop-in replacement for the `svgo` CLI with enhanced features.

### 5.2. As a Rust Library

To integrate `svgn` directly into your Rust project as a dependency, add it to your `Cargo.toml` file:

```toml
[dependencies]
svgn = { git = "https://github.com/twardoch/svgn.git" }
# Or when published to crates.io:
# svgn = "0.1.0"
```

After adding the dependency, you can use the `optimize` function:

```rust
use svgn::{optimize, config::Config};

let svg = "<svg>...</svg>";
let config = Config::default();
let result = optimize(svg, &config)?;
println!("Optimized: {}", result.data);
```
