<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import {EditorState} from "@codemirror/state";
    import {EditorView, basicSetup} from "codemirror";

    let {
        value = $bindable('')
    } = $props();

    let view: EditorView;

    function codemirror(node: Element) {
        const state = EditorState.create({
            doc: value,
            extensions: [
                basicSetup,
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
