use crate::color::Color;

/// Content that can be displayed in a node
///
/// Content nodes are leaf nodes that cannot have children. They represent
/// actual UI elements like text, inputs, images, etc.
#[derive(Debug, Clone)]
pub enum Content {
    /// Text content with styling
    Text(TextContent),
}

/// Text wrapping mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wrap {
    /// No wrapping, text overflows
    None,
    /// Wrap at word boundaries
    Word,
    /// Wrap at character boundaries
    Glyph,
    /// Try word boundaries, fallback to character wrap
    WordOrGlyph,
}

impl Default for Wrap {
    fn default() -> Self {
        Self::Word
    }
}

/// Text content configuration
#[derive(Debug, Clone)]
pub struct TextContent {
    /// The text to display
    pub text: String,
    /// Font size in logical pixels
    pub font_size: crate::layout::Size,
    /// Text color
    pub color: Color,
    /// Horizontal alignment within the node
    pub h_align: HorizontalAlign,
    /// Vertical alignment within the node
    pub v_align: VerticalAlign,
    /// Text wrapping mode
    pub wrap: Wrap,
    /// Line height as a multiplier of font size (default: 1.2)
    pub line_height_multiplier: f32,
}

impl TextContent {
    /// Create new text content with default styling
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: crate::layout::Size::lpx(16.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
            wrap: Wrap::Word,
            line_height_multiplier: 1.2,
        }
    }

    /// Set the font size
    pub fn with_font_size(mut self, size: crate::layout::Size) -> Self {
        self.font_size = size;
        self
    }

    /// Set the text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set horizontal alignment
    pub fn with_h_align(mut self, align: HorizontalAlign) -> Self {
        self.h_align = align;
        self
    }

    /// Set vertical alignment
    pub fn with_v_align(mut self, align: VerticalAlign) -> Self {
        self.v_align = align;
        self
    }

    /// Set text wrapping mode
    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set line height multiplier
    pub fn with_line_height(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier;
        self
    }
}

/// Horizontal text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

/// Vertical text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}
