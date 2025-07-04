# SVGO vs SVGN Plugin Comparison

## Executive Summary

- **SVGO Plugins**: 54 total plugins documented
- **SVGN Plugins**: 47 plugin files implemented
- **Fully Compatible**: ~43 plugins (79.6%)
- **Stub/Incomplete**: 1 plugin (convertPathData)
- **Missing in SVGN**: 10 plugins (18.5%)
- **SVGN-only**: 2 plugins (removeUselessTransforms, removeStyleElement)

## Detailed Comparison Table

| SVGO Plugin Name | SVGN Status | Implementation Notes |
|------------------|-------------|---------------------|
| addAttributesToSVGElement | ✅ Complete | Fully implemented |
| addClassesToSVGElement | ✅ Complete | Fully implemented |
| cleanupAttrs | ✅ Complete | Fully implemented |
| cleanupEnableBackground | ✅ Complete | Implemented but not in default preset |
| cleanupIds | ✅ Complete | Fully implemented |
| cleanupListOfValues | ✅ Complete | Implemented but not in default preset |
| cleanupNumericValues | ✅ Complete | Implemented but not in default preset |
| collapseGroups | ✅ Complete | Fully implemented |
| convertColors | ✅ Complete | Fully implemented |
| convertEllipseToCircle | ✅ Complete | Fully implemented |
| convertOneStopGradients | ✅ Complete | Fully implemented |
| convertPathData | ❌ Stub | Returns error: "not yet implemented" |
| convertShapeToPath | ✅ Complete | Fully implemented |
| convertStyleToAttrs | ✅ Complete | Fully implemented |
| convertTransform | ❌ Missing | Not implemented (requires matrix math) |
| inlineStyles | ❌ Missing | Not implemented (requires CSS parsing) |
| mergePaths | ❌ Missing | Not implemented (requires path analysis) |
| mergeStyles | ✅ Complete | Fully implemented |
| minifyStyles | ✅ Complete | Fully implemented |
| moveElemsAttrsToGroup | ❌ Missing | Not implemented (requires DOM analysis) |
| moveGroupAttrsToElems | ❌ Missing | Not implemented (requires DOM analysis) |
| prefixIds | ✅ Complete | Fully implemented |
| preset-default | N/A | Not a plugin, but a preset configuration |
| removeAttributesBySelector | ⚠️ Complete | Implemented but commented out in registry |
| removeAttrs | ✅ Complete | Fully implemented |
| removeComments | ✅ Complete | Fully implemented |
| removeDeprecatedAttrs | ✅ Complete | Fully implemented |
| removeDesc | ✅ Complete | Fully implemented |
| removeDimensions | ✅ Complete | Fully implemented |
| removeDoctype | ✅ Complete | Fully implemented |
| removeEditorsNSData | ✅ Complete | Fully implemented |
| removeElementsByAttr | ✅ Complete | Fully implemented |
| removeEmptyAttrs | ✅ Complete | Fully implemented |
| removeEmptyContainers | ✅ Complete | Fully implemented |
| removeEmptyText | ✅ Complete | Fully implemented |
| removeHiddenElems | ✅ Complete | Fully implemented |
| removeMetadata | ✅ Complete | Fully implemented |
| removeNonInheritableGroupAttrs | ✅ Complete | Implemented but not in default preset |
| removeOffCanvasPaths | ✅ Complete | Fully implemented |
| removeRasterImages | ✅ Complete | Fully implemented |
| removeScripts | ✅ Complete | Fully implemented |
| removeStyleElement | ✅ SVGN-only | Not in SVGO, SVGN-specific plugin |
| removeTitle | ✅ Complete | Fully implemented |
| removeUnknownsAndDefaults | ✅ Complete | Fully implemented |
| removeUnusedNS | ✅ Complete | Fully implemented |
| removeUselessDefs | ✅ Complete | Fully implemented |
| removeUselessStrokeAndFill | ❌ Missing | Not implemented |
| removeUselessTransforms | ✅ SVGN-only | Not in SVGO, SVGN-specific plugin |
| removeViewBox | ✅ Complete | Fully implemented |
| removeXlink | ✅ Complete | Fully implemented |
| removeXMLNS | ✅ Complete | Fully implemented |
| removeXMLProcInst | ✅ Complete | Fully implemented |
| reusePaths | ❌ Missing | Not implemented |
| sortAttrs | ✅ Complete | Fully implemented |
| sortDefsChildren | ✅ Complete | Fully implemented |

## Missing SVGO Plugins in SVGN

1. **convertTransform** - Requires matrix math implementation
2. **inlineStyles** - Requires CSS parsing capabilities
3. **mergePaths** - Requires path analysis algorithms
4. **moveElemsAttrsToGroup** - Requires DOM analysis
5. **moveGroupAttrsToElems** - Requires DOM analysis
6. **removeUselessStrokeAndFill** - Not implemented
7. **reusePaths** - Not implemented

## SVGN-Specific Plugins (Not in SVGO)

1. **removeStyleElement** - Removes `<style>` elements
2. **removeUselessTransforms** - Removes identity transforms

## Default Preset Differences

### SVGO Default Preset (33 plugins)
cleanupAttrs, cleanupEnableBackground, cleanupIds, cleanupListOfValues, cleanupNumericValues, collapseGroups, convertColors, convertPathData, convertShapeToPath, convertStyleToAttrs, convertTransform, inlineStyles, mergePaths, mergeStyles, minifyStyles, moveElemsAttrsToGroup, moveGroupAttrsToElems, removeComments, removeDesc, removeDoctype, removeEditorsNSData, removeEmptyAttrs, removeEmptyContainers, removeEmptyText, removeHiddenElems, removeMetadata, removeNonInheritableGroupAttrs, removeTitle, removeUnknownsAndDefaults, removeUnusedNS, removeUselessDefs, removeUselessStrokeAndFill, removeXMLProcInst, sortAttrs, sortDefsChildren

### SVGN Default Preset (21 plugins)
removeComments, removeMetadata, removeTitle, removeDesc, removeDoctype, removeXMLProcInst, removeEditorsNSData, cleanupAttrs, removeEmptyAttrs, removeUnknownsAndDefaults, removeUnusedNS, removeUselessDefs, cleanupIds, minifyStyles, convertStyleToAttrs, convertColors, removeEmptyText, removeEmptyContainers, collapseGroups, removeUselessTransforms, sortAttrs

### Notable Differences in Default Presets
- SVGN excludes several cleanup plugins that SVGO includes by default
- SVGN includes removeUselessTransforms which SVGO doesn't have
- SVGN excludes the problematic convertPathData plugin from defaults

## Implementation Quality Notes

1. **convertPathData** is the only stub implementation that will throw an error
2. **removeAttributesBySelector** is implemented but disabled due to CSS selector parsing issues
3. Most other plugins appear to be fully functional implementations
4. SVGN adds two useful plugins not found in SVGO

## Recommendations for Full SVGO Compatibility

1. **High Priority**: Implement convertPathData (currently throws errors)
2. **Medium Priority**: Implement the 7 missing plugins for feature parity
3. **Low Priority**: Fix removeAttributesBySelector CSS parsing issue
4. **Configuration**: Consider aligning default presets more closely with SVGO