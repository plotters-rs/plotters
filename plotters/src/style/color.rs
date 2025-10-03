use super::palette::Palette;
use super::ShapeStyle;

use plotters_backend::{BackendColor, BackendStyle};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

/// Common trait for all color representations.
pub trait Color {
    fn to_backend_color(&self) -> BackendColor;

    #[inline(always)]
    fn rgb(&self) -> (u8, u8, u8) {
        self.to_backend_color().rgb
    }

    #[inline(always)]
    fn alpha(&self) -> f64 {
        self.to_backend_color().alpha
    }

    fn mix(&self, value: f64) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha() * value;
        RGBAColor(r, g, b, a)
    }

    fn to_rgba(&self) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha();
        RGBAColor(r, g, b, a)
    }

    fn filled(&self) -> ShapeStyle
    where
        Self: Sized,
    {
        Into::<ShapeStyle>::into(self).filled()
    }

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct PaletteColor<P: Palette>(usize, PhantomData<P>);

impl<P: Palette> PaletteColor<P> {
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

#[derive(Copy, Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct HSLColor(pub f64, pub f64, pub f64);

impl HSLColor {
    #[inline]
    pub fn from_degrees(h_deg: f64, s: f64, l: f64) -> Self {
        Self(h_deg / 360.0, s, l)
    }
}

impl Color for HSLColor {
    #[inline(always)]
    #[allow(clippy::many_single_char_names)]
    fn to_backend_color(&self) -> BackendColor {
        let h = if self.0 > 1.0 {
            (self.0 / 360.0).rem_euclid(1.0)
        } else {
            self.0.rem_euclid(1.0)
        };
        let s = self.1.clamp(0.0, 1.0);
        let l = self.2.clamp(0.0, 1.0);

        if s == 0.0 {
            let v = (l * 255.0).round() as u8;
            return BackendColor {
                rgb: (v, v, v),
                alpha: 1.0,
            };
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let cvt = |t: f64| {
            let t = t.rem_euclid(1.0);
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
    fn degrees_passed_directly_should_work_for_common_cases() {
        let red = HSLColor(0.0, 1.0, 0.5).to_backend_color().rgb;
        assert_eq!(red, (255, 0, 0));

        let green = HSLColor(120.0, 1.0, 0.5).to_backend_color().rgb;
        assert_eq!(green, (0, 255, 0));

        let blue = HSLColor(240.0, 1.0, 0.5).to_backend_color().rgb;
        assert_eq!(blue, (0, 0, 255));
    }

    #[test]
    fn from_degrees_and_direct_degrees_are_equivalent() {
        for &deg in &[0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0] {
            let a = HSLColor(deg, 1.0, 0.5).to_backend_color().rgb;
            let b = HSLColor::from_degrees(deg, 1.0, 0.5).to_backend_color().rgb;
            assert_eq!(a, b);
        }
    }
}
