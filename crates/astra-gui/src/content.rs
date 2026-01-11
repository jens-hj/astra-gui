use crate::color::Color;

/// Font weight for text rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Thin,        // 100
    ExtraLight,  // 200
    Light,       // 300
    Normal,      // 400 (default)
    Medium,      // 500
    SemiBold,    // 600
    Bold,        // 700
    ExtraBold,   // 800
    Black,       // 900
    Custom(u16), // Exact weight for variable fonts
}

impl FontWeight {
    pub fn to_weight(self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
            FontWeight::Custom(w) => w.clamp(100, 900),
        }
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Normal
    }
}

/// Font style (normal or italic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

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
    /// Font weight (default: Normal/400)
    pub font_weight: FontWeight,
    /// Font style (default: Normal)
    pub font_style: FontStyle,
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
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
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

    /// Set font weight
    pub fn with_font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set font style
    pub fn with_font_style(mut self, style: FontStyle) -> Self {
        self.font_style = style;
        self
    }

    /// Convenience method to set bold weight
    pub fn bold(mut self) -> Self {
        self.font_weight = FontWeight::Bold;
        self
    }

    /// Convenience method to set italic style
    pub fn italic(mut self) -> Self {
        self.font_style = FontStyle::Italic;
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
