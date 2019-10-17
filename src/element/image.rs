use image::{ImageBuffer, Rgb};

use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use crate::drawing::BitMapBackend;

pub struct BitMapElement<Coord> {
    image: Vec<u8>,
    size: (u32, u32),
    pos: Coord,
}

impl<Coord> BitMapElement<Coord> {
    pub fn new(pos: Coord, size: (u32, u32)) -> Self {
        Self {
            image: vec![0; (size.0 * size.1 * 3) as usize],
            size,
            pos,
        }
    }

    #[cfg(feature = "bitmap")]
    pub fn as_bitmap_backend<'a>(&'a mut self) -> BitMapBackend<'a> {
        BitMapBackend::with_buffer(&mut self.image, self.size)
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a BitMapElement<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.pos)
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for BitMapElement<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.blit_bitmap(
                (x, y),
                &(ImageBuffer::<Rgb<u8>, &[u8]>::from_raw(self.size.0, self.size.1, &self.image)
                    .unwrap()),
            );
        }
        Ok(())
    }
}
