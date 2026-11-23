// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
import type { Request, Response } from "./types.ts";

export interface Fonts {
    free(): void;
}

export interface CompilerModule {
    default(init: { module_or_path: string }): Promise<unknown>;
    Fonts: new (data: Uint8Array[]) => Fonts;
    analyze(src: string, fonts: Fonts): string;
}

const version = self.name;

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

async function loadCompiler(): Promise<CompilerModule> {
    let module: CompilerModule;
    try {
        module = await import(`../../../wasm/${version}/typstray_wasm.js`);
    } catch {
        throw new Error(`Unsupported Typst version: ${version}`);
    }

    const wasmUrl = new URL(`../../../wasm/${version}/typstray_wasm_bg.wasm`, import.meta.url);
    await module.default({ module_or_path: wasmUrl.href });
    return module;
}

async function setup(): Promise<{ compiler: CompilerModule; fonts: Fonts }> {
    const [compiler, fontData] = await Promise.all([loadCompiler(), loadFonts()]);
    return { compiler, fonts: new compiler.Fonts(fontData) };
}

const ready = setup();

self.onmessage = async (e: MessageEvent<Request>) => {
    const { id, src } = e.data;
    let res: Response;
    try {
        const { compiler, fonts } = await ready;
        res = { id, ok: true, analysis: JSON.parse(compiler.analyze(src, fonts)) };
    } catch (err) {
        res = { id, ok: false, error: String(err) };
    }
    self.postMessage(res);
};
