use crate::content::{Content, HorizontalAlign, VerticalAlign};
use crate::layout::{
    ComputedLayout, Layout, Overflow, ScrollDirection, Size, Spacing, TransformOrigin, Translation,
    ZIndex,
};
use crate::measure::{ContentMeasurer, IntrinsicSize, MeasureTextRequest};
use crate::primitives::{Rect, Shape};
use crate::style::Style;
use crate::transition::Transition;

/// Unique identifier for a node, used for hit-testing and event routing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new NodeId from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A UI node that can contain a shape, content, and/or children
///
/// Nodes can be either:
/// - Container nodes: Have children and can have an optional background shape
/// - Content nodes: Have content (text, inputs, etc.) and cannot have children
/// - Mixed: Have both a shape and children (container with background)
///
/// All fields are private - use the builder pattern methods (`with_*`) to configure nodes.
pub struct Node {
    /// Optional identifier for this node (used for hit-testing and event routing)
    id: Option<NodeId>,
    /// Width of the node
    width: Size,
    /// Height of the node
    height: Size,
    /// Translation from the default position (post-layout transform)
    translation: Translation,
    /// Rotation in radians, clockwise positive (CSS convention)
    rotation: f32,
    /// Uniform scale factor (1.0 = no scale, 2.0 = double size, 0.5 = half size)
    scale: f32,
    /// Zoom level for browser-style zoom (scales logical pixels to physical pixels)
    /// None means inherit from parent. 1.0 = 100%, 2.0 = 200%, etc.
    zoom: Option<f32>,
    /// Pan offset for camera-style zoom (typically applied at root node)
    pan_offset: Translation,
    /// Transform origin for rotation and scale
    transform_origin: TransformOrigin,
    /// Padding inside the node
    padding: Spacing,
    /// Margin outside the node
    margin: Spacing,
    /// Gap between children in the layout direction
    gap: Size,
    /// Layout mode for children
    layout_direction: Layout,
    /// Horizontal alignment of children within this container
    ///
    /// For Horizontal layout: aligns children along the main axis (justify)
    /// For Vertical layout: aligns children along the cross axis
    /// For Stack layout: horizontal position of stacked children
    ///
    /// Default: `HorizontalAlign::Left`
    h_align: HorizontalAlign,
    /// Vertical alignment of children within this container
    ///
    /// For Horizontal layout: aligns children along the cross axis
    /// For Vertical layout: aligns children along the main axis (justify)
    /// For Stack layout: vertical position of stacked children
    ///
    /// Default: `VerticalAlign::Top`
    v_align: VerticalAlign,
    /// How overflow of content/children is handled.
    ///
    /// Default: `Overflow::Hidden`.
    overflow: Overflow,
    /// Current scroll offset for Overflow::Scroll containers (horizontal, vertical in pixels)
    ///
    /// Default: (0.0, 0.0)
    scroll_offset: (f32, f32),
    /// Target scroll offset for smooth scrolling animation
    ///
    /// Default: (0.0, 0.0)
    scroll_target: (f32, f32),
    /// Scroll speed multiplier for Overflow::Scroll containers
    ///
    /// Default: 2.0
    scroll_speed: f32,
    /// Scroll direction behavior
    ///
    /// Default: ScrollDirection::Inverted (natural scrolling)
    scroll_direction: ScrollDirection,
    /// Opacity of this node and all its children (0.0 = transparent, 1.0 = opaque).
    ///
    /// Default: 1.0 (fully opaque).
    opacity: f32,
    /// Optional shape to render for this node (background)
    shape: Option<Shape>,
    /// Optional content (text, inputs, etc.) - content nodes cannot have children
    content: Option<Content>,
    /// Child nodes (not allowed if content is Some)
    children: Vec<Node>,
    /// Computed layout (filled during layout pass)
    computed: Option<ComputedLayout>,
    /// Base style (always applied)
    base_style: Option<Style>,
    /// Style to apply when hovered (merged with base)
    hover_style: Option<Style>,
    /// Style to apply when active/pressed (merged with base + hover)
    active_style: Option<Style>,
    /// Style to apply when disabled (overrides all other styles)
    disabled_style: Option<Style>,
    /// Whether this node is disabled (cannot be interacted with)
    disabled: bool,
    /// Transition configuration for style changes
    transition: Option<Transition>,
    /// Z-index for controlling rendering order (None = inherit from parent)
    ///
    /// Higher values render on top. Default: None (inherits parent's z-index or 0)
    z_index: Option<ZIndex>,
}

impl Node {
    /// Create a new node with default settings
    pub fn new() -> Self {
        Self {
            id: None,
            width: Size::default(),
            height: Size::default(),
            translation: Translation::ZERO,
            rotation: 0.0,
            scale: 1.0,
            zoom: None,
            pan_offset: Translation::ZERO,
            transform_origin: TransformOrigin::center(),
            padding: Spacing::ZERO,
            margin: Spacing::ZERO,
            gap: Size::Logical(0.0),
            layout_direction: Layout::default(),
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
            overflow: Overflow::default(),
            scroll_offset: (0.0, 0.0),
            scroll_target: (0.0, 0.0),
            scroll_speed: 8.0,
            scroll_direction: ScrollDirection::default(),
            opacity: 1.0,
            shape: None,
            content: None,
            children: Vec::new(),
            computed: None,
            base_style: None,
            hover_style: None,
            active_style: None,
            disabled_style: None,
            disabled: false,
            transition: None,
            z_index: None,
        }
    }

    /// Check if this is a content node (has content, cannot have children)
    pub fn is_content_node(&self) -> bool {
        self.content.is_some()
    }

    /// Set the node ID (used for hit-testing and event routing)
    pub fn with_id(mut self, id: impl Into<NodeId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Get the node ID, if set
    pub fn id(&self) -> Option<&NodeId> {
        self.id.as_ref()
    }

    /// Set an auto-generated ID (internal use only, for interactive styles)
    #[doc(hidden)]
    pub fn set_auto_id(&mut self, id: NodeId) {
        if self.id.is_none() {
            self.id = Some(id);
        }
    }

    /// Set the width
    pub fn with_width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    /// Set the height
    pub fn with_height(mut self, height: Size) -> Self {
        self.height = height;
        self
    }

    /// Set both width and height to fixed pixel values
    pub fn with_size(self, width: f32, height: f32) -> Self {
        self.with_width(Size::lpx(width))
            .with_height(Size::lpx(height))
    }

    /// Set the translation (post-layout offset)
    pub fn with_translation(mut self, translation: Translation) -> Self {
        self.translation = translation;
        self
    }

    /// Set the offset (deprecated, use with_translation)
    #[deprecated(since = "0.2.0", note = "Use with_translation instead")]
    pub fn with_offset(mut self, offset: Translation) -> Self {
        self.translation = offset;
        self
    }

    /// Set the rotation in radians (clockwise positive, CSS convention)
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the scale factor (1.0 = no scale, 2.0 = double size, 0.5 = half size)
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the zoom level for browser-style zoom (scales logical pixels)
    /// 1.0 = 100%, 2.0 = 200%, 0.5 = 50%, etc.
    /// If set, overrides parent's zoom level. If None, inherits from parent.
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = Some(zoom);
        self
    }

    /// Set the pan offset for camera-style zoom (typically used on root node)
    pub fn with_pan_offset(mut self, pan_offset: Translation) -> Self {
        self.pan_offset = pan_offset;
        self
    }

    /// Set the transform origin for rotation and scale
    pub fn with_transform_origin(mut self, origin: TransformOrigin) -> Self {
        self.transform_origin = origin;
        self
    }

    /// Set the z-index for controlling layering order
    ///
    /// Higher values render on top of lower values. If not set, inherits parent's z-index.
    /// Nodes with the same z-index are rendered in tree order.
    pub fn with_z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// Set the padding
    pub fn with_padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    /// Set the margin
    pub fn with_margin(mut self, margin: Spacing) -> Self {
        self.margin = margin;
        self
    }

    /// Set the gap between children
    pub fn with_gap(mut self, gap: Size) -> Self {
        self.gap = gap;
        self
    }

    /// Set the layout mode
    pub fn with_layout_direction(mut self, direction: Layout) -> Self {
        self.layout_direction = direction;
        self
    }

    /// Set horizontal alignment of children within this container
    pub fn with_h_align(mut self, align: HorizontalAlign) -> Self {
        self.h_align = align;
        self
    }

    /// Set vertical alignment of children within this container
    pub fn with_v_align(mut self, align: VerticalAlign) -> Self {
        self.v_align = align;
        self
    }

    /// Set how overflow of content/children is handled (default: `Overflow::Hidden`).
    pub fn with_overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// Set the scroll speed multiplier (default: 1.0)
    pub fn with_scroll_speed(mut self, speed: f32) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// Set the scroll direction behavior (default: `ScrollDirection::Inverted`)
    pub fn with_scroll_direction(mut self, direction: ScrollDirection) -> Self {
        self.scroll_direction = direction;
        self
    }

    /// Set the opacity of this node and all its children (0.0 = transparent, 1.0 = opaque).
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set the shape
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Set the content (makes this a content node that cannot have children)
    pub fn with_content(mut self, content: Content) -> Self {
        assert!(
            self.children.is_empty(),
            "Cannot set content on a node that already has children"
        );
        self.content = Some(content);
        self
    }

    /// Set the base style (always applied)
    pub fn with_style(mut self, style: Style) -> Self {
        // Default shape to rect if not set
        if self.shape.is_none() {
            self.shape = Some(Shape::rect());
        }
        // Apply the style immediately for nodes without interactive states
        // (nodes with IDs will have styles applied via InteractiveStateManager)
        style.apply_to_node(&mut self);
        self.base_style = Some(style);
        self
    }

    /// Set the hover style (merged with base when hovered)
    pub fn with_hover_style(mut self, style: Style) -> Self {
        self.hover_style = Some(style);
        self
    }

    /// Set the active style (merged with base + hover when pressed/active)
    pub fn with_active_style(mut self, style: Style) -> Self {
        self.active_style = Some(style);
        self
    }

    /// Set the disabled style (used when node is disabled, overrides other styles)
    pub fn with_disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = Some(style);
        self
    }

    /// Set whether this node is disabled (cannot be interacted with)
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the transition configuration for style changes
    pub fn with_transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: Node) -> Self {
        assert!(
            self.content.is_none(),
            "Cannot add children to a content node"
        );
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        assert!(
            self.content.is_none(),
            "Cannot add children to a content node"
        );
        self.children.extend(children);
        self
    }

    /// Get the computed layout (if available)
    pub fn computed_layout(&self) -> Option<&ComputedLayout> {
        self.computed.as_ref()
    }

    // Internal getters for fields (used by output.rs and other internal modules)

    /// Get the opacity value
    pub(crate) fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Set the opacity value (used by style system)
    pub(crate) fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    /// Get the translation
    pub(crate) fn translation(&self) -> Translation {
        self.translation
    }

    /// Set the translation (used by style system)
    pub(crate) fn set_translation(&mut self, translation: Translation) {
        self.translation = translation;
    }

    /// Get the rotation
    pub(crate) fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Set the rotation (used by style system)
    pub(crate) fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Get the scale factor
    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    /// Get the pan offset
    pub(crate) fn pan_offset(&self) -> Translation {
        self.pan_offset
    }

    /// Get the zoom level (if set on this node)
    pub(crate) fn zoom_level(&self) -> Option<f32> {
        self.zoom
    }

    /// Get the transform origin
    pub(crate) fn transform_origin(&self) -> TransformOrigin {
        self.transform_origin
    }

    /// Set the transform origin (used by style system)
    pub(crate) fn set_transform_origin(&mut self, origin: TransformOrigin) {
        self.transform_origin = origin;
    }

    /// Get the overflow policy
    pub fn overflow(&self) -> Overflow {
        self.overflow
    }

    /// Get the z-index for controlling layering order
    pub fn z_index(&self) -> Option<ZIndex> {
        self.z_index
    }

    /// Get the scroll offset (horizontal, vertical)
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset
    }

    /// Set the scroll offset (horizontal, vertical)
    pub fn set_scroll_offset(&mut self, offset: (f32, f32)) {
        self.scroll_offset = offset;
    }

    /// Get the scroll target (for smooth scrolling)
    pub fn scroll_target(&self) -> (f32, f32) {
        self.scroll_target
    }

    /// Set the scroll target (for smooth scrolling)
    pub fn set_scroll_target(&mut self, target: (f32, f32)) {
        self.scroll_target = target;
    }

    /// Scroll by a delta (horizontal, vertical) - updates the target for smooth scrolling
    pub fn scroll_by(&mut self, delta: (f32, f32)) {
        self.scroll_target.0 += delta.0;
        self.scroll_target.1 += delta.1;
        // Clamping will be done by scroll processing
    }

    /// Get the scroll speed multiplier
    pub fn scroll_speed(&self) -> f32 {
        self.scroll_speed
    }

    /// Get the scroll direction behavior
    pub fn scroll_direction(&self) -> ScrollDirection {
        self.scroll_direction
    }

    /// Update smooth scrolling animation
    ///
    /// This should be called once per frame with the delta time in seconds.
    /// It interpolates the current scroll offset toward the target scroll offset.
    ///
    /// Returns true if scrolling is in progress (not yet at target).
    pub fn update_scroll_animation(&mut self, dt: f32) -> bool {
        const SCROLL_SMOOTHNESS: f32 = 10.0; // Higher = faster, lower = smoother

        if self.scroll_offset == self.scroll_target {
            return false; // Already at target
        }

        let t = 1.0 - (-SCROLL_SMOOTHNESS * dt).exp(); // Exponential ease-out

        self.scroll_offset.0 += (self.scroll_target.0 - self.scroll_offset.0) * t;
        self.scroll_offset.1 += (self.scroll_target.1 - self.scroll_offset.1) * t;

        // Snap to target if very close (within 0.1 pixels)
        if (self.scroll_target.0 - self.scroll_offset.0).abs() < 0.1 {
            self.scroll_offset.0 = self.scroll_target.0;
        }
        if (self.scroll_target.1 - self.scroll_offset.1).abs() < 0.1 {
            self.scroll_offset.1 = self.scroll_target.1;
        }

        true // Still animating
    }

    /// Recursively update scroll animations for this node and all children
    ///
    /// Returns true if any node is still animating.
    pub fn update_all_scroll_animations(&mut self, dt: f32) -> bool {
        let mut any_animating = self.update_scroll_animation(dt);

        for child in self.children_mut() {
            if child.update_all_scroll_animations(dt) {
                any_animating = true;
            }
        }

        any_animating
    }

    /// Get the shape, if any
    pub(crate) fn shape(&self) -> Option<&Shape> {
        self.shape.as_ref()
    }

    /// Get mutable reference to the shape (used by style system)
    pub(crate) fn shape_mut(&mut self) -> Option<&mut Shape> {
        self.shape.as_mut()
    }

    /// Get the content, if any
    pub(crate) fn content(&self) -> Option<&Content> {
        self.content.as_ref()
    }

    /// Get mutable reference to the content (used by style system)
    pub(crate) fn content_mut(&mut self) -> Option<&mut Content> {
        self.content.as_mut()
    }

    /// Get the padding
    pub fn padding(&self) -> Spacing {
        self.padding
    }

    /// Get the margin
    pub(crate) fn margin(&self) -> Spacing {
        self.margin
    }

    /// Get the gap between children
    pub fn gap(&self) -> Size {
        self.gap
    }

    /// Get the layout mode
    pub fn layout_direction(&self) -> Layout {
        self.layout_direction
    }

    pub fn h_align(&self) -> HorizontalAlign {
        self.h_align
    }

    pub fn v_align(&self) -> VerticalAlign {
        self.v_align
    }

    /// Get the children
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Get mutable reference to children (used by style system)
    pub fn children_mut(&mut self) -> &mut [Node] {
        &mut self.children
    }

    /// Get the base style
    pub fn base_style(&self) -> Option<&Style> {
        self.base_style.as_ref()
    }

    /// Get the hover style
    pub fn hover_style(&self) -> Option<&Style> {
        self.hover_style.as_ref()
    }

    /// Get the active style
    pub fn active_style(&self) -> Option<&Style> {
        self.active_style.as_ref()
    }

    /// Get the disabled style
    pub fn disabled_style(&self) -> Option<&Style> {
        self.disabled_style.as_ref()
    }

    /// Check if this node is disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Get the transition configuration
    pub fn transition(&self) -> Option<&Transition> {
        self.transition.as_ref()
    }

    /// Measure the intrinsic size of this node (content + padding, excluding margins).
    ///
    /// This recursively measures children and applies the same margin/gap collapsing
    /// rules as layout to ensure measured sizes match final layout.
    ///
    /// Returns the node's "border-box" size (content + padding), NOT including margins.
    /// Parent is responsible for adding margins when positioning.
    ///
    /// NOTE: This always measures content size, regardless of the node's Size type.
    /// The Size type only matters when the parent is aggregating children for FitContent sizing.
    fn measure_node(&self, measurer: &mut dyn ContentMeasurer, scale_factor: f32) -> IntrinsicSize {
        // Short-circuit: if both dimensions are Fixed, we can return immediately
        if let (Size::Logical(w), Size::Logical(h)) = (self.width, self.height) {
            return IntrinsicSize::new(w, h);
        }

        // Measure width - only FitContent measures children
        let width = match self.width {
            Size::Logical(w) => w,
            Size::FitContent => {
                let content_width = if let Some(content) = &self.content {
                    match content {
                        Content::Text(text_content) => {
                            let mut request = MeasureTextRequest::from_text_content(text_content);
                            request.font_size *= scale_factor;
                            measurer.measure_text(request).width
                        }
                    }
                } else if !self.children.is_empty() {
                    self.measure_children(measurer, scale_factor).width
                } else {
                    0.0
                };
                let padding_left = self
                    .padding
                    .left
                    .try_resolve_with_scale(content_width, scale_factor)
                    .unwrap_or(0.0);
                let padding_right = self
                    .padding
                    .right
                    .try_resolve_with_scale(content_width, scale_factor)
                    .unwrap_or(0.0);
                content_width + padding_left + padding_right
            }
            _ => {
                // Fill/Relative: don't measure children, no intrinsic size
                0.0
            }
        };

        // Measure height - only FitContent measures children
        let height = match self.height {
            Size::Logical(h) => h,
            Size::FitContent => {
                let content_height = if let Some(content) = &self.content {
                    match content {
                        Content::Text(text_content) => {
                            let mut request = MeasureTextRequest::from_text_content(text_content);
                            request.font_size *= scale_factor;
                            measurer.measure_text(request).height
                        }
                    }
                } else if !self.children.is_empty() {
                    self.measure_children(measurer, scale_factor).height
                } else {
                    0.0
                };
                let padding_top = self
                    .padding
                    .top
                    .try_resolve_with_scale(content_height, scale_factor)
                    .unwrap_or(0.0);
                let padding_bottom = self
                    .padding
                    .bottom
                    .try_resolve_with_scale(content_height, scale_factor)
                    .unwrap_or(0.0);
                content_height + padding_top + padding_bottom
            }
            _ => {
                // Fill/Relative: don't measure children, no intrinsic size
                0.0
            }
        };

        IntrinsicSize::new(width, height)
    }

    /// Measure the intrinsic content size of a container based on its children.
    ///
    /// This uses the same margin/gap collapsing logic as layout to ensure consistency.
    /// IMPORTANT: Only aggregates FitContent children. Fill/Relative children are still
    /// measured (for layout purposes) but don't contribute to parent's intrinsic size.
    ///
    /// OPTIMIZATION: Avoids Vec allocation by computing width/height in a single pass
    fn measure_children(
        &self,
        measurer: &mut dyn ContentMeasurer,
        scale_factor: f32,
    ) -> IntrinsicSize {
        if self.children.is_empty() {
            return IntrinsicSize::zero();
        }

        // Calculate spacing using the same collapsing rules as layout
        // Note: For measurement, we use a large reference size for resolving margins/gaps
        // The actual parent size isn't known yet during intrinsic measurement
        let ref_size = 10000.0; // Large reference size for percentage-based margins
        let scaled_gap = self
            .gap
            .try_resolve_with_scale(ref_size, scale_factor)
            .unwrap_or(0.0);

        let (total_horizontal_spacing, total_vertical_spacing) = match self.layout_direction {
            Layout::Horizontal => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        total += child
                            .margin
                            .left
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                    }

                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        let child_right = child
                            .margin
                            .right
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                        let next_left = next_child
                            .margin
                            .left
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_right.max(next_left);
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        total += child
                            .margin
                            .right
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (total, 0.0)
            }
            Layout::Vertical => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        total += child
                            .margin
                            .top
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                    }

                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        let child_bottom = child
                            .margin
                            .bottom
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                        let next_top = next_child
                            .margin
                            .top
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_bottom.max(next_top);
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        total += child
                            .margin
                            .bottom
                            .try_resolve_with_scale(ref_size, scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (0.0, total)
            }
            Layout::Stack => {
                // In Stack layout, children don't take up space linearly, so no spacing
                (0.0, 0.0)
            }
        };

        // Compute intrinsic size based on layout direction
        // OPTIMIZATION: Measure and aggregate in a single pass to avoid Vec allocation
        match self.layout_direction {
            Layout::Horizontal => {
                // Width: sum of child widths + spacing (main axis)
                // Height: max of child heights (cross axis)
                let mut total_width = 0.0f32;
                let mut max_height = 0.0f32;

                for child in &self.children {
                    let size = child.measure_node(measurer, scale_factor);
                    total_width += size.width;
                    max_height = max_height.max(size.height);
                }

                IntrinsicSize::new(total_width + total_horizontal_spacing, max_height)
            }
            Layout::Vertical => {
                // Height: sum of child heights + spacing (main axis)
                // Width: max of child widths (cross axis)
                let mut total_height = 0.0f32;
                let mut max_width = 0.0f32;

                for child in &self.children {
                    let size = child.measure_node(measurer, scale_factor);
                    total_height += size.height;
                    max_width = max_width.max(size.width);
                }

                IntrinsicSize::new(max_width, total_height + total_vertical_spacing)
            }
            Layout::Stack => {
                // Stack: max of all child sizes (children overlap in Z)
                let mut max_width = 0.0f32;
                let mut max_height = 0.0f32;

                for child in &self.children {
                    let size = child.measure_node(measurer, scale_factor);
                    max_width = max_width.max(size.width);
                    max_height = max_height.max(size.height);
                }

                IntrinsicSize::new(max_width, max_height)
            }
        }
    }

    /// Compute layout for this node and all children
    ///
    /// `available_rect` is the space available for this node (typically parent's content area)
    pub fn compute_layout(&mut self, available_rect: Rect) {
        self.compute_layout_with_scale_factor(available_rect, 1.0);
    }

    /// Compute layout with a scale factor for logical-to-physical pixel conversion
    ///
    /// `scale_factor` is multiplied with all Fixed sizes, padding, margins, gaps, and font sizes
    pub fn compute_layout_with_scale_factor(&mut self, available_rect: Rect, scale_factor: f32) {
        self.compute_layout_with_parent_size(
            available_rect,
            available_rect.width(),
            available_rect.height(),
            scale_factor,
        );
    }

    /// Compute layout with a measurer for resolving `Size::FitContent`.
    ///
    /// This is the recommended entry point when using FitContent sizing.
    pub fn compute_layout_with_measurer(
        &mut self,
        available_rect: Rect,
        measurer: &mut dyn ContentMeasurer,
    ) {
        self.compute_layout_with_measurer_and_scale_factor(available_rect, measurer, 1.0);
    }

    /// Compute layout with both a measurer and scale factor
    pub fn compute_layout_with_measurer_and_scale_factor(
        &mut self,
        available_rect: Rect,
        measurer: &mut dyn ContentMeasurer,
        scale_factor: f32,
    ) {
        self.compute_layout_with_parent_size_and_measurer(
            available_rect,
            available_rect.width(),
            available_rect.height(),
            measurer,
            Overflow::Visible, // Root has no parent, assume Visible
            scale_factor,
        );
    }

    /// Recursively offset this node and all its descendants by the given delta
    fn offset_layout_recursive(&mut self, x_delta: f32, y_delta: f32) {
        if let Some(computed) = &mut self.computed {
            computed.rect.min[0] += x_delta;
            computed.rect.max[0] += x_delta;
            computed.rect.min[1] += y_delta;
            computed.rect.max[1] += y_delta;
        }

        for child in &mut self.children {
            child.offset_layout_recursive(x_delta, y_delta);
        }
    }

    fn compute_layout_with_parent_size_and_measurer(
        &mut self,
        available_rect: Rect,
        parent_width: f32,
        parent_height: f32,
        measurer: &mut dyn ContentMeasurer,
        parent_overflow: Overflow,
        scale_factor: f32,
    ) {
        // Use this node's zoom_level if set, otherwise inherit parent's scale_factor
        let effective_scale_factor = self.zoom.unwrap_or(scale_factor);

        // Account for this node's margins when calculating available space
        // Resolve margin values with effective_scale_factor (logical -> physical pixels)
        let margin_left = self
            .margin
            .left
            .try_resolve_with_scale(parent_width, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_right = self
            .margin
            .right
            .try_resolve_with_scale(parent_width, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_top = self
            .margin
            .top
            .try_resolve_with_scale(parent_height, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_bottom = self
            .margin
            .bottom
            .try_resolve_with_scale(parent_height, effective_scale_factor)
            .unwrap_or(0.0);

        let available_width = (parent_width - margin_left - margin_right).max(0.0);
        let available_height = (parent_height - margin_top - margin_bottom).max(0.0);

        // Resolve width and height
        // IMPORTANT: Only measure FitContent dimensions. For Fixed/Relative/Fill, use constraints directly.
        // This prevents children from incorrectly affecting parent sizes when parent has constrained dimensions.
        //
        // OPTIMIZATION: Cache measurement result to avoid calling measure_node() twice when both
        // width and height are FitContent
        let measured_size = if self.width.is_fit_content() || self.height.is_fit_content() {
            Some(self.measure_node(measurer, effective_scale_factor))
        } else {
            None
        };

        let width = if self.width.is_fit_content() {
            let measured_width = measured_size.as_ref().unwrap().width;

            if parent_overflow == Overflow::Visible {
                // Parent allows overflow, so use full measured width
                measured_width
            } else {
                // Parent clips overflow, so clamp to available width
                measured_width.min(available_width)
            }
        } else {
            self.width
                .try_resolve_with_scale(available_width, effective_scale_factor)
                .unwrap_or(available_width)
        };

        let height = if self.height.is_fit_content() {
            let measured_height = measured_size.as_ref().unwrap().height;

            if parent_overflow == Overflow::Visible {
                // Parent allows overflow, so use full measured height
                measured_height
            } else {
                // Parent clips overflow, so clamp to available height
                measured_height.min(available_height)
            }
        } else {
            self.height
                .try_resolve_with_scale(available_height, effective_scale_factor)
                .unwrap_or(available_height)
        };

        // Position is already adjusted for margins by parent, don't add them again
        let outer_x = available_rect.min[0];
        let outer_y = available_rect.min[1];

        // Content area (after subtracting padding)
        // Resolve padding values with effective_scale_factor (logical -> physical pixels)
        let padding_left = self
            .padding
            .left
            .try_resolve_with_scale(width, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_right = self
            .padding
            .right
            .try_resolve_with_scale(width, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_top = self
            .padding
            .top
            .try_resolve_with_scale(height, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_bottom = self
            .padding
            .bottom
            .try_resolve_with_scale(height, effective_scale_factor)
            .unwrap_or(0.0);

        let content_x = outer_x + padding_left;
        let content_y = outer_y + padding_top;
        let content_width = width - padding_left - padding_right;
        let content_height = height - padding_top - padding_bottom;

        // Store computed layout for this node (untransformed - translation applied during rendering)
        self.computed = Some(ComputedLayout::new(Rect::new(
            [outer_x, outer_y],
            [outer_x + width, outer_y + height],
        )));

        // Layout children (same as original, but passing measurer through)
        let mut current_x = content_x;
        let mut current_y = content_y;

        // Calculate total spacing in the layout direction (margins + gaps)
        // Resolve gap and child margins with effective_scale_factor (logical -> physical pixels)
        let scaled_gap = self
            .gap
            .try_resolve_with_scale(content_width.max(content_height), effective_scale_factor)
            .unwrap_or(0.0);
        let (total_horizontal_spacing, total_vertical_spacing) = match self.layout_direction {
            Layout::Horizontal => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        // First child: left margin doesn't collapse with parent padding
                        total += child
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }

                    // Between this child and the next, collapse gap with margins
                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        // Collapsed margin is the max of the two adjacent margins (scaled)
                        let child_right = child
                            .margin
                            .right
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                        let next_left = next_child
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_right.max(next_left);
                        // Collapse gap with margin - use the larger of gap or collapsed margin
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        // Last child: just add its right margin
                        total += child
                            .margin
                            .right
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (total, 0.0)
            }
            Layout::Vertical => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        // First child: top margin doesn't collapse with parent padding
                        total += child
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }

                    // Between this child and the next, collapse gap with margins
                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        // Collapsed margin is the max of the two adjacent margins (scaled)
                        let child_bottom = child
                            .margin
                            .bottom
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                        let next_top = next_child
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_bottom.max(next_top);
                        // Collapse gap with margin - use the larger of gap or collapsed margin
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        // Last child: just add its bottom margin
                        total += child
                            .margin
                            .bottom
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (0.0, total)
            }
            Layout::Stack => {
                // In Stack layout, children don't take up space linearly, so no spacing
                (0.0, 0.0)
            }
        };

        // Space available for children after subtracting spacing (margins + gaps)
        let available_width = (content_width - total_horizontal_spacing).max(0.0);
        let available_height = (content_height - total_vertical_spacing).max(0.0);

        // Calculate remaining space for Fill children
        let (fill_size_width, fill_size_height) = match self.layout_direction {
            Layout::Horizontal => {
                let mut fill_count = 0;
                let mut used_width = 0.0;

                for child in &self.children {
                    if child.width.is_fill() {
                        fill_count += 1;
                    } else if child.width.is_fit_content() {
                        used_width += child.measure_node(measurer, effective_scale_factor).width;
                    } else {
                        // Must be Fixed or Relative
                        used_width += child
                            .width
                            .try_resolve_with_scale(available_width, effective_scale_factor)
                            .unwrap();
                    }
                }

                let remaining_width = (available_width - used_width).max(0.0);
                let fill_width = if fill_count > 0 {
                    remaining_width / fill_count as f32
                } else {
                    0.0
                };

                (fill_width, available_height)
            }
            Layout::Vertical => {
                let mut fill_count = 0;
                let mut used_height = 0.0;

                for child in &self.children {
                    if child.height.is_fill() {
                        fill_count += 1;
                    } else if child.height.is_fit_content() {
                        used_height += child.measure_node(measurer, effective_scale_factor).height;
                    } else {
                        // Must be Fixed or Relative
                        used_height += child
                            .height
                            .try_resolve_with_scale(available_height, effective_scale_factor)
                            .unwrap();
                    }
                }

                let remaining_height = (available_height - used_height).max(0.0);
                let fill_height = if fill_count > 0 {
                    remaining_height / fill_count as f32
                } else {
                    0.0
                };

                (available_width, fill_height)
            }
            Layout::Stack => {
                // In Stack layout, all children get full available space
                (available_width, available_height)
            }
        };

        // Calculate total size of children for alignment
        let (total_children_width, total_children_height) = match self.layout_direction {
            Layout::Horizontal => {
                let mut total_width = total_horizontal_spacing;
                for child in &self.children {
                    if child.width.is_fill() {
                        total_width += fill_size_width;
                    } else if child.width.is_fit_content() {
                        total_width += child.measure_node(measurer, effective_scale_factor).width;
                    } else {
                        total_width += child
                            .width
                            .try_resolve_with_scale(available_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (total_width, content_height)
            }
            Layout::Vertical => {
                let mut total_height = total_vertical_spacing;
                for child in &self.children {
                    if child.height.is_fill() {
                        total_height += fill_size_height;
                    } else if child.height.is_fit_content() {
                        total_height += child.measure_node(measurer, effective_scale_factor).height;
                    } else {
                        total_height += child
                            .height
                            .try_resolve_with_scale(available_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (content_width, total_height)
            }
            Layout::Stack => {
                // For stack, use content dimensions
                (content_width, content_height)
            }
        };

        // Apply alignment offset
        match self.layout_direction {
            Layout::Horizontal => {
                // h_align controls main axis (justify)
                let remaining_width = (content_width - total_children_width).max(0.0);
                current_x += match self.h_align {
                    HorizontalAlign::Left => 0.0,
                    HorizontalAlign::Center => remaining_width / 2.0,
                    HorizontalAlign::Right => remaining_width,
                };

                // v_align controls cross axis
                current_y += match self.v_align {
                    VerticalAlign::Top => 0.0,
                    VerticalAlign::Center => 0.0, // Will be applied per-child
                    VerticalAlign::Bottom => 0.0, // Will be applied per-child
                };
            }
            Layout::Vertical => {
                // v_align controls main axis (justify)
                let remaining_height = (content_height - total_children_height).max(0.0);
                current_y += match self.v_align {
                    VerticalAlign::Top => 0.0,
                    VerticalAlign::Center => remaining_height / 2.0,
                    VerticalAlign::Bottom => remaining_height,
                };

                // h_align controls cross axis
                current_x += match self.h_align {
                    HorizontalAlign::Left => 0.0,
                    HorizontalAlign::Center => 0.0, // Will be applied per-child
                    HorizontalAlign::Right => 0.0,  // Will be applied per-child
                };
            }
            Layout::Stack => {
                // Both alignments apply to all stacked children
                current_x += match self.h_align {
                    HorizontalAlign::Left => 0.0,
                    HorizontalAlign::Center => 0.0, // Will be applied per-child based on child size
                    HorizontalAlign::Right => 0.0,  // Will be applied per-child based on child size
                };

                current_y += match self.v_align {
                    VerticalAlign::Top => 0.0,
                    VerticalAlign::Center => 0.0, // Will be applied per-child based on child size
                    VerticalAlign::Bottom => 0.0, // Will be applied per-child based on child size
                };
            }
        }

        let num_children = self.children.len();
        for i in 0..num_children {
            if i == 0 {
                match self.layout_direction {
                    Layout::Horizontal => {
                        current_x += self.children[i]
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                    Layout::Vertical => {
                        current_y += self.children[i]
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                    Layout::Stack => {
                        // In Stack layout, don't advance position for first child
                    }
                }
            }

            let child_available_rect = match self.layout_direction {
                Layout::Horizontal => Rect::new(
                    [current_x, current_y],
                    [content_x + content_width, content_y + content_height],
                ),
                Layout::Vertical => Rect::new(
                    [current_x, current_y],
                    [content_x + content_width, content_y + content_height],
                ),
                Layout::Stack => {
                    // In Stack layout, all children start at content origin
                    Rect::new(
                        [content_x, content_y],
                        [content_x + content_width, content_y + content_height],
                    )
                }
            };

            // Resolve child margins with effective_scale_factor before calculating parent dimensions
            let child_margin_left = self.children[i]
                .margin
                .left
                .try_resolve_with_scale(content_width, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_right = self.children[i]
                .margin
                .right
                .try_resolve_with_scale(content_width, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_top = self.children[i]
                .margin
                .top
                .try_resolve_with_scale(content_height, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_bottom = self.children[i]
                .margin
                .bottom
                .try_resolve_with_scale(content_height, effective_scale_factor)
                .unwrap_or(0.0);

            let child_parent_width = if self.children[i].width.is_fill() {
                fill_size_width + child_margin_left + child_margin_right
            } else {
                available_width + child_margin_left + child_margin_right
            };
            let child_parent_height = if self.children[i].height.is_fill() {
                fill_size_height + child_margin_top + child_margin_bottom
            } else {
                available_height + child_margin_top + child_margin_bottom
            };

            self.children[i].compute_layout_with_parent_size_and_measurer(
                child_available_rect,
                child_parent_width,
                child_parent_height,
                measurer,
                self.overflow, // Pass this node's overflow to children
                effective_scale_factor,
            );

            // Apply cross-axis alignment after computing child layout
            if let Some(child_layout) = self.children[i].computed_layout() {
                let child_rect = child_layout.rect;
                let child_width = child_rect.max[0] - child_rect.min[0];
                let child_height = child_rect.max[1] - child_rect.min[1];

                let (x_delta, y_delta) = match self.layout_direction {
                    Layout::Horizontal => {
                        // For horizontal layout, v_align controls cross-axis (vertical) alignment
                        let available_height = content_height;
                        let offset_y = match self.v_align {
                            VerticalAlign::Top => 0.0,
                            VerticalAlign::Center => (available_height - child_height) / 2.0,
                            VerticalAlign::Bottom => available_height - child_height,
                        };
                        let new_y = content_y + offset_y;
                        (0.0, new_y - child_rect.min[1])
                    }
                    Layout::Vertical => {
                        // For vertical layout, h_align controls cross-axis (horizontal) alignment
                        let available_width = content_width;
                        let offset_x = match self.h_align {
                            HorizontalAlign::Left => 0.0,
                            HorizontalAlign::Center => (available_width - child_width) / 2.0,
                            HorizontalAlign::Right => available_width - child_width,
                        };
                        let new_x = content_x + offset_x;
                        (new_x - child_rect.min[0], 0.0)
                    }
                    Layout::Stack => {
                        // For stack layout, apply both alignments
                        let available_width = content_width;
                        let available_height = content_height;

                        let offset_x = match self.h_align {
                            HorizontalAlign::Left => 0.0,
                            HorizontalAlign::Center => (available_width - child_width) / 2.0,
                            HorizontalAlign::Right => available_width - child_width,
                        };
                        let offset_y = match self.v_align {
                            VerticalAlign::Top => 0.0,
                            VerticalAlign::Center => (available_height - child_height) / 2.0,
                            VerticalAlign::Bottom => available_height - child_height,
                        };

                        let new_x = content_x + offset_x;
                        let new_y = content_y + offset_y;
                        (new_x - child_rect.min[0], new_y - child_rect.min[1])
                    }
                };

                // Recursively offset this child and all its descendants
                self.children[i].offset_layout_recursive(x_delta, y_delta);

                // Get updated child_rect after offset for position tracking
                let child_rect = self.children[i].computed_layout().unwrap().rect;

                if i + 1 < num_children {
                    match self.layout_direction {
                        Layout::Horizontal => {
                            // Collapse margins with effective_scale_factor applied
                            let child_right = self.children[i]
                                .margin
                                .right
                                .try_resolve_with_scale(content_width, effective_scale_factor)
                                .unwrap_or(0.0);
                            let next_left = self.children[i + 1]
                                .margin
                                .left
                                .try_resolve_with_scale(content_width, effective_scale_factor)
                                .unwrap_or(0.0);
                            let collapsed_margin = child_right.max(next_left);
                            let spacing = scaled_gap.max(collapsed_margin);
                            current_x = child_rect.max[0] + spacing;
                        }
                        Layout::Vertical => {
                            // Collapse margins with effective_scale_factor applied
                            let child_bottom = self.children[i]
                                .margin
                                .bottom
                                .try_resolve_with_scale(content_height, effective_scale_factor)
                                .unwrap_or(0.0);
                            let next_top = self.children[i + 1]
                                .margin
                                .top
                                .try_resolve_with_scale(content_height, effective_scale_factor)
                                .unwrap_or(0.0);
                            let collapsed_margin = child_bottom.max(next_top);
                            let spacing = scaled_gap.max(collapsed_margin);
                            current_y = child_rect.max[1] + spacing;
                        }
                        Layout::Stack => {
                            // In Stack layout, don't advance position (children overlap)
                        }
                    }
                }
            }
        }
    }

    fn compute_layout_with_parent_size(
        &mut self,
        available_rect: Rect,
        parent_width: f32,
        parent_height: f32,
        scale_factor: f32,
    ) {
        // Use this node's zoom_level if set, otherwise inherit parent's scale_factor
        let effective_scale_factor = self.zoom.unwrap_or(scale_factor);

        // Account for this node's margins when calculating available space
        // Resolve margin values with effective_scale_factor (logical -> physical pixels)
        let margin_left = self
            .margin
            .left
            .try_resolve_with_scale(parent_width, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_right = self
            .margin
            .right
            .try_resolve_with_scale(parent_width, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_top = self
            .margin
            .top
            .try_resolve_with_scale(parent_height, effective_scale_factor)
            .unwrap_or(0.0);
        let margin_bottom = self
            .margin
            .bottom
            .try_resolve_with_scale(parent_height, effective_scale_factor)
            .unwrap_or(0.0);

        let available_width = (parent_width - margin_left - margin_right).max(0.0);
        let available_height = (parent_height - margin_top - margin_bottom).max(0.0);

        // Resolve width and height from available space (after margins)
        // NOTE: Without a measurer, FitContent falls back to available size
        // Apply effective_scale_factor to Fixed sizes (logical -> physical pixels)
        let width = self
            .width
            .try_resolve_with_scale(available_width, effective_scale_factor)
            .unwrap_or(available_width);
        let height = self
            .height
            .try_resolve_with_scale(available_height, effective_scale_factor)
            .unwrap_or(available_height);

        // Position is already adjusted for margins by parent, don't add them again
        let outer_x = available_rect.min[0];
        let outer_y = available_rect.min[1];

        // Content area (after subtracting padding)
        // Resolve padding values with effective_scale_factor (logical -> physical pixels)
        let padding_left = self
            .padding
            .left
            .try_resolve_with_scale(width, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_right = self
            .padding
            .right
            .try_resolve_with_scale(width, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_top = self
            .padding
            .top
            .try_resolve_with_scale(height, effective_scale_factor)
            .unwrap_or(0.0);
        let padding_bottom = self
            .padding
            .bottom
            .try_resolve_with_scale(height, effective_scale_factor)
            .unwrap_or(0.0);

        let content_x = outer_x + padding_left;
        let content_y = outer_y + padding_top;
        let content_width = width - padding_left - padding_right;
        let content_height = height - padding_top - padding_bottom;

        // Store computed layout for this node (untransformed - translation applied during rendering)
        self.computed = Some(ComputedLayout::new(Rect::new(
            [outer_x, outer_y],
            [outer_x + width, outer_y + height],
        )));

        // Layout children
        let mut current_x = content_x;
        let mut current_y = content_y;

        // Calculate total spacing in the layout direction (margins + gaps)
        // Resolve gap and child margins with effective_scale_factor (logical -> physical pixels)
        let scaled_gap = self
            .gap
            .try_resolve_with_scale(content_width.max(content_height), effective_scale_factor)
            .unwrap_or(0.0);
        let (total_horizontal_spacing, total_vertical_spacing) = match self.layout_direction {
            Layout::Horizontal => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        // First child: left margin doesn't collapse with parent padding
                        total += child
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }

                    // Between this child and the next, collapse gap with margins
                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        // Collapsed margin is the max of the two adjacent margins (scaled)
                        let child_right = child
                            .margin
                            .right
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                        let next_left = next_child
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_right.max(next_left);
                        // Collapse gap with margin - use the larger of gap or collapsed margin
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        // Last child: just add its right margin
                        total += child
                            .margin
                            .right
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (total, 0.0)
            }
            Layout::Vertical => {
                let mut total = 0.0f32;
                for (i, child) in self.children.iter().enumerate() {
                    if i == 0 {
                        // First child: top margin doesn't collapse with parent padding
                        total += child
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }

                    // Between this child and the next, collapse gap with margins
                    if i + 1 < self.children.len() {
                        let next_child = &self.children[i + 1];
                        // Collapsed margin is the max of the two adjacent margins (scaled)
                        let child_bottom = child
                            .margin
                            .bottom
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                        let next_top = next_child
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                        let collapsed_margin = child_bottom.max(next_top);
                        // Collapse gap with margin - use the larger of gap or collapsed margin
                        total += scaled_gap.max(collapsed_margin);
                    } else {
                        // Last child: just add its bottom margin
                        total += child
                            .margin
                            .bottom
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                }
                (0.0, total)
            }
            Layout::Stack => {
                // In Stack layout, children don't take up space linearly, so no spacing
                (0.0, 0.0)
            }
        };

        // Space available for children after subtracting spacing (margins + gaps)
        let available_width = (content_width - total_horizontal_spacing).max(0.0);
        let available_height = (content_height - total_vertical_spacing).max(0.0);

        // Calculate remaining space for Fill children
        let (fill_size_width, fill_size_height) = match self.layout_direction {
            Layout::Horizontal => {
                // Count Fill children and calculate space used by non-Fill children
                let mut fill_count = 0;
                let mut used_width = 0.0;

                for child in &self.children {
                    if child.width.is_fill() {
                        fill_count += 1;
                    } else {
                        // For FitContent without measurer, fall back to available width
                        // Apply effective_scale_factor to Fixed sizes
                        used_width += child
                            .width
                            .try_resolve_with_scale(available_width, effective_scale_factor)
                            .unwrap_or(available_width);
                    }
                }

                // Fill children divide the remaining space after non-Fill children
                let remaining_width = (available_width - used_width).max(0.0);
                let fill_width = if fill_count > 0 {
                    remaining_width / fill_count as f32
                } else {
                    0.0
                };

                (fill_width, available_height)
            }
            Layout::Vertical => {
                // Count Fill children and calculate space used by non-Fill children
                let mut fill_count = 0;
                let mut used_height = 0.0;

                for child in &self.children {
                    if child.height.is_fill() {
                        fill_count += 1;
                    } else {
                        // For FitContent without measurer, fall back to available height
                        // Apply effective_scale_factor to Fixed sizes
                        used_height += child
                            .height
                            .try_resolve_with_scale(available_height, effective_scale_factor)
                            .unwrap_or(available_height);
                    }
                }

                // Fill children divide the remaining space after non-Fill children
                let remaining_height = (available_height - used_height).max(0.0);
                let fill_height = if fill_count > 0 {
                    remaining_height / fill_count as f32
                } else {
                    0.0
                };

                (available_width, fill_height)
            }
            Layout::Stack => {
                // In Stack layout, all children get full available space
                (available_width, available_height)
            }
        };

        let num_children = self.children.len();
        for i in 0..num_children {
            // Apply leading margin for first child or collapsed margin was already added for subsequent children
            if i == 0 {
                match self.layout_direction {
                    Layout::Horizontal => {
                        current_x += self.children[i]
                            .margin
                            .left
                            .try_resolve_with_scale(content_width, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                    Layout::Vertical => {
                        current_y += self.children[i]
                            .margin
                            .top
                            .try_resolve_with_scale(content_height, effective_scale_factor)
                            .unwrap_or(0.0);
                    }
                    Layout::Stack => {
                        // In Stack layout, don't advance position for first child
                    }
                }
            }

            let child_available_rect = match self.layout_direction {
                Layout::Horizontal => {
                    // In horizontal layout, each child gets remaining width and full height
                    Rect::new(
                        [current_x, current_y],
                        [content_x + content_width, content_y + content_height],
                    )
                }
                Layout::Vertical => {
                    // In vertical layout, each child gets full width and remaining height
                    Rect::new(
                        [current_x, current_y],
                        [content_x + content_width, content_y + content_height],
                    )
                }
                Layout::Stack => {
                    // In Stack layout, all children start at content origin
                    Rect::new(
                        [content_x, content_y],
                        [content_x + content_width, content_y + content_height],
                    )
                }
            };

            // Pass the available dimensions for size calculations
            // For Fill children, we need to add back their own margins since they'll subtract them
            let child_margin_left = self.children[i]
                .margin
                .left
                .try_resolve_with_scale(content_width, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_right = self.children[i]
                .margin
                .right
                .try_resolve_with_scale(content_width, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_top = self.children[i]
                .margin
                .top
                .try_resolve_with_scale(content_height, effective_scale_factor)
                .unwrap_or(0.0);
            let child_margin_bottom = self.children[i]
                .margin
                .bottom
                .try_resolve_with_scale(content_height, effective_scale_factor)
                .unwrap_or(0.0);

            let child_parent_width = if self.children[i].width.is_fill() {
                fill_size_width + child_margin_left + child_margin_right
            } else {
                available_width + child_margin_left + child_margin_right
            };
            let child_parent_height = if self.children[i].height.is_fill() {
                fill_size_height + child_margin_top + child_margin_bottom
            } else {
                available_height + child_margin_top + child_margin_bottom
            };

            self.children[i].compute_layout_with_parent_size(
                child_available_rect,
                child_parent_width,
                child_parent_height,
                effective_scale_factor,
            );

            // Advance position for next child with collapsed spacing (gap collapsed with margins)
            if let Some(child_layout) = self.children[i].computed_layout() {
                let child_rect = child_layout.rect;

                if i + 1 < num_children {
                    match self.layout_direction {
                        Layout::Horizontal => {
                            // Move to end of current child, then add collapsed spacing
                            let child_right = self.children[i]
                                .margin
                                .right
                                .try_resolve_with_scale(content_width, effective_scale_factor)
                                .unwrap_or(0.0);
                            let next_left = self.children[i + 1]
                                .margin
                                .left
                                .try_resolve_with_scale(content_width, effective_scale_factor)
                                .unwrap_or(0.0);
                            let collapsed_margin = child_right.max(next_left);
                            // Collapse gap with margin - use the larger value
                            let spacing = scaled_gap.max(collapsed_margin);
                            current_x = child_rect.max[0] + spacing;
                        }
                        Layout::Vertical => {
                            // Move to end of current child, then add collapsed spacing
                            let child_bottom = self.children[i]
                                .margin
                                .bottom
                                .try_resolve_with_scale(content_height, effective_scale_factor)
                                .unwrap_or(0.0);
                            let next_top = self.children[i + 1]
                                .margin
                                .top
                                .try_resolve_with_scale(content_height, effective_scale_factor)
                                .unwrap_or(0.0);
                            let collapsed_margin = child_bottom.max(next_top);
                            // Collapse gap with margin - use the larger value
                            let spacing = scaled_gap.max(collapsed_margin);
                            current_y = child_rect.max[1] + spacing;
                        }
                        Layout::Stack => {
                            // In Stack layout, don't advance position (children overlap)
                        }
                    }
                }
            }
        }

        // After children layout, calculate and cache max_scroll if this is a scrollable container
        if self.overflow == Overflow::Scroll {
            let max_scroll = self.calculate_max_scroll_for_node();
            if let Some(computed) = &mut self.computed {
                computed.max_scroll = max_scroll;
            }
        }
    }

    /// Collect all shapes from this node tree for rendering
    pub fn collect_shapes(&self, shapes: &mut Vec<(Rect, Shape)>) {
        self.collect_shapes_with_opacity(shapes, 1.0);
    }

    /// Collect shapes with cumulative opacity
    fn collect_shapes_with_opacity(&self, shapes: &mut Vec<(Rect, Shape)>, parent_opacity: f32) {
        let combined_opacity = parent_opacity * self.opacity;

        // Skip rendering if fully transparent
        if combined_opacity <= 0.0 {
            return;
        }

        if let Some(layout) = &self.computed {
            // Add background shape if present
            if let Some(shape) = &self.shape {
                let mut shape_with_opacity = shape.clone();
                shape_with_opacity.apply_opacity(combined_opacity);
                shapes.push((layout.rect, shape_with_opacity));
            }

            // Add content shape if this is a content node
            if let Some(content) = &self.content {
                match content {
                    crate::content::Content::Text(text_content) => {
                        // Calculate content area (after padding)
                        // NOTE: During rendering, we use scale_factor=1.0 because padding was already
                        // resolved during layout. We just need the logical pixel values here.
                        let width = layout.rect.max[0] - layout.rect.min[0];
                        let height = layout.rect.max[1] - layout.rect.min[1];
                        let padding_left = self
                            .padding
                            .left
                            .try_resolve_with_scale(width, 1.0)
                            .unwrap_or(0.0);
                        let padding_right = self
                            .padding
                            .right
                            .try_resolve_with_scale(width, 1.0)
                            .unwrap_or(0.0);
                        let padding_top = self
                            .padding
                            .top
                            .try_resolve_with_scale(height, 1.0)
                            .unwrap_or(0.0);
                        let padding_bottom = self
                            .padding
                            .bottom
                            .try_resolve_with_scale(height, 1.0)
                            .unwrap_or(0.0);

                        let content_rect = Rect::new(
                            [
                                layout.rect.min[0] + padding_left,
                                layout.rect.min[1] + padding_top,
                            ],
                            [
                                layout.rect.max[0] - padding_right,
                                layout.rect.max[1] - padding_bottom,
                            ],
                        );
                        let mut text_shape =
                            crate::primitives::TextShape::new(content_rect, text_content);
                        text_shape.apply_opacity(combined_opacity);
                        shapes.push((layout.rect, Shape::Text(text_shape)));
                    }
                }
            }
        }

        for child in &self.children {
            child.collect_shapes_with_opacity(shapes, combined_opacity);
        }
    }

    /// Collect debug visualization shapes showing margins, padding, and content areas
    pub fn collect_debug_shapes(
        &self,
        shapes: &mut Vec<(Rect, Shape)>,
        options: &crate::debug::DebugOptions,
    ) {
        use crate::color::Color;
        use crate::primitives::{Stroke, StyledRect};

        if let Some(layout) = &self.computed {
            let rect = layout.rect;
            let width = rect.max[0] - rect.min[0];
            let height = rect.max[1] - rect.min[1];

            // Resolve margin and padding values (use scale_factor=1.0 since layout is already computed)
            let margin_left = self
                .margin
                .left
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);
            let margin_right = self
                .margin
                .right
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);
            let margin_top = self
                .margin
                .top
                .try_resolve_with_scale(height, 1.0)
                .unwrap_or(0.0);
            let margin_bottom = self
                .margin
                .bottom
                .try_resolve_with_scale(height, 1.0)
                .unwrap_or(0.0);
            let padding_left = self
                .padding
                .left
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);
            let padding_right = self
                .padding
                .right
                .try_resolve_with_scale(width, 1.0)
                .unwrap_or(0.0);
            let padding_top = self
                .padding
                .top
                .try_resolve_with_scale(height, 1.0)
                .unwrap_or(0.0);
            let padding_bottom = self
                .padding
                .bottom
                .try_resolve_with_scale(height, 1.0)
                .unwrap_or(0.0);

            // Draw margin area (outermost, semi-transparent red showing margin space)
            if options.show_margins
                && (margin_top > 0.0
                    || margin_right > 0.0
                    || margin_bottom > 0.0
                    || margin_left > 0.0)
            {
                // Draw top margin
                if margin_top > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0] - margin_left, rect.min[1] - margin_top],
                            [rect.max[0] + margin_right, rect.min[1]],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(1.0, 0.0, 0.0, 0.2),
                        )),
                    ));
                }
                // Draw right margin (excluding top and bottom corners)
                if margin_right > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.max[0], rect.min[1]],
                            [rect.max[0] + margin_right, rect.max[1]],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(1.0, 0.0, 0.0, 0.2),
                        )),
                    ));
                }
                // Draw bottom margin (full width including corners)
                if margin_bottom > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0] - margin_left, rect.max[1]],
                            [rect.max[0] + margin_right, rect.max[1] + margin_bottom],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(1.0, 0.0, 0.0, 0.2),
                        )),
                    ));
                }
                // Draw left margin (excluding top and bottom corners)
                if margin_left > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0] - margin_left, rect.min[1]],
                            [rect.min[0], rect.max[1]],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(1.0, 0.0, 0.0, 0.2),
                        )),
                    ));
                }
            }

            // Draw content area (yellow outline - area inside padding)
            if options.show_content_area
                && (padding_top > 0.0
                    || padding_right > 0.0
                    || padding_bottom > 0.0
                    || padding_left > 0.0)
            {
                let content_rect = Rect::new(
                    [rect.min[0] + padding_left, rect.min[1] + padding_top],
                    [rect.max[0] - padding_right, rect.max[1] - padding_bottom],
                );
                shapes.push((
                    content_rect,
                    Shape::Rect(
                        StyledRect::new(Default::default(), Color::transparent()).with_stroke(
                            Stroke::new(Size::lpx(1.0), Color::new(1.0, 1.0, 0.0, 0.5)),
                        ),
                    ),
                ));
            }

            // Draw padding area (semi-transparent blue showing the padding inset)
            if options.show_padding
                && (padding_top > 0.0
                    || padding_right > 0.0
                    || padding_bottom > 0.0
                    || padding_left > 0.0)
            {
                // Draw top padding (full width)
                if padding_top > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0], rect.min[1]],
                            [rect.max[0], rect.min[1] + padding_top],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(0.0, 0.0, 1.0, 0.2),
                        )),
                    ));
                }
                // Draw right padding (excluding top and bottom corners)
                if padding_right > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.max[0] - padding_right, rect.min[1] + padding_top],
                            [rect.max[0], rect.max[1] - padding_bottom],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(0.0, 0.0, 1.0, 0.2),
                        )),
                    ));
                }
                // Draw bottom padding (full width)
                if padding_bottom > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0], rect.max[1] - padding_bottom],
                            [rect.max[0], rect.max[1]],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(0.0, 0.0, 1.0, 0.2),
                        )),
                    ));
                }
                // Draw left padding (excluding top and bottom corners)
                if padding_left > 0.0 {
                    shapes.push((
                        Rect::new(
                            [rect.min[0], rect.min[1] + padding_top],
                            [rect.min[0] + padding_left, rect.max[1] - padding_bottom],
                        ),
                        Shape::Rect(StyledRect::new(
                            Default::default(),
                            Color::new(0.0, 0.0, 1.0, 0.2),
                        )),
                    ));
                }
            }

            // Draw node border (green outline for the actual node rect)
            if options.show_borders {
                shapes.push((
                    rect,
                    Shape::Rect(
                        StyledRect::new(Default::default(), Color::transparent()).with_stroke(
                            Stroke::new(Size::lpx(1.0), Color::new(0.0, 1.0, 0.0, 0.5)),
                        ),
                    ),
                ));
            }
        }

        for child in &self.children {
            child.collect_debug_shapes(shapes, options);
        }
    }

    /// Calculate maximum scroll offset for this container based on children layout
    /// This is called during layout computation to cache the result
    fn calculate_max_scroll_for_node(&self) -> (f32, f32) {
        let Some(layout) = self.computed_layout() else {
            return (0.0, 0.0);
        };

        // Get container dimensions (after padding)
        let width = layout.rect.max[0] - layout.rect.min[0];
        let height = layout.rect.max[1] - layout.rect.min[1];
        let padding_left = self
            .padding
            .left
            .try_resolve_with_scale(width, 1.0)
            .unwrap_or(0.0);
        let padding_right = self
            .padding
            .right
            .try_resolve_with_scale(width, 1.0)
            .unwrap_or(0.0);
        let padding_top = self
            .padding
            .top
            .try_resolve_with_scale(height, 1.0)
            .unwrap_or(0.0);
        let padding_bottom = self
            .padding
            .bottom
            .try_resolve_with_scale(height, 1.0)
            .unwrap_or(0.0);

        let container_width = width - padding_left - padding_right;
        let container_height = height - padding_top - padding_bottom;

        // Calculate total content size based on layout direction
        if self.children.is_empty() {
            return (0.0, 0.0);
        }

        let mut content_width = 0.0f32;
        let mut content_height = 0.0f32;

        match self.layout_direction {
            Layout::Vertical => {
                // For vertical layout: accumulate heights, track max width
                // For nested layouts (like grid), we need to look at the intrinsic width
                for (i, child) in self.children.iter().enumerate() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        // For horizontal child layouts, calculate their full content width
                        let actual_child_width = if child.layout_direction == Layout::Horizontal {
                            let mut row_width = 0.0f32;
                            let child_gap = child
                                .gap
                                .try_resolve_with_scale(child_width, 1.0)
                                .unwrap_or(0.0);

                            for (j, grandchild) in child.children.iter().enumerate() {
                                if let Some(gc_layout) = grandchild.computed_layout() {
                                    row_width += gc_layout.rect.max[0] - gc_layout.rect.min[0];
                                    if j < child.children.len() - 1 {
                                        row_width += child_gap;
                                    }
                                }
                            }
                            let child_padding_left = child
                                .padding
                                .left
                                .try_resolve_with_scale(child_width, 1.0)
                                .unwrap_or(0.0);
                            let child_padding_right = child
                                .padding
                                .right
                                .try_resolve_with_scale(child_width, 1.0)
                                .unwrap_or(0.0);
                            row_width + child_padding_left + child_padding_right
                        } else {
                            child_width
                        };

                        content_width = content_width.max(actual_child_width);
                        content_height += child_height;

                        if i < self.children.len() - 1 {
                            let gap = self
                                .gap
                                .try_resolve_with_scale(container_height, 1.0)
                                .unwrap_or(0.0);
                            content_height += gap;
                        }
                    }
                }
            }
            Layout::Horizontal => {
                // For horizontal layout: accumulate widths, track max height
                for (i, child) in self.children.iter().enumerate() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        content_width += child_width;
                        content_height = content_height.max(child_height);

                        if i < self.children.len() - 1 {
                            let gap = self
                                .gap
                                .try_resolve_with_scale(container_width, 1.0)
                                .unwrap_or(0.0);
                            content_width += gap;
                        }
                    }
                }
            }
            Layout::Stack => {
                // For stack layout: track max width and max height
                for child in self.children.iter() {
                    if let Some(child_layout) = child.computed_layout() {
                        let child_width = child_layout.rect.max[0] - child_layout.rect.min[0];
                        let child_height = child_layout.rect.max[1] - child_layout.rect.min[1];

                        content_width = content_width.max(child_width);
                        content_height = content_height.max(child_height);
                    }
                }
            }
        }

        // Max scroll is the amount content exceeds container size
        let max_scroll_x = (content_width - container_width).max(0.0);
        let max_scroll_y = (content_height - container_height).max(0.0);

        (max_scroll_x, max_scroll_y)
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
