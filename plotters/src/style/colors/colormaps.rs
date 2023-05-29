use crate::style::{HSLColor, RGBAColor, RGBColor};

use num_traits::{Float, FromPrimitive, ToPrimitive};

/// Converts scalar values to colors.
pub trait ColorMap<ColorType: crate::prelude::Color, FloatType = f32>
where
    FloatType: Float,
{
    /// Takes a scalar value 0.0 <= h <= 1.0 and returns the corresponding color.
    /// Typically color-scales are named according to which color-type they return.
    /// To use upper and lower bounds with ths function see [get_color_normalized](ColorMap::get_color_normalized).
    fn get_color(&self, h: FloatType) -> ColorType {
        self.get_color_normalized(h, FloatType::zero(), FloatType::one())
    }

    /// A slight abstraction over [get_color](ColorMap::get_color) function where lower and upper bound can be specified.
    fn get_color_normalized(&self, h: FloatType, min: FloatType, max: FloatType) -> ColorType;
}

/// This struct is used to dynamically construct colormaps by giving it a slice of colors.
/// It can then be used when being intantiated, but not with associated functions.
/// ```
/// use plotters::prelude::{BLACK,BLUE,WHITE,DerivedColorMap,ColorMap};
///
/// let derived_colormap = DerivedColorMap::new(
///     &[BLACK,
///     BLUE,
///     WHITE]
/// );
///
/// assert_eq!(derived_colormap.get_color(0.0), BLACK);
/// assert_eq!(derived_colormap.get_color(0.5), BLUE);
/// assert_eq!(derived_colormap.get_color(1.0), WHITE);
/// ```
pub struct DerivedColorMap<ColorType> {
    colors: Vec<ColorType>,
}

impl<ColorType: crate::style::Color + Clone> DerivedColorMap<ColorType> {
    /// This function lets the user define a new colormap by simply specifying colors in the correct order.
    /// For calculation of the color values, the colors will be spaced evenly apart.
    pub fn new(colors: &[ColorType]) -> Self {
        DerivedColorMap {
            colors: colors.to_vec(),
        }
    }
}

macro_rules! calculate_new_color_value(
    ($relative_difference:expr, $colors:expr, $index_upper:expr, $index_lower:expr, RGBColor) => {
        RGBColor(
            // These equations are a very complicated way of writing a simple linear extrapolation with lots of casting between numerical values
            // In principle every cast should be safe which is why we choose to unwrap
            //           (1.0  - r)                   *                                        color_value_1  +                    r *                                       color_value_2
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].0).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].0).unwrap()).round().to_u8().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].1).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].1).unwrap()).round().to_u8().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].2).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].2).unwrap()).round().to_u8().unwrap()
        )
    };
    ($relative_difference:expr, $colors:expr, $index_upper:expr, $index_lower:expr, RGBAColor) => {
        RGBAColor(
            // These equations are a very complicated way of writing a simple linear extrapolation with lots of casting between numerical values
            // In principle every cast should be safe which is why we choose to unwrap
            //           (1.0  - r)                   *                                        color_value_1  +                    r *                                       color_value_2
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].0).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].0).unwrap()).round().to_u8().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].1).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].1).unwrap()).round().to_u8().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_u8($colors[$index_upper].2).unwrap() + $relative_difference * FloatType::from_u8($colors[$index_lower].2).unwrap()).round().to_u8().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_f64($colors[$index_upper].3).unwrap() + $relative_difference * FloatType::from_f64($colors[$index_lower].3).unwrap()).to_f64().unwrap()
        )
    };
    ($relative_difference:expr, $colors:expr, $index_upper:expr, $index_lower:expr, HSLColor) => {
        HSLColor(
            // These equations are a very complicated way of writing a simple linear extrapolation with lots of casting between numerical values
            // In principle every cast should be safe which is why we choose to unwrap
            //           (1.0  - r)                   *                                         color_value_1  +                    r *                                        color_value_2
            ((FloatType::one() - $relative_difference) * FloatType::from_f64($colors[$index_upper].0).unwrap() + $relative_difference * FloatType::from_f64($colors[$index_lower].0).unwrap()).to_f64().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_f64($colors[$index_upper].1).unwrap() + $relative_difference * FloatType::from_f64($colors[$index_lower].1).unwrap()).to_f64().unwrap(),
            ((FloatType::one() - $relative_difference) * FloatType::from_f64($colors[$index_upper].2).unwrap() + $relative_difference * FloatType::from_f64($colors[$index_lower].2).unwrap()).to_f64().unwrap(),
        )
    };
);

fn calculate_relative_difference_index_lower_upper<
    FloatType: Float + FromPrimitive + ToPrimitive,
>(
    h: FloatType,
    min: FloatType,
    max: FloatType,
    n_colors: usize,
) -> (FloatType, usize, usize) {
    // Ensure that we do have a value in bounds
    let h = num_traits::clamp(h, min, max);
    // Next calculate a normalized value between 0.0 and 1.0
    let t = (h - min) / (max - min);
    let approximate_index =
        t * (FloatType::from_usize(n_colors).unwrap() - FloatType::one()).max(FloatType::zero());
    // Calculate which index are the two most nearest of the supplied value
    let index_lower = approximate_index.floor().to_usize().unwrap();
    let index_upper = approximate_index.ceil().to_usize().unwrap();
    // Calculate the relative difference, ie. is the actual value more towards the color of index_upper or index_lower?
    let relative_difference = approximate_index.ceil() - approximate_index;
    (relative_difference, index_lower, index_upper)
}

macro_rules! implement_color_scale_for_derived_color_map{
    ($($color_type:ident),+) => {
        $(
            impl<FloatType: Float + FromPrimitive + ToPrimitive> ColorMap<$color_type, FloatType> for DerivedColorMap<$color_type> {
                fn get_color_normalized(&self, h: FloatType, min: FloatType, max: FloatType) -> $color_type {
                    let (
                        relative_difference,
                        index_lower,
                        index_upper
                    ) = calculate_relative_difference_index_lower_upper(
                        h,
                        min,
                        max,
                        self.colors.len()
                    );
                    // Interpolate the final color linearly
                    calculate_new_color_value!(
                        relative_difference,
                        self.colors,
                        index_upper,
                        index_lower,
                        $color_type
                    )
                }
            }
        )+
    }
}

implement_color_scale_for_derived_color_map! {RGBAColor, RGBColor, HSLColor}

macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! define_colors_from_list_of_values_or_directly{
    ($color_type:ident, $(($($color_value:expr),+)),+) => {
        [$($color_type($($color_value),+)),+]
    };
    ($($color_complete:tt),+) => {
        [$($color_complete),+]
    };
}

macro_rules! implement_linear_interpolation_color_map {
    ($color_scale_name:ident, $color_type:ident) => {
        impl<FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive>
            ColorMap<$color_type, FloatType> for $color_scale_name
        {
            fn get_color_normalized(
                &self,
                h: FloatType,
                min: FloatType,
                max: FloatType,
            ) -> $color_type {
                let (
                    relative_difference,
                    index_lower,
                    index_upper
                ) = calculate_relative_difference_index_lower_upper(
                    h,
                    min,
                    max,
                    Self::COLORS.len()
                );
                // Interpolate the final color linearly
                calculate_new_color_value!(
                    relative_difference,
                    Self::COLORS,
                    index_upper,
                    index_lower,
                    $color_type
                )
            }
        }

        impl $color_scale_name {
            #[doc = "Get color value from `"]
            #[doc = stringify!($color_scale_name)]
            #[doc = "` by supplying a parameter 0.0 <= h <= 1.0"]
            pub fn get_color<FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive>(
                h: FloatType,
            ) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color(h)
            }

            #[doc = "Get color value from `"]
            #[doc = stringify!($color_scale_name)]
            #[doc = "` by supplying lower and upper bounds min, max and a parameter h where min <= h <= max"]
            pub fn get_color_normalized<
                FloatType: std::fmt::Debug + Float + FromPrimitive + ToPrimitive,
            >(
                h: FloatType,
                min: FloatType,
                max: FloatType,
            ) -> $color_type {
                let color_scale = $color_scale_name {};
                color_scale.get_color_normalized(h, min, max)
            }
        }
    };
}

#[macro_export]
/// Macro to create a new colormap with evenly spaced colors at compile-time.
macro_rules! define_linear_interpolation_color_map{
    ($color_scale_name:ident, $color_type:ident, $doc:expr, $(($($color_value:expr),+)),*) => {
        #[doc = $doc]
        pub struct $color_scale_name {}

        impl $color_scale_name {
            // const COLORS: [$color_type; $number_colors] = [$($color_type($($color_value),+)),+];
            // const COLORS: [$color_type; count!($(($($color_value:expr),+))*)] = [$($color_type($($color_value),+)),+];
            const COLORS: [$color_type; count!($(($($color_value:expr),+))*)] = define_colors_from_list_of_values_or_directly!{$color_type, $(($($color_value),+)),*};
        }

        implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    };
    ($color_scale_name:ident, $color_type:ident, $doc:expr, $($color_complete:tt),+) => {
        #[doc = $doc]
        pub struct $color_scale_name {}

        impl $color_scale_name {
            const COLORS: [$color_type; count!($($color_complete)*)] = define_colors_from_list_of_values_or_directly!{$($color_complete),+};
        }

        implement_linear_interpolation_color_map!{$color_scale_name, $color_type}
    }
}

define_linear_interpolation_color_map! {
    ViridisRGBA,
    RGBAColor,
    "A colormap optimized for visually impaired people (RGBA format).
    It is currently the default colormap also used by [matplotlib](https://matplotlib.org/stable/tutorials/colors/colormaps.html).
    Read more in this [paper](https://doi.org/10.1371/journal.pone.0199239)",
    ( 68,   1,  84, 1.0),
    ( 70,  50, 127, 1.0),
    ( 54,  92, 141, 1.0),
    ( 39, 127, 143, 1.0),
    ( 31, 162, 136, 1.0),
    ( 74, 194, 110, 1.0),
    (160, 219,  57, 1.0),
    (254, 232,  37, 1.0)
}

define_linear_interpolation_color_map! {
    ViridisRGB,
    RGBColor,
    "A colormap optimized for visually impaired people (RGB Format).
    It is currently the default colormap also used by [matplotlib](https://matplotlib.org/stable/tutorials/colors/colormaps.html).
    Read more in this [paper](https://doi.org/10.1371/journal.pone.0199239)",
    ( 68,   1,  84),
    ( 70,  50, 127),
    ( 54,  92, 141),
    ( 39, 127, 143),
    ( 31, 162, 136),
    ( 74, 194, 110),
    (160, 219,  57),
    (254, 232,  37)
}

define_linear_interpolation_color_map! {
    BlackWhite,
    RGBColor,
    "Simple chromatic colormap from black to white.",
    (  0,   0,   0),
    (255, 255,   255)
}

define_linear_interpolation_color_map! {
    MandelbrotHSL,
    HSLColor,
    "Colormap created to replace the one used in the mandelbrot example.",
    (0.0, 1.0, 0.5),
    (1.0, 1.0, 0.5)
}

define_linear_interpolation_color_map! {
    VulcanoHSL,
    HSLColor,
    "A vulcanic colormap that display red/orange and black colors",
    (2.0/3.0, 1.0, 0.7),
    (    0.0, 1.0, 0.7)
}

use super::full_palette::*;
define_linear_interpolation_color_map! {
    Bone,
    RGBColor,
    "Dark colormap going from black over blue to white.",
    BLACK,
    BLUE,
    WHITE
}

define_linear_interpolation_color_map! {
    Copper,
    RGBColor,
    "Friendly black to brown colormap.",
    BLACK,
    BROWN,
    ORANGE
}
