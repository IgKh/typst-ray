// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
use typst::{World, comemo::Track};
use wasm_bindgen::prelude::*;

use crate::world::TypstRayWorld;

mod compiler;
mod model;
mod world;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| {
        error(&info.to_string());
    }));
}

#[wasm_bindgen]
pub fn analyze(src: &str) -> String {
    let world = TypstRayWorld::new(src);

    let result = compiler::compile((&world as &dyn World).track());
    let analysis = model::build_analysis(&result);

    serde_json::to_string(&analysis).unwrap()
}
