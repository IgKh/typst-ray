// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
import type { Request, Response } from "./types.ts"

import init, { analyze } from "../../../wasm/typstray_wasm.js"
import wasmUrl from "../../../wasm/typstray_wasm_bg.wasm?url"

const ready = init({ module_or_path: wasmUrl });

self.onmessage = async (e: MessageEvent<Request>) => {
    const { id, src } = e.data;
    let res: Response;
    try {
        await ready;
        res = { id, ok: true, analysis: JSON.parse(analyze(src)) };
    }
    catch (err) {
        res = { id, ok: false, error: String(err) };
    }
    self.postMessage(res);
}
