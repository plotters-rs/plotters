use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};

use crate::drawing::BitMapBackend;
use std::borrow::{Borrow, Cow};

/// The element that contains a bitmap on it
pub struct BitMapElement<'a, Coord> {
    image: Cow<'a, Vec<u8>>,
    size: (u32, u32),
    pos: Coord,
}

impl<'a, Coord> BitMapElement<'a, Coord> {
    /// Create a new empty bitmap element. This can be use as
    /// the draw and blit pattern.
    ///
    /// - `pos`: The left upper coordinate for the element
    /// - `size`: The size of the bitmap
    pub fn new(pos: Coord, size: (u32, u32)) -> Self {
        Self {
            image: Cow::Owned(vec![0; (size.0 * size.1 * 3) as usize]),
            size,
            pos,
        }
    }

    /// Copy the existing bitmap element to another location
    ///
    /// - `pos`: The new location to copy
    pub fn copy_to<Coord2>(&self, pos: Coord2) -> BitMapElement<Coord2> {
        BitMapElement {
            image: Cow::Borrowed(self.image.borrow()),
            size: self.size,
            pos,
        }
    }

    /// Move the existing bitmap element to a new position
    ///
    /// - `pos`: The new position
    pub fn move_to(&mut self, pos: Coord) {
        self.pos = pos;
    }

    /// Make the bitmap element as a bitmap backend, so that we can use
    /// plotters drawing functionality on the bitmap element
    #[cfg(feature = "bitmap")]
    pub fn as_bitmap_backend(&mut self) -> BitMapBackend {
        BitMapBackend::with_buffer(self.image.to_mut(), self.size)
    }
}

impl<'a, Coord> From<(Coord, DynamicImage)> for BitMapElement<'a, Coord> {
    fn from((pos, image): (Coord, DynamicImage)) -> Self {
        let (w, h) = image.dimensions();
        let rgb_image = image.to_rgb().into_raw();
        Self {
            pos,
            image: Cow::Owned(rgb_image),
            size: (w, h),
        }
    }
}

impl<'a, 'b, Coord> PointCollection<'a, Coord> for &'a BitMapElement<'b, Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.pos)
    }
}

impl<'a, Coord, DB: DrawingBackend> Drawable<DB> for BitMapElement<'a, Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.blit_bitmap(
                (x, y),
                &(ImageBuffer::<Rgb<u8>, &[u8]>::from_raw(
                    self.size.0,
                    self.size.1,
                    &self.image[..],
                )
                .unwrap()),
            );
        }
        Ok(())
    }
}
