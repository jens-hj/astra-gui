use crate::color::Color;
use crate::content::Content;
use crate::layout::{TransformOrigin, Translation};
use crate::node::Node;
use crate::primitives::{CornerShape, Shape};

/// Visual style properties that can be transitioned
///
/// All fields are `Option<T>` to allow partial styles that only override specific properties.
/// This enables style merging where hover/active states only specify the properties that change.
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// Background fill color (for shapes)
    pub fill_color: Option<Color>,

    /// Stroke color (for shapes with borders)
    pub stroke_color: Option<Color>,

    /// Stroke width
    pub stroke_width: Option<f32>,

    /// Corner radius (for Round corner shape)
    pub corner_radius: Option<f32>,

    /// Node opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,

    /// Text color (for text content)
    pub text_color: Option<Color>,

    /// Cursor/caret color (for text input cursors, falls back to text_color if not set)
    pub cursor_color: Option<Color>,

    /// Horizontal translation from default position
    pub translation_x: Option<f32>,

    /// Vertical translation from default position
    pub translation_y: Option<f32>,

    /// Rotation in radians (clockwise positive, CSS convention)
    pub rotation: Option<f32>,

    /// Transform origin for rotation
    pub transform_origin: Option<TransformOrigin>,

    /// Deprecated: use translation_x
    #[deprecated(since = "0.2.0", note = "Use translation_x instead")]
    pub offset_x: Option<f32>,

    /// Deprecated: use translation_y
    #[deprecated(since = "0.2.0", note = "Use translation_y instead")]
    pub offset_y: Option<f32>,
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a style with only fill color
    pub fn fill(color: Color) -> Self {
        Self {
            fill_color: Some(color),
            ..Default::default()
        }
    }

    /// Create a style with only text color
    pub fn text(color: Color) -> Self {
        Self {
            text_color: Some(color),
            ..Default::default()
        }
    }

    /// Create a style with only opacity
    pub fn opacity(opacity: f32) -> Self {
        Self {
            opacity: Some(opacity),
            ..Default::default()
        }
    }

    /// Merge this style with another, preferring values from `other` when present
    ///
    /// This is used to combine base → hover → active styles, where each layer
    /// only specifies the properties that change.
    pub fn merge(&self, other: &Style) -> Style {
        Style {
            fill_color: other.fill_color.or(self.fill_color),
            stroke_color: other.stroke_color.or(self.stroke_color),
            stroke_width: other.stroke_width.or(self.stroke_width),
            corner_radius: other.corner_radius.or(self.corner_radius),
            opacity: other.opacity.or(self.opacity),
            text_color: other.text_color.or(self.text_color),
            cursor_color: other.cursor_color.or(self.cursor_color),
            translation_x: other.translation_x.or(self.translation_x),
            translation_y: other.translation_y.or(self.translation_y),
            rotation: other.rotation.or(self.rotation),
            transform_origin: other.transform_origin.or(self.transform_origin),
            #[allow(deprecated)]
            offset_x: other.offset_x.or(self.offset_x),
            #[allow(deprecated)]
            offset_y: other.offset_y.or(self.offset_y),
        }
    }

    /// Apply this style to a node (modify node properties in-place)
    ///
    /// This is called during rendering to apply computed transition styles.
    /// Public API for backend crates (like astra-gui-wgpu) to apply styles.
    pub fn apply_to_node(&self, node: &mut Node) {
        if let Some(opacity) = self.opacity {
            node.set_opacity(opacity);
        }

        // Apply to shape if present
        if let Some(shape) = node.shape_mut() {
            if let Shape::Rect(ref mut rect) = shape {
                if let Some(color) = self.fill_color {
                    rect.fill = color;
                }
                if let Some(color) = self.stroke_color {
                    if let Some(ref mut stroke) = rect.stroke {
                        stroke.color = color;
                    }
                }
                if let Some(width) = self.stroke_width {
                    if let Some(ref mut stroke) = rect.stroke {
                        stroke.width = width;
                    }
                }
                if let Some(radius) = self.corner_radius {
                    rect.corner_shape = CornerShape::Round(radius);
                }
            }
        }

        // Apply to text content if present
        if let Some(content) = node.content_mut() {
            let Content::Text(ref mut text) = content;
            if let Some(color) = self.text_color {
                text.color = color;
            }
        }

        // Apply translation if present (check both new and deprecated fields)
        #[allow(deprecated)]
        let has_translation = self.translation_x.is_some()
            || self.translation_y.is_some()
            || self.offset_x.is_some()
            || self.offset_y.is_some();

        if has_translation {
            let current_translation = node.translation();
            #[allow(deprecated)]
            let new_x = self
                .translation_x
                .or(self.offset_x)
                .unwrap_or(current_translation.x);
            #[allow(deprecated)]
            let new_y = self
                .translation_y
                .or(self.offset_y)
                .unwrap_or(current_translation.y);
            node.set_translation(Translation::new(new_x, new_y));
        }

        // Apply rotation if present
        if let Some(rotation) = self.rotation {
            node.set_rotation(rotation);
        }

        // Apply transform origin if present
        if let Some(origin) = self.transform_origin {
            node.set_transform_origin(origin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_prefers_other() {
        let base = Style {
            fill_color: Some(Color::rgb(1.0, 0.0, 0.0)),
            opacity: Some(1.0),
            ..Default::default()
        };

        let hover = Style {
            fill_color: Some(Color::rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        };

        let merged = base.merge(&hover);

        assert_eq!(merged.fill_color, Some(Color::rgb(0.0, 1.0, 0.0)));
        assert_eq!(merged.opacity, Some(1.0));
    }

    #[test]
    fn test_merge_preserves_base_when_other_none() {
        let base = Style {
            fill_color: Some(Color::rgb(1.0, 0.0, 0.0)),
            opacity: Some(0.5),
            ..Default::default()
        };

        let hover = Style::default();

        let merged = base.merge(&hover);

        assert_eq!(merged.fill_color, Some(Color::rgb(1.0, 0.0, 0.0)));
        assert_eq!(merged.opacity, Some(0.5));
    }
}
