// SPDX-FileCopyrightText: 2026 Igor Khanin
// SPDX-License-Identifier: AGPL-3.0-or-later
export interface SourceRange {
    start: number,
    end: number
};

export interface AnalysisNode {
    kind: string,
    range: SourceRange | null,
    text: string | null,
    attrs: Record<string, string>,
    error: boolean,
    children: [AnalysisNode]
};

export interface Analysis {
    ast: AnalysisNode,
    content: AnalysisNode | null,
    layouts: [AnalysisNode]
};

export type Request = {
    id: number;
    src: string;
};

export type Response =
    { id: number; ok: true; analysis: Analysis } |
    { id: number; ok: false; error: string };
