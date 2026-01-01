### Critical Files
1. `crates/astra-gui/src/content.rs` - Add wrapping/line-height config
2. `crates/astra-gui-text/src/lib.rs` - Multi-line shaping with cosmic-text
3. `crates/astra-gui/src/measure.rs` - Width-constrained measurement
4. `crates/astra-gui/src/node.rs` - Pass width constraints to measurement
5. `crates/astra-gui-wgpu/src/lib.rs` - Render multiple lines
6. `crates/astra-gui/src/primitives.rs` - TextShape updates

## Implementation Strategy

### Approach
1. Start with core data structures (enables compilation errors to guide changes)
2. Implement multi-line shaping in text engine (testable in isolation)
3. Wire up layout measurement with width constraints
4. Update rendering to handle multiple lines
5. Test with examples showing newlines and wrapping

### Backward Compatibility
- Keep existing `shape_line()` API for single-line use cases
- Provide sensible defaults for new fields
- Existing code may need minor updates to use builder pattern or `..Default::default()`

## Next Steps
Awaiting your answers to the questions above to finalize the implementation details.
#### 7. Output Collection

**File**: `crates/astra-gui/src/output.rs`

Around line 408-414, update to include new fields:
```rust
let mut text_shape = crate::primitives::TextShape::new(content_rect, text_content);
let scaled_font_size = text_content
    .font_size
    .try_resolve_with_scale(width, scale_factor)
    .unwrap_or(16.0);
text_shape.font_size = Size::lpx(scaled_font_size);
text_shape.wrap = text_content.wrap;                              // NEW
text_shape.line_height_multiplier = text_content.line_height_multiplier;  // NEW
```

#### 8. Update Examples

All examples using `TextContent` need updates:
- `examples/text.rs`
- `examples/alignment.rs`
- `examples/catppuccin.rs`
- Any others that create text nodes

Update to use builder pattern or add new fields with defaults.

#### 9. Version Bump

Update `Cargo.toml` files:
- `crates/astra-gui/Cargo.toml`
- `crates/astra-gui-text/Cargo.toml`
- `crates/astra-gui-wgpu/Cargo.toml`

Bump minor version (e.g., 0.1.0 → 0.2.0) to reflect breaking API changes.

### Critical Files Summary

1. **`crates/astra-gui/src/content.rs`** - Add `Wrap` enum, extend `TextContent`
2. **`crates/astra-gui-text/src/lib.rs`** - Implement multi-line shaping with cosmic-text
3. **`crates/astra-gui/src/measure.rs`** - Extend `MeasureTextRequest` with width/wrap/line-height
4. **`crates/astra-gui/src/node.rs`** - Pass `available_width` to text measurement
5. **`crates/astra-gui-wgpu/src/lib.rs`** - Render multiple lines, update cache
6. **`crates/astra-gui/src/primitives.rs`** - Extend `TextShape` with new fields
7. **`crates/astra-gui/src/output.rs`** - Pass new fields to `TextShape`
8. **All examples** - Update to use new `TextContent` API
9. **`Cargo.toml` files** - Version bump

## Implementation Order

1. **Phase 1: Data Structures**
   - Add `Wrap` enum to `content.rs`
   - Extend `TextContent`, `TextShape`, `MeasureTextRequest`
   - Compilation errors will guide remaining updates

2. **Phase 2: Text Engine**
   - Add `ShapedText` and `ShapeTextRequest` structures
   - Implement `shape_text()` in `CosmicEngine`
   - Update `measure_text()` to use multi-line shaping

3. **Phase 3: Layout Integration**
   - Pass `available_width` to measurement in `node.rs`

4. **Phase 4: Rendering**
   - Update WGPU renderer to call `shape_text()` and iterate lines
   - Update shape cache

5. **Phase 5: Cleanup**
   - Update all examples
   - Bump versions
   - Test with various wrapping scenarios

## Testing Strategy

Create/update examples to demonstrate:
- Explicit newlines (`"Line 1\nLine 2"`)
- Word wrapping with constrained width
- `Wrap::None` (overflow behavior)
- `Wrap::Glyph` (character wrap)
- Different alignments (h_align per-line, v_align for block)
- Line height variations
- FitContent with multi-line text

## Expected Behavior

- **FitContent width**: Measures full text width (single line or longest wrapped line)
- **Constrained width**: Text wraps according to `wrap` mode
- **Newlines**: Always create new lines regardless of wrap mode
- **h_align**: Applied per-line (Left/Center/Right justify each line)
- **v_align**: Applied to entire text block within bounding rect
- **Default wrapping**: `Wrap::Word` provides good UX for most cases
