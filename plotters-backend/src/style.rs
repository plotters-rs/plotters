/// The color type that is used by all the backend
#[derive(Clone, Copy)]
pub struct BackendColor {
    pub alpha: f64,
    pub rgb: (u8, u8, u8),
}

impl BackendColor {
    #[inline(always)]
    pub fn mix(&self, alpha: f64) -> Self {
        Self {
            alpha: self.alpha * alpha,
            rgb: self.rgb,
        }
    }
}

/// The style data for the backend drawing API
pub trait BackendStyle {
    /// Get the color of current style
    fn color(&self) -> BackendColor;

    /// Get the stroke width of current style
    fn stroke_width(&self) -> u32 {
        1
    }
}

impl BackendStyle for BackendColor {
    fn color(&self) -> BackendColor {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_color_mix_multiplies_alpha() {
        let color = BackendColor {
            alpha: 0.8,
            rgb: (10, 20, 30),
        };

        let mixed = color.mix(0.5);

        assert!((mixed.alpha - 0.4).abs() < f64::EPSILON);
        assert_eq!(mixed.rgb, (10, 20, 30));
    }

    #[test]
    fn backend_color_mix_preserves_rgb() {
        let color = BackendColor {
            alpha: 1.0,
            rgb: (255, 128, 64),
        };

        let mixed = color.mix(0.25);

        assert_eq!(mixed.rgb, color.rgb);
    }

    #[test]
    fn backend_color_mix_with_zero_alpha_makes_fully_transparent() {
        let color = BackendColor {
            alpha: 1.0,
            rgb: (1, 2, 3),
        };

        let mixed = color.mix(0.0);

        assert_eq!(mixed.alpha, 0.0);
        assert_eq!(mixed.rgb, (1, 2, 3));
    }

    #[test]
    fn backend_color_mix_combines_with_existing_alpha() {
        let color = BackendColor {
            alpha: 0.5,
            rgb: (1, 2, 3),
        };

        let mixed = color.mix(0.5);

        assert_eq!(mixed.alpha, 0.25);
        assert_eq!(mixed.rgb, (1, 2, 3));
    }

    #[test]
    fn backend_color_as_style_returns_itself_as_color() {
        let color = BackendColor {
            alpha: 0.75,
            rgb: (4, 5, 6),
        };

        let style_color = color.color();

        assert_eq!(style_color.alpha, 0.75);
        assert_eq!(style_color.rgb, (4, 5, 6));
    }

    #[test]
    fn backend_color_as_style_uses_default_stroke_width() {
        let color = BackendColor {
            alpha: 1.0,
            rgb: (4, 5, 6),
        };

        assert_eq!(color.stroke_width(), 1);
    }

    struct CustomStyle {
        color: BackendColor,
        stroke_width: u32,
    }

    impl BackendStyle for CustomStyle {
        fn color(&self) -> BackendColor {
            self.color
        }

        fn stroke_width(&self) -> u32 {
            self.stroke_width
        }
    }

    #[test]
    fn custom_backend_style_can_override_stroke_width() {
        let style = CustomStyle {
            color: BackendColor {
                alpha: 1.0,
                rgb: (7, 8, 9),
            },
            stroke_width: 5,
        };

        assert_eq!(style.stroke_width(), 5);
    }

    #[test]
    fn custom_backend_style_returns_its_color() {
        let style = CustomStyle {
            color: BackendColor {
                alpha: 0.6,
                rgb: (7, 8, 9),
            },
            stroke_width: 5,
        };

        let color = style.color();

        assert_eq!(color.alpha, 0.6);
        assert_eq!(color.rgb, (7, 8, 9));
    }
}
