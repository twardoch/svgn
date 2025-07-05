---
layout: default
title: SVGN Tools
description: Online tools for SVG optimization and manipulation
permalink: /tools/
nav_order: 6
---

# SVGN Tools

A collection of web-based tools for SVG optimization and manipulation, all powered by our high-performance Rust implementation.

## Available Tools

### [SVG Optimizer](/tools/optimizer/)
Our flagship online SVG optimization tool. Upload your SVG files and optimize them directly in your browser using WebAssembly technology.

**Features:**
- ✅ **Privacy-first**: All processing happens locally in your browser
- ⚡ **High Performance**: 2-3x faster than JavaScript implementations  
- 🎛️ **Configurable**: Customize optimization settings to your needs
- 📊 **Real-time Preview**: See optimizations applied instantly
- 📦 **Batch Processing**: Upload and optimize multiple files at once
- 🌐 **Cross-platform**: Works on all modern browsers with WebAssembly support

[**Launch Optimizer →**](/tools/optimizer/){: .btn .btn-primary .btn-lg}

---

## Coming Soon

### SVG Analyzer
Analyze SVG files for potential optimization opportunities and structural issues.

### SVG Converter  
Convert between different SVG formats and export to other vector formats.

### SVG Validator
Validate SVG files against web standards and accessibility guidelines.

### Plugin Tester
Test individual SVGO plugins to understand their specific optimizations.

---

## Technical Details

All tools are built using:

- **Backend**: SVGN (Rust) compiled to WebAssembly
- **Frontend**: Modern JavaScript with progressive enhancement
- **UI**: DaisyUI + Tailwind CSS for responsive design
- **Privacy**: No server uploads - everything runs in your browser

## Browser Requirements

These tools require a modern browser with WebAssembly support:

- **Chrome**: 57+ / Chromium 57+
- **Firefox**: 52+
- **Safari**: 11+
- **Edge**: 16+

## Performance

WebAssembly allows us to achieve near-native performance in the browser:

- **File Processing**: 2-3x faster than JavaScript implementations
- **Memory Usage**: Efficient memory management with predictable performance
- **Bundle Size**: Optimized WASM bundles under 2MB compressed
- **Load Time**: Fast initialization and caching for repeated use

## Open Source

All tools are open source and available on GitHub:

- [SVGN Core](https://github.com/twardoch/svgn) - The Rust implementation
- [Web Tools](https://github.com/twardoch/svgn/tree/main/docs) - Frontend code for these tools

## Feedback

Found a bug or have a feature request? Please [open an issue](https://github.com/twardoch/svgn/issues) on GitHub.

---

*Last updated: {{ site.time | date: '%B %d, %Y' }}*