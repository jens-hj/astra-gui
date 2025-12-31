# Dynamic Glyph Atlas Resizing Implementation Plan

## Problem Statement

At high zoom levels (~250-360%), text glyphs disappear because the fixed-size glyph atlas (currently 4096x4096) fills up. When `atlas.insert()` returns `AtlasInsert::Full`, glyphs are silently skipped, causing text to vanish progressively as zoom increases.

**Root Cause**: 
- At high zoom, glyphs are rasterized at large font sizes (e.g., 200px+ at 300% zoom)
- Larger glyphs consume more atlas space
- Fixed atlas size eventually exhausts available space
- No dynamic resizing mechanism exists

**User Requirement**: Implement proper dynamic atlas resizing that detects when the atlas is full and automatically grows it, preventing text from ever disappearing.

## Current System Understanding

### Atlas Implementation (`crates/astra-gui-wgpu/src/text/atlas.rs`)

**Data Structures**:
```rust
pub struct GlyphAtlas {
    width: u32,
    height: u32,
    padding_px: u32,
    shelves: Vec<Shelf>,        // Row-based shelf packer
    next_shelf_y: u32,
    cache: HashMap<GlyphKey, PlacedGlyph>,
}

pub struct PlacedGlyph {
    rect_px: AtlasRectPx,  // Absolute pixel coordinates in atlas
    padding_px: u32,
    uv: UvRect,            // Normalized UV coords [0,1] - DEPENDS ON ATLAS SIZE
}
```

**Key Insight**: UV coordinates are calculated as `pixel_pos / atlas_dimensions`, so they become invalid when atlas is resized and must be recalculated.

### Rendering Pipeline (`crates/astra-gui-wgpu/src/lib.rs`)

**Atlas Full Detection** (line ~1027):
```rust
text::atlas::AtlasInsert::Full => {
    eprintln!("WARNING: Glyph atlas full! ...");
    None  // Glyph is silently skipped
}
```

**Critical Constraint**: This detection happens INSIDE the render loop, while a render pass is active. We cannot use `encoder.copy_texture_to_texture()` here, but we CAN use `queue.write_texture()`.

**Text Rendering Flow**:
1. Before render pass: All preparatory work
2. `encoder.begin_render_pass()` - RENDER PASS STARTS
3. For each text shape:
   - Shape text using text engine
   - For each glyph, check cache
   - If cache miss: rasterize and call `atlas.insert()` ← Full is detected here
   - Upload bitmap via `queue.write_texture()` if placed
4. Render pass ends

### WGPU Patterns in Codebase

**Exponential Buffer Growth Pattern**:
```rust
capacity = (current_size * 2).next_power_of_two();
```

Used consistently for vertex buffers, index buffers throughout the codebase.

## Implementation Approach: Two-Stage Hybrid Resize Strategy

### Stage 1: Proactive Pre-Frame Estimation (Before Render Pass)

**Goal**: Detect potential atlas exhaustion BEFORE the render pass starts, allowing us to resize preemptively.

**Approach**:
- At the start of `render()`, before `encoder.begin_render_pass()`
- Count number of text shapes to be rendered
- Estimate potential new glyphs (conservative heuristic: unique glyphs per shape)
- Calculate estimated space needed vs available
- If threshold exceeded (e.g., >70% utilization predicted), resize immediately

**Benefits**:
- Prevents most atlas Full situations
- Can use encoder operations if needed
- No visible glitches

### Stage 2: Reactive Deferred Resize (Fallback for Unexpected Cases)

**Goal**: Handle cases where Stage 1 estimation was insufficient or unexpected large glyphs appear.

**Approach**:
- When `AtlasInsert::Full` detected during render pass
- Set flag: `atlas_needs_resize = true`
- Continue current frame (glyphs will be skipped this frame)
- Next frame: resize atlas BEFORE render pass starts
- Re-insert all cached glyphs with new UV coordinates

**Benefits**:
- Graceful degradation (1 frame glitch vs crash)
- Works within render pass constraints
- Guarantees eventual success

## Detailed Implementation Steps

### Step 1: Add Atlas Introspection Methods

**File**: `crates/astra-gui-wgpu/src/text/atlas.rs`

Add methods to `GlyphAtlas`:

```rust
impl GlyphAtlas {
    /// Get current atlas utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f32 {
        if self.height == 0 {
            return 0.0;
        }
        (self.next_shelf_y as f32) / (self.height as f32)
    }
    
    /// Get number of cached glyphs
    pub fn glyph_count(&self) -> usize {
        self.cache.len()
    }
    
    /// Get current dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Get iterator over all cached glyphs (for migration during resize)
    pub fn cached_glyphs(&self) -> impl Iterator<Item = (&GlyphKey, &PlacedGlyph)> {
        self.cache.iter()
    }
    
    /// Resize atlas to new dimensions and clear all placements
    /// Returns true if resize was needed
    pub fn resize_to(&mut self, new_width: u32, new_height: u32) -> bool {
        if self.width == new_width && self.height == new_height {
            return false;
        }
        
        self.width = new_width;
        self.height = new_height;
        self.shelves.clear();
        self.next_shelf_y = 0;
        self.cache.clear();
        true
    }
}
```

### Step 2: Add Resize State Tracking to Renderer

**File**: `crates/astra-gui-wgpu/src/lib.rs`

Add fields to `AstraRenderer` struct:

```rust
pub struct AstraRenderer {
    // ... existing fields ...
    
    // Atlas resize tracking
    atlas_needs_resize: bool,
    
    // For proactive estimation
    avg_glyph_size_estimate_px: u32,  // Running average for estimation
}
```

Initialize in `new()`:
```rust
atlas_needs_resize: false,
avg_glyph_size_estimate_px: 32,  // Conservative initial estimate
```

### Step 3: Implement Core Resize Method

**File**: `crates/astra-gui-wgpu/src/lib.rs`

Add private method to `AstraRenderer`:

```rust
fn resize_atlas(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    // Collect all cached glyphs before resize (we need to preserve them)
    let old_glyphs: Vec<(GlyphKey, PlacedGlyph)> = self.atlas
        .cached_glyphs()
        .map(|(k, p)| (k.clone(), *p))
        .collect();
    
    let (old_width, old_height) = self.atlas.dimensions();
    
    // Exponential growth pattern matching buffer growth in codebase
    let new_size = (old_width.max(old_height) * 2).next_power_of_two();
    let new_size = new_size.min(16384); // Cap at 16K to avoid GPU limits
    
    eprintln!(
        "Resizing glyph atlas: {}x{} -> {}x{} ({} cached glyphs)",
        old_width, old_height, new_size, new_size, old_glyphs.len()
    );
    
    // Create new larger atlas texture
    self.atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Astra UI Glyph Atlas"),
        size: wgpu::Extent3d {
            width: new_size,
            height: new_size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    // Resize atlas allocator (clears internal state)
    self.atlas.resize_to(new_size, new_size);
    
    // Re-insert all glyphs (they get new UV coordinates based on new atlas size)
    for (key, old_placed) in &old_glyphs {
        // Get original bitmap dimensions (excluding padding)
        let bitmap_width = ((old_placed.rect_px.width() as i32) - (old_placed.padding_px as i32 * 2)).max(0) as u32;
        let bitmap_height = ((old_placed.rect_px.height() as i32) - (old_placed.padding_px as i32 * 2)).max(0) as u32;
        
        // Re-insert into atlas (gets new placement with corrected UVs)
        match self.atlas.insert(key.clone(), [bitmap_width, bitmap_height]) {
            text::atlas::AtlasInsert::Placed(_) => {
                // Success - will re-rasterize and upload below
            }
            text::atlas::AtlasInsert::AlreadyPresent => {
                // Shouldn't happen since we cleared, but OK
            }
            text::atlas::AtlasInsert::Full => {
                eprintln!("ERROR: Glyph still doesn't fit after resize! key={:?}", key);
                // This is serious - atlas is still too small even after doubling
                // Could try one more doubling or skip this glyph
                continue;
            }
        }
    }
    
    // Re-rasterize and upload all glyphs at their new positions
    for (key, _) in &old_glyphs {
        // Get the new placement
        let Some(new_placed) = self.atlas.get(key) else {
            continue;
        };
        
        // Re-rasterize the glyph
        let bitmap = self.text_engine.rasterize_glyph(
            key.font_id,
            key.glyph_id,
            key.font_px as f32,
        );
        
        if bitmap.pixels.is_empty() {
            continue;
        }
        
        // Upload to new atlas position
        let rect_px = new_placed.rect_px;
        let pad = new_placed.padding_px;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: rect_px.min.x + pad,
                    y: rect_px.min.y + pad,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bitmap.size_px[0]),
                rows_per_image: Some(bitmap.size_px[1]),
            },
            wgpu::Extent3d {
                width: bitmap.size_px[0],
                height: bitmap.size_px[1],
                depth_or_array_layers: 1,
            },
        );
    }
    
    // Update metrics cache with new placements
    // (Keep bearing and size, update placement)
    let mut updated_cache = std::mem::take(&mut self.glyph_metrics_cache);
    for (atlas_key, (bearing, size, _old_placed)) in updated_cache.iter_mut() {
        if let Some(new_placed) = self.atlas.get(atlas_key) {
            *_old_placed = new_placed;
        }
    }
    self.glyph_metrics_cache = updated_cache;
    
    // Recreate bind group with new texture
    let atlas_view = self.atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
    self.atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Astra UI Atlas Bind Group"),
        layout: &self.atlas_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
            },
        ],
    });
    
    self.atlas_needs_resize = false;
}
```

### Step 4: Integrate Into Render Flow

**File**: `crates/astra-gui-wgpu/src/lib.rs`

Modify `render()` method:

**A. Pre-frame resize check** (add at start of `render()`, before render pass):

```rust
pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    output: &FullOutput,
) {
    // STAGE 2: Reactive resize from previous frame
    if self.atlas_needs_resize {
        self.resize_atlas(device, queue);
    }
    
    // STAGE 1: Proactive estimation
    let text_shape_count = output.shapes.iter()
        .filter(|s| matches!(s, output::Shape::Text(_)))
        .count();
    
    if text_shape_count > 0 {
        // Estimate: assume ~10 unique glyphs per text shape (conservative)
        let estimated_new_glyphs = text_shape_count * 10;
        let estimated_space_px = estimated_new_glyphs as u32 * self.avg_glyph_size_estimate_px * self.avg_glyph_size_estimate_px;
        
        let (atlas_w, atlas_h) = self.atlas.dimensions();
        let total_atlas_space = atlas_w * atlas_h;
        let current_utilization = self.atlas.utilization();
        
        // If we'd exceed 70% utilization with new glyphs, resize proactively
        let estimated_utilization = current_utilization + (estimated_space_px as f32 / total_atlas_space as f32);
        
        if estimated_utilization > 0.7 {
            eprintln!(
                "Proactive atlas resize: current={:.1}%, estimated={:.1}%",
                current_utilization * 100.0,
                estimated_utilization * 100.0
            );
            self.resize_atlas(device, queue);
        }
    }
    
    // ... rest of existing render() code ...
```

**B. Update Full handling** (modify existing atlas Full case around line 1027):

```rust
text::atlas::AtlasInsert::Full => {
    eprintln!(
        "WARNING: Glyph atlas full during render! Will resize next frame. \
         (font_id={}, glyph_id={}, size={}px)",
        atlas_key.font_id, atlas_key.glyph_id, atlas_key.font_px
    );
    
    // Mark for resize before next frame
    self.atlas_needs_resize = true;
    
    // Update size estimate for better future predictions
    let glyph_area = bitmap.size_px[0] * bitmap.size_px[1];
    let glyph_size = (glyph_area as f32).sqrt() as u32;
    self.avg_glyph_size_estimate_px = (self.avg_glyph_size_estimate_px + glyph_size) / 2;
    
    None
}
```

**C. Track successful insertions for estimation** (modify Placed case):

```rust
text::atlas::AtlasInsert::Placed(p) => {
    // Upload bitmap
    queue.write_texture(...);
    
    // Update size estimate
    let glyph_area = bitmap.size_px[0] * bitmap.size_px[1];
    let glyph_size = (glyph_area as f32).sqrt() as u32;
    self.avg_glyph_size_estimate_px = (self.avg_glyph_size_estimate_px * 7 + glyph_size) / 8; // Smooth average
    
    Some(p)
}
```

## Critical Files to Modify

1. **`crates/astra-gui-wgpu/src/text/atlas.rs`** (lines 160-370)
   - Add introspection methods: `utilization()`, `glyph_count()`, `dimensions()`, `cached_glyphs()`, `resize_to()`

2. **`crates/astra-gui-wgpu/src/lib.rs`** (lines 100-150 for fields, 1000-1100 for rendering, new method)
   - Add `atlas_needs_resize`, `avg_glyph_size_estimate_px` fields to renderer struct
   - Add `resize_atlas()` private method
   - Modify `render()`: add pre-frame check and proactive estimation
   - Update `AtlasInsert::Full` and `AtlasInsert::Placed` handling

## Testing Strategy

1. **Manual Testing**: 
   - Run zoom example, zoom to 500%+ and verify text stays visible
   - Monitor console for resize messages
   - Check that resizes happen proactively (before Full is hit)

2. **Edge Cases**:
   - Very large glyphs (>1000px)
   - Rapid zoom changes
   - Atlas cap at 16384 (what happens if that's not enough?)

3. **Performance**:
   - Measure resize latency (should be <16ms for 60fps)
   - Check memory usage growth pattern
   - Verify no stuttering during resize

## Performance Considerations

- **Resize Cost**: Re-rasterizing all glyphs is expensive but happens infrequently
- **Memory Growth**: Atlas grows exponentially (4096 → 8192 → 16384), capped at 16K
- **Frame Glitches**: Stage 2 causes 1-frame glitch, but Stage 1 should prevent most cases
- **Estimation Accuracy**: Conservative estimates prevent repeated resizes

## Future Improvements (Out of Scope)

- Eviction strategy for least-recently-used glyphs
- Multiple smaller atlases instead of one giant atlas
- Separate atlases per font size bucket
- GPU-side atlas packing
