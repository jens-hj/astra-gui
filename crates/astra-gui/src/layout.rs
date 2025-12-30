use crate::primitives::Rect;

/// Size specification that can be fixed, relative to parent, or derived from content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    /// Fixed size in pixels
    Fixed(f32),
    /// Relative size as a fraction of parent (0.0 to 1.0)
    Relative(f32),
    /// Fill all remaining available space
    Fill,
    /// Size to the minimum that fits content (text metrics or children), plus padding.
    ///
    /// NOTE: The layout algorithm must measure intrinsic content size to resolve this.
    FitContent,
}

/// Overflow policy for content/children that exceed the node's bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Content can render outside the node's bounds.
    Visible,
    /// Content is clipped to the node's bounds.
    Hidden,
    /// Content is clipped but can be scrolled.
    Scroll,
}

/// Scroll direction behavior
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Normal scrolling: wheel up scrolls content down
    Normal,
    /// Inverted/natural scrolling: wheel up scrolls content up (like touchpad)
    Inverted,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        Self::Inverted
    }
}

impl Size {
    /// Create a fixed size in pixels
    pub const fn px(pixels: f32) -> Self {
        Self::Fixed(pixels)
    }

    /// Create a relative size as a percentage (0.0 to 1.0)
    pub const fn fraction(fraction: f32) -> Self {
        Self::Relative(fraction)
    }

    /// Size to the minimum that fits content.
    pub const fn fit_content() -> Self {
        Self::FitContent
    }

    /// Resolve the size given the parent's dimension
    ///
    /// This only works for `Fixed` and `Relative` sizes. For `Fill` and `FitContent`,
    /// the layout algorithm must compute the size differently:
    /// - `Fill`: Computed based on remaining space after other siblings
    /// - `FitContent`: Computed via intrinsic measurement of content/children
    ///
    /// # Panics
    /// Panics if called on `Fill` or `FitContent` - these must be handled by the layout algorithm.
    pub fn resolve(&self, parent_size: f32) -> f32 {
        match self {
            Size::Fixed(px) => *px,
            Size::Relative(fraction) => parent_size * fraction,
            Size::Fill => panic!("Cannot resolve Size::Fill - must be computed by layout algorithm based on remaining space"),
            Size::FitContent => panic!("Cannot resolve Size::FitContent - must be computed via intrinsic measurement"),
        }
    }

    /// Try to resolve the size, returning None for Fill and FitContent
    ///
    /// This is a non-panicking version of `resolve()` that returns `None`
    /// for sizes that cannot be resolved without additional context.
    pub fn try_resolve(&self, parent_size: f32) -> Option<f32> {
        self.try_resolve_with_scale(parent_size, 1.0)
    }

    /// Try to resolve the size with a scale factor applied to Fixed sizes
    ///
    /// The `scale_factor` converts logical pixels to physical pixels for Fixed sizes.
    /// Relative sizes are not affected by the scale factor as they are already
    /// proportional to the parent size.
    pub fn try_resolve_with_scale(&self, parent_size: f32, scale_factor: f32) -> Option<f32> {
        match self {
            Size::Fixed(px) => Some(*px * scale_factor),
            Size::Relative(fraction) => Some(parent_size * fraction),
            Size::Fill | Size::FitContent => None,
        }
    }

    /// Check if this size is Fill
    pub const fn is_fill(&self) -> bool {
        matches!(self, Size::Fill)
    }

    /// Check if this size is FitContent
    pub const fn is_fit_content(&self) -> bool {
        matches!(self, Size::FitContent)
    }

    /// Check if this size is zero (Fixed(0.0))
    pub fn is_zero(&self) -> bool {
        matches!(self, Size::Fixed(v) if *v == 0.0)
    }

    /// Check if this size is non-zero (any non-zero Fixed value, or Relative/Fill/FitContent)
    pub fn is_non_zero(&self) -> bool {
        !self.is_zero()
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::FitContent
    }
}

impl Default for Overflow {
    fn default() -> Self {
        Self::Visible
    }
}

/// Layout mode for arranging children
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Children are arranged horizontally (left to right)
    Horizontal,
    /// Children are arranged vertically (top to bottom)
    Vertical,
    /// Children are stacked in the Z direction (overlapping)
    Stack,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Vertical
    }
}

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

    pub const fn zero() -> Self {
        Self::ZERO
    }

    pub const fn x(x: f32) -> Self {
        Self { x, y: 0.0 }
    }

    pub const fn y(y: f32) -> Self {
        Self { x: 0.0, y }
    }
}

/// Backward compatibility alias
#[deprecated(since = "0.2.0", note = "Use Translation instead")]
pub type Offset = Translation;

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

    pub const fn top_right() -> Self {
        Self {
            x_percent: 1.0,
            y_percent: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub const fn bottom_left() -> Self {
        Self {
            x_percent: 0.0,
            y_percent: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub const fn bottom_right() -> Self {
        Self {
            x_percent: 1.0,
            y_percent: 1.0,
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

/// 2D transform combining translation, rotation, scale, and origin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub translation: Translation,
    pub rotation: f32, // Radians, clockwise positive (CSS convention)
    pub scale: f32,    // Uniform scale factor (1.0 = no scale)
    pub origin: TransformOrigin,
    /// Absolute world-space origin position (resolved during transform composition)
    /// This is used for hierarchical rotations - children rotate around this point
    pub absolute_origin: Option<[f32; 2]>,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        translation: Translation::ZERO,
        rotation: 0.0,
        scale: 1.0,
        origin: TransformOrigin {
            x_percent: 0.5,
            y_percent: 0.5,
            x_offset: 0.0,
            y_offset: 0.0,
        },
        absolute_origin: None,
    };

    /// Apply transform to a point (forward transform)
    /// Order: Scale → Rotate → Translate (around origin)
    pub fn apply(&self, point: [f32; 2], rect_size: [f32; 2]) -> [f32; 2] {
        let (origin_x, origin_y) = self.origin.resolve(rect_size[0], rect_size[1]);

        // Translate to origin
        let x = point[0] - origin_x;
        let y = point[1] - origin_y;

        // Scale
        let sx = x * self.scale;
        let sy = y * self.scale;

        // Rotate (clockwise positive)
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = sx * cos_r + sy * sin_r;
        let ry = -sx * sin_r + sy * cos_r;

        // Translate back and apply translation
        [
            rx + origin_x + self.translation.x,
            ry + origin_y + self.translation.y,
        ]
    }

    /// Apply inverse transform (for hit testing)
    ///
    /// Inverse of: Scale → Rotate → Translate
    /// So we: Inverse Translate → Inverse Rotate → Inverse Scale
    pub fn apply_inverse(&self, point: [f32; 2], rect_size: [f32; 2]) -> [f32; 2] {
        // Use absolute_origin if set, otherwise resolve the percentage-based origin
        let (origin_x, origin_y) = if let Some(abs_origin) = self.absolute_origin {
            (abs_origin[0], abs_origin[1])
        } else {
            self.origin.resolve(rect_size[0], rect_size[1])
        };

        // 1. Remove translation
        let mut x = point[0] - self.translation.x;
        let mut y = point[1] - self.translation.y;

        // 2. Translate to origin for inverse rotation
        x -= origin_x;
        y -= origin_y;

        // 3. Inverse rotate (negate angle)
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rx = x * cos_r - y * sin_r;
        let ry = x * sin_r + y * cos_r;

        x = rx;
        y = ry;

        // 4. Inverse scale (divide by scale)
        x /= self.scale;
        y /= self.scale;

        // 5. Translate back from origin
        x += origin_x;
        y += origin_y;

        [x, y]
    }

    /// Compose two transforms (apply self, then other)
    /// Scales multiply, rotations add, translations add
    pub fn then(&self, other: &Transform2D, _rect_size: [f32; 2]) -> Transform2D {
        // If parent has rotation or an absolute origin, use parent's absolute origin
        // Otherwise, use child's origin (will be resolved later)
        let (effective_origin, absolute_origin) =
            if self.rotation.abs() > 0.0001 || self.absolute_origin.is_some() {
                // Parent is rotated or has inherited rotation - use parent's absolute origin
                (self.origin, self.absolute_origin)
            } else {
                // Parent is not rotated - use child's origin (no absolute origin yet)
                (other.origin, None)
            };

        Transform2D {
            translation: Translation {
                x: self.translation.x + other.translation.x,
                y: self.translation.y + other.translation.y,
            },
            rotation: self.rotation + other.rotation,
            scale: self.scale * other.scale, // Multiply scales
            origin: effective_origin,
            absolute_origin,
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Computed layout information after tree traversal
#[derive(Clone, Copy, Debug)]
pub struct ComputedLayout {
    /// Absolute position in screen coordinates
    pub rect: Rect,
    /// Maximum scroll offset for scrollable containers (cached during layout)
    pub max_scroll: (f32, f32),
}

impl ComputedLayout {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            max_scroll: (0.0, 0.0),
        }
    }

    pub fn with_max_scroll(rect: Rect, max_scroll: (f32, f32)) -> Self {
        Self { rect, max_scroll }
    }
}

/// Spacing/padding around content
#[derive(Clone, Copy, Debug, Default)]
pub struct Spacing {
    pub top: Size,
    pub right: Size,
    pub bottom: Size,
    pub left: Size,
}

impl Spacing {
    /// Zero spacing constant
    pub const ZERO: Self = Self {
        top: Size::Fixed(0.0),
        right: Size::Fixed(0.0),
        bottom: Size::Fixed(0.0),
        left: Size::Fixed(0.0),
    };

    /// Create spacing with all sides equal
    pub const fn all(value: Size) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create zero spacing
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// Create spacing with symmetric horizontal and vertical values (CSS-style)
    ///
    /// ```
    /// # use astra_gui::{Spacing, Size};
    /// let spacing = Spacing::symmetric(Size::px(10.0), Size::px(20.0));
    /// ```
    pub const fn symmetric(horizontal: Size, vertical: Size) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create spacing from individual top, right, bottom, left values (CSS-style)
    ///
    /// ```
    /// # use astra_gui::{Spacing, Size};
    /// let spacing = Spacing::trbl(Size::px(10.0), Size::px(20.0), Size::px(30.0), Size::px(40.0));
    /// ```
    pub const fn trbl(top: Size, right: Size, bottom: Size, left: Size) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn horizontal(horizontal: Size) -> Self {
        Self {
            top: Size::Fixed(0.0),
            right: horizontal,
            bottom: Size::Fixed(0.0),
            left: horizontal,
        }
    }

    pub const fn vertical(vertical: Size) -> Self {
        Self {
            top: vertical,
            right: Size::Fixed(0.0),
            bottom: vertical,
            left: Size::Fixed(0.0),
        }
    }

    pub const fn top(top: Size) -> Self {
        Self {
            top,
            right: Size::Fixed(0.0),
            bottom: Size::Fixed(0.0),
            left: Size::Fixed(0.0),
        }
    }

    pub const fn right(right: Size) -> Self {
        Self {
            top: Size::Fixed(0.0),
            right,
            bottom: Size::Fixed(0.0),
            left: Size::Fixed(0.0),
        }
    }

    pub const fn bottom(bottom: Size) -> Self {
        Self {
            top: Size::Fixed(0.0),
            right: Size::Fixed(0.0),
            bottom,
            left: Size::Fixed(0.0),
        }
    }

    pub const fn left(left: Size) -> Self {
        Self {
            top: Size::Fixed(0.0),
            right: Size::Fixed(0.0),
            bottom: Size::Fixed(0.0),
            left,
        }
    }

    /// Get the sum of horizontal spacing (left + right)
    ///
    /// Resolves Size values to f32. For Fill or FitContent, returns 0.0.
    pub fn get_horizontal(&self) -> f32 {
        self.left.try_resolve(1.0).unwrap_or(0.0) + self.right.try_resolve(1.0).unwrap_or(0.0)
    }

    /// Get the sum of vertical spacing (top + bottom)
    ///
    /// Resolves Size values to f32. For Fill or FitContent, returns 0.0.
    pub fn get_vertical(&self) -> f32 {
        self.top.try_resolve(1.0).unwrap_or(0.0) + self.bottom.try_resolve(1.0).unwrap_or(0.0)
    }
}
