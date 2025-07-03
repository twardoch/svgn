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

This document describes `svgn`'s structure, API, and plugins, drawing parallels and highlighting differences with the original JavaScript `svgo` reference implementation.

## 2. Why SVGN?

The primary motivations behind developing `svgn` are:

-   **Performance**: Rust's focus on performance and memory safety allows `svgn` to process SVG files significantly faster than its JavaScript counterpart, making it ideal for large-scale optimization tasks or real-time applications.
-   **Native Integration**: As a native Rust library, `svgn` can be seamlessly integrated into other Rust applications, desktop tools, and command-line interfaces without the overhead of a Node.js runtime.
-   **WebAssembly (WASM) Compatibility**: `svgn` is designed with WebAssembly compilation in mind, enabling high-performance SVG optimization directly within web browsers or other WASM environments.
-   **API Compatibility**: `svgn` strives for API compatibility with `svgo` v4.0.0, making it easier for developers familiar with `svgo` to transition to `svgn`.

## 3. Project Structure

The `svgn` repository is organized to reflect its native Rust implementation while referencing the original `svgo` structure:

-   **/svgn**: Contains the core Rust library and CLI application.
-   **/src**: Rust source code for `svgn`, including the parser, optimizer, stringifier, and plugin implementations.
-   **/ref/svgo**: The complete `svgo` v4.0.0 JavaScript reference implementation, used for functional parity testing and architectural guidance.
-   **/docs**: This documentation.
-   **/tests**: Comprehensive test suites for `svgn`, including integration and unit tests, often mirroring `svgo`'s test cases.

## 4. Installation

To get started with `svgn`, you'll need to have Rust and Cargo installed. If you don't have them, you can install them via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once Rust is set up, you can install `svgn` as a command-line tool:

```bash
cargo install svgn
```

Or, if you want to use `svgn` as a library in your Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
svgn = "0.1.0" # Use the latest version
```