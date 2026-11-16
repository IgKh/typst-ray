// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::BTreeMap;

use serde::Serialize;
use typst::{
    World, WorldExt,
    ecow::EcoVec,
    foundations::{Content, Repr, Value},
    layout::{Frame, FrameItem},
};
use typst_layout::PagedDocument;
use typst_syntax::LinkedNode;

use crate::compiler::CompilationResult;

#[derive(Serialize)]
pub struct AnalysisNode {
    kind: String,
    start: Option<usize>,
    end: Option<usize>,
    text: Option<String>,
    attrs: BTreeMap<String, String>,
    error: bool,
    children: Vec<AnalysisNode>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    message: String,
    start: Option<usize>,
    end: Option<usize>,
}

#[derive(Serialize)]
pub struct Analysis {
    ast: AnalysisNode,
    content: Option<AnalysisNode>,
    layouts: Vec<AnalysisNode>,
    diagnostics: Vec<Diagnostic>,
}

pub fn build_analysis(result: &CompilationResult, world: &dyn World) -> Analysis {
    let ast = convert_ast_node(LinkedNode::new(result.source.root()));

    let content = result
        .content
        .as_ref()
        .map(|content| convert_content(content, world));

    let layouts = result
        .documents
        .iter()
        .map(|document| convert_document(document, world))
        .collect();

    let diagnostics = convert_diagnostics(&result.diagnostics, world);

    Analysis {
        ast,
        content,
        layouts,
        diagnostics,
    }
}

fn convert_ast_node(node: LinkedNode<'_>) -> AnalysisNode {
    let range = node.range();
    let text = Some(node.leaf_text().to_string()).filter(|s| !s.is_empty());
    let children: Vec<_> = node.children().map(convert_ast_node).collect();

    AnalysisNode {
        kind: format!("{:?}", node.kind()),
        start: Some(range.start),
        end: Some(range.end),
        text,
        attrs: BTreeMap::new(),
        error: node.diagnosis().errors,
        children,
    }
}

fn collect_content_attributes(
    name: &str,
    value: &Value,
    world: &dyn World,
    attrs: &mut BTreeMap<String, String>,
    children: &mut Vec<AnalysisNode>,
) {
    match value {
        Value::Content(c) => children.push(convert_content(c, world)),
        Value::Array(arr) => {
            for item in arr {
                collect_content_attributes(name, item, world, attrs, children);
            }
        }
        Value::Dict(dict) => {
            for (_, value) in dict {
                collect_content_attributes(name, value, world, attrs, children);
            }
        }
        _ => {
            attrs.insert(name.to_string(), value.repr().to_string());
        }
    }
}

fn convert_content(content: &Content, world: &dyn World) -> AnalysisNode {
    let range = world.range(content.span());
    let start = range.as_ref().map(|r| r.start);
    let end = range.as_ref().map(|r| r.end);

    let mut attrs = BTreeMap::new();
    let mut children = vec![];

    if let Some(seq) = content.to_packed::<typst::foundations::SequenceElem>() {
        children.extend(seq.children.iter().map(|c| convert_content(c, world)));
    } else {
        for (name, value) in content.fields() {
            collect_content_attributes(&name, &value, world, &mut attrs, &mut children);
        }
    }

    // TODO - decide how to present StyledElem

    let text = attrs.remove("text");

    AnalysisNode {
        kind: content.elem().name().to_string(),
        start,
        end,
        text,
        attrs,
        error: false,
        children,
    }
}

fn convert_document(document: &PagedDocument, world: &dyn World) -> AnalysisNode {
    let pages = document.pages().iter().map(|page| AnalysisNode {
        kind: format!("Page {}", page.number),
        start: None,
        end: None,
        text: None,
        attrs: BTreeMap::new(),
        error: false,
        children: convert_frame(&page.frame, world),
    });

    AnalysisNode {
        kind: String::from("Document"),
        start: None,
        end: None,
        text: None,
        attrs: BTreeMap::new(),
        error: false,
        children: pages.collect(),
    }
}

fn convert_frame(frame: &Frame, world: &dyn World) -> Vec<AnalysisNode> {
    let mut result = vec![];

    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => result.push(AnalysisNode {
                kind: String::from("Group"),
                start: None,
                end: None,
                text: None,
                attrs: BTreeMap::new(),
                error: false,
                children: convert_frame(&group.frame, world),
            }),
            FrameItem::Text(text) => {
                let (start, end) = if !text.glyphs.is_empty() {
                    let ranges = text.glyphs.iter().filter_map(|g| world.range(g.span.0));

                    let mut start = usize::MAX;
                    let mut end = usize::MIN;
                    for range in ranges {
                        start = start.min(range.start);
                        end = end.max(range.end);
                    }

                    (Some(start), Some(end))
                } else {
                    (None, None)
                };

                result.push(AnalysisNode {
                    kind: String::from("Text"),
                    start: start,
                    end: end,
                    text: Some(text.text.to_string()),
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Shape(_shape, span) => {
                let range = world.range(*span);

                result.push(AnalysisNode {
                    kind: String::from("Shape"),
                    start: range.as_ref().map(|r| r.start),
                    end: range.as_ref().map(|r| r.end),
                    text: None,
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Image(_image, _axes, span) => {
                let range = world.range(*span);

                result.push(AnalysisNode {
                    kind: String::from("Image"),
                    start: range.as_ref().map(|r| r.start),
                    end: range.as_ref().map(|r| r.end),
                    text: None,
                    attrs: BTreeMap::new(),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Link(_destination, _axes) => result.push(AnalysisNode {
                kind: String::from("Link"),
                start: None,
                end: None,
                text: None,
                attrs: BTreeMap::new(),
                error: false,
                children: vec![],
            }),
            FrameItem::Tag(_tag) => result.push(AnalysisNode {
                kind: String::from("Tag"),
                start: None,
                end: None,
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
    world: &dyn World,
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diag| {
            let range = world.range(diag.span);

            Diagnostic {
                message: diag.message.to_string(),
                start: range.as_ref().map(|r| r.start),
                end: range.as_ref().map(|r| r.end),
            }
        })
        .collect()
}
