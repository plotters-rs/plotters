use std::pin::Pin;
use std::marker::PhantomPinned;
use std::slice::from_raw_parts;
use std::collections::HashMap;
use std::sync::Mutex;
use std::i32;
use std::rc::Rc;

use rusttype::{Error, Scale, Font, point};

use lazy_static::lazy_static;

use font_loader::system_fonts;

use super::FontData;

type FontResult<T> = Result<T, FontError>;


#[derive(Debug, Clone)]
pub enum FontError {
    LockError,
    NoSuchFont,
    FontLoadError(Rc<Error>),
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

struct OwnedFont {
    data: Vec<u8>,
    font: Option<Font<'static>>,
    _pinned: PhantomPinned,
}

impl OwnedFont {
    fn new(data:Vec<u8>) -> Result<Pin<Box<Self>>, Error> {
        let font_obj = OwnedFont {
            data,
            font: None,
            _pinned: PhantomPinned,
        };

        let mut boxed_font_obj = Box::pin(font_obj);

        unsafe {
            let addr = &boxed_font_obj.data[0] as *const u8;
            let font = Font::from_bytes(from_raw_parts(addr, boxed_font_obj.data.len()))?;
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed_font_obj);
            Pin::get_unchecked_mut(mut_ref).font = std::mem::transmute::<_, Option<Font<'static>>>(Some(font));
        }

        return Ok(boxed_font_obj);
    }
}

impl Drop for OwnedFont {
    fn drop(&mut self) {
        self.font = None;
    }
}

impl <'a> Into<&'a Font<'a>> for &'a OwnedFont {
    fn into(self) -> &'a Font<'a> {
        return self.font.as_ref().unwrap();
    }
}

lazy_static! {
    static ref FONT_DATA_CACHE: Mutex<HashMap<String, Pin<Box<OwnedFont>>>> = { Mutex::new(HashMap::new()) };
}

#[allow(dead_code)]
fn load_font_data(face: &str) -> FontResult<&'static Font<'static>> {
    match FONT_DATA_CACHE.lock().map(|mut cache| {
        if !cache.contains_key(face) {
            let query = system_fonts::FontPropertyBuilder::new()
                .family(face)
                .build();
            if let Some((data, _)) = system_fonts::get(&query) {
                let font = OwnedFont::new(data).map_err(|e| FontError::FontLoadError(Rc::new(e)))?;
                cache.insert(face.to_string(), font);
            } else {
                return Err(FontError::NoSuchFont);
            }
        }
        let font_ref:&'static OwnedFont = unsafe{ std::mem::transmute(cache.get(face).unwrap().as_ref().get_ref()) };
        let addr = Into::<&'static Font<'static>>::into(font_ref) as *const Font<'static>;
        return Ok(unsafe {addr.as_ref().unwrap()});
    }) {
        Ok(what) => what,
        Err(_) => Err(FontError::LockError),
    }
}

/// STOP! This is generally a bad idea, because all the font we borrowed out should have a static life
/// time, thus clear the font cache may cause problem.
#[allow(dead_code)]
pub unsafe fn clear_font_cache() -> FontResult<()> {
    if let Ok(mut cache) = FONT_DATA_CACHE.lock() {
        *cache = HashMap::new();
    }
    return Err(FontError::LockError);
}

#[derive(Clone)]
pub struct FontDataInternal(&'static Font<'static>);

impl FontData for FontDataInternal {
    type ErrorType = FontError;
    fn new(face:&str) -> Result<Self, FontError> {
        return Ok(FontDataInternal(load_font_data(face)?));
    }
    fn estimate_layout(&self, size:f64, text:&str) -> Result<((i32, i32), (i32, i32)), Self::ErrorType> {
        let scale = Scale::uniform(size as f32);

        let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
        let (mut max_x, mut max_y) = (0, 0);

        let font = self.0;

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
    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(&self, (x,y): (i32, i32), size:f64, text:&str, mut draw: DrawFunc) -> Result<Result<(), E>, Self::ErrorType> {
        let ((_, b), (_, _)) = self.estimate_layout(size, text)?;

        let scale = Scale::uniform(size as f32);
        let mut result = Ok(());
        let font = self.0;
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
}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_font_cache() -> FontResult<()> {
        let font1 = load_font_data("Arial")?;
        let font2 = load_font_data("Arial")?;

        assert_eq!(font1 as *const Font<'static>, font2 as *const Font<'static>);

        return Ok(());
    }
}
