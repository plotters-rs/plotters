use crate::element::Path;
use crate::style::ShapeStyle;

/// The line series object, which takes an iterator of points in guest coordinate system
/// and creates the element rendering the line plot
pub struct LineSeries<Coord, I: IntoIterator<Item = Coord>> {
    style: ShapeStyle,
    data_iter: Option<I::IntoIter>,
}

impl<Coord, I: IntoIterator<Item = Coord>> Iterator for LineSeries<Coord, I> {
    type Item = Path<Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            Some(Path::new(
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
