use fontique::{
    Attributes, Collection, CollectionOptions, FallbackKey, FontStyle as FontiqueStyle,
    FontWeight, FontWidth, GenericFamily, QueryFamily, QueryStatus, Script, SourceCache,
};
use plotters_backend::{FontFamily, FontStyle};
use std::sync::Arc;

pub struct SystemFontSource {
    collection: Collection,
    source_cache: SourceCache,
}

pub struct FontCandidate {
    pub data: Arc<[u8]>,
    pub index: u32,
}

impl SystemFontSource {
    pub fn new(enable_system: bool) -> Self {
        Self {
            collection: Collection::new(CollectionOptions {
                system_fonts: enable_system,
                ..Default::default()
            }),
            source_cache: SourceCache::default(),
        }
    }

    /// Resolve `family` against the configured collection. When
    /// `with_fallback` is true, fontique chains through Latin-script fallback
    /// families if the named family isn't installed -- mirroring the implicit
    /// fontconfig fallback that callers used to get from the legacy
    /// font-kit/`ttf` backend, so a chart asking for a font that isn't on the
    /// host (e.g. "Calibri" on Linux) renders via the closest match instead
    /// of erroring. Strict resolution (`with_fallback = false`) is kept for
    /// explicit `with_fonts(...)` contexts where every name must match
    /// exactly.
    pub fn resolve(
        &mut self,
        family: FontFamily<'_>,
        style: FontStyle,
        with_fallback: bool,
    ) -> Option<FontCandidate> {
        let mut query = self.collection.query(&mut self.source_cache);
        match family {
            FontFamily::Serif => query.set_families([QueryFamily::Generic(GenericFamily::Serif)]),
            FontFamily::SansSerif => {
                query.set_families([QueryFamily::Generic(GenericFamily::SansSerif)])
            }
            FontFamily::Monospace => {
                query.set_families([QueryFamily::Generic(GenericFamily::Monospace)])
            }
            FontFamily::Name(name) => query.set_families([QueryFamily::Named(name)]),
        }
        query.set_attributes(attributes(style));
        if with_fallback {
            // Latin script covers the ASCII/Latin-1 ranges that chart labels
            // are overwhelmingly drawn from; fontique iterates `families`
            // first and only consults the fallback list when nothing in
            // `families` matched.
            query.set_fallbacks(FallbackKey::new(Script::from_bytes(*b"Latn"), None));
        }

        let mut candidate = None;
        query.matches_with(|font| {
            candidate = Some(FontCandidate {
                data: Arc::from(font.blob.data()),
                index: font.index,
            });
            QueryStatus::Stop
        });
        candidate
    }
}

fn attributes(style: FontStyle) -> Attributes {
    match style {
        FontStyle::Normal => Attributes::new(
            FontWidth::default(),
            FontiqueStyle::Normal,
            FontWeight::NORMAL,
        ),
        FontStyle::Italic => Attributes::new(
            FontWidth::default(),
            FontiqueStyle::Italic,
            FontWeight::NORMAL,
        ),
        FontStyle::Oblique => Attributes::new(
            FontWidth::default(),
            FontiqueStyle::Oblique(Some(14.0)),
            FontWeight::NORMAL,
        ),
        FontStyle::Bold => Attributes::new(
            FontWidth::default(),
            FontiqueStyle::Normal,
            FontWeight::BOLD,
        ),
    }
}
