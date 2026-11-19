<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import Diagnostics from "./lib/Diagnostics.svelte";
    import Editor from "./lib/Editor.svelte";
    import TreeNode from "./lib/TreeNode.svelte";

    import { analyze } from "./lib/compiler/client";
    import type { Analysis, AnalysisNode, Diagnostic } from "./lib/compiler/types";

    let src = $state("FOO");
    let compiledSrc = $state<string | null>(null);
    let busy = $state(false);
    let result = $state<Analysis | null>(null);
    let systemError = $state<string | null>(null);
    let activeTab = $state("ast");
    let activeLayoutIndex = $state(0);
    let stale = $derived(result !== null && src !== compiledSrc);

    let editor: Editor;

    async function compile() {
        if (busy) {
            return;
        }

        busy = true;
        const source = src;
        try {
            result = await analyze(source);
            compiledSrc = source;
            systemError = null;

            if (activeLayoutIndex >= result.layouts.length) {
                activeLayoutIndex = Math.max(0, result.layouts.length - 1);
            }
            if (!result.content || (activeTab === "layout" && result.layouts.length === 0)) {
                activeTab = "ast";
            }

            editor.setCompilerDiagnostics(result.diagnostics);
        } catch (err) {
            if (err instanceof Error) {
                systemError = err.message;
            } else {
                systemError = String(err);
            }
        } finally {
            busy = false;
        }
    }

    function onTreeNodeHover(node: AnalysisNode) {
        const range = $state.snapshot(node?.range);
        editor.highlightRange(range);
    }

    function onTreeNodeClick(node: AnalysisNode) {
        const range = $state.snapshot(node?.range);
        editor.scrollToRange(range, false);
    }

    function onDiagnosticClick(diag: Diagnostic) {
        editor.scrollToRange(diag.range, true);
    }
</script>

<header>
    <h1><span style="color: #239dad">Typst</span>Ray</h1>
</header>

<main>
    <section id="editor-section">
        <nav>
            <button
                class:stale={stale && !busy}
                onclick={compile}
                disabled={busy}
                title={stale
                    ? "Source changed since last compile (Ctrl+Enter)"
                    : "Compile (Ctrl+Enter)"}
            >
                <svg width="12" height="12" aria-hidden="true">
                    <use href="/icons.svg#play-icon" />
                </svg>
                {busy ? "Compiling..." : "Compile"}
            </button>
        </nav>
        <div id="editor-wrapper">
            <Editor onsubmit={compile} bind:value={src} bind:this={editor} />
        </div>
        <div id="diagnostics-wrapper" role="region" aria-label="Diagnostics">
            <Diagnostics
                diagnostics={result?.diagnostics}
                {systemError}
                onclick={onDiagnosticClick}
            />
        </div>
    </section>

    <section id="results-section">
        <nav>
            <button
                class:active={activeTab === "ast"}
                onclick={() => (activeTab = "ast")}
                disabled={!result}
            >
                AST
            </button>
            <button
                class:active={activeTab === "content"}
                onclick={() => (activeTab = "content")}
                disabled={!result?.content}
            >
                Content
            </button>
            {#each result?.layouts ?? [] as _, index}
                <button
                    class:active={activeTab === "layout" && activeLayoutIndex === index}
                    onclick={() => ((activeTab = "layout"), (activeLayoutIndex = index))}
                >
                    Layout #{index + 1}
                </button>
            {:else}
                <button disabled>Layout</button>
            {/each}
        </nav>
        <div class="result-panel">
            {#if result}
                {#if activeTab === "ast"}
                    <TreeNode
                        node={result.ast}
                        onhover={onTreeNodeHover}
                        onclick={onTreeNodeClick}
                    />
                {:else if activeTab === "content"}
                    <TreeNode
                        node={result.content}
                        onhover={onTreeNodeHover}
                        onclick={onTreeNodeClick}
                    />
                {:else if activeTab === "layout"}
                    {#key activeLayoutIndex}
                        <TreeNode
                            node={result.layouts[activeLayoutIndex]}
                            onhover={onTreeNodeHover}
                            onclick={onTreeNodeClick}
                        />
                    {/key}
                {/if}
            {:else}
                <span style="color: var(--muted)">Please click on Compile</span>
            {/if}
        </div>
    </section>
</main>

<footer>
    <!-- TODO put links, version, etc -->
</footer>

<style>
    header {
        display: flex;
        gap: 16px;
        align-items: center;
        box-sizing: border-box;
        height: 56px;
        padding: 0 20px;
        background: color-mix(in srgb, var(--fg) 5%, var(--bg));
        border-bottom: 1px solid var(--line);
    }

    footer {
        display: flex;
        align-items: center;
        box-sizing: border-box;
        flex: none;
        height: 28px;
        padding: 0 20px;
        background: color-mix(in srgb, var(--fg) 5%, var(--bg));
        border-top: 1px solid var(--line);
        color: var(--muted);
        font-size: 12px;
    }

    header h1 {
        font-size: 22px;
        font-weight: 700;
        letter-spacing: -0.02em;
        margin: 0;
        flex: 1;
    }

    main {
        flex: 1;
        min-height: 0;
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
        grid-template-rows: minmax(0, 1fr);
    }

    nav {
        display: flex;
        align-items: stretch;
        box-sizing: border-box;
        height: 40px;
        flex: none;
        border-bottom: 1px solid var(--line);
    }

    nav button {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        background: none;
        border: 0;
        border-bottom: 2px solid transparent;
        padding: 0 14px;
        color: var(--fg);
        cursor: pointer;
        font: inherit;
    }

    nav button:hover:not(:disabled),
    nav button:focus-visible,
    nav button:active:not(:disabled) {
        color: var(--accent);
    }

    nav button:disabled {
        color: var(--muted);
        cursor: default;
    }

    nav button.stale::after {
        content: "";
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: currentColor;
    }

    nav button.active {
        border-bottom-color: var(--accent);
        color: var(--accent);
    }

    #editor-section {
        border-right: 1px solid var(--line);
        min-height: 0;
        min-width: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    #editor-wrapper {
        flex: 1;
        min-height: 0;
        height: 100%;
    }

    #diagnostics-wrapper {
        flex: none;
        max-height: 30%;
        overflow: auto;
    }

    #results-section {
        min-height: 0;
        min-width: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .result-panel {
        flex: 1;
        overflow: auto;
        padding: 8px 12px;
        font:
            13px ui-monospace,
            monospace;
    }
</style>
