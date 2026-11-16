// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{collections::BTreeMap, ops::Range};

use serde::Serialize;
use typst::{
    ecow::EcoVec,
    foundations::{Content, Repr, Value},
    layout::{Frame, FrameItem},
};
use typst_layout::PagedDocument;
use typst_syntax::{DiagSpan, DiagSpanKind, LinkedNode, Source};

use crate::compiler::CompilationResult;

#[derive(Serialize)]
pub struct SourceRange {
    start: usize,
    end: usize,
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
    kind: String,
    range: Option<SourceRange>,
    text: Option<String>,
    attrs: BTreeMap<String, String>,
    error: bool,
    children: Vec<AnalysisNode>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    message: String,
    range: Option<SourceRange>,
}

#[derive(Serialize)]
pub struct Analysis {
    ast: AnalysisNode,
    content: Option<AnalysisNode>,
    layouts: Vec<AnalysisNode>,
    diagnostics: Vec<Diagnostic>,
}

pub fn build_analysis(result: &CompilationResult) -> Analysis {
    let source = &result.source;
    let ast = convert_ast_node(LinkedNode::new(source.root()), source);

    let content = result
        .content
        .as_ref()
        .map(|content| convert_content(content, source));

    let layouts = result
        .documents
        .iter()
        .map(|document| convert_document(document, source))
        .collect();

    let diagnostics = convert_diagnostics(&result.diagnostics, source);

    Analysis {
        ast,
        content,
        layouts,
        diagnostics,
    }
}

fn convert_ast_node(node: LinkedNode<'_>, source: &Source) -> AnalysisNode {
    let text = Some(node.leaf_text().to_string()).filter(|s| !s.is_empty());
    let children: Vec<_> = node
        .children()
        .map(|child| convert_ast_node(child, source))
        .collect();

    AnalysisNode {
        kind: format!("{:?}", node.kind()),
        range: process_range(node.range(), source),
        text,
        attrs: BTreeMap::new(),
        error: node.diagnosis().errors,
        children,
    }
}

fn collect_content_attributes(
    name: &str,
    value: &Value,
    source: &Source,
    attrs: &mut BTreeMap<String, String>,
    children: &mut Vec<AnalysisNode>,
) {
    match value {
        Value::Content(c) => children.push(convert_content(c, source)),
        Value::Array(arr) => {
            for item in arr {
                collect_content_attributes(name, item, source, attrs, children);
            }
        }
        Value::Dict(dict) => {
            for (_, value) in dict {
                collect_content_attributes(name, value, source, attrs, children);
            }
        }
        _ => {
            attrs.insert(name.to_string(), value.repr().to_string());
        }
    }
}

fn convert_content(content: &Content, source: &Source) -> AnalysisNode {
    let range = process_span(content.span(), source);

    let mut attrs = BTreeMap::new();
    let mut children = vec![];

    if let Some(seq) = content.to_packed::<typst::foundations::SequenceElem>() {
        children.extend(seq.children.iter().map(|c| convert_content(c, source)));
    } else {
        for (name, value) in content.fields() {
            collect_content_attributes(&name, &value, source, &mut attrs, &mut children);
        }
    }

    // TODO - decide how to present StyledElem

    let text = attrs.remove("text");

    AnalysisNode {
        kind: content.elem().name().to_string(),
        range,
        text,
        attrs,
        error: false,
        children,
    }
}

fn convert_document(document: &PagedDocument, source: &Source) -> AnalysisNode {
    let pages = document.pages().iter().map(|page| AnalysisNode {
        kind: format!("Page {}", page.number),
        range: None,
        text: None,
        attrs: BTreeMap::new(),
        error: false,
        children: convert_frame(&page.frame, source),
    });

    AnalysisNode {
        kind: String::from("Document"),
        range: None,
        text: None,
        attrs: BTreeMap::new(),
        error: false,
        children: pages.collect(),
    }
}

fn convert_frame(frame: &Frame, source: &Source) -> Vec<AnalysisNode> {
    let mut result = vec![];

    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => result.push(AnalysisNode {
                kind: String::from("Group"),
                range: None,
                text: None,
                attrs: BTreeMap::new(),
                error: false,
                children: convert_frame(&group.frame, source),
            }),
            FrameItem::Text(text) => {
                let range = if !text.glyphs.is_empty() {
                    let ranges = text
                        .glyphs
                        .iter()
                        .filter_map(|g| process_span(g.span.0, source));

                    let mut start = usize::MAX;
                    let mut end = usize::MIN;
                    for range in ranges {
                        start = start.min(range.start);
                        end = end.max(range.end);
                    }

                    Some((start..end).into())
                } else {
                    None
                };

                result.push(AnalysisNode {
                    kind: String::from("Text"),
                    range,
                    text: Some(text.text.to_string()),
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Shape(_shape, span) => {
                let range = process_span(*span, source);

                result.push(AnalysisNode {
                    kind: String::from("Shape"),
                    range,
                    text: None,
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Image(_image, _axes, span) => {
                let range = process_span(*span, source);

                result.push(AnalysisNode {
                    kind: String::from("Image"),
                    range,
                    text: None,
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Link(_destination, _axes) => result.push(AnalysisNode {
                kind: String::from("Link"),
                range: None,
                text: None,
                attrs: BTreeMap::new(),
                error: false,
                children: vec![],
            }),
            FrameItem::Tag(_tag) => result.push(AnalysisNode {
                kind: String::from("Tag"),
                range: None,
                text: None,
                attrs: BTreeMap::new(),
                error: false,
                children: vec![],
            }),
        }
    }

    result
}

fn convert_diagnostics(
    diagnostics: &EcoVec<typst::diag::SourceDiagnostic>,
    source: &Source,
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diag| Diagnostic {
            message: diag.message.to_string(),
            range: process_span(diag.span, source),
        })
        .collect()
}

fn process_span(span: impl Into<DiagSpan>, source: &Source) -> Option<SourceRange> {
    let span = span.into();
    match span.get() {
        DiagSpanKind::Number { id, num, sub_range } if id == source.id() => {
            let range = source.range(num, sub_range)?;
            process_range(range, source)
        }
        _ => None,
    }
}

fn process_range(range: Range<usize>, source: &Source) -> Option<SourceRange> {
    let start = source.lines().byte_to_utf16(range.start)?;
    let end = source.lines().byte_to_utf16(range.end)?;

    Some((start..end).into())
}
