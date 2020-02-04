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
