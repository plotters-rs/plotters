use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use fontique::{
    Attributes, Collection, CollectionOptions, FontStyle as FontiqueStyle, FontWeight, FontWidth,
    GenericFamily, QueryFamily, QueryStatus, SourceCache,
};
use lazy_static::lazy_static;

use super::{FontError, FontFamily, FontResult, FontStyle};

#[derive(Clone, Debug)]
pub(super) struct SystemFontData {
    pub(super) bytes: Arc<Vec<u8>>,
    pub(super) index: usize,
}

lazy_static! {
    static ref COLLECTION: Mutex<(Collection, SourceCache)> = Mutex::new((
        Collection::new(CollectionOptions {
            system_fonts: true,
            ..CollectionOptions::default()
        }),
        SourceCache::new_shared(),
    ));
    static ref BYTE_CACHE: RwLock<HashMap<String, FontResult<SystemFontData>>> =
        RwLock::new(HashMap::new());
}

pub(super) fn load(family: FontFamily<'_>, style: FontStyle) -> FontResult<SystemFontData> {
    let key = cache_key(family, style);

    if let Some(data) = BYTE_CACHE
        .read()
        .map_err(|_| FontError::LockError)?
        .get(&key)
    {
        return data.clone();
    }

    let loaded = load_uncached(family, style);
    BYTE_CACHE
        .write()
        .map_err(|_| FontError::LockError)?
        .insert(key, loaded.clone());
    loaded
}

#[cfg(test)]
fn cache_contains(family: FontFamily<'_>, style: FontStyle) -> bool {
    BYTE_CACHE
        .read()
        .map(|cache| cache.contains_key(&cache_key(family, style)))
        .unwrap_or(false)
}

fn load_uncached(family: FontFamily<'_>, style: FontStyle) -> FontResult<SystemFontData> {
    let mut hit = None;
    let mut collection = COLLECTION.lock().map_err(|_| FontError::LockError)?;
    let (collection, source_cache) = &mut *collection;
    let mut query = collection.query(source_cache);

    let query_families = query_families(family);
    query.set_families(query_families.iter().copied());
    query.set_attributes(attributes(style));
    query.matches_with(|font| {
        hit = Some(SystemFontData {
            bytes: Arc::new(font.blob.as_ref().to_vec()),
            index: font.index as usize,
        });
        QueryStatus::Stop
    });

    hit.ok_or_else(|| FontError::NoSuchFont(family.as_str().to_owned(), style.as_str().to_owned()))
}

fn cache_key(family: FontFamily<'_>, style: FontStyle) -> String {
    match style {
        FontStyle::Normal => family.as_str().to_owned(),
        _ => format!("{}, {}", family.as_str(), style.as_str()),
    }
}

fn query_families(family: FontFamily<'_>) -> Vec<QueryFamily<'_>> {
    match family {
        FontFamily::Serif => vec![
            QueryFamily::Generic(GenericFamily::Serif),
            QueryFamily::Generic(GenericFamily::SansSerif),
        ],
        FontFamily::SansSerif => vec![QueryFamily::Generic(GenericFamily::SansSerif)],
        FontFamily::Monospace => vec![
            QueryFamily::Generic(GenericFamily::Monospace),
            QueryFamily::Generic(GenericFamily::SansSerif),
        ],
        FontFamily::Name(name) => vec![
            QueryFamily::Named(name),
            QueryFamily::Generic(GenericFamily::SansSerif),
        ],
    }
}

fn attributes(style: FontStyle) -> Attributes {
    let (font_style, font_weight) = match style {
        FontStyle::Normal => (FontiqueStyle::Normal, FontWeight::NORMAL),
        FontStyle::Italic => (FontiqueStyle::Italic, FontWeight::NORMAL),
        FontStyle::Oblique => (FontiqueStyle::Oblique(None), FontWeight::NORMAL),
        FontStyle::Bold => (FontiqueStyle::Normal, FontWeight::BOLD),
    };
    Attributes::new(FontWidth::NORMAL, font_style, font_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_system_font_variants() -> FontResult<()> {
        let cases = [
            ("serif_normal", FontFamily::Serif, FontStyle::Normal),
            ("sans_bold", FontFamily::SansSerif, FontStyle::Bold),
            ("monospace_italic", FontFamily::Monospace, FontStyle::Italic),
        ];

        for (name, family, style) in cases {
            let font = load(family, style).unwrap_or_else(|err| {
                panic!("case {} failed to load a system font: {}", name, err)
            });
            assert!(
                !font.bytes.is_empty(),
                "case {} loaded empty font data",
                name
            );
            assert!(
                cache_contains(family, style),
                "case {} did not populate the font byte cache",
                name
            );
        }

        Ok(())
    }

    #[test]
    fn missing_named_font_falls_back_to_sans_serif() -> FontResult<()> {
        let family = FontFamily::Name("plotters-font-that-should-not-exist");
        let font = load(family, FontStyle::Normal)?;
        assert!(!font.bytes.is_empty());
        Ok(())
    }
}
