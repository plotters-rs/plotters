use crate::style::ShapeStyle;
use crate::element::Path;

pub struct LineSeries<'a, Coord,I:IntoIterator<Item=Coord>> {
    style: &'a ShapeStyle<'a>,
    data_iter:Option<I::IntoIter>,
}

impl <'b, 'a:'b, Coord, I:IntoIterator<Item=Coord>> Iterator for LineSeries<'b, Coord, I> {
    type Item = Path<'b, Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.data_iter.is_some() {
            let mut data_iter = None;
            std::mem::swap(&mut self.data_iter, &mut data_iter);
            return Some(Path::new(data_iter.unwrap().collect::<Vec<_>>(), self.style.clone()));
        } else {
            return None;
        }
    }
}

impl <'a, Coord, I:IntoIterator<Item=Coord>> LineSeries<'a, Coord, I> {
    pub fn new<'b:'a>(iter:I, style: &'a ShapeStyle<'b>) -> Self {
        return Self {
            style,
            data_iter: Some(iter.into_iter())
        };
    }
}
