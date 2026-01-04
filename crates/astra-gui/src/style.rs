use crate::color::Color;
use crate::content::Content;
use crate::layout::{TransformOrigin, Translation};
use crate::node::Node;
use crate::primitives::{CornerShape, Shape, Stroke};

/// Visual style properties that can be transitioned
///
/// All fields are `Option<T>` to allow partial styles that only override specific properties.
/// This enables style merging where hover/active states only specify the properties that change.
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// Background fill color (for shapes)
    pub fill_color: Option<Color>,

    /// Stroke configuration (width and color)
    pub stroke: Option<Stroke>,

    /// Corner shape (supports all variants: None, Round, Cut, InverseRound, Squircle)
    pub corner_shape: Option<CornerShape>,

    /// Node opacity (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f32>,

    /// Text color (for text content)
    pub text_color: Option<Color>,

    /// Cursor/caret color (for text input cursors, falls back to text_color if not set)
    pub cursor_color: Option<Color>,

    // Note: Translation is stored as separate x/y fields (not `Option<Translation>`)
    // to allow partial style overrides. This enables hover/active styles to change
    // just one axis (e.g., "shift 5px right") without affecting the other axis.
    // This is consistent with CSS where you can set `transform: translateX(5px)`
    // without overriding any existing translateY.
    /// Horizontal translation from default position
    pub translation_x: Option<crate::layout::Size>,

    /// Vertical translation from default position
    pub translation_y: Option<crate::layout::Size>,

    /// Rotation in radians (clockwise positive, CSS convention)
    pub rotation: Option<f32>,

    /// Transform origin for rotation
    pub transform_origin: Option<TransformOrigin>,

    /// Override width in physical pixels (post-resolution, from transition system)
    ///
    /// When set, this bypasses the normal Size→pixels resolution during layout
    /// and uses this value directly. Used for animating between different Size variants.
    pub width_override: Option<f32>,

    /// Override height in physical pixels (post-resolution, from transition system)
    ///
    /// When set, this bypasses the normal Size→pixels resolution during layout
    /// and uses this value directly. Used for animating between different Size variants.
    pub height_override: Option<f32>,
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
            stroke: other.stroke.or(self.stroke),
            corner_shape: other.corner_shape.or(self.corner_shape),
            opacity: other.opacity.or(self.opacity),
            text_color: other.text_color.or(self.text_color),
            cursor_color: other.cursor_color.or(self.cursor_color),
            translation_x: other.translation_x.or(self.translation_x),
            translation_y: other.translation_y.or(self.translation_y),
            rotation: other.rotation.or(self.rotation),
            transform_origin: other.transform_origin.or(self.transform_origin),
            width_override: other.width_override.or(self.width_override),
            height_override: other.height_override.or(self.height_override),
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
            match shape {
                Shape::Rect(ref mut rect) => {
                    if let Some(color) = self.fill_color {
                        rect.fill = color;
                    }

                    // Apply stroke via unified field only.
                    if let Some(stroke) = self.stroke {
                        rect.stroke = Some(stroke);
                    }

                    // Apply corner shape
                    if let Some(corner_shape) = self.corner_shape {
                        rect.corner_shape = corner_shape;
                    }
                }
                Shape::Triangle(ref mut tri) => {
                    if let Some(color) = self.fill_color {
                        tri.fill = color;
                    }

                    // Apply stroke
                    if let Some(stroke) = self.stroke {
                        tri.stroke = Some(stroke);
                    }
                }
                Shape::Text(_) => {
                    // Text shapes don't have fill/stroke
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

        // Apply translation if present
        let has_translation = self.translation_x.is_some() || self.translation_y.is_some();

        if has_translation {
            let current_translation = node.translation();
            let new_x = self.translation_x.unwrap_or(current_translation.x);
            let new_y = self.translation_y.unwrap_or(current_translation.y);
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

        // Apply width/height overrides if present
        if let Some(width) = self.width_override {
            node.set_width_override(width);
        }
        if let Some(height) = self.height_override {
            node.set_height_override(height);
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
