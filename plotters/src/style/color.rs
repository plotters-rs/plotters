use super::palette::Palette;
use super::ShapeStyle;

use plotters_backend::{BackendColor, BackendStyle};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::marker::PhantomData;

/// Any color representation
pub trait Color {
    /// Normalize this color representation to the backend color
    fn to_backend_color(&self) -> BackendColor;

    /// Convert the RGB representation to the standard RGB tuple
    #[inline(always)]
    fn rgb(&self) -> (u8, u8, u8) {
        self.to_backend_color().rgb
    }

    /// Get the alpha channel of the color
    #[inline(always)]
    fn alpha(&self) -> f64 {
        self.to_backend_color().alpha
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

impl<T: Color> Color for &'_ T {
    fn to_backend_color(&self) -> BackendColor {
        <T as Color>::to_backend_color(*self)
    }
}

/// The RGBA representation of the color, Plotters use RGBA as the internal representation
/// of color
///
/// If you want to directly create a RGB color with transparency use [RGBColor::mix]
#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct RGBAColor(pub u8, pub u8, pub u8, pub f64);

impl Color for RGBAColor {
    #[inline(always)]
    fn to_backend_color(&self) -> BackendColor {
        BackendColor {
            rgb: (self.0, self.1, self.2),
            alpha: self.3,
        }
    }
}

impl From<RGBColor> for RGBAColor {
    fn from(rgb: RGBColor) -> Self {
        Self(rgb.0, rgb.1, rgb.2, 1.0)
    }
}

/// A color in the given palette
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct PaletteColor<P: Palette>(usize, PhantomData<P>);

impl<P: Palette> PaletteColor<P> {
    /// Pick a color from the palette
    pub fn pick(idx: usize) -> PaletteColor<P> {
        PaletteColor(idx % P::COLORS.len(), PhantomData)
    }
}

impl<P: Palette> Color for PaletteColor<P> {
    #[inline(always)]
    fn to_backend_color(&self) -> BackendColor {
        BackendColor {
            rgb: P::COLORS[self.0],
            alpha: 1.0,
        }
    }
}

/// The color described by its RGB value
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl BackendStyle for RGBAColor {
    fn color(&self) -> BackendColor {
        self.to_backend_color()
    }
}

impl Color for RGBColor {
    #[inline(always)]
    fn to_backend_color(&self) -> BackendColor {
        BackendColor {
            rgb: (self.0, self.1, self.2),
            alpha: 1.0,
        }
    }
}
impl BackendStyle for RGBColor {
    fn color(&self) -> BackendColor {
        self.to_backend_color()
    }
}

/// The color described by HSL color space
#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct HSLColor(pub f64, pub f64, pub f64);

/// Errors that can occur when constructing an `HSLColor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HSLColorError {
    /// Hue (or degrees input) must be finite.
    NonFiniteHue,
    /// Saturation must be in the closed interval `[0, 1]`.
    SaturationOutOfRange,
    /// Lightness must be in the closed interval `[0, 1]`.
    LightnessOutOfRange,
}

impl fmt::Display for HSLColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HSLColorError::NonFiniteHue => f.write_str("hue must be finite"),
            HSLColorError::SaturationOutOfRange => f.write_str("saturation must be in [0, 1]"),
            HSLColorError::LightnessOutOfRange => f.write_str("lightness must be in [0, 1]"),
        }
    }
}

impl std::error::Error for HSLColorError {}

impl HSLColor {
    /// Creates an `HSLColor` from normalized components, returning an error if any are out of range.
    pub fn try_new(h: f64, s: f64, l: f64) -> Result<Self, HSLColorError> {
        if !h.is_finite() {
            return Err(HSLColorError::NonFiniteHue);
        }
        if !s.is_finite() || s < 0.0 || s > 1.0 {
            return Err(HSLColorError::SaturationOutOfRange);
        }
        if !l.is_finite() || l < 0.0 || l > 1.0 {
            return Err(HSLColorError::LightnessOutOfRange);
        }
        Ok(Self(h, s, l))
    }

    /// Creates an `HSLColor` from degrees, wrapping into `[0, 360)` before normalizing.
    /// Prefer this helper when specifying hue in degrees. Returns an error if saturation
    /// or lightness fall outside `[0, 1]` or if the input hue is non-finite.
    #[inline]
    pub fn from_degrees(h_deg: f64, s: f64, l: f64) -> Result<Self, HSLColorError> {
        if !h_deg.is_finite() {
            return Err(HSLColorError::NonFiniteHue);
        }
        Self::try_new(h_deg.rem_euclid(360.0) / 360.0, s, l)
    }
}

impl Color for HSLColor {
    #[inline(always)]
    #[allow(clippy::many_single_char_names)]
    fn to_backend_color(&self) -> BackendColor {
        // Hue is expected normalized in [0,1); wrap to keep negative or slightly
        // out-of-range inputs usable, but do not reinterpret raw degrees.
        let h = self.0.rem_euclid(1.0);

        // Saturation & lightness remain clamped to valid ranges
        let s = self.1.clamp(0.0, 1.0);
        let l = self.2.clamp(0.0, 1.0);

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

#[cfg(test)]
mod hue_robustness_tests {
    use super::*;

    #[test]
    fn degrees_passed_via_helper_should_work_for_common_cases() {
        let red = HSLColor::from_degrees(0.0, 1.0, 0.5)
            .unwrap()
            .to_backend_color()
            .rgb;
        assert_eq!(red, (255, 0, 0));

        let green = HSLColor::from_degrees(120.0, 1.0, 0.5)
            .unwrap()
            .to_backend_color()
            .rgb;
        assert_eq!(green, (0, 255, 0));

        let blue = HSLColor::from_degrees(240.0, 1.0, 0.5)
            .unwrap()
            .to_backend_color()
            .rgb;
        assert_eq!(blue, (0, 0, 255));
    }

    #[test]
    fn from_degrees_wraps_and_matches_normalized() {
        let normalized = HSLColor(120.0 / 360.0, 1.0, 0.5).to_backend_color().rgb;
        let via_helper = HSLColor::from_degrees(120.0, 1.0, 0.5)
            .unwrap()
            .to_backend_color()
            .rgb;
        assert_eq!(normalized, via_helper);

        let wrap_positive =
            HSLColor::from_degrees(720.0, 1.0, 0.5).unwrap().to_backend_color().rgb;
        let wrap_negative =
            HSLColor::from_degrees(-120.0, 1.0, 0.5).unwrap().to_backend_color().rgb;
        let canonical =
            HSLColor::from_degrees(0.0, 1.0, 0.5).unwrap().to_backend_color().rgb;

        assert_eq!(wrap_positive, canonical);
        assert_eq!(
            wrap_negative,
            HSLColor::from_degrees(240.0, 1.0, 0.5)
                .unwrap()
                .to_backend_color()
                .rgb
        );
    }

    #[test]
    fn from_degrees_rejects_out_of_range_components() {
        assert!(matches!(
            HSLColor::from_degrees(0.0, -0.1, 0.5),
            Err(HSLColorError::SaturationOutOfRange)
        ));
        assert!(matches!(
            HSLColor::from_degrees(0.0, 0.5, 1.1),
            Err(HSLColorError::LightnessOutOfRange)
        ));
        assert!(matches!(
            HSLColor::from_degrees(f64::INFINITY, 0.5, 0.5),
            Err(HSLColorError::NonFiniteHue)
        ));
    }
}
