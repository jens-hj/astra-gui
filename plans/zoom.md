# Zoom Feature Implementation Plan

## User Requirements Summary

### Two Zoom Types Required:

1. **Scale Zoom (Browser-style)**
   - Uniform enlargement of everything (text, borders, spacing)
   - No specific zoom center - just scales everything proportionally
   - Layout maintains same relative sizes, just larger

2. **Pan Zoom (Camera-style)** 
   - Zooms into a specific point (mouse cursor position)
   - Like pinch-to-zoom on mobile
   - Allows focusing on a specific area

### API Design:
- Per-node property: `Zoom` enum with `Enabled` and `Disabled` variants
- Default: `Disabled`
- When enabled on root node → affects all children (global effect)
- Allows per-container zoom if needed

### Constraints:
- Layout maintains exact same relative sizes
- Text must render crisp at all zoom levels (re-rasterize glyphs)
- Both zoom types should work together

## Current System Understanding

From exploration:
- Transform system: Translation + Rotation (no scale yet)
- Transforms applied post-layout in shaders
- Three rendering paths: SDF (primary), Mesh, Text
- Screen-space coordinate system with Y-down
- Hit-testing uses inverse transforms

## Implementation Approach

[To be filled after Plan agent analysis]

## Critical Files

[To be identified by Plan agent]
