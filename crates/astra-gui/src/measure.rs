//! Intrinsic content measurement for layout resolution.
//!
//! This module provides a backend-agnostic trait for measuring intrinsic content size
//! (e.g., text metrics) during layout. It enables `Size::FitContent` to resolve to
//! actual dimensions rather than falling back to parent size.

use crate::content::{HorizontalAlign, TextContent, VerticalAlign, Wrap};

/// Request to measure the intrinsic size of text (single or multi-line).
#[derive(Debug, Clone)]
pub struct MeasureTextRequest<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
    /// Optional font family name (backend-defined meaning)
    pub family: Option<&'a str>,
    /// Maximum width constraint for wrapping (None = no constraint)
    pub max_width: Option<f32>,
    /// Text wrapping mode
    pub wrap: Wrap,
    /// Line height as a multiplier of font size
    pub line_height_multiplier: f32,
}

impl<'a> MeasureTextRequest<'a> {
    pub fn from_text_content(content: &'a TextContent) -> Self {
        // Resolve font_size to f32 - use a reference size for measurement
        // During actual layout, the scale_factor will be applied
        let font_size = content
            .font_size
            .try_resolve_with_scale(1000.0, 1.0)
            .unwrap_or(16.0);

        Self {
            text: &content.text,
            font_size,
            h_align: content.h_align,
            v_align: content.v_align,
            family: None,
            max_width: None,
            wrap: content.wrap,
            line_height_multiplier: content.line_height_multiplier,
        }
    }
}

/// Intrinsic size measurement result.
#[derive(Debug, Clone, Copy, Default)]
pub struct IntrinsicSize {
    pub width: f32,
    pub height: f32,
}

impl IntrinsicSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Backend-agnostic content measurement.
///
/// Implementors provide intrinsic size information for content types (primarily text).
/// The layout algorithm uses this trait to resolve `Size::FitContent`.
///
/// This trait is intentionally minimal and backend-agnostic: core layout must not
/// depend on any specific text engine (cosmic-text, etc.). Backends like
/// `astra-gui-text` implement this trait.
pub trait ContentMeasurer {
    /// Measure the intrinsic size of a single line of text.
    ///
    /// This should return the minimum bounding box that fits the shaped text,
    /// excluding any padding or margins (those are handled by layout).
    fn measure_text(&mut self, request: MeasureTextRequest<'_>) -> IntrinsicSize;
}
