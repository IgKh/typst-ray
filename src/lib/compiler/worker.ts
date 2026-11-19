// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
import type { Request, Response } from "./types.ts"

import init, { Fonts, analyze } from "../../../wasm/typstray_wasm.js"
import wasmUrl from "../../../wasm/typstray_wasm_bg.wasm?url"

const fontsBase = `${import.meta.env.BASE_URL}fonts/`;

async function fetchBytes(url: string): Promise<Uint8Array> {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`Failed to fetch ${url}: ${res.status}`);
    return new Uint8Array(await res.arrayBuffer());
}

async function loadFonts() {
    const manifest: string[] = await (await fetch(`${fontsBase}manifest.json`)).json();
    return Promise.all(manifest.map((name) => fetchBytes(fontsBase + name)));
}

async function setup(): Promise<Fonts> {
    const [, fontData] = await Promise.all([
        init({ module_or_path: wasmUrl }),
        loadFonts(),
    ]);
    return new Fonts(fontData);
}

const ready = setup();

self.onmessage = async (e: MessageEvent<Request>) => {
    const { id, src } = e.data;
    let res: Response;
    try {
        const fonts = await ready;
        res = { id, ok: true, analysis: JSON.parse(analyze(src, fonts)) };
    }
    catch (err) {
        res = { id, ok: false, error: String(err) };
    }
    self.postMessage(res);
}
