# Plan: Performance Optimization for astra-gui Framework

## Problem Statement

The scroll.rs example (188 nodes, 325 shapes) is experiencing choppy performance. Analysis identified critical bottlenecks across rendering, layout, and event systems. The workload size is NOT inherently expensive - the issue is algorithmic inefficiency with multiple tree traversals and unnecessary allocations per frame.

**Current State**: ~30 FPS (choppy)  
**Target**: 60+ FPS (smooth)

## Performance Analysis Summary

### Workload Metrics
- **Total nodes per frame**: 188
- **Total drawable shapes**: ~325 (164 rects + 161 text)
- **Tree depth**: 7-8 levels
- **Scrollable containers**: 3
- **Frame operations**: 5-6 separate tree traversals

### Critical Bottlenecks Identified

**Rendering Pipeline (HIGH IMPACT):**
1. Shape cloning for opacity (output.rs:198) - 325 clones/frame
2. Multiple vector allocations per frame (lib.rs:576-834)
3. Multiple shape type iterations (lib.rs:581-1030)
4. Per-glyph texture uploads (lib.rs:890-920)
5. Redundant scissor rect computations (lib.rs:589-853)
6. Per-glyph trig calculations (lib.rs:930-970)

**Layout & Events (HIGH IMPACT):**
1. Redundant spacing calculations (node.rs:631-666, 1050-1095)
2. Double scroll state traversals (events.rs:393-443)
3. calculate_max_scroll O(n×m) nested loops (events.rs:547-635)
4. Linear node searches per scroll event (events.rs:532-541)
5. HashMap string allocations in hot path (events.rs:434-435)

## Implementation Strategy

Optimize in 3 phases prioritizing high-impact, low-effort improvements first.

---

## PHASE 1: Quick Wins (Expected: ~20-30% improvement)

### 1.1 Remove Shape Cloning for Opacity

**File**: `crates/astra-gui/src/output.rs`  
**Location**: Line 198

**Current Code**:
```rust
let mut shape_with_opacity = shape.clone();
if combined_opacity < 1.0 {
    shape_with_opacity.apply_opacity(combined_opacity);
}
out.push((node_rect, inherited_clip_rect, shape_with_opacity, world_transform));
```

**Optimized Code**:
```rust
// Option 1: Store opacity separately in ClippedShape
pub struct ClippedShape {
    pub node_rect: Rect,
    pub clip_rect: Rect,
    pub shape: Shape,  // No clone needed
    pub transform: Transform2D,
    pub opacity: f32,  // NEW: Store opacity separately
}

// In collect_clipped_shapes_with_opacity:
out.push(ClippedShape {
    node_rect,
    clip_rect: inherited_clip_rect,
    shape: shape.clone(),  // Still need clone, but only once
    transform: world_transform,
    opacity: combined_opacity,
});

// Apply opacity during rendering in lib.rs instead of pre-processing
```

**Impact**: Eliminates 325 shape clones per frame  
**Breaking**: Changes ClippedShape from tuple to struct (already planned in rotation work)

---

### 1.2 Cache Max Scroll Calculation

**File**: `crates/astra-gui-wgpu/src/events.rs`  
**Location**: Lines 547-635

**Problem**: `calculate_max_scroll()` is called on EVERY scroll event with O(n×m) nested loops for grid layouts.

**Solution**: Cache max_scroll in ComputedLayout during layout computation.

**Step 1**: Add field to ComputedLayout
```rust
// In crates/astra-gui/src/layout.rs
pub struct ComputedLayout {
    pub rect: Rect,
    pub max_scroll: (f32, f32),  // NEW: Cached max scroll
}
```

**Step 2**: Calculate during layout (node.rs)
```rust
// In compute_layout_with_parent_size_and_measurer, after children layout:
let max_scroll = if self.overflow == Overflow::Scroll {
    Self::calculate_max_scroll_for_node(self)
} else {
    (0.0, 0.0)
};

self.computed = Some(ComputedLayout {
    rect: Rect::new([outer_x, outer_y], [outer_x + width, outer_y + height]),
    max_scroll,
});

// Move calculate_max_scroll logic to Node impl
fn calculate_max_scroll_for_node(&self) -> (f32, f32) {
    // Same logic as events.rs calculate_max_scroll but as Node method
}
```

**Step 3**: Use cached value in events.rs
```rust
// In process_scroll_event:
let max_scroll = node.computed_layout()
    .map(|l| l.max_scroll)
    .unwrap_or((0.0, 0.0));

// Remove calculate_max_scroll function entirely
```

**Impact**: Moves O(n×m) calculation from per-scroll-event to per-layout (much less frequent)

---

### 1.3 Combine Scroll State Traversals

**File**: `crates/astra-gui-wgpu/src/events.rs`  
**Location**: Lines 393-443

**Problem**: Two separate tree traversals: `restore_scroll_state` + `sync_scroll_state`

**Solution**: Single traversal that both restores and syncs.

```rust
// Replace both restore_scroll_state and sync_scroll_state with:
pub fn sync_scroll_state_bidirectional(&mut self, root: &mut Node) {
    Self::sync_bidirectional_recursive(root, &mut self.scroll_state);
}

fn sync_bidirectional_recursive(
    node: &mut Node,
    scroll_state: &mut HashMap<String, ((f32, f32), (f32, f32))>,
) {
    // First restore from persistent state
    if let Some(node_id) = node.id() {
        if let Some(&(offset, target)) = scroll_state.get(node_id.as_str()) {
            node.set_scroll_offset(offset);
            node.set_scroll_target(target);
        }
        
        // Then sync back after potential changes
        if node.overflow() == astra_gui::Overflow::Scroll {
            let current_offset = node.scroll_offset();
            let current_target = node.scroll_target();
            scroll_state.insert(
                node_id.as_str().to_string(),
                (current_offset, current_target),
            );
        }
    }
    
    // Recurse
    for child in node.children_mut() {
        Self::sync_bidirectional_recursive(child, scroll_state);
    }
}
```

**Update scroll.rs example**:
```rust
// Replace:
gpu_state.event_dispatcher.restore_scroll_state(&mut ui);
// ... later ...
gpu_state.event_dispatcher.sync_scroll_state(&ui);

// With single call:
gpu_state.event_dispatcher.sync_scroll_state_bidirectional(&mut ui);
```

**Impact**: Reduces 6 tree traversals to 5 per frame

---

### 1.4 Pre-allocate Rendering Vectors

**File**: `crates/astra-gui-wgpu/src/lib.rs`  
**Location**: Lines 576-834

**Problem**: Fresh allocations for `indices` and `geometry_draws` every frame.

**Solution**: Make them persistent fields on Renderer.

**Step 1**: Add fields to Renderer struct
```rust
pub struct Renderer {
    // ... existing fields ...
    
    // Persistent buffers (reused across frames)
    frame_indices: Vec<u32>,
    frame_geometry_draws: Vec<ClippedDraw>,
}
```

**Step 2**: Initialize in new()
```rust
impl Renderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        Self {
            // ... existing initialization ...
            frame_indices: Vec::new(),
            frame_geometry_draws: Vec::new(),
        }
    }
}
```

**Step 3**: Reuse in render()
```rust
// Replace:
let mut indices: Vec<u32> = Vec::new();
indices.reserve(self.last_frame_index_count);
let mut geometry_draws: Vec<ClippedDraw> = Vec::new();

// With:
self.frame_indices.clear();
self.frame_indices.reserve(self.last_frame_index_count);
self.frame_geometry_draws.clear();
```

**Impact**: Eliminates 2 allocations per frame

---

### 1.5 Hoist Transform Calculations Outside Glyph Loop

**File**: `crates/astra-gui-wgpu/src/lib.rs`  
**Location**: Lines 930-970

**Problem**: `cos_r` and `sin_r` calculated inside per-glyph loop.

**Solution**: Pre-calculate outside loop.

```rust
// Before glyph loop (line ~925):
let (cos_r, sin_r) = if rotation.abs() > 0.0001 {
    (rotation.cos(), rotation.sin())
} else {
    (1.0, 0.0)  // Identity
};

// Inside glyph loop, replace closure with direct calculation:
for g in &shaped.glyphs {
    // ... existing code ...
    
    // Calculate transformed vertices directly
    let mut apply_transform = |pos: [f32; 2]| -> [f32; 2] {
        let mut x = pos[0] + translation.x;
        let mut y = pos[1] + translation.y;
        
        if rotation.abs() > 0.0001 {
            x -= transform_origin[0];
            y -= transform_origin[1];
            
            // Use pre-calculated cos_r and sin_r
            let rx = x * cos_r + y * sin_r;
            let ry = -x * sin_r + y * cos_r;
            
            x = rx + transform_origin[0];
            y = ry + transform_origin[1];
        }
        
        [x, y]
    };
    
    // ... rest of glyph processing ...
}
```

**Impact**: Eliminates redundant trig calculations per glyph

---

## PHASE 2: Medium-Impact Optimizations (Expected: +15-25% improvement)

### 2.1 Single-Pass Shape Type Iteration

**File**: `crates/astra-gui-wgpu/src/lib.rs`  
**Location**: Lines 581-1030

**Problem**: Iterate shapes 2-3 times (once for rects, once for text, once for mesh).

**Solution**: Pre-classify shapes during output generation or use single iteration.

**Option A**: Classify in output.rs
```rust
// In output.rs, FullOutput struct:
pub struct FullOutput {
    pub shapes: Vec<ClippedShape>,
    pub rect_indices: Vec<usize>,  // NEW: Indices of rect shapes
    pub text_indices: Vec<usize>,  // NEW: Indices of text shapes
    pub window_size: (f32, f32),
}

// During collect_clipped_shapes, track indices by type
```

**Option B**: Single-pass iteration with match
```rust
for (idx, clipped) in output.shapes.iter().enumerate() {
    match &clipped.shape {
        Shape::Rect(rect) => {
            // Process rect immediately
            if use_sdf {
                // SDF processing
            } else {
                // Mesh processing
            }
        }
        Shape::Text(text) => {
            // Process text immediately
        }
    }
}
```

**Impact**: Reduces wasted iterations over non-matching shapes

---

### 2.2 Batch Texture Uploads

**File**: `crates/astra-gui-wgpu/src/lib.rs`  
**Location**: Lines 890-920

**Problem**: `queue.write_texture()` called for each new glyph in tight loop.

**Solution**: Collect all new glyphs first, then batch upload.

```rust
// Collect new glyphs
let mut pending_uploads: Vec<(GlyphKey, Bitmap, AtlasPlacement)> = Vec::new();

for g in &shaped.glyphs {
    let Some(bitmap) = self.text_engine.rasterize_glyph(g.key) else {
        continue;
    };
    
    let placed = match self.atlas.insert(key, bitmap.size_px) {
        AtlasInsert::Placed(p) => {
            pending_uploads.push((key, bitmap, p));
            p
        }
        AtlasInsert::Cached(p) => p,
    };
    
    // ... rest of processing ...
}

// Batch upload all new glyphs
for (key, bitmap, placement) in pending_uploads {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &self.atlas_texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: placement.x,
                y: placement.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &bitmap.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bitmap.size_px.0),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: bitmap.size_px.0,
            height: bitmap.size_px.1,
            depth_or_array_layers: 1,
        },
    );
}
```

**Impact**: Better GPU command batching, fewer pipeline stalls

---

### 2.3 Build Node ID Index

**File**: `crates/astra-gui-wgpu/src/events.rs`  
**Location**: Lines 532-541 (find_node_by_id_mut)

**Problem**: Linear O(n) search on every scroll event.

**Solution**: Build HashMap index during dispatch.

```rust
// Add to EventDispatcher:
pub struct EventDispatcher {
    // ... existing fields ...
    node_index: HashMap<String, *mut Node>,  // Cache of node pointers
}

// Build index during dispatch (which already traverses tree):
pub fn dispatch(&mut self, input: &InputState, root: &mut Node) 
    -> (Vec<TargetedEvent>, HashMap<NodeId, InteractionState>) 
{
    // Clear and rebuild index
    self.node_index.clear();
    Self::build_node_index(root, &mut self.node_index);
    
    // ... rest of dispatch logic ...
}

fn build_node_index(node: &mut Node, index: &mut HashMap<String, *mut Node>) {
    if let Some(id) = node.id() {
        index.insert(id.as_str().to_string(), node as *mut Node);
    }
    for child in node.children_mut() {
        Self::build_node_index(child, index);
    }
}

// Replace find_node_by_id_mut with O(1) lookup:
fn find_node_by_id_mut<'a>(&self, id: &str) -> Option<&'a mut Node> {
    self.node_index.get(id).map(|ptr| unsafe { &mut **ptr })
}
```

**Impact**: O(1) node lookup instead of O(n) search per scroll event

**Note**: Requires careful lifetime management with raw pointers.

---

### 2.4 Reduce HashMap String Allocations

**File**: `crates/astra-gui-wgpu/src/events.rs`  
**Location**: Lines 434-435, 522-523

**Problem**: `to_string()` on every insert creates unnecessary allocation.

**Solution**: Use `Cow<'static, str>` or change HashMap key to `&str` with proper lifetimes.

```rust
// Simple fix: Reuse node_id string
if let Some(node_id) = node.id() {
    if node.overflow() == astra_gui::Overflow::Scroll {
        let current_offset = node.scroll_offset();
        let current_target = node.scroll_target();
        
        // Reuse the String from NodeId instead of cloning
        scroll_state.insert(
            node_id.into_string(),  // Move ownership instead of clone
            (current_offset, current_target),
        );
    }
}
```

**Impact**: Eliminates string allocations in hot path

---

## PHASE 3: Architectural Improvements (Expected: +20-40% improvement)

### 3.1 Visibility Culling

**File**: `crates/astra-gui/src/output.rs`  
**Location**: collect_clipped_shapes_with_opacity

**Problem**: All 325 shapes collected even if outside viewport.

**Solution**: Skip shapes completely outside clip rect early.

```rust
fn collect_clipped_shapes_with_opacity(
    node: &Node,
    window_rect: Rect,
    inherited_clip_rect: Rect,
    parent_transform: Transform2D,
    debug_options: Option<DebugOptions>,
    out: &mut Vec<ClippedShape>,
    parent_opacity: f32,
) {
    // ... existing code ...
    
    // EARLY EXIT: Skip if completely outside viewport
    if is_empty_rect(effective_clip_rect) {
        return;  // Already exists
    }
    
    // NEW: Skip if AABB completely outside window
    let transformed_aabb = compute_transformed_aabb(node_rect, &world_transform);
    if !rects_overlap(transformed_aabb, window_rect) {
        return;  // Don't traverse children if parent invisible
    }
    
    // ... rest of function ...
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    !(a.max[0] < b.min[0] || a.min[0] > b.max[0] ||
      a.max[1] < b.min[1] || a.min[1] > b.max[1])
}
```

**Impact**: Significantly reduces shape count for scrolled containers

---

### 3.2 Pre-compute Scissor Rects

**File**: `crates/astra-gui/src/output.rs` and `crates/astra-gui-wgpu/src/lib.rs`

**Problem**: Scissor rect computed 3 times per shape in lib.rs (lines 589-603, 653-667, 839-853).

**Solution**: Compute once in output.rs and store in ClippedShape.

```rust
// In output.rs, add to ClippedShape:
pub struct ClippedShape {
    pub node_rect: Rect,
    pub clip_rect: Rect,
    pub shape: Shape,
    pub transform: Transform2D,
    pub opacity: f32,
    pub scissor: (u32, u32, u32, u32),  // NEW: Pre-computed (x, y, w, h)
}

// Compute during collection:
let scissor = compute_scissor_rect(effective_clip_rect, screen_width, screen_height);
out.push(ClippedShape {
    // ... other fields ...
    scissor,
});

fn compute_scissor_rect(clip_rect: Rect, screen_w: f32, screen_h: f32) -> (u32, u32, u32, u32) {
    let sc_min_x = clip_rect.min[0].max(0.0).floor() as i32;
    let sc_min_y = clip_rect.min[1].max(0.0).floor() as i32;
    let sc_max_x = clip_rect.max[0].min(screen_w).ceil() as i32;
    let sc_max_y = clip_rect.max[1].min(screen_h).ceil() as i32;
    
    let sc_w = (sc_max_x - sc_min_x).max(0) as u32;
    let sc_h = (sc_max_y - sc_min_y).max(0) as u32;
    
    (sc_min_x as u32, sc_min_y as u32, sc_w, sc_h)
}
```

**Impact**: Eliminates redundant float→int conversions and clamping

---

### 3.3 Eliminate Redundant Spacing Calculation

**File**: `crates/astra-gui/src/node.rs`  
**Location**: Lines 631-666 and 1050-1095 (duplicated code)

**Problem**: Two nearly identical functions with duplicated spacing calculation.

**Solution**: Extract shared logic.

```rust
// Extract common spacing calculation
fn calculate_spacing_and_margins(
    children: &[Node],
    gap: f32,
    direction: Layout,
) -> (f32, f32) {
    if children.is_empty() {
        return (0.0, 0.0);
    }
    
    let mut total_spacing = 0.0;
    let mut total_cross = 0.0;
    
    for (i, child) in children.iter().enumerate() {
        if i == 0 {
            total_spacing += match direction {
                Layout::Horizontal => child.margin.left,
                Layout::Vertical => child.margin.top,
                _ => 0.0,
            };
        }
        
        if i + 1 < children.len() {
            let next = &children[i + 1];
            let collapsed = match direction {
                Layout::Horizontal => child.margin.right.max(next.margin.left),
                Layout::Vertical => child.margin.bottom.max(next.margin.top),
                _ => 0.0,
            };
            total_spacing += gap.max(collapsed);
        } else {
            total_spacing += match direction {
                Layout::Horizontal => child.margin.right,
                Layout::Vertical => child.margin.bottom,
                _ => 0.0,
            };
        }
    }
    
    (total_spacing, total_cross)
}

// Use in both compute_layout functions
```

**Impact**: Code deduplication, easier maintenance

---

## Critical Files to Modify

### Phase 1 (Quick Wins):
1. `crates/astra-gui/src/output.rs` - Remove shape cloning
2. `crates/astra-gui/src/layout.rs` - Add max_scroll field
3. `crates/astra-gui/src/node.rs` - Calculate max_scroll during layout
4. `crates/astra-gui-wgpu/src/events.rs` - Combine traversals, use cached max_scroll
5. `crates/astra-gui-wgpu/src/lib.rs` - Pre-allocate vectors, hoist trig
6. `crates/astra-gui-wgpu/examples/scroll.rs` - Use combined sync function

### Phase 2 (Medium Impact):
1. `crates/astra-gui-wgpu/src/lib.rs` - Single-pass iteration, batch uploads
2. `crates/astra-gui-wgpu/src/events.rs` - Node ID index, reduce allocations

### Phase 3 (Architectural):
1. `crates/astra-gui/src/output.rs` - Visibility culling, pre-compute scissor
2. `crates/astra-gui/src/node.rs` - Extract shared spacing logic

## Expected Performance Improvements

- **Phase 1**: 30 FPS → 45-50 FPS (~50% improvement)
- **Phase 2**: 45-50 FPS → 55-65 FPS (~30% improvement)
- **Phase 3**: 55-65 FPS → 70-80+ FPS (~40% improvement)

**Total Expected**: 30 FPS → 70-80+ FPS (2.3-2.6x improvement)

## Implementation Order

1. **Start with Phase 1.2** (Cache max_scroll) - Highest impact, isolated change
2. **Then Phase 1.1** (Remove shape cloning) - Requires ClippedShape struct change
3. **Then Phase 1.3-1.5** (Other Phase 1 optimizations) - Independent, low risk
4. **Measure performance** - Verify Phase 1 gains before continuing
5. **Phase 2** - Only if Phase 1 doesn't reach 60 FPS target
6. **Phase 3** - For handling larger workloads (500+ nodes)

## Notes

- All Phase 1 optimizations are internal implementation changes
- No breaking API changes until ClippedShape struct modification
- Each optimization is independently valuable
- Immediate-mode architecture is preserved
- Performance gains are cumulative
