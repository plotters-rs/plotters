use super::{BackendCoordAndZ, Drawable, PointCollection};
use crate::style::ShapeStyle;
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

pub struct Cubiod<X, Y, Z> {
    face_style: ShapeStyle,
    edge_style: ShapeStyle,
    vert: [(X, Y, Z); 8],
}

impl<X: Clone, Y: Clone, Z: Clone> Cubiod<X, Y, Z> {
    pub fn new<FS: Into<ShapeStyle>, ES: Into<ShapeStyle>>(
        [(x0, y0, z0), (x1, y1, z1)]: [(X, Y, Z); 2],
        face_style: FS,
        edge_style: ES,
    ) -> Self {
        Self {
            face_style: face_style.into(),
            edge_style: edge_style.into(),
            vert: [
                (x0.clone(), y0.clone(), z0.clone()),
                (x0.clone(), y0.clone(), z1.clone()),
                (x0.clone(), y1.clone(), z0.clone()),
                (x0.clone(), y1.clone(), z1.clone()),
                (x1.clone(), y0.clone(), z0.clone()),
                (x1.clone(), y0.clone(), z1.clone()),
                (x1.clone(), y1.clone(), z0.clone()),
                (x1.clone(), y1.clone(), z1.clone()),
            ],
        }
    }
}

impl<'a, X: 'a, Y: 'a, Z: 'a> PointCollection<'a, (X, Y, Z), BackendCoordAndZ>
    for &'a Cubiod<X, Y, Z>
{
    type Point = &'a (X, Y, Z);
    type IntoIter = &'a [(X, Y, Z)];
    fn point_iter(self) -> Self::IntoIter {
        &self.vert
    }
}

impl<X, Y, Z, DB: DrawingBackend> Drawable<DB, BackendCoordAndZ> for Cubiod<X, Y, Z> {
    fn draw<I: Iterator<Item = (BackendCoord, i32)>>(
        &self,
        points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let vert: Vec<_> = points.collect();
        let mut polygon = vec![];
        for mask in vec![1, 2, 4] {
            let mask_a = if mask == 4 { 1 } else { mask * 2 };
            let mask_b = if mask == 1 { 4 } else { mask / 2 };
            let a = 0;
            let b = a | mask_a;
            let c = a | mask_a | mask_b;
            let d = a | mask_b;
            polygon.push([vert[a], vert[b], vert[c], vert[d]]);
            polygon.push([
                vert[a | mask],
                vert[b | mask],
                vert[c | mask],
                vert[d | mask],
            ]);
        }
        polygon.sort_by_cached_key(|t| std::cmp::Reverse(t[0].1 + t[1].1 + t[2].1 + t[3].1));

        for p in polygon {
            backend.fill_polygon(p.iter().map(|(coord, _)| *coord), &self.face_style)?;
            backend.draw_path(
                p.iter()
                    .map(|(coord, _)| *coord)
                    .chain(std::iter::once(p[0].0)),
                &self.edge_style,
            )?;
        }

        Ok(())
    }
}
