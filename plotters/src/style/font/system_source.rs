//! Native system font discovery via [`fontique`].
//!
//! Compiled only with the `ttf` feature; `ttf.rs` is the sole consumer.
//! The byte cache here is the canonical store for font data — `ttf.rs`
//! keeps a thread-local `FontExt` cache on top but does not re-cache the
//! bytes themselves.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use fontique::{
    Attributes, Collection, CollectionOptions, FontStyle as FontiqueStyle, FontWeight, FontWidth,
    GenericFamily, QueryFamily, QueryStatus, SourceCache,
};
use lazy_static::lazy_static;
use swash::FontRef;
use ttf_parser::Face;

use super::{FontError, FontFamily, FontResult, FontStyle};

/// Bytes for one resolved font face. Cheap to clone.
#[derive(Clone, Debug)]
pub(super) struct SystemFontData {
    /// Font binary, kept alive for as long as any consumer references it.
    pub(super) bytes: Arc<Vec<u8>>,
    /// Index of the face inside the font collection (TTC).
    pub(super) index: usize,
    /// Stable cache identity allocated at first load. Used by swash's
    /// `ScaleContext` so glyphs cannot be served from a freed-and-reused
    /// pointer if the underlying `Arc` is ever pruned.
    pub(super) id: u64,
}

static FONT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_font_id() -> u64 {
    FONT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
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

    // Resolve outside the write lock so concurrent loads of *different*
    // fonts do not serialize on font I/O.
    let loaded = load_uncached(family, style);

    // Take the write lock and re-check: if another thread inserted while we
    // were loading, prefer its entry so the cache stays canonical and
    // `Arc::ptr_eq` holds across repeated loads of the same key.
    let mut cache = BYTE_CACHE.write().map_err(|_| FontError::LockError)?;
    cache.entry(key).or_insert(loaded).clone()
}

#[cfg(test)]
fn cache_contains(family: FontFamily<'_>, style: FontStyle) -> bool {
    BYTE_CACHE
        .read()
        .map(|cache| cache.contains_key(&cache_key(family, style)))
        .unwrap_or(false)
}

fn load_uncached(family: FontFamily<'_>, style: FontStyle) -> FontResult<SystemFontData> {
    // Resolve the font under the collection lock, but drop the lock before
    // copying the byte buffer so concurrent loads of *different* fonts do
    // not serialize on a multi-MB Vec::to_vec.
    let mut hit = None;
    {
        let mut collection = COLLECTION.lock().map_err(|_| FontError::LockError)?;
        let (collection, source_cache) = &mut *collection;
        let mut query = collection.query(source_cache);

        let query_families = query_families(family);
        query.set_families(query_families.iter().copied());
        query.set_attributes(attributes(style));
        query.matches_with(|font| {
            // peniko::Blob is internally reference-counted; clone is cheap.
            hit = Some((font.blob.clone(), font.index));
            QueryStatus::Stop
        });
    }

    let (blob, index) = hit.ok_or_else(|| {
        FontError::NoSuchFont(family.as_str().to_owned(), style.as_str().to_owned())
    })?;

    let bytes = Arc::new(blob.as_ref().to_vec());
    let index = index as usize;

    // Validate once at load time so consumers can treat parsed FontRef /
    // Face as infallible from the cached bytes.
    FontRef::from_index(bytes.as_slice(), index).ok_or_else(|| {
        FontError::FontLoadError(format!("invalid font data at index {}", index))
    })?;
    Face::parse(bytes.as_slice(), index as u32)
        .map_err(|err| FontError::FaceParseError(err.to_string()))?;

    Ok(SystemFontData {
        bytes,
        index,
        id: next_font_id(),
    })
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
            assert!(font.id > 0, "case {} got an unallocated font id", name);
        }

        Ok(())
    }

    #[test]
    fn missing_named_font_falls_back_to_sans_serif() -> FontResult<()> {
        // Matches font-kit's behavior: select_best_match was called with
        // [requested, SansSerif] as the candidate list, so unknown names
        // resolve to the sans-serif fallback rather than erroring.
        let family = FontFamily::Name("plotters-font-that-should-not-exist");
        let font = load(family, FontStyle::Normal)?;
        assert!(!font.bytes.is_empty());
        Ok(())
    }

    #[test]
    fn cached_loads_share_arc() -> FontResult<()> {
        let a = load(FontFamily::Serif, FontStyle::Normal)?;
        let b = load(FontFamily::Serif, FontStyle::Normal)?;
        assert!(
            Arc::ptr_eq(&a.bytes, &b.bytes),
            "cached loads should return the same Arc"
        );
        assert_eq!(a.id, b.id, "cached loads should share font id");
        Ok(())
    }
}
