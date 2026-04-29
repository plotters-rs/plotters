// pattern: Imperative Shell

use fontique::{
    Attributes, Collection, CollectionOptions, FontStyle as FontiqueStyle, FontWeight, FontWidth,
    GenericFamily, QueryFamily, QueryStatus, SourceCache,
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

    pub fn resolve(&mut self, family: FontFamily<'_>, style: FontStyle) -> Option<FontCandidate> {
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
