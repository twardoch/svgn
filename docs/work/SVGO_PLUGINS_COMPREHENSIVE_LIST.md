# SVGO Plugins Comprehensive List

This document provides a complete analysis of all SVGO plugins from the reference implementation, including their descriptions, default preset status, and parameters.

## Plugin Classification

### Default Preset Plugins (35 plugins)
These plugins are included in the SVGO default preset and are enabled by default:

| Plugin Name | Description | Parameters |
|-------------|-------------|------------|
| **removeDoctype** | removes doctype declarations | None |
| **removeXMLProcInst** | removes XML processing instructions | None |
| **removeComments** | removes comments | `preservePatterns` (RegExp[] or false) |
| **removeDeprecatedAttrs** | removes deprecated attributes | None |
| **removeMetadata** | removes `<metadata>` elements | None |
| **removeEditorsNSData** | removes editor-specific namespaces, elements, and attributes | None |
| **cleanupAttrs** | cleanups attributes from newlines, trailing and repeating spaces | `newlines` (boolean), `trim` (boolean), `spaces` (boolean) |
| **mergeStyles** | merge multiple style elements into one | None |
| **inlineStyles** | inline styles (additional options) | `onlyMatchedOnce` (boolean), `removeMatchedSelectors` (boolean), `useMqs` (string[]), `usePseudos` (string[]) |
| **minifyStyles** | minifies styles and removes unused styles | `restructure` (boolean), `forceMediaMerge` (boolean), `comments` (string or boolean), `usage` (boolean or Usage object) |
| **cleanupIds** | removes unused IDs and minifies used | `remove` (boolean), `minify` (boolean), `preserve` (string[]), `preservePrefixes` (string[]), `force` (boolean) |
| **removeUselessDefs** | removes elements in `<defs>` without an `id` | None |
| **cleanupNumericValues** | rounds numeric values to the fixed precision, removes default "px" units | `floatPrecision` (number), `leadingZero` (boolean), `defaultPx` (boolean), `convertToPx` (boolean) |
| **convertColors** | converts colors: rgb() to #rrggbb and #rrggbb to #rgb | `currentColor` (boolean/string/RegExp), `names2hex` (boolean), `rgb2hex` (boolean), `convertCase` (string), `shorthex` (boolean), `shortname` (boolean) |
| **removeUnknownsAndDefaults** | removes unknown elements content and attributes, removes attrs with default values | `unknownContent` (boolean), `unknownAttrs` (boolean), `defaultAttrs` (boolean), `defaultMarkupDeclarations` (boolean), `uselessOverrides` (boolean), `keepDataAttrs` (boolean), `keepAriaAttrs` (boolean), `keepRoleAttr` (boolean) |
| **removeNonInheritableGroupAttrs** | removes non-inheritable group's "presentation" attributes | None |
| **removeUselessStrokeAndFill** | removes useless `stroke` and `fill` attributes | None |
| **cleanupEnableBackground** | remove or cleanup enable-background attribute when possible | None |
| **removeHiddenElems** | removes hidden elements (`display="none"` or `visibility="hidden"`) | None |
| **removeEmptyText** | removes empty text elements | None |
| **convertShapeToPath** | converts basic shapes to more compact path form | `convertArcs` (boolean), `floatPrecision` (number) |
| **convertEllipseToCircle** | converts non-eccentric `<ellipse>`s to `<circle>`s | None |
| **moveElemsAttrsToGroup** | Move common attributes of group children to the group | None |
| **moveGroupAttrsToElems** | moves some group attributes to the content elements | None |
| **collapseGroups** | collapses useless groups | None |
| **convertPathData** | optimizes path data: writes in shorter form, applies transformations | Very extensive parameter set (see details below) |
| **convertTransform** | collapses multiple transformations and optimizes it | `convertToShorts` (boolean), `degPrecision` (number), `floatPrecision` (number), `transformPrecision` (number), `matrixToTransform` (boolean), `shortTranslate` (boolean), `shortScale` (boolean), `shortRotate` (boolean), `removeUseless` (boolean), `collapseIntoOne` (boolean), `leadingZero` (boolean), `negativeExtraSpace` (boolean) |
| **removeEmptyAttrs** | removes empty attributes | None |
| **removeEmptyContainers** | removes empty container elements | None |
| **mergePaths** | merges multiple paths in one if possible | `force` (boolean), `floatPrecision` (number), `noSpaceAfterFlags` (boolean) |
| **removeUnusedNS** | removes unused namespace declarations | None |
| **sortAttrs** | Sort element attributes for better compression | `order` (string[]), `xmlnsOrder` (string) |
| **sortDefsChildren** | Sorts children of `<defs>` to improve compression | None |
| **removeDesc** | removes `<desc>` | `removeAny` (boolean) |

### Non-Default Plugins (18 plugins)
These plugins are available but not included in the default preset:

| Plugin Name | Description | Parameters |
|-------------|-------------|------------|
| **addAttributesToSVGElement** | adds attributes to an outer `<svg>` element | `attribute` (string or object), `attributes` (array) |
| **addClassesToSVGElement** | adds classnames to an outer `<svg>` element | `className` (string or function), `classNames` (array) |
| **applyTransforms** | Apply transformation(s) to the Path data | `transformPrecision` (number), `applyTransformsStroked` (boolean) |
| **cleanupListOfValues** | rounds list of values to the fixed precision | `floatPrecision` (number), `leadingZero` (boolean), `defaultPx` (boolean), `convertToPx` (boolean) |
| **convertOneStopGradients** | converts one-stop (single color) gradients to a plain color | None |
| **convertStyleToAttrs** | converts style to attributes | `keepImportant` (boolean) |
| **prefixIds** | prefix IDs | `prefix` (boolean/string/function), `delim` (string), `prefixIds` (boolean), `prefixClassNames` (boolean) |
| **removeAttributesBySelector** | removes attributes of elements that match a CSS selector | Parameters vary |
| **removeAttrs** | removes attributes by name | Parameters vary |
| **removeDimensions** | removes `width` and `height` attributes and preserves the `viewBox` | Parameters vary |
| **removeElementsByAttr** | removes elements by ID or class name | Parameters vary |
| **removeOffCanvasPaths** | removes elements that are drawn outside of the `viewBox` | Parameters vary |
| **removeRasterImages** | removes raster images | Parameters vary |
| **removeScripts** | removes `<script>` elements | Parameters vary |
| **removeStyleElement** | removes `<style>` elements | Parameters vary |
| **removeTitle** | removes `<title>` elements | `removeAny` (boolean) |
| **removeViewBox** | removes the `viewBox` attribute when possible | Parameters vary |
| **removeXlink** | removes `xlink` attributes | Parameters vary |
| **removeXMLNS** | removes the `xmlns` attribute from the root `<svg>` element | Parameters vary |
| **reusePaths** | Finds `<path>` elements with the same d, fill, and stroke, and converts them to `<use>` elements | None |

### Special Files
- **preset-default.js** - Defines the default plugin preset
- **_collections.js** - Shared collections and constants
- **_path.js** - Path data utilities
- **_transforms.js** - Transform utilities

## Detailed Parameter Analysis

### convertPathData Plugin Parameters
This is the most complex plugin with extensive configuration options:

```typescript
{
  applyTransforms?: boolean;
  applyTransformsStroked?: boolean;
  makeArcs?: {
    threshold: number;
    tolerance: number;
  };
  straightCurves?: boolean;
  convertToQ?: boolean;
  lineShorthands?: boolean;
  convertToZ?: boolean;
  curveSmoothShorthands?: boolean;
  floatPrecision?: number | false;
  transformPrecision?: number;
  smartArcRounding?: boolean;
  removeUseless?: boolean;
  collapseRepeated?: boolean;
  utilizeAbsolute?: boolean;
  leadingZero?: boolean;
  negativeExtraSpace?: boolean;
  noSpaceAfterFlags?: boolean;
  forceAbsolutePath?: boolean;
}
```

## Plugin Architecture Insights

1. **Plugin Structure**: Each plugin exports `name`, `description`, and `fn` (the main function)
2. **Parameter Types**: Most plugins use TypeScript interfaces for type safety
3. **Visitor Pattern**: Plugins use a visitor pattern with `enter` and `exit` hooks for elements
4. **Shared Utilities**: Common functionality is shared through utility modules
5. **Style Processing**: Several plugins handle CSS style processing and inlining
6. **Path Optimization**: Multiple plugins focus on SVG path data optimization

## Plugin Dependencies and Interactions

- **Style Processing Chain**: mergeStyles → inlineStyles → minifyStyles
- **Transform Chain**: convertTransform → applyTransforms (if enabled)
- **Cleanup Chain**: Various remove* plugins work together
- **Path Processing**: convertShapeToPath → convertPathData → mergePaths

## Usage Patterns

1. **Default Preset**: Use for general SVG optimization
2. **Aggressive Optimization**: Enable additional plugins like reusePaths, convertOneStopGradients
3. **Accessibility Preservation**: Carefully configure removeDesc, removeTitle with appropriate parameters
4. **Development vs Production**: Different configurations for different environments

This comprehensive list provides the foundation for implementing equivalent functionality in SVGN (the Rust port).