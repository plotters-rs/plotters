use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::i32;
use std::io::Read;
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use rusttype::{point, Error, Font, FontCollection, Scale, SharedBytes};

use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;

use super::{FontData, FontFamily, FontStyle, LayoutBox};

type FontResult<T> = Result<T, FontError>;

#[derive(Debug, Clone)]
pub enum FontError {
    LockError,
    NoSuchFont(String, String),
    FontLoadError(Arc<Error>),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FontError::LockError => write!(fmt, "Could not lock mutex"),
            FontError::NoSuchFont(family, style) => {
                write!(fmt, "No such font: {} {}", family, style)
            }
            FontError::FontLoadError(e) => write!(fmt, "Font loading error: {}", e),
        }
    }
}

impl std::error::Error for FontError {}

lazy_static! {
    static ref CACHE: RwLock<HashMap<String, FontResult<Font<'static>>>> =
        RwLock::new(HashMap::new());
}

thread_local! {
    static FONT_SOURCE: SystemSource = SystemSource::new();
}

#[allow(dead_code)]
/// Lazily load font data. Font type doesn't own actual data, which
/// lives in the cache.
fn load_font_data(face: FontFamily, style: FontStyle) -> FontResult<Font<'static>> {
    let key = match style {
        FontStyle::Normal => Cow::Borrowed(face.as_str()),
        _ => Cow::Owned(format!("{}, {}", face.as_str(), style.as_str())),
    };
    let cache = CACHE.read().unwrap();
    if let Some(cached) = cache.get(Borrow::<str>::borrow(&key)) {
        return cached.clone();
    }
    drop(cache);

    let mut properties = Properties::new();
    match style {
        FontStyle::Normal => properties.style(Style::Normal),
        FontStyle::Italic => properties.style(Style::Italic),
        FontStyle::Oblique => properties.style(Style::Oblique),
        FontStyle::Bold => properties.weight(Weight::BOLD),
    };

    let family = match face {
        FontFamily::Serif => FamilyName::Serif,
        FontFamily::SansSerif => FamilyName::SansSerif,
        FontFamily::Monospace => FamilyName::Monospace,
        FontFamily::Name(name) => FamilyName::Title(name.to_owned()),
    };

    let make_not_found_error =
        || FontError::NoSuchFont(face.as_str().to_owned(), style.as_str().to_owned());

    if let Ok(handle) = FONT_SOURCE
        .with(|source| source.select_best_match(&[family, FamilyName::SansSerif], &properties))
    {
        let (data, id) = match handle {
            Handle::Path {
                path,
                font_index: idx,
            } => {
                let mut buf = vec![];
                std::fs::File::open(path)
                    .map_err(|_| make_not_found_error())?
                    .read_to_end(&mut buf)
                    .map_err(|_| make_not_found_error())?;
                (buf, idx)
            }
            Handle::Memory {
                bytes,
                font_index: idx,
            } => (bytes[..].to_owned(), idx),
        };
        // TODO: font-kit actually have rasterizer, so consider remove dependency for rusttype as
        // well
        let result = FontCollection::from_bytes(Into::<SharedBytes>::into(data))
            .map_err(|err| FontError::FontLoadError(Arc::new(err)))?
            .font_at(id.max(0) as usize)
            .map_err(|err| FontError::FontLoadError(Arc::new(err)));

        CACHE
            .write()
            .map_err(|_| FontError::LockError)?
            .insert(key.into_owned(), result.clone());

        return result;
    }
    Err(make_not_found_error())
}

/// Remove all cached fonts data.
#[allow(dead_code)]
pub fn clear_font_cache() -> FontResult<()> {
    let mut cache = CACHE.write().map_err(|_| FontError::LockError)?;
    cache.clear();
    Ok(())
}

#[derive(Clone)]
pub struct FontDataInternal(Font<'static>);

impl FontData for FontDataInternal {
    type ErrorType = FontError;

    fn new(family: FontFamily, style: FontStyle) -> Result<Self, FontError> {
        Ok(FontDataInternal(load_font_data(family, style)?))
    }

    fn estimate_layout(&self, size: f64, text: &str) -> Result<LayoutBox, Self::ErrorType> {
        let scale = Scale::uniform(size as f32);

        let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
        let (mut max_x, mut max_y) = (0, 0);

        let font = &self.0;

        for g in font.layout(text, scale, point(0.0, 0.0)) {
            if let Some(rect) = g.pixel_bounding_box() {
                min_x = min_x.min(rect.min.x);
                min_y = min_y.min(rect.min.y);
                max_x = max_x.max(rect.max.x);
                max_y = max_y.max(rect.max.y);
            }
        }

        if min_x == i32::MAX || min_y == i32::MAX {
            return Ok(((0, 0), (0, 0)));
        }

        Ok(((min_x, min_y), (max_x, max_y)))
    }

    fn draw<E, DrawFunc: FnMut(i32, i32, f32) -> Result<(), E>>(
        &self,
        (base_x, base_y): (i32, i32),
        size: f64,
        text: &str,
        mut draw: DrawFunc,
    ) -> Result<Result<(), E>, Self::ErrorType> {
        let scale = Scale::uniform(size as f32);
        let mut result = Ok(());
        let font = &self.0;

        for g in font.layout(text, scale, point(0.0, 0.0)) {
            if let Some(rect) = g.pixel_bounding_box() {
                let (x0, y0) = (rect.min.x, rect.min.y);
                g.draw(|x, y, v| {
                    let (x, y) = (x as i32 + x0, y as i32 + y0);
                    result = draw(x + base_x, y + base_y, v);
                });
                if result.is_err() {
                    break;
                }
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

        // We cannot only check the size of font cache, because
        // the test case may be run in parallel. Thus the font cache
        // may contains other fonts.
        let _a = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        assert!(CACHE.read().unwrap().contains_key("serif"));

        let _b = load_font_data(FontFamily::Serif, FontStyle::Normal)?;
        assert!(CACHE.read().unwrap().contains_key("serif"));

        // TODO: Check they are the same

        return Ok(());
    }
}
