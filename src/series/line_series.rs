use crate::element::Path;
use crate::style::ShapeStyle;

/// The line series object, which takes an iterator of points in guest coordinate system
/// and creates the element rendering the line plot
pub struct LineSeries<'a, Coord, I: IntoIterator<Item = Coord>> {
    style: ShapeStyle<'a>,
    data_iter: Option<I::IntoIter>,
}

impl<'b, Coord, I: IntoIterator<Item = Coord>> Iterator for LineSeries<'b, Coord, I> {
    type Item = Path<'b, Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            return Some(Path::new(
                data_iter.unwrap().collect::<Vec<_>>(),
                self.style.clone(),
            ));
        } else {
            return None;
        }
    }
}

impl<'a, Coord, I: IntoIterator<Item = Coord>> LineSeries<'a, Coord, I> {
    pub fn new<S: Into<ShapeStyle<'a>>>(iter: I, style: S) -> Self {
        return Self {
            style: style.into(),
            data_iter: Some(iter.into_iter()),
        };
    }
}

