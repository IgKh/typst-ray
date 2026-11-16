<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import {EditorState, Prec, StateEffect, StateField} from "@codemirror/state";
    import {EditorView, Decoration, keymap} from "@codemirror/view";
    import {basicSetup} from "codemirror";

    import type {DecorationSet} from "@codemirror/view";
    import type {SourceRange} from "./compiler/types";

    let {
        value = $bindable(''),
        onsubmit
    } = $props();

    let view: EditorView;

    const setHighlight = StateEffect.define<SourceRange | null>({
        map(range, change) {
            if (!range) {
                return range;
            }
            return {
                start: change.mapPos(range.start),
                end: change.mapPos(range.end)
            };
        }
    });

    const highlightMark = Decoration.mark({
        class: "cm-hl"
    });

    const highlightField = StateField.define<DecorationSet>({
        create() {
            return Decoration.none;
        },
        update(decorationSet, tx) {
            for (let effect of tx.effects) {
                if (effect.is(setHighlight)) {
                    const range = effect.value;
                    if (range && range.start < range.end) {
                        return Decoration.set([highlightMark.range(range.start, range.end)]);
                    } else {
                        return Decoration.none;
                    }
                }
            }
            return decorationSet.map(tx.changes);
        },
        provide(field) {
            return EditorView.decorations.from(field);
        }
    });

    export function highlightRange(range: SourceRange | null) {
        view?.dispatch({
            effects: setHighlight.of(range)
        });
    }

    function codemirror(node: Element) {
        const state = EditorState.create({
            doc: value,
            extensions: [
                Prec.highest(keymap.of([
                    {
                        key: "Mod-Enter",
                        run: () => {
                            onsubmit?.();
                            return true;
                        }
                    }
                ])),
                basicSetup,
                highlightField,
                EditorView.updateListener.of((update) => {
                    if (update.docChanged) {
                        value = update.state.doc.toString();
                    }
                })
            ]
        });

        view = new EditorView({
            state,
            parent: node,
        });

        return {
            destroy() {
                view?.destroy();
            }
        }
    }

    $effect(() => {
        if (view && value !== view.state.doc.toString()) {
            view.dispatch({
				changes: {
                    from: 0,
                    to: view.state.doc.length,
                    insert: value
                }
			});
        }
    })
</script>

<div class="editor-container" use:codemirror></div>

<style>
    .editor-container {
        height: 100%;
        min-height: 0;
    }

    .editor-container :global(.cm-editor) {
        height: 100%;
    }

    .editor-container :global(.cm-scroller) {
        overflow: auto;
    }

    :global(.cm-hl) {
        background: var(--hl);
    }
</style>
