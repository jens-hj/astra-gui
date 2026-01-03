use crate::layout::{Overflow, Size, Transform2D};
use crate::measure::ContentMeasurer;
use crate::node::Node;
use crate::primitives::{ClippedShape, Rect, Shape, Stroke};

/// Output from the UI system containing all shapes to render
#[derive(Clone, Debug, Default)]
pub struct FullOutput {
    pub shapes: Vec<ClippedShape>,
    pub debug_options: Option<crate::debug::DebugOptions>,
}

impl FullOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shapes(shapes: Vec<ClippedShape>) -> Self {
        Self {
            shapes,
            debug_options: None,
        }
    }

    /// Create output from a node tree
    ///
    /// `window_size` is the (width, height) of the window
    pub fn from_node(root: Node, window_size: (f32, f32)) -> Self {
        Self::from_node_with_scale_factor(root, window_size, 1.0)
    }

    /// Create output from a node tree with a scale factor for logical-to-physical pixel conversion
    ///
    /// `window_size` is the (width, height) of the window
    /// `scale_factor` is multiplied with all Fixed sizes, padding, margins, gaps, and font sizes
    pub fn from_node_with_scale_factor(
        root: Node,
        window_size: (f32, f32),
        scale_factor: f32,
    ) -> Self {
        Self::from_node_with_debug_and_scale_factor(root, window_size, None, scale_factor)
    }

    /// Create output from a node tree with optional debug visualization
    ///
    /// `window_size` is the (width, height) of the window
    /// `debug_options` configures which debug visualizations to show
    pub fn from_node_with_debug(
        root: Node,
        window_size: (f32, f32),
        debug_options: Option<crate::debug::DebugOptions>,
    ) -> Self {
        Self::from_node_with_debug_and_scale_factor(root, window_size, debug_options, 1.0)
    }

    /// Create output from a node tree with debug visualization and scale factor
    ///
    /// `window_size` is the (width, height) of the window
    /// `debug_options` configures which debug visualizations to show
    /// `scale_factor` is multiplied with all Fixed sizes, padding, margins, gaps, and font sizes
    pub fn from_node_with_debug_and_scale_factor(
        root: Node,
        window_size: (f32, f32),
        debug_options: Option<crate::debug::DebugOptions>,
        scale_factor: f32,
    ) -> Self {
        Self::from_node_with_debug_measurer_and_scale_factor(
            root,
            window_size,
            debug_options,
            None,
            scale_factor,
        )
    }

    /// Create output from a node tree with optional debug visualization and measurer
    ///
    /// `window_size` is the (width, height) of the window
    /// `debug_options` configures which debug visualizations to show
    /// `measurer` enables `Size::FitContent` to resolve to intrinsic content size
    pub fn from_node_with_debug_and_measurer(
        root: Node,
        window_size: (f32, f32),
        debug_options: Option<crate::debug::DebugOptions>,
        measurer: Option<&mut dyn ContentMeasurer>,
    ) -> Self {
        Self::from_node_with_debug_measurer_and_scale_factor(
            root,
            window_size,
            debug_options,
            measurer,
            1.0,
        )
    }

    /// Create output from a node tree with debug visualization, measurer, and scale factor
    ///
    /// `window_size` is the (width, height) of the window
    /// `debug_options` configures which debug visualizations to show
    /// `measurer` enables `Size::FitContent` to resolve to intrinsic content size
    /// `scale_factor` is multiplied with all Fixed sizes, padding, margins, gaps, and font sizes
    pub fn from_node_with_debug_measurer_and_scale_factor(
        mut root: Node,
        window_size: (f32, f32),
        debug_options: Option<crate::debug::DebugOptions>,
        measurer: Option<&mut dyn ContentMeasurer>,
        scale_factor: f32,
    ) -> Self {
        // Get the effective scale factor: use root's zoom_level if set, otherwise the provided scale_factor
        let effective_scale_factor = root.zoom().unwrap_or(scale_factor);

        // Compute layout starting from the full window
        let window_rect = Rect::new([0.0, 0.0], [window_size.0, window_size.1]);

        if let Some(m) = measurer {
            root.compute_layout_with_measurer_and_scale_factor(
                window_rect,
                m,
                effective_scale_factor,
            );
        } else {
            root.compute_layout_with_scale_factor(window_rect, effective_scale_factor);
        }

        // Convert to ClippedShapes (including optional debug shapes), with overflow-aware clip rects.
        //
        // We derive `clip_rect` from the node tree's overflow policy:
        // - If any ancestor has `Overflow::Hidden` (or `Scroll`, for now), shapes are clipped to the
        //   intersection of those ancestor rects.
        // - If all ancestors are `Overflow::Visible`, the clip rect remains the full window rect.

        // Apply pan offset from root node for camera-style zoom
        let initial_transform = Transform2D {
            translation: root.pan_offset().resolve(
                window_size.0,
                window_size.1,
                effective_scale_factor,
            ),
            rotation: 0.0,
            scale: 1.0,
            origin: crate::layout::TransformOrigin::center(),
            absolute_origin: None,
        };

        let mut raw_shapes = Vec::new();
        let mut tree_index = 0;
        collect_clipped_shapes(
            &root,
            window_rect,
            window_rect,
            initial_transform, // Start with pan offset applied
            debug_options,
            &mut raw_shapes,
            crate::layout::ZIndex::DEFAULT, // Initial z_index
            &mut tree_index,                // Track tree order
            effective_scale_factor,
        );

        // Sort shapes by (z_index, tree_index) for correct layering
        // Lower z_index renders first (bottom), higher z_index renders last (top)
        // Within same z_index, tree order is preserved (stable sort)
        raw_shapes.sort_by_key(|(_, _, _, _, _, z_index, tree_idx)| (*z_index, *tree_idx));

        let shapes = raw_shapes
            .into_iter()
            .map(
                |(rect, clip_rect, shape, transform, opacity, z_index, tree_idx)| {
                    // Apply the rect to the shape if it's a StyledRect.
                    // Text already carries its own bounding rect internally (TextShape::rect).
                    let shape_with_rect = match shape {
                        Shape::Rect(mut styled_rect) => {
                            styled_rect.rect = rect;
                            Shape::Rect(styled_rect)
                        }
                        Shape::Triangle(mut styled_triangle) => {
                            styled_triangle.rect = rect;
                            Shape::Triangle(styled_triangle)
                        }
                        Shape::Text(text_shape) => Shape::Text(text_shape),
                    };

                    let mut clipped =
                        ClippedShape::with_transform(clip_rect, rect, shape_with_rect, transform)
                            .with_opacity(opacity);
                    clipped.z_index = z_index;
                    clipped.tree_index = tree_idx;
                    clipped
                },
            )
            .collect();

        Self {
            shapes,
            debug_options,
        }
    }
}

// Recursively walk the node tree to associate a clip rect with each collected shape.
fn collect_clipped_shapes(
    node: &Node,
    window_rect: Rect,
    inherited_clip_rect: Rect,
    parent_transform: Transform2D,
    debug_options: Option<crate::debug::DebugOptions>,
    out: &mut Vec<(
        Rect,
        Rect,
        Shape,
        Transform2D,
        f32,
        crate::layout::ZIndex,
        usize,
    )>,
    parent_z_index: crate::layout::ZIndex,
    tree_index: &mut usize,
    scale_factor: f32,
) {
    collect_clipped_shapes_with_opacity(
        node,
        window_rect,
        inherited_clip_rect,
        parent_transform,
        debug_options,
        out,
        1.0,
        parent_z_index,
        tree_index,
        scale_factor,
    );
}

// Recursively walk the node tree with cumulative opacity.
fn collect_clipped_shapes_with_opacity(
    node: &Node,
    window_rect: Rect,
    inherited_clip_rect: Rect,
    parent_transform: Transform2D,
    debug_options: Option<crate::debug::DebugOptions>,
    out: &mut Vec<(
        Rect,
        Rect,
        Shape,
        Transform2D,
        f32,
        crate::layout::ZIndex,
        usize,
    )>,
    parent_opacity: f32,
    parent_z_index: crate::layout::ZIndex,
    tree_index: &mut usize,
    scale_factor: f32,
) {
    let combined_opacity = parent_opacity * node.opacity();

    // Determine this node's z_index (inherit from parent if not set)
    let current_z_index = node.z_index().unwrap_or(parent_z_index);

    // Skip rendering if fully transparent
    if combined_opacity <= 0.0 {
        return;
    }

    let Some(layout) = node.computed_layout() else {
        return;
    };

    let node_rect = layout.rect;

    // Compute rect size for transform operations
    let rect_size = [
        node_rect.max[0] - node_rect.min[0],
        node_rect.max[1] - node_rect.min[1],
    ];

    // Build local transform from node properties
    let local_transform = Transform2D {
        translation: node
            .translation()
            .resolve(rect_size[0], rect_size[1], scale_factor),
        rotation: node.rotation(),
        scale: node.scale(),
        origin: node.transform_origin(),
        absolute_origin: None, // Will be set during composition if needed
    };

    // Accumulate transforms: parent → local
    let mut world_transform = parent_transform.then(&local_transform, rect_size);

    // If this node has rotation and no absolute origin is set yet, resolve it now
    if world_transform.rotation.abs() > 0.0001 && world_transform.absolute_origin.is_none() {
        let (origin_x, origin_y) = world_transform.origin.resolve(rect_size[0], rect_size[1]);
        world_transform.absolute_origin =
            Some([node_rect.min[0] + origin_x, node_rect.min[1] + origin_y]);
    }

    // Update effective clip rect based on this node's overflow policy.
    let effective_clip_rect = match node.overflow() {
        Overflow::Visible => inherited_clip_rect,
        Overflow::Hidden | Overflow::Scroll => {
            // For Hidden/Scroll, clip to the node rect (including padding)
            // Transform the node rect to get its AABB
            let node_aabb = compute_transformed_aabb(node_rect, &world_transform);
            intersect_rect(inherited_clip_rect, node_aabb)
        }
    };

    // If a node is fully clipped out, we can early-out (and skip its subtree).
    if is_empty_rect(effective_clip_rect) {
        return;
    }

    // Background shape (if any)
    // The node's own shape uses the inherited clip rect (from parent), not effective_clip_rect.
    // This ensures the container's border/background is not clipped by its own overflow policy.
    if let Some(shape) = node.shape() {
        // OPTIMIZATION: Store opacity in ClippedShape instead of applying it to the shape
        // This eliminates 325 shape clones per frame - opacity will be applied during rendering

        // Scale stroke width (logical -> physical pixels)
        let scaled_shape = match shape {
            Shape::Rect(styled_rect) => {
                let mut scaled_rect = styled_rect.clone();
                let width = node_rect.max[0] - node_rect.min[0];
                let height = node_rect.max[1] - node_rect.min[1];
                let min_dim = width.min(height);

                if let Some(ref stroke) = scaled_rect.stroke {
                    // Resolve stroke width with scale_factor
                    let scaled_width = stroke
                        .width
                        .try_resolve_with_scale(width, scale_factor)
                        .unwrap_or(1.0);
                    scaled_rect.stroke = Some(Stroke::new(Size::ppx(scaled_width), stroke.color));
                }

                // Resolve corner shape
                scaled_rect.corner_shape = match scaled_rect.corner_shape {
                    crate::CornerShape::Round(size) => crate::CornerShape::Round(Size::ppx(
                        size.try_resolve_with_scale(min_dim, scale_factor)
                            .unwrap_or(0.0),
                    )),
                    crate::CornerShape::Cut(size) => crate::CornerShape::Cut(Size::ppx(
                        size.try_resolve_with_scale(min_dim, scale_factor)
                            .unwrap_or(0.0),
                    )),
                    crate::CornerShape::InverseRound(size) => {
                        crate::CornerShape::InverseRound(Size::ppx(
                            size.try_resolve_with_scale(min_dim, scale_factor)
                                .unwrap_or(0.0),
                        ))
                    }
                    crate::CornerShape::Squircle { radius, smoothness } => {
                        crate::CornerShape::Squircle {
                            radius: Size::ppx(
                                radius
                                    .try_resolve_with_scale(min_dim, scale_factor)
                                    .unwrap_or(0.0),
                            ),
                            smoothness,
                        }
                    }
                    crate::CornerShape::None => crate::CornerShape::None,
                };

                Shape::Rect(scaled_rect)
            }
            Shape::Triangle(styled_triangle) => {
                let mut scaled_triangle = styled_triangle.clone();
                let width = node_rect.max[0] - node_rect.min[0];

                if let Some(ref stroke) = scaled_triangle.stroke {
                    // Resolve stroke width with scale_factor
                    let scaled_width = stroke
                        .width
                        .try_resolve_with_scale(width, scale_factor)
                        .unwrap_or(1.0);
                    scaled_triangle.stroke =
                        Some(Stroke::new(Size::ppx(scaled_width), stroke.color));
                }

                Shape::Triangle(scaled_triangle)
            }
            Shape::Text(_) => shape.clone(),
        };

        out.push((
            node_rect,
            inherited_clip_rect,
            scaled_shape,
            world_transform,
            combined_opacity,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }

    // Content (if any)
    if let Some(content) = node.content() {
        match content {
            crate::content::Content::Text(text_content) => {
                // Content uses the node's content rect (after padding) as its bounding box,
                // but still inherits the node/ancestor clip rect.
                let padding = node.padding();
                let width = node_rect.max[0] - node_rect.min[0];
                let height = node_rect.max[1] - node_rect.min[1];
                let padding_left = padding
                    .left
                    .try_resolve_with_scale(width, scale_factor)
                    .unwrap_or(0.0);
                let padding_right = padding
                    .right
                    .try_resolve_with_scale(width, scale_factor)
                    .unwrap_or(0.0);
                let padding_top = padding
                    .top
                    .try_resolve_with_scale(height, scale_factor)
                    .unwrap_or(0.0);
                let padding_bottom = padding
                    .bottom
                    .try_resolve_with_scale(height, scale_factor)
                    .unwrap_or(0.0);

                let content_rect = Rect::new(
                    [
                        node_rect.min[0] + padding_left,
                        node_rect.min[1] + padding_top,
                    ],
                    [
                        node_rect.max[0] - padding_right,
                        node_rect.max[1] - padding_bottom,
                    ],
                );
                let mut text_shape = crate::primitives::TextShape::new(content_rect, text_content);
                // Scale font size by scale_factor for zoom
                let scaled_font_size = text_content
                    .font_size
                    .try_resolve_with_scale(width, scale_factor)
                    .unwrap_or(16.0);
                text_shape.font_size = Size::lpx(scaled_font_size);
                text_shape.wrap = text_content.wrap;
                text_shape.line_height_multiplier = text_content.line_height_multiplier;
                // OPTIMIZATION: Store opacity instead of applying it to shape
                out.push((
                    node_rect,
                    effective_clip_rect,
                    Shape::Text(text_shape),
                    world_transform,
                    combined_opacity,
                    current_z_index,
                    *tree_index,
                ));
                *tree_index += 1;
            }
        }
    }

    // Debug overlays (if enabled) must also be overflow-clipped consistently.
    if let Some(options) = debug_options {
        if options.is_enabled() {
            collect_debug_shapes_clipped(
                node,
                node_rect,
                effective_clip_rect,
                &options,
                &world_transform,
                out,
                scale_factor,
                current_z_index,
                tree_index,
            );
        }
    }

    // Collect gap debug shapes between children
    if let Some(options) = debug_options {
        if options.show_gaps && node.gap().is_non_zero() {
            collect_gap_debug_shapes(
                node,
                effective_clip_rect,
                &options,
                &world_transform,
                out,
                scale_factor,
                current_z_index,
                tree_index,
            );
        }
    }

    // Apply scroll offset to children if this is a scroll container
    let child_transform = if node.overflow() == Overflow::Scroll {
        let scroll_offset = node.scroll_offset();
        let mut scrolled_transform = world_transform;
        scrolled_transform.translation.x -= scroll_offset.0;
        scrolled_transform.translation.y -= scroll_offset.1;
        scrolled_transform
    } else {
        world_transform
    };

    for child in node.children() {
        collect_clipped_shapes_with_opacity(
            child,
            window_rect,
            effective_clip_rect,
            child_transform, // Pass accumulated transform with scroll offset
            debug_options,
            out,
            combined_opacity,
            current_z_index, // Pass down current z_index
            tree_index,      // Pass through tree_index counter
            scale_factor,
        );
    }
}

fn intersect_rect(a: Rect, b: Rect) -> Rect {
    Rect::new(
        [a.min[0].max(b.min[0]), a.min[1].max(b.min[1])],
        [a.max[0].min(b.max[0]), a.max[1].min(b.max[1])],
    )
}

fn is_empty_rect(r: Rect) -> bool {
    r.max[0] <= r.min[0] || r.max[1] <= r.min[1]
}

/// Compute axis-aligned bounding box of a transformed rect
fn compute_transformed_aabb(rect: Rect, transform: &Transform2D) -> Rect {
    let width = rect.max[0] - rect.min[0];
    let height = rect.max[1] - rect.min[1];

    // Transform all four corners
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

fn collect_debug_shapes_clipped(
    node: &Node,
    node_rect: Rect,
    clip_rect: Rect,
    options: &crate::debug::DebugOptions,
    transform: &Transform2D,
    out: &mut Vec<(
        Rect,
        Rect,
        Shape,
        Transform2D,
        f32,
        crate::layout::ZIndex,
        usize,
    )>,
    scale_factor: f32,
    current_z_index: crate::layout::ZIndex,
    tree_index: &mut usize,
) {
    use crate::color::Color;
    use crate::primitives::StyledRect;

    let margin = node.margin();
    let padding = node.padding();

    // Resolve margin and padding sizes for arithmetic operations
    let width = node_rect.max[0] - node_rect.min[0];
    let height = node_rect.max[1] - node_rect.min[1];

    let margin_top = margin
        .top
        .try_resolve_with_scale(height, scale_factor)
        .unwrap_or(0.0);
    let margin_right = margin
        .right
        .try_resolve_with_scale(width, scale_factor)
        .unwrap_or(0.0);
    let margin_bottom = margin
        .bottom
        .try_resolve_with_scale(height, scale_factor)
        .unwrap_or(0.0);
    let margin_left = margin
        .left
        .try_resolve_with_scale(width, scale_factor)
        .unwrap_or(0.0);

    // Draw margin area (outermost, semi-transparent red showing margin space)
    if options.show_margins
        && (margin.top.is_non_zero()
            || margin.right.is_non_zero()
            || margin.bottom.is_non_zero()
            || margin.left.is_non_zero())
    {
        // Draw top margin
        if margin.top.is_non_zero() {
            out.push((
                Rect::new(
                    [
                        node_rect.min[0] - margin_left,
                        node_rect.min[1] - margin_top,
                    ],
                    [node_rect.max[0] + margin_right, node_rect.min[1]],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw right margin (excluding top and bottom corners)
        if margin.right.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.max[0], node_rect.min[1]],
                    [node_rect.max[0] + margin_right, node_rect.max[1]],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw bottom margin (full width including corners)
        if margin.bottom.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.min[0] - margin_left, node_rect.max[1]],
                    [
                        node_rect.max[0] + margin_right,
                        node_rect.max[1] + margin_bottom,
                    ],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw left margin (excluding top and bottom corners)
        if margin.left.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.min[0] - margin_left, node_rect.min[1]],
                    [node_rect.min[0], node_rect.max[1]],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(1.0, 0.0, 0.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
    }

    let padding_top = padding
        .top
        .try_resolve_with_scale(height, scale_factor)
        .unwrap_or(0.0);
    let padding_right = padding
        .right
        .try_resolve_with_scale(width, scale_factor)
        .unwrap_or(0.0);
    let padding_bottom = padding
        .bottom
        .try_resolve_with_scale(height, scale_factor)
        .unwrap_or(0.0);
    let padding_left = padding
        .left
        .try_resolve_with_scale(width, scale_factor)
        .unwrap_or(0.0);

    // Draw content area (yellow outline - area inside padding)
    if options.show_content_area
        && (padding.top.is_non_zero()
            || padding.right.is_non_zero()
            || padding.bottom.is_non_zero()
            || padding.left.is_non_zero())
    {
        let content_rect = Rect::new(
            [
                node_rect.min[0] + padding_left,
                node_rect.min[1] + padding_top,
            ],
            [
                node_rect.max[0] - padding_right,
                node_rect.max[1] - padding_bottom,
            ],
        );
        out.push((
            content_rect,
            clip_rect,
            Shape::Rect(
                StyledRect::new(Default::default(), Color::transparent())
                    .with_stroke(Stroke::new(Size::lpx(1.0), Color::rgb(1.0, 1.0, 0.0))),
            ),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }

    // Draw padding area (semi-transparent blue showing the padding inset)
    if options.show_padding
        && (padding.top.is_non_zero()
            || padding.right.is_non_zero()
            || padding.bottom.is_non_zero()
            || padding.left.is_non_zero())
    {
        // Draw top padding (full width)
        if padding.top.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.min[0], node_rect.min[1]],
                    [node_rect.max[0], node_rect.min[1] + padding_top],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(0.0, 0.0, 1.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw right padding (excluding top and bottom corners)
        if padding.right.is_non_zero() {
            out.push((
                Rect::new(
                    [
                        node_rect.max[0] - padding_right,
                        node_rect.min[1] + padding_top,
                    ],
                    [node_rect.max[0], node_rect.max[1] - padding_bottom],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(0.0, 0.0, 1.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw bottom padding (full width)
        if padding.bottom.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.min[0], node_rect.max[1] - padding_bottom],
                    [node_rect.max[0], node_rect.max[1]],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(0.0, 0.0, 1.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
        // Draw left padding (excluding top and bottom corners)
        if padding.left.is_non_zero() {
            out.push((
                Rect::new(
                    [node_rect.min[0], node_rect.min[1] + padding_top],
                    [
                        node_rect.min[0] + padding_left,
                        node_rect.max[1] - padding_bottom,
                    ],
                ),
                clip_rect,
                Shape::Rect(StyledRect::new(
                    Default::default(),
                    Color::rgba(0.0, 0.0, 1.0, 0.2),
                )),
                *transform,
                1.0,
                current_z_index,
                *tree_index,
            ));
            *tree_index += 1;
        }
    }

    // Draw node border (green outline for the actual node rect)
    if options.show_borders {
        out.push((
            node_rect,
            clip_rect,
            Shape::Rect(
                StyledRect::new(Default::default(), Color::transparent())
                    .with_stroke(Stroke::new(Size::ppx(1.0), Color::rgb(0.0, 1.0, 0.0))),
            ),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }

    // Draw clip rectangle (red outline showing the clipping boundary)
    // Clip rects are in world space and should NOT be transformed
    if options.show_clip_rects {
        out.push((
            clip_rect,
            clip_rect, // Don't clip the clip rect visualization itself
            Shape::Rect(
                StyledRect::new(Default::default(), Color::transparent())
                    .with_stroke(Stroke::new(Size::ppx(2.0), Color::rgb(1.0, 0.0, 0.0))),
            ),
            Transform2D::IDENTITY, // Clip rects are already in world space
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }

    // Draw transform origin crosshair
    if options.show_transform_origins {
        let width = node_rect.max[0] - node_rect.min[0];
        let height = node_rect.max[1] - node_rect.min[1];
        let (origin_x, origin_y) = node.transform_origin().resolve(width, height);
        let origin_world_x = node_rect.min[0] + origin_x;
        let origin_world_y = node_rect.min[1] + origin_y;

        let crosshair_size = 10.0;
        let crosshair_thickness = 2.0;

        // Horizontal line
        out.push((
            Rect::new(
                [
                    origin_world_x - crosshair_size,
                    origin_world_y - crosshair_thickness / 2.0,
                ],
                [
                    origin_world_x + crosshair_size,
                    origin_world_y + crosshair_thickness / 2.0,
                ],
            ),
            clip_rect,
            Shape::Rect(StyledRect::new(
                Default::default(),
                Color::rgb(1.0, 0.5, 0.0), // Orange
            )),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;

        // Vertical line
        out.push((
            Rect::new(
                [
                    origin_world_x - crosshair_thickness / 2.0,
                    origin_world_y - crosshair_size,
                ],
                [
                    origin_world_x + crosshair_thickness / 2.0,
                    origin_world_y + crosshair_size,
                ],
            ),
            clip_rect,
            Shape::Rect(StyledRect::new(
                Default::default(),
                Color::rgb(1.0, 0.5, 0.0), // Orange
            )),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;

        // Circle at center (hollow with stroke)
        use crate::primitives::CornerShape;
        let circle_radius = 8.0;
        let circle_rect = Rect::new(
            [
                origin_world_x - circle_radius,
                origin_world_y - circle_radius,
            ],
            [
                origin_world_x + circle_radius,
                origin_world_y + circle_radius,
            ],
        );
        out.push((
            circle_rect,
            clip_rect,
            Shape::Rect(
                StyledRect::new(circle_rect, Color::transparent())
                    .with_corner_shape(CornerShape::Round(Size::ppx(circle_radius)))
                    .with_stroke(Stroke::new(Size::ppx(2.0), Color::rgb(1.0, 0.5, 0.0))), // Orange stroke
            ),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }
}

fn collect_gap_debug_shapes(
    node: &Node,
    clip_rect: Rect,
    _options: &crate::debug::DebugOptions,
    transform: &Transform2D,
    out: &mut Vec<(
        Rect,
        Rect,
        Shape,
        Transform2D,
        f32,
        crate::layout::ZIndex,
        usize,
    )>,
    _scale_factor: f32,
    current_z_index: crate::layout::ZIndex,
    tree_index: &mut usize,
) {
    use crate::color::Color;
    use crate::layout::Layout;
    use crate::primitives::StyledRect;

    let children = node.children();
    if children.len() < 2 {
        return; // No gaps to visualize if fewer than 2 children
    }

    let layout_direction = node.layout_direction();

    // Draw gap rectangles between consecutive children
    for i in 0..children.len() - 1 {
        let current_child = &children[i];
        let next_child = &children[i + 1];

        // Get computed layouts for both children
        let Some(current_layout) = current_child.computed_layout() else {
            continue;
        };
        let Some(next_layout) = next_child.computed_layout() else {
            continue;
        };

        let current_rect = current_layout.rect;
        let next_rect = next_layout.rect;

        // Calculate gap rect based on layout direction
        let gap_rect = match layout_direction {
            Layout::Horizontal => {
                // Gap is between right edge of current and left edge of next
                Rect::new(
                    [current_rect.max[0], current_rect.min[1]],
                    [next_rect.min[0], current_rect.max[1]],
                )
            }
            Layout::Vertical => {
                // Gap is between bottom edge of current and top edge of next
                Rect::new(
                    [current_rect.min[0], current_rect.max[1]],
                    [current_rect.max[0], next_rect.min[1]],
                )
            }
            Layout::Stack => {
                // No gaps in Stack layout (children overlap)
                continue;
            }
        };

        // Draw purple semi-transparent rectangle for gap
        out.push((
            gap_rect,
            clip_rect,
            Shape::Rect(StyledRect::new(
                Default::default(),
                Color::rgba(0.5, 0.0, 0.5, 0.3), // Purple with 30% opacity
            )),
            *transform,
            1.0,
            current_z_index,
            *tree_index,
        ));
        *tree_index += 1;
    }
}
