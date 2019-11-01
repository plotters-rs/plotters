use crate::element::PathElement;
use crate::style::ShapeStyle;

/// The line series object, which takes an iterator of points in guest coordinate system
/// and creates the element rendering the line plot
pub struct LineSeries<Coord, I: IntoIterator<Item = Coord>> {
    style: ShapeStyle,
    data_iter: Option<I::IntoIter>,
}

impl<Coord, I: IntoIterator<Item = Coord>> Iterator for LineSeries<Coord, I> {
    type Item = PathElement<Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            Some(PathElement::new(
                data_iter.unwrap().collect::<Vec<_>>(),
                self.style.clone(),
            ))
        } else {
            None
        }
    }
}

impl<Coord, I: IntoIterator<Item = Coord>> LineSeries<Coord, I> {
    pub fn new<S: Into<ShapeStyle>>(iter: I, style: S) -> Self {
        Self {
            style: style.into(),
            data_iter: Some(iter.into_iter()),
        }
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
                    assert_eq!(path[i], (i as i32 * 2, 200 - i as i32 * 2));
                }
            });

            m.drop_check(|b| {
                assert_eq!(b.num_draw_path_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        });

        let mut chart = ChartBuilder::on(&drawing_area)
            .build_ranged(0..100, 0..100)
            .expect("Build chart error");

        chart
            .draw_series(LineSeries::new(
                (0..100).map(|x| (x, x)),
                Into::<ShapeStyle>::into(&RED).stroke_width(3),
            ))
            .expect("Drawing Error");
    }
}
