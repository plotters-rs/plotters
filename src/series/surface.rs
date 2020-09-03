use crate::element::Polygon;
use crate::style::ShapeStyle;
/// The surface series.
///
/// Currently the surface is representing any surface in form
/// y = f(x,z)
///
/// TODO: make this more general
pub struct SurfaceSeries<X, Y, Z> {
    x_data: Vec<X>,
    y_data: Vec<Y>,
    z_data: Vec<Z>,
    style: ShapeStyle,
    size: usize,
    state: usize,
}

impl<X, Y, Z> SurfaceSeries<X, Y, Z> {
    pub fn new<XS, ZS, YF, S>(xs: XS, zs: ZS, y_func: YF, style: S) -> Self
    where
        YF: Fn(&X, &Z) -> Y,
        XS: Iterator<Item = X>,
        ZS: Iterator<Item = Z>,
        S: Into<ShapeStyle>,
    {
        let x_data: Vec<_> = xs.collect();
        let z_data: Vec<_> = zs.collect();
        let y_data: Vec<_> = x_data
            .iter()
            .map(|x| z_data.iter().map(move |z| (x, z)))
            .flatten()
            .map(|(x, z)| y_func(x, z))
            .collect();
        let size = (x_data.len().max(1) - 1) * (z_data.len().max(1) - 1);
        Self {
            x_data,
            y_data,
            z_data,
            style: style.into(),
            size,
            state: 0,
        }
    }

    fn point_at(&self, x: usize, z: usize) -> (X, Y, Z)
    where
        X: Clone,
        Y: Clone,
        Z: Clone,
    {
        (
            self.x_data[x].clone(),
            self.y_data[x * self.z_data.len() + z].clone(),
            self.z_data[z].clone(),
        )
    }
}

impl<X: Clone, Y: Clone, Z: Clone> Iterator for SurfaceSeries<X, Y, Z> {
    type Item = Polygon<(X, Y, Z)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size <= self.state {
            return None;
        }

        let x = self.state / (self.z_data.len() - 1);
        let z = self.state % (self.z_data.len() - 1);

        self.state += 1;

        Some(Polygon::new(
            vec![
                self.point_at(x, z),
                self.point_at(x, z + 1),
                self.point_at(x + 1, z + 1),
                self.point_at(x + 1, z),
            ],
            self.style.clone(),
        ))
    }
}
