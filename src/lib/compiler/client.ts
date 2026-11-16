// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
import type { Analysis, Response } from './types'

const TIMEOUT_MS = 5000;

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, { resolve: (a: Analysis) => void; reject: (e: Error) => void }>();

function spawn(): Worker {
    const w = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

    w.onmessage = (e: MessageEvent<Response>) => {
        const p = pending.get(e.data.id);
        if (!p) {
            // Request is stale
            return;
        }
        pending.delete(e.data.id);

        if (e.data.ok) {
            p.resolve(e.data.analysis);
        }
        else {
            p.reject(new Error(e.data.error));
        }
    };
    w.onerror = () => kill('Compiler crashed');

    return w;
}

function kill(reason: string) {
    worker?.terminate();
    worker = null;
    for (const p of pending.values()) {
        p.reject(new Error(reason));
    }
    pending.clear();
}

export function analyze(src: string): Promise<Analysis> {
    worker ??= spawn();

    const id = nextId++;

    return new Promise<Analysis>((resolve, reject) => {
        const timer = setTimeout(() => kill('Compilation timed out'), TIMEOUT_MS);
        pending.set(id, {
            resolve: (a) => {
                clearTimeout(timer);
                resolve(a);
            },
            reject: (e) => {
                clearTimeout(timer);
                reject(e);
            }
        });
        worker!.postMessage({ id, src });
    });
}
