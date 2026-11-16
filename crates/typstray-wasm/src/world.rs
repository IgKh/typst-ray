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
use typst_syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

static SOURCE_ID: LazyLock<FileId> = LazyLock::new(|| {
    FileId::unique(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("SOURCE").unwrap(),
    ))
});

pub struct TypstRayWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
}

impl TypstRayWorld {
    pub fn new(content: &str) -> Self {
        let fonts: Vec<_> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        let book = FontBook::from_fonts(&fonts);

        let library = Library::builder()
            .with_features(typst::Features::all())
            .build();

        let source = Source::new(*SOURCE_ID, String::from(content));

        Self {
            library: LazyHash::new(library),
            book: LazyHash::new(book),
            fonts,
            source,
        }
    }
}

impl World for TypstRayWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
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

    fn today(
        &self,
        _offset: Option<typst::foundations::Duration>,
    ) -> Option<typst::foundations::Datetime> {
        None
    }
}
