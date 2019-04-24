/// The abstraction of a drawing area
use super::backend::{DrawingBackend, DrawingErrorKind, BackendCoord};
use super::coord::{CoordTranslate, Shift, MeshLine, RangedCoord, Ranged};
use crate::element::{Drawable, PointCollection};
use crate::style::{Color, TextStyle};

use std::iter::{once,repeat};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt::Debug;
use std::borrow::Borrow;
use std::ops::Range;

/// The representation of the rectangle in backend canvas
#[derive(Clone, Debug)]
struct Rect {
    x0:i32,
    y0:i32,
    x1:i32,
    y1:i32,
}

impl Rect {
    /// Split the rectangle into a few smaller rectnagles
    fn split<'a, BPI:IntoIterator<Item=&'a i32> + 'a>(&'a self, break_points: BPI, vertical: bool) -> impl Iterator<Item = Rect> + 'a{
        let (mut x0, mut y0) = (self.x0, self.y0);
        let (full_x, full_y) = (self.x1, self.y1);
        return break_points.into_iter().chain(once(if vertical {&self.y1} else {&self.x1})).map(move |&p| {
            let x1 = if vertical { full_x } else { p };
            let y1 = if vertical { p }  else { full_y };
            let ret = Rect { x0, y0, x1, y1 };

            if vertical { 
                y0 = y1 
            } else {
                x0 = x1;
            }

            return ret;
        });
    }

    /// Evently split the regtangle to a row * col mesh
    fn split_evenly<'a>(&'a self, (row,col):(usize, usize)) -> impl Iterator<Item = Rect> + 'a {
        fn compute_evenly_split(from:i32, to:i32, n:usize, idx:usize) -> i32 {
            let size = (to - from) as usize;
            return from + idx as i32 * (size / n) as i32 + if size % n < idx {1} else { 0 }; 
        }
        return (0..row).into_iter().map(move |x| repeat(x).zip(0..col)).flatten().map(move |(ri,ci)| {
            return Self {
                y0: compute_evenly_split(self.y0, self.y1, row, ri),
                y1: compute_evenly_split(self.y0, self.y1, row, ri + 1),
                x0: compute_evenly_split(self.x0, self.x1, col, ci),
                x1: compute_evenly_split(self.x0, self.x1, col, ci + 1),
            };
        });
    }
}

/// The abstraction of a region
pub struct DrawingArea<DB:DrawingBackend, CT:CoordTranslate> {
    backend: Rc<RefCell<DB>>,
    rect : Rect,
    coord: CT,
}

impl <DB: DrawingBackend, CT:CoordTranslate + Clone> Clone for DrawingArea<DB, CT> {
    fn clone(&self) -> Self {
        return Self {
            backend: self.copy_backend_ref(),
            rect: self.rect.clone(),
            coord: self.coord.clone()
        };
    }
}

#[derive(Debug)]
pub enum DrawingAreaErrorKind<E:Debug> {
    BackendError(DrawingErrorKind<E>),
    SharingError,
    LayoutError,
}

#[allow(type_alias_bounds)]  
type DrawingAreaError<T:DrawingBackend> = DrawingAreaErrorKind<T::ErrorType>;

impl <DB:DrawingBackend> From<DB> for DrawingArea<DB, Shift> {
    fn from(backend:DB) -> Self {
        let (x1,y1) = backend.get_size();
        return Self {
            rect: Rect {x0:0,y0:0,x1: x1 as i32,y1: y1 as i32},
            backend: Rc::new(RefCell::new(backend)),
            coord: Shift((0,0))
        };
    }
}

impl <DB:DrawingBackend, X:Ranged, Y:Ranged> DrawingArea<DB, RangedCoord<X,Y>> {
    /// Draw the mesh on a area
    pub fn draw_mesh<DrawFunc>(&self, mut draw_func:DrawFunc, y_count_max: usize, x_count_max: usize) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> 
        where DrawFunc:FnMut(&mut DB, MeshLine<X,Y>)->Result<(), DrawingErrorKind<DB::ErrorType>>
    {
        return self.backend_ops(move |b| {
            return self.coord.draw_mesh(y_count_max, x_count_max, |line| {
                return draw_func(b, line);
            });
        });
    }

    pub fn get_x_range(&self) -> Range<X::ValueType> {
        return self.coord.get_x_range();
    }

    pub fn get_y_range(&self) -> Range<Y::ValueType> {
        return self.coord.get_y_range();
    }
}

impl <DB:DrawingBackend, CT:CoordTranslate> DrawingArea<DB, CT> {
    pub fn get_base_pixel(&self) -> BackendCoord {
        return (self.rect.x0, self.rect.y0);
    }
    /// Get the area dimension in pixel
    pub fn dim_in_pixel(&self) -> (u32, u32) {
        return ((self.rect.x1 - self.rect.x0) as u32,
                (self.rect.y1 - self.rect.y0) as u32);
    }

    /// Get the pixel range of this area
    pub fn get_pixel_range(&self) -> (Range<i32>, Range<i32>) {
        return (self.rect.x0..self.rect.x1, self.rect.y0..self.rect.y1);
    }

    /// Copy the drawing contenxt
    fn copy_backend_ref(&self) -> Rc<RefCell<DB>> {
        return self.backend.clone(); 
    }

    /// Perform operation on the drawing backend
    fn backend_ops<R, O:FnOnce(&mut DB) -> Result<R, DrawingErrorKind<DB::ErrorType>>>(&self, ops:O) -> Result<R, DrawingAreaError<DB>> {
        if let Ok(mut db) = self.backend.try_borrow_mut() {
            return ops(&mut db).map_err(|what| DrawingAreaErrorKind::BackendError(what));
        } else {
            return Err(DrawingAreaErrorKind::SharingError);
        }
    }

    /// Fill the entire drawing area with a color
    pub fn fill<ColorType:Color>(&self, color:&ColorType) -> Result<(), DrawingAreaError<DB>> {
        return self.backend_ops(|backend| {
            backend.draw_rect((self.rect.x0, self.rect.y0), (self.rect.x1, self.rect.y1), color, true)
        });
    }

    /// Close the backend
    pub fn close(&self) -> Result<(), DrawingAreaError<DB>> {
        return self.backend_ops(|b| {
            return b.close();
        });
    }

    /// Draw an high-level element
    pub fn draw<'a, E>(&self, element: &'a E) -> Result<(), DrawingAreaError<DB>> 
        where &'a E: PointCollection<'a, CT::From>,
              E: Drawable
    {
        let backend_coords = element.point_iter().into_iter().map(|p| {
            let b = p.borrow();
            return self.coord.translate(b);
        });
        return self.backend_ops(move |b| element.draw(backend_coords, b));
    }

}

impl <DB:DrawingBackend> DrawingArea<DB, Shift> {
    
    /// Apply the coord transformation object and returns a new drawing area
    pub fn apply_coord_spec<CT:CoordTranslate>(&self, coord_spec:CT) -> DrawingArea<DB, CT> {
        return DrawingArea {
            rect: self.rect.clone(),
            backend: self.copy_backend_ref(),
            coord: coord_spec,
        };
    }

    /// Split the drawing area vertically
    pub fn split_vertically(&self, y:i32) -> (Self, Self) {
        let split_point = [y + self.rect.y0];
        let mut ret = self.rect.split(split_point.iter(), true).map(|rect| {
            return Self {
                rect: rect.clone(),
                backend: self.copy_backend_ref(),
                coord: Shift((rect.x0, rect.y0))
            };
        });

        return (ret.next().unwrap(), ret.next().unwrap());
    }

    /// Split the drawing area horizentally
    pub fn split_horizentally(&self, x:i32) -> (Self, Self) {
        let split_point = [x + self.rect.x0];
        let mut ret = self.rect.split(split_point.iter(), false).map(|rect| {
            return Self {
                rect: rect.clone(),
                backend: self.copy_backend_ref(),
                coord: Shift((rect.x0, rect.y0))
            };
        });

        return (ret.next().unwrap(), ret.next().unwrap());
    }

    /// Split the drawing area evenly
    pub fn split_evenly(&self, (row, col):(usize, usize)) -> Vec<Self> {
        return self.rect.split_evenly((row, col)).map(|rect| {
            return Self {
                rect: rect.clone(),
                backend: self.copy_backend_ref(),
                coord: Shift((rect.x0, rect.y0))
            };
        }).collect();
    }

    pub fn titled(&self, text:&str, style:TextStyle) -> Result<Self, DrawingAreaError<DB>> {
        let (text_w, text_h) = match style.font.box_size(text) {
            Ok(what) => what,
            Err(what) => {
                return Err(DrawingAreaErrorKind::BackendError(DrawingErrorKind::FontError(what)));
            }
        };
        let padding = if self.rect.x1 - self.rect.x0 > text_w as i32 { (self.rect.x1 - self.rect.x0 - text_w as i32) / 2 } else { 0 };

        self.backend_ops(|b| {
            b.draw_text(text, style.font, (self.rect.x0 + padding, self.rect.y0 + 5), &Box::new(style.color))
        })?;

        return Ok(Self {
            rect: Rect { x0: self.rect.x0, y0: self.rect.y0 + 10 + text_h as i32, x1: self.rect.x1, y1: self.rect.y1 },
            backend: self.copy_backend_ref(),
            coord: Shift((self.rect.x0, self.rect.y0 + 10 + text_h as i32))
        });
    }
    
    /// Draw text on the drawing area
    pub fn draw_text(&self, text: &str, style:&TextStyle, pos:BackendCoord) -> Result<(), DrawingAreaError<DB>> {
        return self.backend_ops(|b| {
            b.draw_text(text, style.font, (pos.0 + self.rect.x0, pos.1 + self.rect.y0), &Box::new(style.color))
        });
    }
}
