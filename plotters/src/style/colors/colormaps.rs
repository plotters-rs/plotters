use crate::style::{HSLColor,RGBAColor,RGBColor};

use num_traits::{Float,ToPrimitive,FromPrimitive};

pub trait ColorScale<ColorType: crate::prelude::Color, FloatType=f32>
where
    FloatType: Float,
{
    fn get_color(&self, h: FloatType) -> ColorType {
        self.get_color_normalized(h, FloatType::zero(), FloatType::one())
    }

    fn get_color_normalized(&self, h: FloatType, min: FloatType, max: FloatType) -> ColorType;
}


macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}


macro_rules! define_colors_from_list_of_values_or_directly{
    ($color_type:tt, $(($($color_value:expr),+)),+) => {
        [$($color_type($($color_value),+)),+]
    };
    ($($color_complete:tt),+) => {
        [$($color_complete),+]
    };
}


macro_rules! implement_linear_interpolation_color_map{
    ($color_scale_name:ident, $color_type:tt) => {
        impl<FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive> ColorScale<$color_type, FloatType> for $color_scale_name {
            fn get_color_normalized(&self, h: FloatType, min: FloatType, max: FloatType) -> $color_type {
                // Ensure that we do have a value in bounds
                let h = h.max(min).min(max);
                // Make sure that we really have a minimal value which is smaller than the maximal value
                assert_eq!(min<max, true);
                // Next calculate a normalized value between 0.0 and 1.0
                let t = (h - min)/(max-min);
                let approximate_index = t * (FloatType::from_usize(Self::COLORS.len()).unwrap() - FloatType::one()).max(FloatType::zero());
                // Calculate which index are the two most nearest of the supplied value
                let index_lower = approximate_index.floor().to_usize().unwrap();
                let index_upper = approximate_index.ceil().to_usize().unwrap();
                // Calculate the relative difference, ie. is the actual value more towards the color of index_upper or index_lower?
                let relative_difference = approximate_index.ceil() - approximate_index;
                // Interpolate the final color linearly
                calculate_new_color_value!(relative_difference, Self::COLORS, index_upper, index_lower, $color_type)
            }
        }

        impl $color_scale_name {
            pub fn get_color<FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive>(h: FloatType) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color(h)
            }

            pub fn get_color_normalized<FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive>(h: FloatType, min: FloatType, max: FloatType) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color_normalized(h, min, max)
            }
        }
    }
}


#[macro_export]
macro_rules! define_linear_interpolation_color_map{
    ($color_scale_name:ident, $color_type:tt, $(($($color_value:expr),+)),*) => {
        pub struct $color_scale_name {}

        impl $color_scale_name {
            // const COLORS: [$color_type; $number_colors] = [$($color_type($($color_value),+)),+];
            // const COLORS: [$color_type; count!($(($($color_value:expr),+))*)] = [$($color_type($($color_value),+)),+];
            const COLORS: [$color_type; count!($(($($color_value:expr),+))*)] = define_colors_from_list_of_values_or_directly!{$color_type, $(($($color_value),+)),*};
        }

        implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    };
    ($color_scale_name:ident, $color_type:tt, $($color_complete:tt),+) => {
        pub struct $color_scale_name {}

        impl $color_scale_name {
            const COLORS: [$color_type; count!($($color_complete)*)] = define_colors_from_list_of_values_or_directly!{$($color_complete),+};
        }

        implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    }
}


define_linear_interpolation_color_map!{
    ViridisRGBA,
    RGBAColor,
    ( 68,   1,  84, 1.0),
    ( 70,  50, 127, 1.0),
    ( 54,  92, 141, 1.0),
    ( 39, 127, 143, 1.0),
    ( 31, 162, 136, 1.0),
    ( 74, 194, 110, 1.0),
    (160, 219,  57, 1.0),
    (254, 232,  37, 1.0)
}


define_linear_interpolation_color_map!{
    ViridisRGB,
    RGBColor,
    ( 68,   1,  84),
    ( 70,  50, 127),
    ( 54,  92, 141),
    ( 39, 127, 143),
    ( 31, 162, 136),
    ( 74, 194, 110),
    (160, 219,  57),
    (254, 232,  37)
}


define_linear_interpolation_color_map!{
    BlackWhite,
    RGBColor,
    (  0,   0,   0),
    (255, 255,   255)
}


define_linear_interpolation_color_map!{
    MandelbrotHSL,
    HSLColor,
    (0.0, 1.0, 0.5),
    (1.0, 1.0, 0.5)
}


define_linear_interpolation_color_map!{
    VulcanoHSL,
    HSLColor,
    (2.0/3.0, 1.0, 0.7),
    (    0.0, 1.0, 0.7)
}


use super::full_palette::*;
define_linear_interpolation_color_map!{
    Bone,
    RGBColor,
    BLACK,
    BLUE,
    WHITE
}


define_linear_interpolation_color_map!{
    Copper,
    RGBColor,
    BLACK,
    BROWN,
    ORANGE
}
