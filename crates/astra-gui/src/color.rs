/// RGBA color in linear space with values in [0, 1]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    pub const fn transparent() -> Self {
        Self::rgba(0.0, 0.0, 0.0, 0.0)
    }

    /// Convert sRGB color (0-255) to linear space
    /// Uses proper sRGB gamma correction (ITU-R BT.709)
    #[inline]
    pub const fn srgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        const fn srgb_to_linear(c: u8) -> f32 {
            let x = c as f32 / 255.0;
            // Standard sRGB to linear conversion (ITU-R BT.709)
            if x <= 0.04045 {
                x / 12.92
            } else {
                // Approximate ((x + 0.055) / 1.055)^2.4
                // Using simple polynomial approximation (original)
                let t = (x + 0.055) / 1.055;
                t * t * (0.5870 * t + 0.4130)
            }
        }

        Self::rgba(
            srgb_to_linear(r),
            srgb_to_linear(g),
            srgb_to_linear(b),
            a as f32 / 255.0,
        )
    }

    /// with alpha builder method taking u8
    pub fn with_alpha_u8(mut self, alpha: u8) -> Self {
        self.a = alpha as f32 / 255.0;
        self
    }

    /// with alpha builder method taking f32
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }

    /// Calculate relative luminance (0.0 to 1.0)
    /// Uses standard coefficients for linear RGB: 0.2126 R + 0.7152 G + 0.0722 B
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Calculate contrast ratio with another color (1.0 to 21.0)
    pub fn contrast_ratio(&self, other: &Color) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// CSS color constants
pub mod css {
    use super::Color;

    pub const AQUA: Color = Color::srgba(0, 255, 255, 255);
    pub const BLACK: Color = Color::srgba(0, 0, 0, 255);
    pub const BLUE: Color = Color::srgba(0, 0, 255, 255);
    pub const FUCHSIA: Color = Color::srgba(255, 0, 255, 255);
    pub const GRAY: Color = Color::srgba(128, 128, 128, 255);
    pub const GREEN: Color = Color::srgba(0, 128, 0, 255);
    pub const LIME: Color = Color::srgba(0, 255, 0, 255);
    pub const MAROON: Color = Color::srgba(128, 0, 0, 255);
    pub const NAVY: Color = Color::srgba(0, 0, 128, 255);
    pub const OLIVE: Color = Color::srgba(128, 128, 0, 255);
    pub const PURPLE: Color = Color::srgba(128, 0, 128, 255);
    pub const RED: Color = Color::srgba(255, 0, 0, 255);
    pub const SILVER: Color = Color::srgba(192, 192, 192, 255);
    pub const TEAL: Color = Color::srgba(0, 128, 128, 255);
    pub const WHITE: Color = Color::srgba(255, 255, 255, 255);
    pub const YELLOW: Color = Color::srgba(255, 255, 0, 255);
}

/// Catppuccin color palette
pub mod catppuccin {
    use super::Color;

    pub mod mocha {
        use super::Color;

        pub const ROSEWATER: Color = Color::srgba(245, 224, 220, 255);
        pub const FLAMINGO: Color = Color::srgba(242, 205, 205, 255);
        pub const PINK: Color = Color::srgba(245, 194, 231, 255);
        pub const MAUVE: Color = Color::srgba(203, 166, 247, 255);
        pub const RED: Color = Color::srgba(243, 139, 168, 255);
        pub const MAROON: Color = Color::srgba(235, 160, 172, 255);
        pub const PEACH: Color = Color::srgba(250, 179, 135, 255);
        pub const YELLOW: Color = Color::srgba(249, 226, 175, 255);
        pub const GREEN: Color = Color::srgba(166, 227, 161, 255);
        pub const TEAL: Color = Color::srgba(148, 226, 213, 255);
        pub const SKY: Color = Color::srgba(137, 220, 235, 255);
        pub const SAPPHIRE: Color = Color::srgba(116, 199, 236, 255);
        pub const BLUE: Color = Color::srgba(137, 180, 250, 255);
        pub const LAVENDER: Color = Color::srgba(180, 190, 254, 255);
        pub const TEXT: Color = Color::srgba(205, 214, 244, 255);
        pub const SUBTEXT1: Color = Color::srgba(186, 194, 222, 255);
        pub const SUBTEXT0: Color = Color::srgba(166, 173, 200, 255);
        pub const OVERLAY2: Color = Color::srgba(147, 153, 178, 255);
        pub const OVERLAY1: Color = Color::srgba(127, 132, 156, 255);
        pub const OVERLAY0: Color = Color::srgba(108, 112, 134, 255);
        pub const SURFACE2: Color = Color::srgba(88, 91, 112, 255);
        pub const SURFACE1: Color = Color::srgba(69, 71, 90, 255);
        pub const SURFACE0: Color = Color::srgba(49, 50, 68, 255);
        pub const BASE: Color = Color::srgba(30, 30, 46, 255);
        pub const MANTLE: Color = Color::srgba(24, 24, 37, 255);
        pub const CRUST: Color = Color::srgba(17, 17, 27, 255);
    }

    pub mod latte {
        use super::Color;

        pub const ROSEWATER: Color = Color::srgba(220, 138, 120, 255);
        pub const FLAMINGO: Color = Color::srgba(221, 120, 120, 255);
        pub const PINK: Color = Color::srgba(234, 118, 203, 255);
        pub const MAUVE: Color = Color::srgba(136, 57, 239, 255);
        pub const RED: Color = Color::srgba(210, 15, 57, 255);
        pub const MAROON: Color = Color::srgba(230, 69, 83, 255);
        pub const PEACH: Color = Color::srgba(254, 100, 11, 255);
        pub const YELLOW: Color = Color::srgba(223, 142, 29, 255);
        pub const GREEN: Color = Color::srgba(64, 160, 43, 255);
        pub const TEAL: Color = Color::srgba(23, 146, 153, 255);
        pub const SKY: Color = Color::srgba(4, 165, 229, 255);
        pub const SAPPHIRE: Color = Color::srgba(32, 159, 181, 255);
        pub const BLUE: Color = Color::srgba(30, 102, 245, 255);
        pub const LAVENDER: Color = Color::srgba(114, 135, 253, 255);
        pub const TEXT: Color = Color::srgba(76, 79, 105, 255);
        pub const SUBTEXT1: Color = Color::srgba(92, 95, 119, 255);
        pub const SUBTEXT0: Color = Color::srgba(108, 111, 133, 255);
        pub const OVERLAY2: Color = Color::srgba(124, 127, 147, 255);
        pub const OVERLAY1: Color = Color::srgba(140, 143, 161, 255);
        pub const OVERLAY0: Color = Color::srgba(156, 160, 176, 255);
        pub const SURFACE2: Color = Color::srgba(172, 176, 190, 255);
        pub const SURFACE1: Color = Color::srgba(188, 192, 204, 255);
        pub const SURFACE0: Color = Color::srgba(204, 208, 218, 255);
        pub const BASE: Color = Color::srgba(239, 241, 245, 255);
        pub const MANTLE: Color = Color::srgba(230, 233, 239, 255);
        pub const CRUST: Color = Color::srgba(220, 224, 232, 255);
    }

    pub mod frappe {
        use super::Color;

        pub const ROSEWATER: Color = Color::srgba(242, 213, 207, 255);
        pub const FLAMINGO: Color = Color::srgba(238, 190, 190, 255);
        pub const PINK: Color = Color::srgba(244, 184, 228, 255);
        pub const MAUVE: Color = Color::srgba(202, 158, 230, 255);
        pub const RED: Color = Color::srgba(231, 130, 132, 255);
        pub const MAROON: Color = Color::srgba(234, 153, 156, 255);
        pub const PEACH: Color = Color::srgba(239, 159, 118, 255);
        pub const YELLOW: Color = Color::srgba(229, 200, 144, 255);
        pub const GREEN: Color = Color::srgba(166, 209, 137, 255);
        pub const TEAL: Color = Color::srgba(129, 200, 190, 255);
        pub const SKY: Color = Color::srgba(153, 209, 219, 255);
        pub const SAPPHIRE: Color = Color::srgba(133, 193, 220, 255);
        pub const BLUE: Color = Color::srgba(140, 170, 238, 255);
        pub const LAVENDER: Color = Color::srgba(186, 187, 241, 255);
        pub const TEXT: Color = Color::srgba(198, 208, 245, 255);
        pub const SUBTEXT1: Color = Color::srgba(181, 191, 226, 255);
        pub const SUBTEXT0: Color = Color::srgba(165, 173, 206, 255);
        pub const OVERLAY2: Color = Color::srgba(148, 156, 187, 255);
        pub const OVERLAY1: Color = Color::srgba(131, 139, 167, 255);
        pub const OVERLAY0: Color = Color::srgba(115, 121, 148, 255);
        pub const SURFACE2: Color = Color::srgba(98, 104, 128, 255);
        pub const SURFACE1: Color = Color::srgba(81, 87, 109, 255);
        pub const SURFACE0: Color = Color::srgba(65, 69, 89, 255);
        pub const BASE: Color = Color::srgba(48, 52, 70, 255);
        pub const MANTLE: Color = Color::srgba(41, 44, 60, 255);
        pub const CRUST: Color = Color::srgba(35, 38, 52, 255);
    }

    pub mod macchiato {
        use super::Color;

        pub const ROSEWATER: Color = Color::srgba(244, 219, 214, 255);
        pub const FLAMINGO: Color = Color::srgba(240, 198, 198, 255);
        pub const PINK: Color = Color::srgba(245, 189, 230, 255);
        pub const MAUVE: Color = Color::srgba(198, 160, 246, 255);
        pub const RED: Color = Color::srgba(237, 135, 150, 255);
        pub const MAROON: Color = Color::srgba(238, 153, 160, 255);
        pub const PEACH: Color = Color::srgba(245, 169, 127, 255);
        pub const YELLOW: Color = Color::srgba(238, 212, 159, 255);
        pub const GREEN: Color = Color::srgba(166, 218, 149, 255);
        pub const TEAL: Color = Color::srgba(139, 213, 202, 255);
        pub const SKY: Color = Color::srgba(145, 215, 227, 255);
        pub const SAPPHIRE: Color = Color::srgba(125, 196, 228, 255);
        pub const BLUE: Color = Color::srgba(138, 173, 244, 255);
        pub const LAVENDER: Color = Color::srgba(183, 189, 248, 255);
        pub const TEXT: Color = Color::srgba(202, 211, 245, 255);
        pub const SUBTEXT1: Color = Color::srgba(184, 192, 224, 255);
        pub const SUBTEXT0: Color = Color::srgba(165, 173, 203, 255);
        pub const OVERLAY2: Color = Color::srgba(147, 154, 183, 255);
        pub const OVERLAY1: Color = Color::srgba(128, 135, 162, 255);
        pub const OVERLAY0: Color = Color::srgba(110, 115, 141, 255);
        pub const SURFACE2: Color = Color::srgba(91, 96, 120, 255);
        pub const SURFACE1: Color = Color::srgba(73, 77, 100, 255);
        pub const SURFACE0: Color = Color::srgba(54, 58, 79, 255);
        pub const BASE: Color = Color::srgba(36, 39, 58, 255);
        pub const MANTLE: Color = Color::srgba(30, 32, 48, 255);
        pub const CRUST: Color = Color::srgba(24, 25, 38, 255);
    }
}
