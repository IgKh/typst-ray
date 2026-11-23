// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use std::sync::LazyLock;

use typst::{
    Library, LibraryExt, World,
    diag::FileResult,
    foundations::Bytes,
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_syntax::{FileId, Source, VirtualPath};

static SOURCE_ID: LazyLock<FileId> =
    LazyLock::new(|| FileId::new(None, VirtualPath::new("SOURCE")));

#[derive(Default)]
pub struct Fonts {
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl Fonts {
    pub fn new(data: impl Iterator<Item = Vec<u8>>) -> Self {
        let fonts: Vec<Font> = data.flat_map(|data| Font::iter(Bytes::new(data))).collect();
        let book = FontBook::from_fonts(&fonts);
        Self {
            book: LazyHash::new(book),
            fonts,
        }
    }
}

pub struct TypstRayWorld<'a> {
    library: LazyHash<Library>,
    fonts: &'a Fonts,
    source: Source,
    now: time::OffsetDateTime,
}

impl<'a> TypstRayWorld<'a> {
    pub fn new(content: &str, fonts: &'a Fonts) -> Self {
        let library = Library::builder()
            .with_features(
                [typst::Feature::Html, typst::Feature::A11yExtras]
                    .into_iter()
                    .collect(),
            )
            .build();

        let source = Source::new(*SOURCE_ID, String::from(content));

        Self {
            library: LazyHash::new(library),
            fonts,
            source,
            now: time::OffsetDateTime::now_utc(),
        }
    }
}

impl World for TypstRayWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts.book
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts.get(index).cloned()
    }

    fn main(&self) -> FileId {
        *SOURCE_ID
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == *SOURCE_ID {
            Ok(self.source.clone())
        } else {
            Err(typst::diag::FileError::AccessDenied)
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(typst::diag::FileError::AccessDenied)
    }

    fn today(&self, offset: Option<i64>) -> Option<typst::foundations::Datetime> {
        let date = match offset {
            Some(hours) => {
                let hours = i8::try_from(hours).ok()?;
                let off = time::UtcOffset::from_hms(hours, 0, 0).ok()?;
                self.now.to_offset(off).date()
            }
            None => self.now.date(),
        };

        Some(typst::foundations::Datetime::Date(date))
    }
}
