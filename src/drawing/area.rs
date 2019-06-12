/// The abstraction of a drawing area
use super::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::coord::{CoordTranslate, MeshLine, Ranged, RangedCoord, Shift};
use crate::element::{Drawable, PointCollection};
use crate::style::{Color, TextStyle};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::error::Error;
use std::iter::{once, repeat};
use std::ops::Range;
use std::rc::Rc;

/// The representation of the rectangle in backend canvas
#[derive(Clone, Debug)]
struct Rect {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

impl Rect {
    /// Split the rectangle into a few smaller rectnagles
    fn split<'a, BPI: IntoIterator<Item = &'a i32> + 'a>(
        &'a self,
        break_points: BPI,
        vertical: bool,
    ) -> impl Iterator<Item = Rect> + 'a {
        let (mut x0, mut y0) = (self.x0, self.y0);
        let (full_x, full_y) = (self.x1, self.y1);
        break_points
            .into_iter()
            .chain(once(if vertical { &self.y1 } else { &self.x1 }))
            .map(move |&p| {
                let x1 = if vertical { full_x } else { p };
                let y1 = if vertical { p } else { full_y };
                let ret = Rect { x0, y0, x1, y1 };

                if vertical {
                    y0 = y1
                } else {
                    x0 = x1;
                }

                ret
            })
    }

    /// Evently split the regtangle to a row * col mesh
    fn split_evenly<'a>(&'a self, (row, col): (usize, usize)) -> impl Iterator<Item = Rect> + 'a {
        fn compute_evenly_split(from: i32, to: i32, n: usize, idx: usize) -> i32 {
            let size = (to - from) as usize;
            from + idx as i32 * (size / n) as i32 + if size % n < idx { 1 } else { 0 }
        }
        (0..row)
            .map(move |x| repeat(x).zip(0..col))
            .flatten()
            .map(move |(ri, ci)| Self {
                y0: compute_evenly_split(self.y0, self.y1, row, ri),
                y1: compute_evenly_split(self.y0, self.y1, row, ri + 1),
                x0: compute_evenly_split(self.x0, self.x1, col, ci),
                x1: compute_evenly_split(self.x0, self.x1, col, ci + 1),
            })
    }

    fn split_grid(&self, x_breaks: &[i32], y_breaks: &[i32]) -> impl Iterator<Item = Rect> {
        let mut xs = vec![self.x0, self.x1];
        let mut ys = vec![self.y0, self.y1];
        xs.extend(x_breaks.iter().map(|v| v + self.x0));
        ys.extend(y_breaks.iter().map(|v| v + self.y0));

        xs.sort();
        ys.sort();

        let xsegs: Vec<_> = xs
            .iter()
            .zip(xs.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();
        let ysegs: Vec<_> = ys
            .iter()
            .zip(ys.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();

        ysegs
            .into_iter()
            .map(move |(y0, y1)| {
                xsegs
                    .clone()
                    .into_iter()
                    .map(move |(x0, x1)| Self { x0, y0, x1, y1 })
            })
            .flatten()
    }

    /// Make the coordinate in the range of the rectangle
    fn truncate(&self, p: (i32, i32)) -> (i32, i32) {
        (p.0.min(self.x1).max(self.x0), p.1.min(self.y1).max(self.y0))
    }
}

/// The abstraction of a region
pub struct DrawingArea<DB: DrawingBackend, CT: CoordTranslate> {
    backend: Rc<RefCell<DB>>,
    rect: Rect,
    coord: CT,
}

impl<DB: DrawingBackend, CT: CoordTranslate + Clone> Clone for DrawingArea<DB, CT> {
    fn clone(&self) -> Self {
        Self {
            backend: self.copy_backend_ref(),
            rect: self.rect.clone(),
            coord: self.coord.clone(),
        }
    }
}

/// The error description of any drawing area API
#[derive(Debug)]
pub enum DrawingAreaErrorKind<E: Error + Send + Sync> {
    /// The error is due to drawing backend failure
    BackendError(DrawingErrorKind<E>),
    /// We are not able to get the mutable reference of the backend,
    /// which indicates the drawing backend is current used by other
    /// drawing operation
    SharingError,
    /// The error caused by invalid layout
    LayoutError,
}

impl<E: Error + Send + Sync> std::fmt::Display for DrawingAreaErrorKind<E> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DrawingAreaErrorKind::BackendError(e) => write!(fmt, "backend error: {}", e),
            DrawingAreaErrorKind::SharingError => {
                write!(fmt, "Mulitple backend operation in progress")
            }
            DrawingAreaErrorKind::LayoutError => write!(fmt, "Bad layout"),
        }
    }
}

impl<E: Error + Send + Sync> Error for DrawingAreaErrorKind<E> {}

#[allow(type_alias_bounds)]
type DrawingAreaError<T: DrawingBackend> = DrawingAreaErrorKind<T::ErrorType>;

impl<DB: DrawingBackend> From<DB> for DrawingArea<DB, Shift> {
    fn from(backend: DB) -> Self {
        Self::with_rc_cell(Rc::new(RefCell::new(backend)))
    }
}

impl<'a, DB: DrawingBackend> From<&'a Rc<RefCell<DB>>> for DrawingArea<DB, Shift> {
    fn from(backend: &'a Rc<RefCell<DB>>) -> Self {
        Self::with_rc_cell(backend.clone())
    }
}

/// A type which can be converted into a root drawing area
pub trait IntoDrawingArea: DrawingBackend + Sized {
    /// Convert the type into a root drawing area
    fn into_drawing_area(self) -> DrawingArea<Self, Shift>;
}

impl<T: DrawingBackend> IntoDrawingArea for T {
    fn into_drawing_area(self) -> DrawingArea<T, Shift> {
        self.into()
    }
}

impl<DB: DrawingBackend, X: Ranged, Y: Ranged> DrawingArea<DB, RangedCoord<X, Y>> {
    /// Draw the mesh on a area
    pub fn draw_mesh<DrawFunc>(
        &self,
        mut draw_func: DrawFunc,
        y_count_max: usize,
        x_count_max: usize,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>>
    where
        DrawFunc: FnMut(&mut DB, MeshLine<X, Y>) -> Result<(), DrawingErrorKind<DB::ErrorType>>,
    {
        self.backend_ops(move |b| {
            self.coord
                .draw_mesh(y_count_max, x_count_max, |line| draw_func(b, line))
        })
    }

    /// Get the range of X of the guest coordinate for current drawing area
    pub fn get_x_range(&self) -> Range<X::ValueType> {
        self.coord.get_x_range()
    }

    /// Get the range of Y of the guest coordinate for current drawing area
    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        self.coord.get_y_range()
    }
}

impl<DB: DrawingBackend, CT: CoordTranslate> DrawingArea<DB, CT> {
    /// Get the left upper conner of this area in the drawing backend
    pub fn get_base_pixel(&self) -> BackendCoord {
        (self.rect.x0, self.rect.y0)
    }

    /// Strip the applied coordinate specification and returns a shift-based drawing area
    pub fn strip_coord_spec(&self) -> DrawingArea<DB, Shift> {
        DrawingArea {
            rect: self.rect.clone(),
            backend: self.copy_backend_ref(),
            coord: Shift((self.rect.x0, self.rect.y0)),
        }
    }

    /// Get the area dimension in pixel
    pub fn dim_in_pixel(&self) -> (u32, u32) {
        (
            (self.rect.x1 - self.rect.x0) as u32,
            (self.rect.y1 - self.rect.y0) as u32,
        )
    }

    /// Compute the relative size based on the drawing area's height
    pub fn relative_to_height(&self, p: f64) -> f64 {
        f64::from((self.rect.y1 - self.rect.y0).max(0)) * (p.min(1.0).max(0.0))
    }

    /// Compute the relative size based on the drawing area's width
    pub fn relative_to_width(&self, p: f64) -> f64 {
        f64::from((self.rect.y1 - self.rect.y0).max(0)) * (p.min(1.0).max(0.0))
    }

    /// Get the pixel range of this area
    pub fn get_pixel_range(&self) -> (Range<i32>, Range<i32>) {
        (self.rect.x0..self.rect.x1, self.rect.y0..self.rect.y1)
    }

    /// Copy the drawing contenxt
    fn copy_backend_ref(&self) -> Rc<RefCell<DB>> {
        self.backend.clone()
    }

    /// Perform operation on the drawing backend
    fn backend_ops<R, O: FnOnce(&mut DB) -> Result<R, DrawingErrorKind<DB::ErrorType>>>(
        &self,
        ops: O,
    ) -> Result<R, DrawingAreaError<DB>> {
        if let Ok(mut db) = self.backend.try_borrow_mut() {
            db.ensure_prepared()
                .map_err(DrawingAreaErrorKind::BackendError)?;
            ops(&mut db).map_err(DrawingAreaErrorKind::BackendError)
        } else {
            Err(DrawingAreaErrorKind::SharingError)
        }
    }

    /// Fill the entire drawing area with a color
    pub fn fill<ColorType: Color>(&self, color: &ColorType) -> Result<(), DrawingAreaError<DB>> {
        self.backend_ops(|backend| {
            backend.draw_rect(
                (self.rect.x0, self.rect.y0),
                (self.rect.x1, self.rect.y1),
                color,
                true,
            )
        })
    }

    /// Draw a single pixel
    pub fn draw_pixel<ColorType: Color>(
        &self,
        pos: CT::From,
        color: &ColorType,
    ) -> Result<(), DrawingAreaError<DB>> {
        let pos = self.coord.translate(&pos);
        self.backend_ops(|b| b.draw_pixel(pos, &color.to_rgba()))
    }

    /// Present all the pending changes to the backend
    pub fn present(&self) -> Result<(), DrawingAreaError<DB>> {
        self.backend_ops(|b| b.present())
    }

    /// Draw an high-level element
    pub fn draw<'a, E>(&self, element: &'a E) -> Result<(), DrawingAreaError<DB>>
    where
        &'a E: PointCollection<'a, CT::From>,
        E: Drawable<DB>,
    {
        let backend_coords = element.point_iter().into_iter().map(|p| {
            let b = p.borrow();
            self.rect.truncate(self.coord.translate(b))
        });
        self.backend_ops(move |b| element.draw(backend_coords, b))
    }

    /// Map coordinate to the backend coordinate
    pub fn map_coordinate(&self, coord: &CT::From) -> BackendCoord {
        self.coord.translate(coord)
    }
}

impl<DB: DrawingBackend> DrawingArea<DB, Shift> {
    fn with_rc_cell(backend: Rc<RefCell<DB>>) -> Self {
        let (x1, y1) = RefCell::borrow(backend.borrow()).get_size();
        Self {
            rect: Rect {
                x0: 0,
                y0: 0,
                x1: x1 as i32,
                y1: y1 as i32,
            },
            backend,
            coord: Shift((0, 0)),
        }
    }

    /// Shrink the region, note all the locaitions are in guest coordinate
    pub fn shrink(
        mut self,
        left_upper: (u32, u32),
        dimension: (u32, u32),
    ) -> DrawingArea<DB, Shift> {
        self.rect.x0 = self.rect.x1.min(self.rect.x0 + left_upper.0 as i32);
        self.rect.y0 = self.rect.y1.min(self.rect.y0 + left_upper.1 as i32);

        self.rect.x1 = self.rect.x0.max(self.rect.x0 + dimension.0 as i32);
        self.rect.y1 = self.rect.y0.max(self.rect.y0 + dimension.1 as i32);

        self.coord = Shift((self.rect.x0, self.rect.y0));

        self
    }

    /// Apply a new coord transformation object and returns a new drawing area
    pub fn apply_coord_spec<CT: CoordTranslate>(&self, coord_spec: CT) -> DrawingArea<DB, CT> {
        DrawingArea {
            rect: self.rect.clone(),
            backend: self.copy_backend_ref(),
            coord: coord_spec,
        }
    }

    /// Create a margin for the given drawing area and returns the new drawing area
    pub fn margin(&self, top: i32, bottom: i32, left: i32, right: i32) -> DrawingArea<DB, Shift> {
        DrawingArea {
            rect: Rect {
                x0: self.rect.x0 + left,
                y0: self.rect.y0 + top,
                x1: self.rect.x1 - right,
                y1: self.rect.y1 - bottom,
            },
            backend: self.copy_backend_ref(),
            coord: Shift((self.rect.x0 + left, self.rect.y0 + top)),
        }
    }

    /// Split the drawing area vertically
    pub fn split_vertically(&self, y: i32) -> (Self, Self) {
        let split_point = [y + self.rect.y0];
        let mut ret = self.rect.split(split_point.iter(), true).map(|rect| Self {
            rect: rect.clone(),
            backend: self.copy_backend_ref(),
            coord: Shift((rect.x0, rect.y0)),
        });

        (ret.next().unwrap(), ret.next().unwrap())
    }

    /// Split the drawing area horizentally
    pub fn split_horizentally(&self, x: i32) -> (Self, Self) {
        let split_point = [x + self.rect.x0];
        let mut ret = self.rect.split(split_point.iter(), false).map(|rect| Self {
            rect: rect.clone(),
            backend: self.copy_backend_ref(),
            coord: Shift((rect.x0, rect.y0)),
        });

        (ret.next().unwrap(), ret.next().unwrap())
    }

    /// Split the drawing area evenly
    pub fn split_evenly(&self, (row, col): (usize, usize)) -> Vec<Self> {
        self.rect
            .split_evenly((row, col))
            .map(|rect| Self {
                rect: rect.clone(),
                backend: self.copy_backend_ref(),
                coord: Shift((rect.x0, rect.y0)),
            })
            .collect()
    }

    /// Split the drawing area into a grid with specified breakpoints on both X axis and Y axis
    pub fn split_by_breakpoints<XS: AsRef<[i32]>, YS: AsRef<[i32]>>(
        &self,
        xs: XS,
        ys: YS,
    ) -> Vec<Self> {
        self.rect
            .split_grid(xs.as_ref(), ys.as_ref())
            .map(|rect| Self {
                rect: rect.clone(),
                backend: self.copy_backend_ref(),
                coord: Shift((rect.x0, rect.y0)),
            })
            .collect()
    }

    /// Draw a title of the drawing area and return the remaining drawing area
    pub fn titled<'a, S: Into<TextStyle<'a>>>(
        &self,
        text: &str,
        style: S,
    ) -> Result<Self, DrawingAreaError<DB>> {
        let style = style.into();

        let (text_w, text_h) = match style.font.box_size(text) {
            Ok(what) => what,
            Err(what) => {
                return Err(DrawingAreaErrorKind::BackendError(
                    DrawingErrorKind::FontError(what),
                ));
            }
        };
        let padding = if self.rect.x1 - self.rect.x0 > text_w as i32 {
            (self.rect.x1 - self.rect.x0 - text_w as i32) / 2
        } else {
            0
        };

        self.backend_ops(|b| {
            b.draw_text(
                text,
                &style.font,
                (self.rect.x0 + padding, self.rect.y0 + 5),
                &style.color,
            )
        })?;

        Ok(Self {
            rect: Rect {
                x0: self.rect.x0,
                y0: self.rect.y0 + 10 + text_h as i32,
                x1: self.rect.x1,
                y1: self.rect.y1,
            },
            backend: self.copy_backend_ref(),
            coord: Shift((self.rect.x0, self.rect.y0 + 10 + text_h as i32)),
        })
    }

    /// Draw text on the drawing area
    pub fn draw_text(
        &self,
        text: &str,
        style: &TextStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingAreaError<DB>> {
        self.backend_ops(|b| {
            b.draw_text(
                text,
                &style.font,
                (pos.0 + self.rect.x0, pos.1 + self.rect.y0),
                &style.color,
            )
        })
    }
}

impl<DB: DrawingBackend, CT: CoordTranslate> DrawingArea<DB, CT> {
    pub fn into_coord_spec(self) -> CT {
        self.coord
    }
}
