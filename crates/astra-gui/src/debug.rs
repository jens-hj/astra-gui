/// Configuration for debug visualization
#[derive(Clone, Copy, Debug, Default)]
pub struct DebugOptions {
    /// Show margin areas (red overlay)
    pub show_margins: bool,
    /// Show padding areas (blue overlay)
    pub show_padding: bool,
    /// Show node borders (green outline)
    pub show_borders: bool,
    /// Show content areas (yellow outline)
    pub show_content_area: bool,
    /// Show clip rectangles (red outline)
    pub show_clip_rects: bool,
    /// Show gaps between children (purple overlay)
    pub show_gaps: bool,
    /// Show transform origins (crosshair)
    pub show_transform_origins: bool,
    /// Show text line bounds (cyan outline for each line)
    pub show_text_bounds: bool,
}

impl DebugOptions {
    /// Create debug options with nothing enabled
    pub const fn none() -> Self {
        Self {
            show_margins: false,
            show_padding: false,
            show_borders: false,
            show_content_area: false,
            show_clip_rects: false,
            show_gaps: false,
            show_transform_origins: false,
            show_text_bounds: false,
        }
    }

    /// Create debug options with all visualizations enabled
    pub const fn all() -> Self {
        Self {
            show_margins: true,
            show_padding: true,
            show_borders: true,
            show_content_area: true,
            show_clip_rects: true,
            show_gaps: true,
            show_transform_origins: true,
            show_text_bounds: true,
        }
    }

    /// Enable margin visualization
    pub const fn with_margins(mut self, enabled: bool) -> Self {
        self.show_margins = enabled;
        self
    }

    /// Enable padding visualization
    pub const fn with_padding(mut self, enabled: bool) -> Self {
        self.show_padding = enabled;
        self
    }

    /// Enable border visualization
    pub const fn with_borders(mut self, enabled: bool) -> Self {
        self.show_borders = enabled;
        self
    }

    /// Enable content area visualization
    pub const fn with_content_area(mut self, enabled: bool) -> Self {
        self.show_content_area = enabled;
        self
    }

    /// Enable clip rect visualization
    pub const fn with_clip_rects(mut self, enabled: bool) -> Self {
        self.show_clip_rects = enabled;
        self
    }

    /// Enable gap visualization
    pub const fn with_gaps(mut self, enabled: bool) -> Self {
        self.show_gaps = enabled;
        self
    }

    /// Enable transform origin visualization
    pub const fn with_transform_origins(mut self, enabled: bool) -> Self {
        self.show_transform_origins = enabled;
        self
    }

    /// Check if any debug visualization is enabled
    pub const fn is_enabled(&self) -> bool {
        self.show_margins
            || self.show_padding
            || self.show_borders
            || self.show_content_area
            || self.show_clip_rects
            || self.show_gaps
            || self.show_transform_origins
            || self.show_text_bounds
    }
}
