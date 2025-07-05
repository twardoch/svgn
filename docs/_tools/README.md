# SVGN Tools Collection

This directory contains the Jekyll collection for SVGN web tools. Each tool is a separate Markdown file that uses the `tool` layout to provide a consistent interface.

## Structure

```
_tools/
├── README.md          # This file
├── index.md           # Tools collection landing page
└── optimizer.md       # SVG Optimizer tool
```

## Tool Layout

Tools use the special `tool` layout (`_layouts/tool.html`) which provides:

- Full-width responsive layout optimized for tool interfaces
- Integrated configuration sidebar with plugin toggles
- Statistics panel for optimization metrics
- Help and tips section
- Consistent header/footer with theme switching

## Creating New Tools

To add a new tool:

1. Create a new `.md` file in this directory
2. Use the `tool` layout in the front matter
3. Set appropriate title, description, and permalink
4. Include the tool's HTML interface in the content
5. Create corresponding JavaScript in `/assets/js/` if needed

### Example Front Matter

```yaml
---
layout: tool
title: Your Tool Name
description: Tool description for SEO and meta tags
permalink: /tools/your-tool/
nav_order: 2
---
```

## JavaScript Integration

Tools can include custom JavaScript by:

1. Creating a script file in `/assets/js/`
2. Including it in the tool layout or individual tool page
3. Following the naming convention: `tool-name.js`

The optimizer tool demonstrates this pattern with `/assets/js/svg-optimizer.js`.

## Styling

Tools inherit all styling from:

- **Base styles**: DaisyUI + Tailwind CSS
- **Tool-specific styles**: `.tool-*` classes in `/assets/css/input.css`
- **Component styles**: `.svg-preview-container`, `.config-panel`, etc.

Key style classes for tools:

- `.tool-grid` - Main responsive grid layout
- `.tool-main` - Main content area (left/top)
- `.tool-sidebar` - Configuration sidebar (right/bottom)
- `.svg-preview-container` - SVG display areas
- `.config-panel` - Configuration sections
- `.file-drop-zone` - File upload areas

## WebAssembly Integration

Tools are designed to integrate with SVGN's WebAssembly module:

1. **Thread D**: WASM build infrastructure
2. **Thread E**: WASM optimization
3. **Thread F**: JavaScript API layer

Current tools include placeholder code that will be replaced with actual WASM integration once those threads are complete.

## Responsive Design

All tools are designed mobile-first with breakpoints:

- **Mobile**: Single column, sidebar below main content
- **Tablet**: Two columns, sidebar on right
- **Desktop**: Three columns with expanded sidebar

## Accessibility

Tools follow accessibility best practices:

- Semantic HTML structure
- ARIA labels for interactive elements
- Keyboard navigation support
- High contrast theme support
- Screen reader compatible

## Testing

Test tools locally by:

1. Running the Jekyll development server: `bundle exec jekyll serve`
2. Opening `http://localhost:4000/tools/`
3. Testing responsive layouts at different screen sizes
4. Verifying dark/light theme switching

## Future Tools

Planned tools (see Thread I for details):

- **SVG Analyzer**: Code analysis and optimization suggestions
- **SVG Converter**: Format conversion and export options
- **SVG Validator**: Standards compliance checking
- **Plugin Tester**: Individual plugin testing interface
- **Batch Processor**: Advanced batch operations
- **Code Generator**: HTML/CSS embedding assistance

## Dependencies

Tools depend on:

- Jekyll collections (configured in `_config.yml`)
- DaisyUI + Tailwind CSS for styling
- Custom JavaScript for interactivity
- WebAssembly module (when available)
- Modern browser with WASM support