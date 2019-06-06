use super::palette::Palette;
/// The abstraction of a color
use std::marker::PhantomData;

/// Any color representation
pub trait Color {
    /// Convert the RGB representation to the standard RGB tuple
    fn rgb(&self) -> (u8, u8, u8);

    /// Get the alpha channel of the color
    fn alpha(&self) -> f64;

    /// Mix the color with given opacity
    fn mix(&self, value: f64) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha() * value;
        RGBAColor(r, g, b, a)
    }

    /// Convert the color into the RGBA color which is intrenally used by Plotters
    fn to_rgba(&self) -> RGBAColor {
        let (r, g, b) = self.rgb();
        let a = self.alpha();
        RGBAColor(r, g, b, a)
    }
}

/// The RGBA representation of the color, Plotters use RGBA as the internal representation
/// of color
#[derive(Clone, PartialEq, Debug)]
pub struct RGBAColor(u8, u8, u8, f64);

impl Color for RGBAColor {
    fn rgb(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }

    fn alpha(&self) -> f64 {
        self.3
    }

    fn to_rgba(&self) -> RGBAColor {
        self.clone()
    }
}

/// Color without alpha channel
pub trait SimpleColor {
    fn rgb(&self) -> (u8, u8, u8);
}

impl<T: SimpleColor> Color for T {
    fn rgb(&self) -> (u8, u8, u8) {
        SimpleColor::rgb(self)
    }

    fn alpha(&self) -> f64 {
        1.0
    }
}

/// A color in the given palette
pub struct PaletteColor<P: Palette>(usize, PhantomData<P>);

impl<P: Palette> PaletteColor<P> {
    /// Pick a color from the palette
    pub fn pick(idx: usize) -> PaletteColor<P> {
        PaletteColor(idx % P::COLORS.len(), PhantomData)
    }
}

impl<P: Palette> SimpleColor for PaletteColor<P> {
    fn rgb(&self) -> (u8, u8, u8) {
        P::COLORS[self.0]
    }
}

/// The color described by it's RGB value
pub struct RGBColor(pub u8, pub u8, pub u8);

impl SimpleColor for RGBColor {
    fn rgb(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }
}

macro_rules! predefined_color {
    ($name:ident, $r:expr, $g:expr, $b:expr, $doc: expr) => {
        #[doc = $doc]
        pub struct $name;
        impl SimpleColor for $name {
            fn rgb(&self) -> (u8,u8,u8) {
                return ($r, $g, $b);
            }
        }
    };

    ($name:ident, $r:expr, $g:expr, $b:expr, $a: expr, $doc: expr) => {
        #[doc = $doc]
        pub struct $name;
        impl Color for $name {
            fn rgb(&self) -> (u8,u8,u8) {
                return ($r, $g, $b);
            }
            fn alpha(&self) -> f64 {
                $a
            }
        }
    }
}

predefined_color!(White, 255, 255, 255, "The predefined white color");
predefined_color!(Black, 0, 0, 0, "The predefined black color");
predefined_color!(Red, 255, 0, 0, "The predefined red color");
predefined_color!(Green, 0, 255, 0, "The predefined green color");
predefined_color!(Blue, 0, 0, 255, "The predefined blue color");
predefined_color!(Yellow, 255, 255, 0, "The predefined yellow color");
predefined_color!(Cyan, 0, 255, 255, "The predefined cyan color");
predefined_color!(Magenta, 255, 0, 255, "The predefined magenta color");
predefined_color!(Transparent, 0, 0, 0, 0.0, "The predefined transparent");

/// The color described by HSL color space
pub struct HSLColor(pub f64, pub f64, pub f64);

impl SimpleColor for HSLColor {
    #[allow(clippy::many_single_char_names)]
    fn rgb(&self) -> (u8, u8, u8) {
        let (h, s, l) = (
            self.0.min(1.0).max(0.0),
            self.1.min(1.0).max(0.0),
            self.2.min(1.0).max(0.0),
        );

        if s == 0.0 {
            let value = (l * 255.0).round() as u8;
            return (value, value, value);
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

        (cvt(h + 1.0 / 3.0), cvt(h), cvt(h - 1.0 / 3.0))
    }
}
