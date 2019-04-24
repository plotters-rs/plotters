use font_loader::system_fonts;
use lazy_static::lazy_static;
/// The font management utilities
/// This file retains a font cache and makes font object.
///
use rusttype::{point, Error, Font, Scale};
use std::cell::RefCell;
use std::collections::HashMap;
use std::i32;
use std::sync::Mutex;

#[derive(Debug)]
pub enum FontError {
    LockError,
    NoSuchFont,
    FontLoadError(Error),
}

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

impl<'a> FontDesc<'a> {
    /// Create a new font
    pub fn new(typeface: &'a str, size: f64) -> Self {
        return Self {
            size,
            name: typeface,
            font: RefCell::new(None),
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
    pub fn box_size(&self, text: &str) -> FontResult<(u32, u32)> {
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
                return Ok((0, 0));
            }

            return Ok(((max_x - min_x) as u32, (0 - min_y) as u32));
        }
        unreachable!();
    }

    /// Actually draws a font with a drawing function
    pub fn draw<E, DrawFunc: FnMut(u32, u32, f32) -> Result<(), E>>(
        &self,
        text: &str,
        (x, y): (u32, u32),
        mut draw: DrawFunc,
    ) -> FontResult<Result<(), E>> {
        let (_, h) = self.box_size(text)?;

        let scale = Scale::uniform(self.size as f32);
        if self.font.borrow().is_none() {
            let font = get_system_font(self.name)?;
            self.font.replace(Some(font));
        }

        if let Some(ref font) = *self.font.borrow() {
            let mut result = Ok(());
            for g in font.layout(text, scale, point(x as f32, y as f32 + h as f32)) {
                if let Some(rect) = g.pixel_bounding_box() {
                    let x0 = rect.min.x;
                    let y0 = rect.min.y;
                    g.draw(|x, y, v| {
                        if x as i32 + x0 >= 0 && y as i32 + y0 >= 0 && result.is_ok() {
                            result = draw((x as i32 + x0) as u32, (y as i32 + y0) as u32, v);
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
        let font_desc = FontDesc::new("ArialMT", 30.0);
        assert_eq!(font_desc.get_name(), "ArialMT");

        let (box_w, box_h) = font_desc.box_size("hello!")?;
        assert!(box_w > 0);
        assert!(box_h > 0);

        Ok(())
    }
}
