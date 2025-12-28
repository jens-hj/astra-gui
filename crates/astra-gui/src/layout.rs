use crate::primitives::Rect;

/// Size specification that can be fixed, relative to parent, or derived from content.
#[derive(Clone, Copy, Debug)]
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
    /// Content is clipped but can be scrolled (not implemented yet).
    Scroll,
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
        match self {
            Size::Fixed(px) => Some(*px),
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

/// 2D transform combining translation, rotation, and origin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub translation: Translation,
    pub rotation: f32, // Radians, clockwise positive (CSS convention)
    pub origin: TransformOrigin,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        translation: Translation::ZERO,
        rotation: 0.0,
        origin: TransformOrigin {
            x_percent: 0.5,
            y_percent: 0.5,
            x_offset: 0.0,
            y_offset: 0.0,
        },
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
    pub fn then(&self, other: &Transform2D, _rect_size: [f32; 2]) -> Transform2D {
        // For hierarchical transforms, accumulate rotations and translations
        // This is a simplified composition that works for most cases
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

/// Computed layout information after tree traversal
#[derive(Clone, Copy, Debug)]
pub struct ComputedLayout {
    /// Absolute position in screen coordinates
    pub rect: Rect,
}

impl ComputedLayout {
    pub fn new(rect: Rect) -> Self {
        Self { rect }
    }
}

/// Spacing/padding around content
#[derive(Clone, Copy, Debug, Default)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Spacing {
    /// Create spacing with all sides equal
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create zero spacing
    pub const fn zero() -> Self {
        Self::all(0.0)
    }

    /// Create spacing with symmetric horizontal and vertical values (CSS-style)
    ///
    /// ```
    /// # use astra_gui::Spacing;
    /// let spacing = Spacing::symmetric(10.0, 20.0);
    /// assert_eq!(spacing.left, 10.0);
    /// assert_eq!(spacing.right, 10.0);
    /// assert_eq!(spacing.top, 20.0);
    /// assert_eq!(spacing.bottom, 20.0);
    /// ```
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
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
    /// # use astra_gui::Spacing;
    /// let spacing = Spacing::trbl(10.0, 20.0, 30.0, 40.0);
    /// assert_eq!(spacing.top, 10.0);
    /// assert_eq!(spacing.right, 20.0);
    /// assert_eq!(spacing.bottom, 30.0);
    /// assert_eq!(spacing.left, 40.0);
    /// ```
    pub const fn trbl(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn horizontal(horizontal: f32) -> Self {
        Self {
            top: 0.0,
            right: horizontal,
            bottom: 0.0,
            left: horizontal,
        }
    }

    pub const fn vertical(vertical: f32) -> Self {
        Self {
            top: vertical,
            right: 0.0,
            bottom: vertical,
            left: 0.0,
        }
    }

    pub const fn top(top: f32) -> Self {
        Self {
            top,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub const fn right(right: f32) -> Self {
        Self {
            top: 0.0,
            right,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub const fn bottom(bottom: f32) -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom,
            left: 0.0,
        }
    }

    pub const fn left(left: f32) -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left,
        }
    }

    pub const fn get_vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub const fn get_horizontal(&self) -> f32 {
        self.right + self.left
    }
}
