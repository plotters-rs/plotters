use crate::element::{Circle, DynElement, IntoDynElement, PathElement};
use crate::style::ShapeStyle;
use plotters_backend::DrawingBackend;
use std::marker::PhantomData;

/// The line series object, which takes an iterator of points in guest coordinate system
/// and creates the element rendering the line plot
pub struct LineSeries<DB: DrawingBackend, Coord> {
    style: ShapeStyle,
    data: Vec<Coord>,
    point_idx: usize,
    point_size: u32,
    phantom: PhantomData<DB>,
}

impl<DB: DrawingBackend, Coord: Clone + 'static> Iterator for LineSeries<DB, Coord> {
    type Item = DynElement<'static, DB, Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.data.is_empty() {
            if self.point_size > 0 && self.point_idx < self.data.len() {
                let idx = self.point_idx;
                self.point_idx += 1;
                return Some(
                    Circle::new(self.data[idx].clone(), self.point_size, self.style.clone())
                        .into_dyn(),
                );
            }
            let mut data = vec![];
            std::mem::swap(&mut self.data, &mut data);
            Some(PathElement::new(data, self.style.clone()).into_dyn())
        } else {
            None
        }
    }
}

impl<DB: DrawingBackend, Coord> LineSeries<DB, Coord> {
    pub fn new<I: IntoIterator<Item = Coord>, S: Into<ShapeStyle>>(iter: I, style: S) -> Self {
        Self {
            style: style.into(),
            data: iter.into_iter().collect(),
            point_size: 0,
            point_idx: 0,
            phantom: PhantomData,
        }
    }

    pub fn point_size(mut self, size: u32) -> Self {
        self.point_size = size;
        self
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_line_series() {
        let drawing_area = create_mocked_drawing_area(200, 200, |m| {
            m.check_draw_path(|c, s, path| {
                assert_eq!(c, RED.to_rgba());
                assert_eq!(s, 3);
                for i in 0..100 {
                    assert_eq!(path[i], (i as i32 * 2, 200 - i as i32 * 2 - 1));
                }
            });

            m.drop_check(|b| {
                assert_eq!(b.num_draw_path_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        });

        let mut chart = ChartBuilder::on(&drawing_area)
            .build_cartesian_2d(0..100, 0..100)
            .expect("Build chart error");

        chart
            .draw_series(LineSeries::new(
                (0..100).map(|x| (x, x)),
                Into::<ShapeStyle>::into(&RED).stroke_width(3),
            ))
            .expect("Drawing Error");
    }
}
