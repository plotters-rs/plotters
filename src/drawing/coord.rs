/// The abstraction of the coordinate system
use super::backend::{BackendCoord, DrawingErrorKind, DrawingBackend};
use crate::style::ShapeStyle;

use std::ops::Range;


/// The trait that translates some customized object to the backend coordinate
pub trait CoordTranslate {
    type From;
    fn translate(&self, from: &Self::From) -> BackendCoord;
}

/// The coordinate translation that only impose shift 
#[derive(Debug,Clone)]
pub struct Shift(pub BackendCoord);

impl CoordTranslate for Shift {
    type From = BackendCoord;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        return (from.0 + (self.0).0, from.1 + (self.0).1);
    }
}

/// We can compose an abitray transformation with a shift
pub struct ShiftAndTrans<T:CoordTranslate>(Shift, T);

impl <T:CoordTranslate> CoordTranslate for ShiftAndTrans<T> {
    type From = T::From;
    fn translate(&self, from: &Self::From) -> BackendCoord {
        let temp = self.1.translate(from);
        return self.0.translate(&temp);
    }
}

/// The trait that indicates we have a ordered and ranged value
/// Which is used to describe the axis
pub trait Ranged {
    /// The type of this value
    type ValueType;

    /// This function maps the value to i32, which is the drawing coordinate
    fn map(&self, value: &Self::ValueType, limit: (i32,i32)) -> i32;

    /// This function gives the key points that we can draw a grid based on this
    fn key_points(&self, max_points:usize) -> Vec<Self::ValueType>;

    /// Get the range of this value
    fn range(&self) -> Range<Self::ValueType>;
}

/// The coordinate description 
pub struct RangedCoord<X:Ranged, Y:Ranged>{
    logic_x:X,
    logic_y:Y, 
    back_x:(i32,i32), 
    back_y:(i32,i32)
}

/// Represent a line in the mesh
pub enum MeshLine <'a, X:Ranged, Y:Ranged>{
    XMesh(BackendCoord, BackendCoord, &'a X::ValueType),
    YMesh(BackendCoord, BackendCoord, &'a Y::ValueType)
}

impl <'a, X:Ranged, Y:Ranged> MeshLine<'a,X,Y> {
    pub fn draw<DB:DrawingBackend>(&self, backend:&mut DB, style: &ShapeStyle) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let (&left, &right) = match self {
            MeshLine::XMesh(a,b, _) => (a,b),
            MeshLine::YMesh(a,b, _) => (a,b),
        };
        return backend.draw_line(left, right, &Box::new(style.color));
    }
}

impl <X:Ranged, Y:Ranged> RangedCoord<X,Y> {
    pub fn new<IntoX: Into<X>, IntoY: Into<Y>>(logic_x:IntoX, logic_y:IntoY, actual:(Range<i32>, Range<i32>)) -> Self {
        return Self {
            logic_x: logic_x.into(),
            logic_y: logic_y.into(),
            back_x : (actual.0.start, actual.0.end),
            back_y : (actual.1.start, actual.1.end),
        };
    }

    pub fn draw_mesh<E, DrawMesh: FnMut(MeshLine<X,Y>) -> Result<(),E>>(&self, h_limit:usize, v_limit:usize, mut draw_mesh:DrawMesh) -> Result<(), E> {
        let (xkp, ykp) = (self.logic_x.key_points(v_limit), self.logic_y.key_points(h_limit));

        for logic_x in xkp {
            let x = self.logic_x.map(&logic_x, self.back_x);
            draw_mesh(MeshLine::XMesh((x, self.back_y.0), (x, self.back_y.1), &logic_x))?;
        }

        for logic_y in ykp {
            let y = self.logic_y.map(&logic_y, self.back_y);
            draw_mesh(MeshLine::YMesh((self.back_x.0, y), (self.back_x.1, y), &logic_y))?;
        }

        return Ok(());
    }

    pub fn get_x_range(&self) -> Range<X::ValueType> {
        return self.logic_x.range();
    }

    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        return self.logic_y.range();
    }
}

impl <X:Ranged, Y:Ranged> CoordTranslate for RangedCoord<X,Y> {
    type From = (X::ValueType, Y::ValueType);

    fn translate(&self, from: &Self::From) -> BackendCoord {
        return (self.logic_x.map(&from.0, self.back_x),
                self.logic_y.map(&from.1, self.back_y));
    }
}

macro_rules! make_numeric_coord {
    ($type:ty, $name:ident, $key_points:ident) => {
        pub struct $name($type, $type);
        impl From<Range<$type>> for $name {
            fn from(range:Range<$type>) -> Self {
                return Self(range.start, range.end);
            }
        }
        impl Ranged for $name {
            type ValueType = $type;
            fn map(&self, v:&$type, limit:(i32,i32)) -> i32 {
                let logic_length = (*v - self.0) as f64 / (self.1 - self.0) as f64;
                let actual_length = limit.1 - limit.0;

                if actual_length == 0 { return limit.1; }

                return limit.0 + (actual_length as f64 * logic_length) as i32;
            }
            fn key_points(&self, max_points: usize) -> Vec<$type> {
                $key_points((self.0, self.1), max_points)
            }
            fn range(&self) -> Range<$type> {
                return self.0..self.1;
            }
        }
    }
}

macro_rules! gen_key_points_comp {
    (float, $name:ident, $type:ty) => {
        fn $name(range:($type, $type), max_points: usize) -> Vec<$type> {
            let mut scale = (10 as $type).powf((range.1 - range.0).log(10.0).floor());
            fn rem_euclid(a:$type, b:$type) -> $type {
                if b > 0.0 {
                    a - (a/b).floor() * b
                } else {
                    a - (a/b).ceil() * b
                }
            }
            let mut left = range.0 + rem_euclid(range.0, -scale);
            let mut right = range.1 - rem_euclid(range.1, scale);
            'outer: loop {
                let old_scale = scale;
                for nxt in [2.0, 5.0, 10.0].iter() {
                    let new_left = range.0 + rem_euclid(range.0, -scale/nxt);
                    let new_right = range.1 - rem_euclid(range.1, scale/nxt);
                    
                    let npoints = 1 + ((new_right - new_left) / old_scale  * nxt) as usize;
                    
                    if npoints > max_points {
                        break 'outer;
                    }

                    scale = old_scale / nxt;
                    left = new_left;
                    right = new_right;
                }
            }

            let mut ret = vec![];
            while left <= right {
                ret.push(left as $type);
                left += scale;
            }

            return ret;
        }
    };
    (integer, $name:ident, $type:ty) => {
        fn $name(range:($type, $type), max_points: usize) -> Vec<$type> {
            let mut scale:$type = 1;
            'outter: while (range.1 - range.0 + scale - 1) as usize / (scale as usize) > max_points {
                let next_scale = scale * 10;
                for new_scale in [scale*2, scale*5, scale*10].iter() {
                    scale = *new_scale;
                    if (range.1 - range.0 + *new_scale - 1) as usize / (*new_scale as usize) < max_points {
                        break 'outter;
                    }
                
                }
                scale = next_scale;
            }

            let (mut left, right) = (range.0 + (scale - range.0) % scale, range.1 - range.1 % scale);
            
            let mut ret = vec![];
            while left <= right {
                ret.push(left as $type);
                left += scale;
            }

            return ret;
        }
    }
}

gen_key_points_comp!(float, compute_f32_key_points, f32);
gen_key_points_comp!(float, compute_f64_key_points, f64);
gen_key_points_comp!(integer, compute_i32_key_points, i32);
gen_key_points_comp!(integer, compute_u32_key_points, u32);
gen_key_points_comp!(integer, compute_i64_key_points, i64);
gen_key_points_comp!(integer, compute_u64_key_points, u64);

make_numeric_coord!(f32, RangedCoordf32, compute_f32_key_points);
make_numeric_coord!(f64, RangedCoordf64, compute_f64_key_points);
make_numeric_coord!(u32, RangedCoordu32, compute_u32_key_points);
make_numeric_coord!(i32, RangedCoordi32, compute_i32_key_points);
make_numeric_coord!(u64, RangedCoordu64, compute_u64_key_points);
make_numeric_coord!(i64, RangedCoordi64, compute_i64_key_points);



#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_key_points() {

        let kp = compute_i32_key_points((0,999), 28);

        assert!(kp.len() > 0);
        assert!(kp.len() <= 28);
    }

    #[test] 
    fn test_linear_coord_map() {
        let coord: RangedCoordu32 = (0..20).into();
        assert_eq!(coord.key_points(11).len(), 11);
        assert_eq!(coord.key_points(11)[0], 0);
        assert_eq!(coord.key_points(11)[10], 20);
        assert_eq!(coord.map(&5, (0,100)), 25);
        
        let coord: RangedCoordf32 = (0f32..20f32).into();
        assert_eq!(coord.map(&5.0, (0,100)), 25);
    }

    #[test]
    fn test_linear_coord_system() {
        let coord = RangedCoord::<RangedCoordu32, RangedCoordu32>::new(0..10, 0..10, (0..1024, 0..768));
    }
}
