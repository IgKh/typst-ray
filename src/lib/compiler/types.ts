// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
export interface AnalysisNode {
    kind: string,
    start: number | null,
    end: number | null,
    text: string | null,
    error: boolean,
    children: [AnalysisNode]
};

export interface Analysis {
    ast: AnalysisNode,
    content: AnalysisNode | null,
    layouts: [AnalysisNode]
}

export type Request = {
    id: number;
    src: string;
};

export type Response =
    { id: number; ok: true; analysis: Analysis } |
    { id: number; ok: false; error: string };
