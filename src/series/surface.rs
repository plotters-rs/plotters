use crate::element::Polygon;
use crate::style::{colors::BLUE, Color, ShapeStyle};
use std::marker::PhantomData;
pub trait Direction<X, Y, Z> {
    type Input1Type;
    type Input2Type;
    type OutputType;
    fn make_coord(
        free_vars: (Self::Input1Type, Self::Input2Type),
        result: Self::OutputType,
    ) -> (X, Y, Z);
}

macro_rules! define_panel_descriptor {
    ($name: ident, $var1: ident, $var2: ident, $out: ident, ($first: ident, $second:ident) -> $result: ident = $output: expr) => {
        pub struct $name;
        impl<X, Y, Z> Direction<X, Y, Z> for $name {
            type Input1Type = $var1;
            type Input2Type = $var2;
            type OutputType = $out;
            fn make_coord(
                ($first, $second): (Self::Input1Type, Self::Input2Type),
                $result: Self::OutputType,
            ) -> (X, Y, Z) {
                $output
            }
        }
    };
}

define_panel_descriptor!(XOY, X, Y, Z, (x, y) -> z = (x,y,z));
define_panel_descriptor!(XOZ, X, Z, Y, (x, z) -> y = (x,y,z));
define_panel_descriptor!(YOZ, Y, Z, X, (y, z) -> x = (x,y,z));

enum StyleConfig<'a, T> {
    Fixed(ShapeStyle),
    Function(&'a dyn Fn(&T) -> ShapeStyle),
}

impl<T> StyleConfig<'_, T> {
    fn get_style(&self, v: &T) -> ShapeStyle {
        match self {
            StyleConfig::Fixed(s) => s.clone(),
            StyleConfig::Function(f) => f(v),
        }
    }
}

/// The surface series.
///
/// Currently the surface is representing any surface in form
/// y = f(x,z)
///
/// TODO: make this more general
pub struct SurfaceSeries<'a, X, Y, Z, D, SurfaceFunc>
where
    D: Direction<X, Y, Z>,
    SurfaceFunc: Fn(D::Input1Type, D::Input2Type) -> D::OutputType,
{
    free_var_1: Vec<D::Input1Type>,
    free_var_2: Vec<D::Input2Type>,
    surface_f: SurfaceFunc,
    style: StyleConfig<'a, D::OutputType>,
    vidx_1: usize,
    vidx_2: usize,
    _phantom: PhantomData<(X, Y, Z, D)>,
}

impl<'a, X, Y, Z, D, SurfaceFunc> SurfaceSeries<'a, X, Y, Z, D, SurfaceFunc>
where
    D: Direction<X, Y, Z>,
    SurfaceFunc: Fn(D::Input1Type, D::Input2Type) -> D::OutputType,
{
    pub fn new<IterA: Iterator<Item = D::Input1Type>, IterB: Iterator<Item = D::Input2Type>>(
        first_iter: IterA,
        second_iter: IterB,
        func: SurfaceFunc,
    ) -> Self {
        Self {
            free_var_1: first_iter.collect(),
            free_var_2: second_iter.collect(),
            surface_f: func,
            style: StyleConfig::Fixed(BLUE.mix(0.4).filled()),
            vidx_1: 0,
            vidx_2: 0,
            _phantom: PhantomData,
        }
    }

    pub fn style_func<F: Fn(&D::OutputType) -> ShapeStyle>(mut self, f: &'a F) -> Self {
        self.style = StyleConfig::Function(f);
        self
    }

    pub fn style<S: Into<ShapeStyle>>(mut self, s: S) -> Self {
        self.style = StyleConfig::Fixed(s.into());
        self
    }
}

macro_rules! impl_constructor {
    ($dir: ty, $name: ident) => {
        impl<'a, X, Y, Z, SurfaceFunc> SurfaceSeries<'a, X, Y, Z, $dir, SurfaceFunc>
        where
            SurfaceFunc: Fn(
                <$dir as Direction<X, Y, Z>>::Input1Type,
                <$dir as Direction<X, Y, Z>>::Input2Type,
            ) -> <$dir as Direction<X, Y, Z>>::OutputType,
        {
            pub fn $name<IterA, IterB>(a: IterA, b: IterB, f: SurfaceFunc) -> Self
            where
                IterA: Iterator<Item = <$dir as Direction<X, Y, Z>>::Input1Type>,
                IterB: Iterator<Item = <$dir as Direction<X, Y, Z>>::Input2Type>,
            {
                Self::new(a, b, f)
            }
        }
    };
}

impl_constructor!(XOY, xoy);
impl_constructor!(XOZ, xoz);
impl_constructor!(YOZ, yoz);
impl<'a, X, Y, Z, D, SurfaceFunc> Iterator for SurfaceSeries<'a, X, Y, Z, D, SurfaceFunc>
where
    D: Direction<X, Y, Z>,
    D::Input1Type: Clone,
    D::Input2Type: Clone,
    SurfaceFunc: Fn(D::Input1Type, D::Input2Type) -> D::OutputType,
{
    type Item = Polygon<(X, Y, Z)>;
    fn next(&mut self) -> Option<Self::Item> {
        let (b0, b1) = if let (Some(b0), Some(b1)) = (
            self.free_var_2.get(self.vidx_2),
            self.free_var_2.get(self.vidx_2 + 1),
        ) {
            self.vidx_2 += 1;
            (b0, b1)
        } else {
            self.vidx_1 += 1;
            self.vidx_2 = 1;
            if let (Some(b0), Some(b1)) = (self.free_var_2.get(0), self.free_var_2.get(1)) {
                (b0, b1)
            } else {
                return None;
            }
        };

        match (
            self.free_var_1.get(self.vidx_1),
            self.free_var_1.get(self.vidx_1 + 1),
        ) {
            (Some(a0), Some(a1)) => {
                let value = (self.surface_f)(a0.clone(), b0.clone());
                let style = self.style.get_style(&value);
                let vert = vec![
                    D::make_coord((a0.clone(), b0.clone()), value),
                    D::make_coord(
                        (a0.clone(), b1.clone()),
                        (self.surface_f)(a0.clone(), b1.clone()),
                    ),
                    D::make_coord(
                        (a1.clone(), b1.clone()),
                        (self.surface_f)(a1.clone(), b1.clone()),
                    ),
                    D::make_coord(
                        (a1.clone(), b0.clone()),
                        (self.surface_f)(a1.clone(), b0.clone()),
                    ),
                ];
                return Some(Polygon::new(vert, style));
            }
            _ => {
                return None;
            }
        }
    }
}
