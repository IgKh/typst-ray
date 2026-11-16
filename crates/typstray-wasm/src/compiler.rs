// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use typst::{
    World,
    comemo::{Track, Tracked},
    ecow::EcoVec,
    engine::{Route, Sink, Traced},
    foundations::{StyleChain, Target, TargetElem},
    introspection::Introspector,
    utils::Protected,
};
use typst_layout::PagedDocument;

pub struct CompilationResult {
    pub source: typst_syntax::Source,
    pub content: Option<typst::foundations::Content>,
    pub documents: Vec<PagedDocument>,
    pub diagnostics: EcoVec<typst::diag::SourceDiagnostic>,
}

pub fn compile(world: Tracked<dyn World + '_>) -> CompilationResult {
    let source = world.source(world.main()).unwrap();
    let library = world.library();

    let traced = Traced::default();
    let mut sink = Sink::new();

    //
    // Evaluate
    //

    let route = Route::default();
    let module = typst_eval::eval(
        world,
        library,
        traced.track(),
        sink.track_mut(),
        route.track(),
        &source,
    );

    let content = match module {
        Ok(module) => module.content(),
        Err(diagnostics) => {
            return CompilationResult {
                source,
                content: None,
                documents: vec![],
                diagnostics,
            };
        }
    };

    //
    // Realize + Layout
    //

    let base = StyleChain::new(&world.library().styles);
    let target = TargetElem::target.set(Target::Paged).wrap();
    let styles = base.chain(&target);
    let empty_introspector = typst::introspection::EmptyIntrospector;

    let mut history: Vec<PagedDocument> = vec![];
    let mut document: PagedDocument;

    loop {
        let introspector = history
            .last()
            .map(|doc| doc.introspector().as_ref() as &dyn Introspector)
            .unwrap_or(&empty_introspector);

        let constraint = typst::comemo::Constraint::new();

        let mut subsink = Sink::new();
        let mut engine = typst::engine::Engine {
            library,
            world,
            introspector: Protected::new(introspector.track_with(&constraint)),
            traced: traced.track(),
            sink: subsink.track_mut(),
            route: Route::default(),
        };

        document = match typst_layout::layout_document(&mut engine, &content, styles) {
            Ok(document) => document,
            Err(diagnostics) => {
                return CompilationResult {
                    source,
                    content: Some(content),
                    documents: history,
                    diagnostics,
                };
            }
        };

        if constraint.validate(document.introspector().as_ref() as &dyn Introspector) {
            sink.extend_from_sink(subsink);
            history.push(document);
            break;
        }

        if history.len() >= 4 {
            // TODO return an error
            sink.extend_from_sink(subsink);
            break;
        }

        history.push(document);
    }

    CompilationResult {
        source,
        content: Some(content),
        documents: history,
        diagnostics: EcoVec::new(),
    }
}
