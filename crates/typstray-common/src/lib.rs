// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{collections::BTreeMap, ops::Range};

use serde::Serialize;

/*
 * Keep these definitions in sync with `src/lib/compiler/types.ts`
 */

#[derive(Serialize)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

impl From<Range<usize>> for SourceRange {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Serialize)]
pub struct AnalysisNode {
    pub kind: String,
    pub range: Option<SourceRange>,
    pub text: Option<String>,
    pub attrs: BTreeMap<String, String>,
    pub error: bool,
    pub children: Vec<AnalysisNode>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub range: Option<SourceRange>,
}

#[derive(Serialize)]
pub struct Analysis {
    pub ast: AnalysisNode,
    pub content: Option<AnalysisNode>,
    pub layouts: Vec<AnalysisNode>,
    pub diagnostics: Vec<Diagnostic>,
}
