---
layout: tool
title: SVG Optimizer
description: Optimize your SVG files online with our powerful WASM-powered tool
permalink: /tools/optimizer/
nav_order: 1
---

<!-- File Upload Area -->
<div class="file-drop-zone" id="file-drop-zone">
  <div class="space-y-4">
    <svg class="w-16 h-16 mx-auto text-base-content/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
    </svg>
    <div>
      <h3 class="text-lg font-semibold">Drop SVG files here</h3>
      <p class="text-sm text-base-content/70">or click to select files</p>
      <p class="text-xs text-base-content/50 mt-2">Supports .svg files up to 10MB</p>
    </div>
    <button class="btn btn-upload" id="file-select-btn">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
      </svg>
      Select Files
    </button>
  </div>
  <input type="file" id="file-input" accept=".svg,image/svg+xml" multiple style="display: none;">
</div>

<!-- Processing Status -->
<div id="processing-status" class="alert" style="display: none;">
  <div class="flex items-center gap-3">
    <div class="progress-circle"></div>
    <div>
      <h4 class="font-medium">Processing SVG files...</h4>
      <p class="text-sm">Please wait while we optimize your files.</p>
    </div>
  </div>
</div>

<!-- Results Area -->
<div id="results-area" style="display: none;">
  
  <!-- SVG Comparison -->
  <div class="svg-comparison">
    <!-- Original SVG -->
    <div class="svg-comparison-panel">
      <div class="svg-comparison-label">Original</div>
      <div class="svg-preview-container" id="original-preview">
        <div class="text-base-content/50">No SVG loaded</div>
      </div>
      <div class="mt-2">
        <div class="svg-code-container">
          <div class="svg-code-header">
            <span class="text-sm font-medium">Original Code</span>
            <div class="svg-code-actions">
              <span class="file-size-display" id="original-size-display">- KB</span>
              <button class="btn-icon" id="copy-original" title="Copy original code">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                </svg>
              </button>
            </div>
          </div>
          <div class="svg-code-display" id="original-code">
            <!-- Original SVG code will be displayed here -->
          </div>
        </div>
      </div>
    </div>

    <!-- Optimized SVG -->
    <div class="svg-comparison-panel">
      <div class="svg-comparison-label">Optimized</div>
      <div class="svg-preview-container" id="optimized-preview">
        <div class="text-base-content/50">No optimization yet</div>
      </div>
      <div class="mt-2">
        <div class="svg-code-container">
          <div class="svg-code-header">
            <span class="text-sm font-medium">Optimized Code</span>
            <div class="svg-code-actions">
              <span class="file-size-display" id="optimized-size-display">- KB</span>
              <button class="btn-icon" id="copy-optimized" title="Copy optimized code">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                </svg>
              </button>
              <button class="btn btn-download btn-sm" id="download-optimized" title="Download optimized SVG">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                </svg>
                Download
              </button>
            </div>
          </div>
          <div class="svg-code-display" id="optimized-code">
            <!-- Optimized SVG code will be displayed here -->
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Batch Download -->
  <div class="text-center mt-6" id="batch-download" style="display: none;">
    <button class="btn btn-success btn-wide" id="download-all">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
      </svg>
      Download All as ZIP
    </button>
  </div>
</div>

<!-- Error Display -->
<div id="error-display" class="alert alert-error" style="display: none;">
  <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
  </svg>
  <div>
    <h4 class="font-medium">Error</h4>
    <p class="text-sm" id="error-message">An error occurred while processing your files.</p>
  </div>
</div>

<!-- Information Section -->
<div class="mt-12 prose prose-slate max-w-none dark:prose-invert">
  <h2>About the SVG Optimizer</h2>
  
  <p>This online SVG optimizer is powered by <strong>SVGN</strong>, a high-performance Rust implementation of the popular SVGO tool. It runs entirely in your browser using WebAssembly, ensuring your files never leave your device.</p>
  
  <h3>Features</h3>
  
  <ul>
    <li><strong>Privacy-first:</strong> All processing happens locally in your browser</li>
    <li><strong>High Performance:</strong> 2-3x faster than JavaScript implementations</li>
    <li><strong>Full Compatibility:</strong> 48/53 SVGO plugins implemented</li>
    <li><strong>Real-time Preview:</strong> See optimizations applied instantly</li>
    <li><strong>Batch Processing:</strong> Upload and optimize multiple files at once</li>
    <li><strong>Customizable:</strong> Configure optimization settings to your needs</li>
  </ul>
  
  <h3>Supported Optimizations</h3>
  
  <p>The optimizer includes all major SVGO optimizations:</p>
  
  <ul>
    <li>Remove metadata, comments, and unnecessary elements</li>
    <li>Cleanup and minify attributes and values</li>
    <li>Convert colors to shorter formats</li>
    <li>Optimize path data and transforms</li>
    <li>Inline styles and merge similar elements</li>
    <li>Remove unused namespace declarations</li>
  </ul>
  
  <h3>Browser Compatibility</h3>
  
  <p>This tool requires a modern browser with WebAssembly support:</p>
  
  <ul>
    <li>Chrome 57+ / Chromium 57+</li>
    <li>Firefox 52+</li>
    <li>Safari 11+</li>
    <li>Edge 16+</li>
  </ul>
</div>