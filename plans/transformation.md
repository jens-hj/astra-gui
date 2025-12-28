# Plan: Rename Offset to Translation and Add Rotation Support

## Design Requirements

1. **Rotation direction**: Clockwise positive (CSS convention)
2. **Transform origin**: Configurable `TransformOrigin` property (percentage-based like CSS)
3. **Child coordinate spaces**: Children layout in parent's transformed reference frame
4. **Clipping**: Expanded AABB approach for v1 (defer rotated clipping)
5. **Hit testing**: Inverse transform for accurate click detection

## Critical Architectural Change

**Current**: `offset` is baked into `ComputedLayout::rect` during layout computation  
**New**: Store untransformed rects, apply transforms during rendering via transform hierarchy

This enables:
- Rotation around arbitrary origins
- Proper transform composition (parent + child)
- Inverse transforms for hit testing

## Type System

### 1. Translation (layout.rs)

```rust
/// 2D translation offset
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Translation {
    pub x: f32,
    pub y: f32,
}

impl Translation {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// Backward compatibility
#[deprecated(since = "0.2.0", note = "Use Translation instead")]
pub type Offset = Translation;
```

### 2. TransformOrigin (layout.rs)

```rust
/// Transform origin for rotation (CSS-like percentage + pixel offset)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    /// X position as percentage of width (0.0 = left, 0.5 = center, 1.0 = right)
    pub x_percent: f32,
    /// Y position as percentage of height (0.0 = top, 0.5 = center, 1.0 = bottom)
    pub y_percent: f32,
    /// Additional X offset in pixels
    pub x_offset: f32,
    /// Additional Y offset in pixels
    pub y_offset: f32,
}

impl TransformOrigin {
    pub const fn center() -> Self {
        Self {
            x_percent: 0.5,
            y_percent: 0.5,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }
    
    pub const fn top_left() -> Self {
        Self {
            x_percent: 0.0,
            y_percent: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }
    
    /// Compute absolute position given rect size
    pub fn resolve(&self, width: f32, height: f32) -> (f32, f32) {
        (
            self.x_percent * width + self.x_offset,
            self.y_percent * height + self.y_offset,
        )
    }
}

impl Default for TransformOrigin {
    fn default() -> Self {
        Self::center()
    }
}
```

### 3. Transform2D (layout.rs)

```rust
/// 2D transform combining translation, rotation, and origin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub translation: Translation,
    pub rotation: f32, // Radians, clockwise positive
    pub origin: TransformOrigin,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        translation: Translation::ZERO,
        rotation: 0.0,
        origin: TransformOrigin::center(),
    };
    
    /// Apply transform to a point (forward transform)
    pub fn apply(&self, point: [f32; 2], rect_size: [f32; 2]) -> [f32; 2] {
        let (origin_x, origin_y) = self.origin.resolve(rect_size[0], rect_size[1]);
        
        // Translate to origin
        let x = point[0] - origin_x;
        let y = point[1] - origin_y;
        
        // Rotate (clockwise positive)
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = x * cos_r + y * sin_r;
        let ry = -x * sin_r + y * cos_r;
        
        // Translate back and apply translation
        [
            rx + origin_x + self.translation.x,
            ry + origin_y + self.translation.y,
        ]
    }
    
    /// Apply inverse transform (for hit testing)
    pub fn apply_inverse(&self, point: [f32; 2], rect_size: [f32; 2]) -> [f32; 2] {
        let (origin_x, origin_y) = self.origin.resolve(rect_size[0], rect_size[1]);
        
        // Remove translation
        let x = point[0] - self.translation.x - origin_x;
        let y = point[1] - self.translation.y - origin_y;
        
        // Inverse rotate (negate angle)
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = x * cos_r - y * sin_r;
        let ry = x * sin_r + y * cos_r;
        
        // Translate back from origin
        [rx + origin_x, ry + origin_y]
    }
    
    /// Compose two transforms (apply self, then other)
    pub fn then(&self, other: &Transform2D, rect_size: [f32; 2]) -> Transform2D {
        // For hierarchical transforms, we need to:
        // 1. Apply parent transform
        // 2. Apply child transform in the transformed space
        
        // Simplified: just accumulate (works for most cases)
        Transform2D {
            translation: Translation {
                x: self.translation.x + other.translation.x,
                y: self.translation.y + other.translation.y,
            },
            rotation: self.rotation + other.rotation,
            origin: other.origin, // Use child's origin
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}
```

## Implementation Phases

### Phase 1: Add Types & Rename (layout.rs, node.rs, style.rs)

**Files**: `crates/astra-gui/src/layout.rs`, `crates/astra-gui/src/node.rs`, `crates/astra-gui/src/style.rs`

1. Add `Translation`, `TransformOrigin`, `Transform2D` to `layout.rs`
2. Add backward-compatible `Offset` type alias
3. Update `Node`:
   ```rust
   pub struct Node {
       // ... existing fields ...
       translation: Translation,
       rotation: f32,
       transform_origin: TransformOrigin,
   }
   
   // Add builder methods
   pub fn with_translation(mut self, translation: Translation) -> Self
   pub fn with_rotation(mut self, rotation: f32) -> Self
   pub fn with_transform_origin(mut self, origin: TransformOrigin) -> Self
   
   // Add deprecated aliases
   #[deprecated(since = "0.2.0")]
   pub fn with_offset(self, offset: Offset) -> Self
   ```

4. Update `Style`:
   ```rust
   pub struct Style {
       // ... existing fields ...
       pub translation_x: Option<f32>,
       pub translation_y: Option<f32>,
       pub rotation: Option<f32>,
       pub transform_origin: Option<TransformOrigin>,
       
       // Deprecated
       #[deprecated(since = "0.2.0")]
       pub offset_x: Option<f32>,
       #[deprecated(since = "0.2.0")]
       pub offset_y: Option<f32>,
   }
   ```

### Phase 2: Decouple Layout from Transform (node.rs)

**File**: `crates/astra-gui/src/node.rs`

**Critical change** in `compute_layout_with_parent_size()`:

```rust
// BEFORE (bakes translation into rect):
self.computed = Some(ComputedLayout::new(Rect::new(
    [outer_x + self.offset.x, outer_y + self.offset.y],
    [outer_x + width + self.offset.x, outer_y + height + self.offset.y],
)));

// AFTER (store untransformed rect):
self.computed = Some(ComputedLayout::new(Rect::new(
    [outer_x, outer_y],
    [outer_x + width, outer_y + height],
)));
```

### Phase 3: Transform Accumulation (output.rs)

**File**: `crates/astra-gui/src/output.rs`

1. Change `ClippedShape` from tuple to struct:
   ```rust
   pub struct ClippedShape {
       pub node_rect: Rect,          // Untransformed
       pub clip_rect: Rect,          // Expanded AABB in world space
       pub shape: Shape,
       pub transform: Transform2D,   // Accumulated parent + local
   }
   ```

2. Update `collect_clipped_shapes_with_opacity()`:
   ```rust
   fn collect_clipped_shapes_with_opacity(
       node: &Node,
       window_rect: Rect,
       inherited_clip_rect: Rect,
       parent_transform: Transform2D,  // NEW parameter
       debug_options: Option<DebugOptions>,
       out: &mut Vec<ClippedShape>,
       parent_opacity: f32,
   ) {
       let Some(layout) = node.computed_layout() else { return };
       
       // Compose transforms
       let local_transform = Transform2D {
           translation: node.translation(),
           rotation: node.rotation(),
           origin: node.transform_origin(),
       };
       
       let rect_size = [
           layout.rect.max[0] - layout.rect.min[0],
           layout.rect.max[1] - layout.rect.min[1],
       ];
       
       let world_transform = parent_transform.then(&local_transform, rect_size);
       
       // Compute AABB for clipping
       let transformed_aabb = compute_transformed_aabb(layout.rect, &world_transform);
       let effective_clip_rect = clip_rects(inherited_clip_rect, transformed_aabb);
       
       // Collect shape with transform
       if let Some(shape) = node.shape() {
           out.push(ClippedShape {
               node_rect: layout.rect,
               clip_rect: effective_clip_rect,
               shape: shape_with_opacity,
               transform: world_transform,
           });
       }
       
       // Recurse to children with accumulated transform
       for child in node.children() {
           collect_clipped_shapes_with_opacity(
               child,
               window_rect,
               effective_clip_rect,
               world_transform,  // Pass accumulated transform
               debug_options,
               out,
               combined_opacity,
           );
       }
   }
   
   fn compute_transformed_aabb(rect: Rect, transform: &Transform2D) -> Rect {
       let width = rect.max[0] - rect.min[0];
       let height = rect.max[1] - rect.min[1];
       let (origin_x, origin_y) = transform.origin.resolve(width, height);
       
       // Transform four corners
       let corners = [
           [rect.min[0], rect.min[1]],
           [rect.max[0], rect.min[1]],
           [rect.max[0], rect.max[1]],
           [rect.min[0], rect.max[1]],
       ];
       
       let mut min_x = f32::INFINITY;
       let mut max_x = f32::NEG_INFINITY;
       let mut min_y = f32::INFINITY;
       let mut max_y = f32::NEG_INFINITY;
       
       for corner in corners {
           let transformed = transform.apply(corner, [width, height]);
           min_x = min_x.min(transformed[0]);
           max_x = max_x.max(transformed[0]);
           min_y = min_y.min(transformed[1]);
           max_y = max_y.max(transformed[1]);
       }
       
       Rect::new([min_x, min_y], [max_x, max_y])
   }
   ```

### Phase 4: Instance Data (instance.rs)

**File**: `crates/astra-gui-wgpu/src/instance.rs`

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectInstance {
    // Geometry (16 bytes)
    pub center: [f32; 2],
    pub half_size: [f32; 2],
    
    // Transform (24 bytes)
    pub translation: [f32; 2],
    pub rotation: f32,
    pub _padding1: f32,
    pub transform_origin: [f32; 2],  // Resolved absolute position
    pub _padding2: [f32; 2],
    
    // Appearance (48 bytes)
    pub fill_color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub stroke_width: f32,
    pub corner_type: u32,
    pub corner_param1: f32,
    pub corner_param2: f32,
}

impl RectInstance {
    pub fn from_clipped_shape(clipped: &ClippedShape) -> Self {
        let Shape::Rect(rect) = &clipped.shape else {
            panic!("Expected Rect shape");
        };
        
        let width = clipped.node_rect.max[0] - clipped.node_rect.min[0];
        let height = clipped.node_rect.max[1] - clipped.node_rect.min[1];
        
        let center = [
            (clipped.node_rect.min[0] + clipped.node_rect.max[0]) * 0.5,
            (clipped.node_rect.min[1] + clipped.node_rect.max[1]) * 0.5,
        ];
        
        let half_size = [width * 0.5, height * 0.5];
        
        let (origin_x, origin_y) = clipped.transform.origin.resolve(width, height);
        
        Self {
            center,
            half_size,
            translation: [clipped.transform.translation.x, clipped.transform.translation.y],
            rotation: clipped.transform.rotation,
            _padding1: 0.0,
            transform_origin: [origin_x, origin_y],
            _padding2: [0.0, 0.0],
            fill_color: rect.fill_color,
            stroke_color: rect.stroke_color,
            stroke_width: rect.stroke_width,
            corner_type: rect.corner_type as u32,
            corner_param1: rect.corner_param1,
            corner_param2: rect.corner_param2,
        }
    }
}
```

### Phase 5: Shader Updates (ui_sdf.wgsl)

**File**: `crates/astra-gui-wgpu/src/shaders/ui_sdf.wgsl`

```wgsl
struct InstanceInput {
    @location(1) center: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) translation: vec2<f32>,
    @location(4) rotation: f32,
    // location 5: _padding1 (skip)
    @location(6) transform_origin: vec2<f32>,
    // location 7-8: _padding2 (skip)
    @location(9) fill_color: vec4<f32>,
    @location(10) stroke_color: vec4<f32>,
    @location(11) stroke_width: f32,
    @location(12) corner_type: u32,
    @location(13) corner_param1: f32,
    @location(14) corner_param2: f32,
}

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Expand to local rect
    let padding = inst.stroke_width * 0.5;
    let expanded_size = inst.half_size + vec2<f32>(padding);
    let local_pos = inst.center + vert.pos * expanded_size;
    
    // Apply transform: origin → rotate → translate
    let centered = local_pos - inst.transform_origin;
    let cos_r = cos(inst.rotation);
    let sin_r = sin(inst.rotation);
    let rotated = vec2<f32>(
        centered.x * cos_r + centered.y * sin_r,    // Clockwise
        -centered.x * sin_r + centered.y * cos_r
    );
    out.world_pos = rotated + inst.transform_origin + inst.translation;

    // NDC conversion
    let ndc = ((out.world_pos + 0.5) / uniforms.screen_size) * 2.0 - 1.0;
    out.clip_pos = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    // SDF uses unrotated local space
    out.local_pos = vert.pos * inst.half_size;
    
    // Pass other attributes
    out.fill_color = inst.fill_color;
    out.stroke_color = inst.stroke_color;
    out.stroke_width = inst.stroke_width;
    out.corner_type = inst.corner_type;
    out.corner_param1 = inst.corner_param1;
    out.corner_param2 = inst.corner_param2;
    
    return out;
}
```

### Phase 6: Vertex Buffer Layout (lib.rs)

**File**: `crates/astra-gui-wgpu/src/lib.rs`

Update vertex attributes to match new instance layout with proper offsets and skip padding locations.

### Phase 7: Hit Testing (node.rs)

**File**: `crates/astra-gui/src/node.rs`

```rust
impl Node {
    /// Test if a world-space point is inside this node (accounting for transforms)
    pub fn hit_test(&self, point: [f32; 2], parent_transform: Transform2D) -> bool {
        let Some(layout) = self.computed_layout() else {
            return false;
        };
        
        // Build accumulated transform
        let local_transform = Transform2D {
            translation: self.translation,
            rotation: self.rotation,
            origin: self.transform_origin,
        };
        
        let width = layout.rect.max[0] - layout.rect.min[0];
        let height = layout.rect.max[1] - layout.rect.min[1];
        
        let world_transform = parent_transform.then(&local_transform, [width, height]);
        
        // Transform point to local space
        let local_point = world_transform.apply_inverse(point, [width, height]);
        
        // Test against untransformed rect
        local_point[0] >= layout.rect.min[0] &&
        local_point[0] <= layout.rect.max[0] &&
        local_point[1] >= layout.rect.min[1] &&
        local_point[1] <= layout.rect.max[1]
    }
}
```

## Critical Files to Modify

1. **crates/astra-gui/src/layout.rs** - Add `Translation`, `TransformOrigin`, `Transform2D`
2. **crates/astra-gui/src/node.rs** - Add transform properties, decouple layout, add hit testing
3. **crates/astra-gui/src/style.rs** - Add transform style properties
4. **crates/astra-gui/src/output.rs** - Transform accumulation, AABB computation
5. **crates/astra-gui-wgpu/src/instance.rs** - Extend instance data with transform
6. **crates/astra-gui-wgpu/src/shaders/ui_sdf.wgsl** - Apply transforms in vertex shader
7. **crates/astra-gui-wgpu/src/lib.rs** - Update vertex buffer layout, instance creation

## Testing Strategy

1. Create example with rotated rects at 0°, 45°, 90°, 180°
2. Test transform origin variants (center, corner, custom)
3. Test nested transforms (parent rotation + child rotation)
4. Test hit testing with rotated elements
5. Test combined translation + rotation
6. Performance test: verify no regression with identity transforms

## Breaking Changes

- `offset` → `translation` (with deprecation aliases)
- `ComputedLayout::rect` no longer includes translation (now untransformed)
- `ClippedShape` changes from tuple to struct
- Version bump: 0.1.x → 0.2.0
