use super::palette::Palette;
use super::ShapeStyle;

use plotters_backend::{BackendColor, BackendStyle};

use std::marker::PhantomData;

/// Any color representation
pub trait Color: BackendStyle {
    /// Convert the RGB representation to the standard RGB tuple
    #[inline(always)]
    fn rgb(&self) -> (u8, u8, u8) {
        self.color().rgb
    }

    /// Get the alpha channel of the color
    #[inline(always)]
    fn alpha(&self) -> f64 {
        self.color().alpha
    }

    /// Mix the color with given opacity
    fn mix(&self, value: f64) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha() * value;
        RGBAColor(r, g, b, a)
    }

    /// Convert the color into the RGBA color which is internally used by Plotters
    fn to_rgba(&self) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha();
        RGBAColor(r, g, b, a)
    }

    /// Make a filled style form the color
    fn filled(&self) -> ShapeStyle
    where
        Self: Sized,
    {
        Into::<ShapeStyle>::into(self).filled()
    }

    /// Make a shape style with stroke width from a color
    fn stroke_width(&self, width: u32) -> ShapeStyle
    where
        Self: Sized,
    {
        Into::<ShapeStyle>::into(self).stroke_width(width)
    }
}

/// The RGBA representation of the color, Plotters use RGBA as the internal representation
/// of color
#[derive(Clone, PartialEq, Debug)]
pub struct RGBAColor(pub(crate) u8, pub(crate) u8, pub(crate) u8, pub(crate) f64);

impl BackendStyle for RGBAColor {
    #[inline(always)]
    fn color(&self) -> BackendColor {
        BackendColor {
            rgb: (self.0, self.1, self.2),
            alpha: self.3,
        }
    }
}
impl Color for RGBAColor {}

/// A color in the given palette
pub struct PaletteColor<P: Palette>(usize, PhantomData<P>);

impl<P: Palette> PaletteColor<P> {
    /// Pick a color from the palette
    pub fn pick(idx: usize) -> PaletteColor<P> {
        PaletteColor(idx % P::COLORS.len(), PhantomData)
    }
}

impl<P: Palette> BackendStyle for PaletteColor<P> {
    #[inline(always)]
    fn color(&self) -> BackendColor {
        BackendColor {
            rgb: P::COLORS[self.0],
            alpha: 1.0,
        }
    }
}

impl<P: Palette> Color for PaletteColor<P> {}

/// The color described by its RGB value
#[derive(Debug)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl BackendStyle for RGBColor {
    #[inline(always)]
    fn color(&self) -> BackendColor {
        BackendColor {
            rgb: (self.0, self.1, self.2),
            alpha: 1.0,
        }
    }
}

impl Color for RGBColor {}

/// The color described by HSL color space
pub struct HSLColor(pub f64, pub f64, pub f64);

impl BackendStyle for HSLColor {
    #[inline(always)]
    #[allow(clippy::many_single_char_names)]
    fn color(&self) -> BackendColor {
        let (h, s, l) = (
            self.0.min(1.0).max(0.0),
            self.1.min(1.0).max(0.0),
            self.2.min(1.0).max(0.0),
        );

        if s == 0.0 {
            let value = (l * 255.0).round() as u8;
            return BackendColor {
                rgb: (value, value, value),
                alpha: 1.0,
            };
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let cvt = |mut t| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            let value = if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 1.0 / 2.0 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            };
            (value * 255.0).round() as u8
        };

        BackendColor {
            rgb: (cvt(h + 1.0 / 3.0), cvt(h), cvt(h - 1.0 / 3.0)),
            alpha: 1.0,
        }
    }
}

impl Color for HSLColor {}
