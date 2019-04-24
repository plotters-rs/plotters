use crate::font::FontDesc;
use crate::color::Color;
use crate::drawing::DrawingBackend;
use crate::region::{DrawingRegion,Shift, Splitable, CoordTrans};
use crate::element::{Text, GridLineIter, Path, Grid, GridDirection};

use std::ops::Range;
use std::marker::PhantomData;

pub trait RangedCoord{
    type CoordType : Sized;
    fn map(&self, range: (u32, u32), v:&Self::CoordType) -> u32;
    fn key_points(&self, max_points: usize) -> Vec<Self::CoordType>;
}

pub struct RangedCoordTranslation<X:RangedCoord, Y:RangedCoord>{
    x_axis:X, 
    y_axis:Y, 
    x_range: (u32, u32), 
    y_range: (u32, u32),
}

impl <X:RangedCoord, Y:RangedCoord> CoordTrans for RangedCoordTranslation<X,Y> {
    type CoordType = (X::CoordType, Y::CoordType);
    fn translate(&self, coord: &Self::CoordType) -> (u32, u32) {
        return (self.x_axis.map(self.x_range, &coord.0), self.y_axis.map(self.y_range, &coord.1));
    }
}

pub trait NumericCoordKind{}
pub struct Linear;
impl NumericCoordKind for Linear{}
pub struct Logarithm;
impl NumericCoordKind for Logarithm {}

macro_rules! gen_key_points_comp {
    (float, $name:ident, $type:ty) => {
        fn $name(range:($type, $type), max_points: usize) -> Vec<$type> {
            let mut scale = (range.1 - range.0).log(10.0).floor().powf(10.0);
            fn rem_euclid(a:$type, b:$type) -> $type {
                a - (a/b).floor() * b
            }
            let (mut left, right) = (range.0 + rem_euclid(range.0, -scale), range.1 - rem_euclid(range.1, scale));
            let mut npoints = ((right - left) / scale) as usize + 1;
            while npoints * 10 < max_points {
                npoints *= 10;
                scale /= 10.0;
            }

            let mut ret = vec![];
            while left < right {
                ret.push(left as $type);
                left += scale;
            }

            return ret;
        }
    };
    (integer, $name:ident, $type:ty) => {
        fn $name(range:($type, $type), max_points: usize) -> Vec<$type> {
            let mut scale:$type = 1;
            while (range.1 - range.0 + scale * 10 - 1) / (scale * 10) < max_points {
                scale *= 10;
            }

            let (left, right) = (range.0 + scale - range.0 % scale, range.1 - range.1 % scale);
            
            let mut ret = vec![];
            while left < right {
                ret.push(left as $type);
                left += scale;
            }

            return ret;
        }
    }
}

macro_rules! make_numeric_coord {
    ($type:ty, $name:ident, $key_points:ident) => {
        pub struct $name<K:NumericCoordKind>($type, $type, std::marker::PhantomData<K>);
        impl <K:NumericCoordKind> From<Range<$type>> for $name<K> {
            fn from(range:Range<$type>) -> Self {
                return Self(range.start, range.end, std::marker::PhantomData);
            }
        }
        impl RangedCoord for $name<Linear> {
            type CoordType = $type;
            fn map(&self, range: (u32, u32), v:&$type) -> u32 {
                let logic_length = (*v - self.0) as f64 / (self.1 - self.0) as f64;
                let actual_length = range.1 as f64 - range.0 as f64;

                if actual_length == 0.0 { return range.1; }

                return range.0 + (actual_length * logic_length) as u32;
            }
            fn key_points(&self, max_points: usize) -> Vec<$type> {
                $key_points((self.0, self.1), max_points)
            }
        }
    }
}

gen_key_points_comp!(float, compute_f32_key_points, f32);
make_numeric_coord!(f32, RangedCoordF32, compute_f32_key_points);

gen_key_points_comp!(float, compute_f64_key_points, f64);
make_numeric_coord!(f64, RangedCoordF64, compute_f64_key_points);

pub struct Plot<DC:DrawingBackend, XCoord:RangedCoord, YCoord:RangedCoord> {
    drawing_area: DrawingRegion<DC, RangedCoordTranslation<XCoord, YCoord>>,
    x_keys: Vec<XCoord::CoordType>,
    y_keys: Vec<YCoord::CoordType>,
    x_label_area: Option<DrawingRegion<DC, Shift>>,
    y_label_area: Option<DrawingRegion<DC, Shift>>,
}

impl <DC:DrawingBackend, XCoord: RangedCoord, YCoord:RangedCoord> Plot<DC, XCoord, YCoord> {
    pub fn new(mut root_region: DrawingRegion<DC, Shift>, x_range: XCoord, y_range: YCoord, x_label_size: u32, y_label_size:u32) -> Option<Self> {
        let mut x_label_area = None;
        let mut y_label_area = None;
        if y_label_size > 0 {
            let (left, right) = root_region.split_horizentally(root_region.size_in_pixels().1 - y_label_size)?;
            y_label_area = Some(left);
            root_region = right;
        }

        if x_label_size > 0 {
            let (upper, lower) = root_region.split_horizentally(x_label_size)?;
            x_label_area = Some(lower);
            root_region = upper;
        }

        let x_keys = x_range. key_points(10);
        let y_keys = y_range.key_points(10);
        let dim = root_region.size_in_pixels();
        let transtion = RangedCoordTranslation {
          x_axis : x_range,
          y_axis:  y_range,
          x_range: (0, dim.0),
          y_range: (0, dim.1),
        };

        let ret = Self {
            drawing_area: DrawingRegion::new(&root_region, transtion),
            x_keys,
            y_keys,
            x_label_area,
            y_label_area,
        };

        return Some(ret);
    }

}

