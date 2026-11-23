// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};

use typst::{
    ecow::EcoVec,
    foundations::{Content, Repr, Value},
    introspection::Tag,
    layout::{Frame, FrameItem},
    visualize::FixedStroke,
};
use typst_layout::PagedDocument;
use typst_syntax::{DiagSpan, DiagSpanKind, LinkedNode, Source};
use typstray_common::{Analysis, AnalysisNode, Diagnostic, SourceRange};

use crate::compiler::CompilationResult;

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

    let diagnostics = convert_diagnostics(result.diagnostics.clone(), source);

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
            for (idx, item) in arr.iter().enumerate() {
                collect_content_attributes(
                    &format!("{name}[{idx}]"),
                    item,
                    source,
                    attrs,
                    children,
                );
            }
        }
        Value::Dict(dict) => {
            for (key, value) in dict {
                collect_content_attributes(
                    &format!("{name}.{key}"),
                    value,
                    source,
                    attrs,
                    children,
                );
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

    let text = attrs
        .remove("text")
        .map(|s| s.trim_prefix("\"").trim_suffix("\"").to_string());

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
                let mut ranges = text
                    .glyphs
                    .iter()
                    .filter_map(|g| process_span(g.span.0, source))
                    .peekable();

                let range = if ranges.peek().is_some() {
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
                    attrs: get_text_item_attrs(text),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Shape(shape, span) => {
                let range = process_span(*span, source);

                result.push(AnalysisNode {
                    kind: String::from("Shape"),
                    range,
                    text: None,
                    attrs: get_shape_attrs(shape),
                    error: false,
                    children: vec![],
                });
            }
            FrameItem::Image(_image, _, span) => {
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
            FrameItem::Link(destination, _) => result.push(AnalysisNode {
                kind: String::from("Link"),
                range: None,
                text: None,
                attrs: get_link_attrs(destination),
                error: false,
                children: vec![],
            }),
            FrameItem::Tag(Tag::Start(content, _)) => result.push(AnalysisNode {
                kind: String::from("Tag::Start"),
                range: process_span(content.span(), source),
                text: None,
                attrs: get_start_tag_attrs(content),
                error: false,
                children: vec![],
            }),
            FrameItem::Tag(Tag::End(_, _, _)) => result.push(AnalysisNode {
                kind: String::from("Tag::End"),
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

fn get_text_item_attrs(item: &typst::text::TextItem) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();

    let font = item.font.font().info();

    attrs.insert(
        String::from("font"),
        format!("{} ({:?})", font.family, font.variant.style),
    );
    attrs.insert(String::from("size"), item.size.repr().to_string());
    attrs.insert(String::from("lang"), item.lang.as_str().to_string());

    if item.fill != typst::visualize::Paint::Solid(typst::visualize::Color::BLACK) {
        attrs.insert(String::from("fill"), item.fill.repr().to_string());
    }
    if let Some(ref stroke) = item.stroke {
        attrs.insert(String::from("stroke"), describe_stroke(stroke));
    }
    if let Some(ref region) = item.region {
        attrs.insert(String::from("region"), region.as_str().to_string());
    }
    if !item.glyphs.is_empty() {
        attrs.insert(String::from("glyphs"), format!("{}", item.glyphs.len()));
    }

    attrs
}

fn get_shape_attrs(shape: &typst::visualize::Shape) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();

    let geometry = match shape.geometry {
        typst::visualize::Geometry::Line(_) => String::from("Line"),
        typst::visualize::Geometry::Rect(_) => String::from("Rect"),
        typst::visualize::Geometry::Curve(_) => String::from("Curve"),
    };

    attrs.insert(String::from("geometry"), geometry);

    if let Some(ref fill) = shape.fill {
        attrs.insert(String::from("fill"), fill.repr().to_string());
    }
    if let Some(ref stroke) = shape.stroke {
        attrs.insert(String::from("stroke"), describe_stroke(stroke));
    }

    attrs
}

fn get_link_attrs(destination: &typst::model::Destination) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();

    attrs.insert(String::from("destination"), destination.repr().to_string());

    attrs
}

fn get_start_tag_attrs(content: &Content) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();

    attrs.insert(String::from("elem"), content.elem().name().to_string());

    if let Some(label) = content.label() {
        attrs.insert(String::from("label"), label.resolve().to_string());
    }

    attrs
}

fn describe_stroke(stroke: &FixedStroke) -> String {
    // FIXME do better than this
    format!("{:?}", stroke)
}

fn convert_diagnostics(
    mut diagnostics: EcoVec<typst::diag::SourceDiagnostic>,
    source: &Source,
) -> Vec<Diagnostic> {
    // Dedup diagnostics
    let mut unique = HashSet::new();
    diagnostics.retain(|diag| {
        let hash = typst::utils::hash128(&(&diag.span, &diag.message));
        unique.insert(hash)
    });

    diagnostics
        .into_iter()
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
