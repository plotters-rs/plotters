//! Basic predefined colors.
use super::{RGBAColor, RGBColor};

macro_rules! predefined_color {
    ($name:ident, $r:expr, $g:expr, $b:expr, $doc:expr) => {
        #[doc = $doc]
        pub const $name: RGBColor = RGBColor($r, $g, $b);
    };

    ($name:ident, $r:expr, $g:expr, $b:expr, $a: expr, $doc:expr) => {
        #[doc = $doc]
        pub const $name: RGBAColor = RGBAColor($r, $g, $b, $a);
    }
}

predefined_color!(WHITE, 255, 255, 255, "The predefined white color");
predefined_color!(BLACK, 0, 0, 0, "The predefined black color");
predefined_color!(RED, 255, 0, 0, "The predefined red color");
predefined_color!(GREEN, 0, 255, 0, "The predefined green color");
predefined_color!(BLUE, 0, 0, 255, "The predefined blue color");
predefined_color!(YELLOW, 255, 255, 0, "The predefined yellow color");
predefined_color!(CYAN, 0, 255, 255, "The predefined cyan color");
predefined_color!(MAGENTA, 255, 0, 255, "The predefined magenta color");
predefined_color!(TRANSPARENT, 0, 0, 0, 0.0, "The predefined transparent");

/// Predefined Color definitions using the [palette](https://docs.rs/palette/) color types
#[cfg(feature = "palette_ext")]
pub mod palette_ext {
    use palette::rgb::Srgb;
    use palette::Alpha;

    use std::marker::PhantomData;

    macro_rules! predefined_color_pal {
        ($name:ident, $r:expr, $g:expr, $b:expr, $doc:expr) => {
            #[doc = $doc]
            pub const $name: Srgb<u8> = predefined_color_pal!(@gen_c $r, $g, $b);
        };
        ($name:ident, $r:expr, $g:expr, $b:expr, $a:expr, $doc:expr) => {
            #[doc = $doc]
            pub const $name: Alpha<Srgb<u8>, f64> = Alpha{ alpha: $a, color: predefined_color_pal!(@gen_c $r, $g, $b) };
        };
        (@gen_c $r:expr, $g:expr, $b:expr) => {
            Srgb { red: $r, green: $g, blue: $b, standard: PhantomData }
        };
    }

    predefined_color_pal!(WHITE, 255, 255, 255, "The predefined white color");
    predefined_color_pal!(BLACK, 0, 0, 0, "The predefined black color");
    predefined_color_pal!(RED, 255, 0, 0, "The predefined red color");
    predefined_color_pal!(GREEN, 0, 255, 0, "The predefined green color");
    predefined_color_pal!(BLUE, 0, 0, 255, "The predefined blue color");
    predefined_color_pal!(YELLOW, 255, 255, 0, "The predefined yellow color");
    predefined_color_pal!(CYAN, 0, 255, 255, "The predefined cyan color");
    predefined_color_pal!(MAGENTA, 255, 0, 255, "The predefined magenta color");
    predefined_color_pal!(TRANSPARENT, 0, 0, 0, 0.0, "The predefined transparent");
}
