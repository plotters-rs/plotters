/// The font management utilities
/// This file retains a font cache and makes font object.
///
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::From;
use std::i32;
use std::sync::Mutex;

use font_loader::system_fonts;
use lazy_static::lazy_static;
use rusttype::{point, Error, Font, Scale};

#[derive(Debug)]
pub enum FontError {
    LockError,
    NoSuchFont,
    FontLoadError(Error),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return match self {
            FontError::LockError => write!(fmt, "Could not lock mutex"),
            FontError::NoSuchFont => write!(fmt, "No such font"),
            FontError::FontLoadError(e) => write!(fmt, "Font loading error: {}", e),
        };
    }
}

impl std::error::Error for FontError {}

/// The type we used to represent a result of any font operations
pub type FontResult<T> = Result<T, FontError>;

lazy_static! {
    static ref FONT_DATA_CACHE: Mutex<HashMap<String, Vec<u8>>> = { Mutex::new(HashMap::new()) };
}

fn load_font_data(face: &str) -> FontResult<&'static [u8]> {
    match FONT_DATA_CACHE.lock().map(|mut cache| {
        if !cache.contains_key(face) {
            let query = system_fonts::FontPropertyBuilder::new()
                .family(face)
                .build();
            if let Some((data, _)) = system_fonts::get(&query) {
                cache.insert(face.to_string(), data);
            } else {
                return Err(FontError::NoSuchFont);
            }
        }
        return Ok(unsafe {
            std::mem::transmute::<_, &'static [u8]>(&cache.get(face).unwrap()[..])
        });
    }) {
        Ok(what) => what,
        Err(_) => Err(FontError::LockError),
    }
}

fn get_system_font(face: &str) -> FontResult<Font<'static>> {
    return match load_font_data(face) {
        Ok(bytes) => Font::from_bytes(bytes).map_err(|x| FontError::FontLoadError(x)),
        Err(what) => Err(what),
    };
}

/// Describes a font
pub struct FontDesc<'a> {
    size: f64,
    name: &'a str,
    font: RefCell<Option<Font<'a>>>,
}

impl<'a> From<&'a str> for FontDesc<'a> {
    fn from(from: &'a str) -> FontDesc<'a> {
        return FontDesc::new(from, 1.0);
    }
}

impl<'a> FontDesc<'a> {
    /// Create a new font
    pub fn new(typeface: &'a str, size: f64) -> Self {
        return Self {
            size,
            name: typeface,
            font: RefCell::new(None),
        };
    }

    /// Create a new font desc with the same font but different size
    pub fn resize(&self, size: f64) -> FontDesc<'a> {
        return Self {
            size,
            name: self.name,
            font: self.font.clone(),
        };
    }

    /// Get the name of the font
    pub fn get_name(&self) -> &'a str {
        return self.name;
    }

    /// Get the size of font
    pub fn get_size(&self) -> f64 {
        return self.size;
    }

    /// Get the size of the text if rendered in this font
    pub fn layout_box(&self, text: &str) -> FontResult<((i32, i32), (i32, i32))> {
        let scale = Scale::uniform(self.size as f32);

        if self.font.borrow().is_none() {
            let font = get_system_font(self.name)?;
            self.font.replace(Some(font));
        }

        if let Some(ref font) = *self.font.borrow() {
            let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
            let (mut max_x, mut max_y) = (0, 0);

            font.layout(text, scale, point(0 as f32, 0 as f32))
                .for_each(|g| {
                    if let Some(rect) = g.pixel_bounding_box() {
                        min_x = min_x.min(rect.min.x);
                        min_y = min_y.min(rect.min.y);
                        max_x = max_x.max(rect.max.x);
                        max_y = max_y.max(rect.max.y);
                    }
                });

            if min_x == i32::MAX || min_y == i32::MAX {
                return Ok(((0, 0), (0, 0)));
            }

            return Ok(((min_x, min_y), (max_x, max_y)));
        }
        unreachable!();
    }
    
    /// Get the size of the text if rendered in this font
    pub fn box_size(&self, text: &str) -> FontResult<(u32, u32)> {
        let ((min_x, min_y), (max_x, max_y)) = self.layout_box(text)?;
        return Ok(((max_x - min_x) as u32, (max_y - min_y) as u32));
    }

    /// Actually draws a font with a drawing function
    pub fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        text: &str,
        (x, y): (i32, i32),
        mut draw: DrawFunc,
    ) -> FontResult<Result<(), E>> {
        let ((_,b),(_,_)) = self.layout_box(text)?;

        let scale = Scale::uniform(self.size as f32);
        if self.font.borrow().is_none() {
            let font = get_system_font(self.name)?;
            self.font.replace(Some(font));
        }

        if let Some(ref font) = *self.font.borrow() {
            let mut result = Ok(());
            for g in font.layout(text, scale, point(x as f32, y as f32 - b as f32)) {
                if let Some(rect) = g.pixel_bounding_box() {
                    let x0 = rect.min.x;
                    let y0 = rect.min.y;
                    g.draw(|x, y, v| {
                        if x as i32 + x0 >= 0 && y as i32 + y0 >= 0 && result.is_ok() {
                            result = draw(x as i32 + x0, y as i32 + y0, v);
                        }
                    });
                }
            }
            return Ok(result);
        }
        return Ok(Ok(()));
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_load_system_font() -> FontResult<()> {
        let font_desc = FontDesc::new("Arial", 30.0);
        assert_eq!(font_desc.get_name(), "Arial");

        let (box_w, box_h) = font_desc.box_size("hello!")?;
        assert!(box_w > 0);
        assert!(box_h > 0);

        Ok(())
    }
}
