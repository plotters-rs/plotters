use std::collections::HashMap;
use std::i32;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use rusttype::{point, Error, Font, Scale, SharedBytes};

use font_loader::system_fonts;
use font_loader::system_fonts::FontPropertyBuilder;

use super::{FontData, FontTransform, LayoutBox};

type FontResult<T> = Result<T, FontError>;

#[derive(Debug, Clone)]
pub enum FontError {
    LockError,
    NoSuchFont,
    FontLoadError(Arc<Error>),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FontError::LockError => write!(fmt, "Could not lock mutex"),
            FontError::NoSuchFont => write!(fmt, "No such font"),
            FontError::FontLoadError(e) => write!(fmt, "Font loading error: {}", e),
        }
    }
}

impl std::error::Error for FontError {}

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, Option<SharedBytes<'static>>>> =
        Mutex::new(HashMap::new());
}

#[allow(dead_code)]
/// Lazily load font data. Font type doesn't own actual data, which
/// lives in the cache.
fn load_font_data(face: &str) -> FontResult<Font<'static>> {
    CACHE
        .lock()
        .map_err(|_| FontError::LockError)?
        .entry(face.into())
        .or_insert_with(|| {
            let query = FontPropertyBuilder::new().family(face).build();
            system_fonts::get(&query).map(|(data, _)| data.into())
        })
        .clone()
        .ok_or(FontError::NoSuchFont)
        .and_then(|cached| {
            Font::from_bytes(cached).map_err(|err| FontError::FontLoadError(Arc::new(err)))
        })
}

/// Remove all cached fonts data.
#[allow(dead_code)]
pub fn clear_font_cache() -> FontResult<()> {
    let mut cache = CACHE.lock().map_err(|_| FontError::LockError)?;
    cache.clear();
    Ok(())
}

#[derive(Clone)]
pub struct FontDataInternal(Font<'static>);

impl FontData for FontDataInternal {
    type ErrorType = FontError;
    fn new(face: &str) -> Result<Self, FontError> {
        Ok(FontDataInternal(load_font_data(face)?))
    }
    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let scale = Scale::uniform(size as f32);

        let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
        let (mut max_x, mut max_y) = (0, 0);

        let font = &self.0;

        font.layout(text, scale, point(0.0, 0.0)).for_each(|g| {
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

        Ok(((min_x, min_y), (max_x, max_y)))
    }
    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        (x, y): (i32, i32),
        size: f64,
        text: &str,
        trans: FontTransform,
        mut draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        //let ((_, b), (_, _)) = self.estimate_layout(size, text)?;
        let layout = self.estimate_layout(size, text)?;

        let scale = Scale::uniform(size as f32);
        let mut result = Ok(());
        let font = &self.0;

        let base_x = x + trans.offset(layout).0;
        let base_y = y + trans.offset(layout).1;

        for g in font.layout(text, scale, point(0.0, 0.0)) {
            if let Some(rect) = g.pixel_bounding_box() {
                let x0 = rect.min.x;
                let y0 = rect.min.y - (layout.0).1;
                g.draw(|x, y, v| {
                    let (x, y) = trans.transform(x as i32 + x0, y as i32 + y0);
                    if x + base_x >= 0 && y + base_y >= 0 && result.is_ok() {
                        result = draw(x + base_x, y + base_y, v);
                    }
                });
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_font_cache() -> FontResult<()> {
        
        clear_font_cache()?;

        assert_eq!(CACHE.lock().unwrap().len(), 0);

        load_font_data("sans")?;
        assert_eq!(CACHE.lock().unwrap().len(), 1);

        load_font_data("sans")?;
        assert_eq!(CACHE.lock().unwrap().len(), 1);

        return Ok(());
    }
}
