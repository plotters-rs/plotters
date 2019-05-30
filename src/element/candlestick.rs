/*!
  The candelstick element, which showing the high/low/open/close price
*/

use std::cmp::Ordering;

use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::element::{Drawable, PointCollection};
use crate::style::ShapeStyle;

/// The candelstick data point element
pub struct CandleStick<'a, X, Y: PartialOrd> {
    style: ShapeStyle<'a>,
    width: u32,
    points: [(X, Y); 4],
}

impl<'a, X: Clone, Y: PartialOrd> CandleStick<'a, X, Y> {
    /// Create a new candlestick element, which requires the Y coordinate can be compared
    #[allow(clippy::too_many_arguments)]
    pub fn new<GS: Into<ShapeStyle<'a>>, LS: Into<ShapeStyle<'a>>>(
        x: X,
        open: Y,
        high: Y,
        low: Y,
        close: Y,
        gain_style: GS,
        loss_style: LS,
        width: u32,
    ) -> Self {
        Self {
            style: match open.partial_cmp(&close) {
                Some(Ordering::Less) => gain_style.into(),
                _ => loss_style.into(),
            },
            width,
            points: [
                (x.clone(), open),
                (x.clone(), high),
                (x.clone(), low),
                (x.clone(), close),
            ],
        }
    }
}

impl<'b, 'a, X: 'a, Y: PartialOrd + 'a> PointCollection<'a, (X, Y)> for &'a CandleStick<'b, X, Y> {
    type Borrow = &'a (X, Y);
    type IntoIter = &'a [(X, Y)];
    fn point_iter(self) -> &'a [(X, Y)] {
        &self.points
    }
}

impl<'a, X: 'a, Y: 'a + PartialOrd, DB:DrawingBackend> Drawable<DB> for CandleStick<'a, X, Y> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let mut points: Vec<_> = points.take(4).collect();
        if points.len() == 4 {
            let fill = false;
            if points[0].1 > points[3].1 {
                points.swap(0, 3);
            }
            let (l, r) = (
                self.width as i32 / 2,
                self.width as i32 - self.width as i32 / 2,
            );

            backend.draw_line(points[0], points[1], &Box::new(self.style.color))?;
            backend.draw_line(points[2], points[3], &Box::new(self.style.color))?;

            points[0].0 -= l;
            points[3].0 += r;

            backend.draw_rect(points[0], points[3], &Box::new(self.style.color), fill)?;
        }
        Ok(())
    }
}
