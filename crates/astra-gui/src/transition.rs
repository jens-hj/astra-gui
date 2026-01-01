use crate::color::Color;
use crate::primitives::{CornerShape, Stroke};
use crate::style::Style;

/// Easing function type: takes progress (0.0 to 1.0) and returns eased value (0.0 to 1.0)
pub type EasingFn = fn(f32) -> f32;

/// Linear interpolation (no easing)
pub fn linear(t: f32) -> f32 {
    t
}

/// Ease in (quadratic) - slow start, accelerating
pub fn ease_in(t: f32) -> f32 {
    t * t
}

/// Ease out (quadratic) - fast start, decelerating
pub fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Ease in-out (quadratic) - slow start and end, fast middle
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// Ease in (cubic) - stronger slow start effect
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease out (cubic) - stronger fast start effect
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

/// Ease in-out (cubic) - stronger slow start/end effect
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = t - 1.0;
        1.0 + 4.0 * t * t * t
    }
}

/// Linearly interpolate between two f32 values
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linearly interpolate between two colors
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp_f32(a.r, b.r, t),
        g: lerp_f32(a.g, b.g, t),
        b: lerp_f32(a.b, b.b, t),
        a: lerp_f32(a.a, b.a, t),
    }
}

/// Linearly interpolate between two Size values
///
/// Only interpolates if both sizes are the same variant. Otherwise returns b.
pub fn lerp_size(a: crate::layout::Size, b: crate::layout::Size, t: f32) -> crate::layout::Size {
    use crate::layout::Size;
    match (a, b) {
        (Size::Logical(v1), Size::Logical(v2)) => Size::Logical(lerp_f32(v1, v2, t)),
        (Size::Physical(v1), Size::Physical(v2)) => Size::Physical(lerp_f32(v1, v2, t)),
        (Size::Relative(v1), Size::Relative(v2)) => Size::Relative(lerp_f32(v1, v2, t)),
        // For incompatible types or Fill/FitContent, snap to target
        _ => b,
    }
}

/// Linearly interpolate between two strokes
pub fn lerp_stroke(a: Stroke, b: Stroke, t: f32) -> Stroke {
    Stroke {
        width: lerp_size(a.width, b.width, t),
        color: lerp_color(a.color, b.color, t),
    }
}

/// Linearly interpolate between two corner shapes
///
/// Only interpolates if both shapes are the same variant with compatible parameters.
/// Otherwise, snaps to the target shape at t >= 0.5.
pub fn lerp_corner_shape(a: CornerShape, b: CornerShape, t: f32) -> CornerShape {
    match (a, b) {
        (CornerShape::None, CornerShape::None) => CornerShape::None,
        (CornerShape::Round(r1), CornerShape::Round(r2)) => {
            CornerShape::Round(lerp_size(r1, r2, t))
        }
        (CornerShape::Cut(d1), CornerShape::Cut(d2)) => CornerShape::Cut(lerp_size(d1, d2, t)),
        (CornerShape::InverseRound(r1), CornerShape::InverseRound(r2)) => {
            CornerShape::InverseRound(lerp_size(r1, r2, t))
        }
        (
            CornerShape::Squircle {
                radius: r1,
                smoothness: s1,
            },
            CornerShape::Squircle {
                radius: r2,
                smoothness: s2,
            },
        ) => CornerShape::Squircle {
            radius: lerp_size(r1, r2, t),
            smoothness: lerp_f32(s1, s2, t),
        },
        // Different variants: snap at halfway point
        (_, b) if t >= 0.5 => b,
        (a, _) => a,
    }
}

/// Interpolate between two styles
///
/// For each property, if both styles have a value, interpolate between them.
/// Otherwise, use whichever value is present (or None if neither has a value).
pub fn lerp_style(from: &Style, to: &Style, t: f32) -> Style {
    Style {
        fill_color: match (from.fill_color, to.fill_color) {
            (Some(a), Some(b)) => Some(lerp_color(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        stroke_color: match (from.stroke_color, to.stroke_color) {
            (Some(a), Some(b)) => Some(lerp_color(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        stroke: match (from.stroke, to.stroke) {
            (Some(a), Some(b)) => Some(lerp_stroke(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        stroke_width: match (from.stroke_width, to.stroke_width) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        corner_shape: match (from.corner_shape, to.corner_shape) {
            (Some(a), Some(b)) => Some(lerp_corner_shape(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        #[allow(deprecated)]
        corner_radius: match (from.corner_radius, to.corner_radius) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        opacity: match (from.opacity, to.opacity) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        text_color: match (from.text_color, to.text_color) {
            (Some(a), Some(b)) => Some(lerp_color(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        cursor_color: match (from.cursor_color, to.cursor_color) {
            (Some(a), Some(b)) => Some(lerp_color(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        translation_x: match (from.translation_x, to.translation_x) {
            (Some(a), Some(b)) => Some(lerp_size(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        translation_y: match (from.translation_y, to.translation_y) {
            (Some(a), Some(b)) => Some(lerp_size(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        rotation: match (from.rotation, to.rotation) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        transform_origin: match (from.transform_origin, to.transform_origin) {
            // For transform origin, we don't interpolate - just snap to the target
            (_, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        #[allow(deprecated)]
        offset_x: match (from.offset_x, to.offset_x) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        #[allow(deprecated)]
        offset_y: match (from.offset_y, to.offset_y) {
            (Some(a), Some(b)) => Some(lerp_f32(a, b, t)),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
    }
}

/// Transition configuration
///
/// Defines how long a transition takes and what easing function to use.
#[derive(Debug, Clone, Copy)]
pub struct Transition {
    /// Duration in seconds
    pub duration: f32,

    /// Easing function to apply
    pub easing: EasingFn,
}

impl Transition {
    /// Create a new transition with custom duration and easing
    pub fn new(duration: f32, easing: EasingFn) -> Self {
        Self { duration, easing }
    }

    /// Instant transition (no animation, duration = 0)
    pub fn instant() -> Self {
        Self {
            duration: 0.0,
            easing: linear,
        }
    }

    /// Quick transition (150ms, ease-out)
    ///
    /// Good for hover states and quick feedback
    pub fn quick() -> Self {
        Self {
            duration: 0.15,
            easing: ease_out,
        }
    }

    /// Standard transition (250ms, ease-in-out)
    ///
    /// Good for most state changes
    pub fn standard() -> Self {
        Self {
            duration: 0.25,
            easing: ease_in_out,
        }
    }

    /// Slow transition (400ms, ease-in-out)
    ///
    /// Good for emphasized state changes
    pub fn slow() -> Self {
        Self {
            duration: 0.4,
            easing: ease_in_out,
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn test_ease_in() {
        assert_eq!(ease_in(0.0), 0.0);
        assert!(ease_in(0.5) < 0.5); // Slower at start
        assert_eq!(ease_in(1.0), 1.0);
    }

    #[test]
    fn test_ease_out() {
        assert_eq!(ease_out(0.0), 0.0);
        assert!(ease_out(0.5) > 0.5); // Faster at start
        assert_eq!(ease_out(1.0), 1.0);
    }

    #[test]
    fn test_lerp_f32() {
        assert_eq!(lerp_f32(0.0, 100.0, 0.0), 0.0);
        assert_eq!(lerp_f32(0.0, 100.0, 0.5), 50.0);
        assert_eq!(lerp_f32(0.0, 100.0, 1.0), 100.0);
    }

    #[test]
    fn test_lerp_color() {
        let black = Color::rgb(0.0, 0.0, 0.0);
        let white = Color::rgb(1.0, 1.0, 1.0);
        let gray = lerp_color(black, white, 0.5);

        assert_eq!(gray.r, 0.5);
        assert_eq!(gray.g, 0.5);
        assert_eq!(gray.b, 0.5);
    }
}
