<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import Diagnostics from "./lib/Diagnostics.svelte";
    import Editor from "./lib/Editor.svelte";
    import TreeNode from "./lib/TreeNode.svelte";

    import { analyze } from "./lib/compiler/client";
    import { DEFAULT_VERSION, VERSIONS } from "./lib/compiler/versions";
    import {
        clearSharingParam,
        getInitialSource,
        makeSharingUrl,
        saveSource,
    } from "./lib/util/source";
    import type { Analysis, AnalysisNode, Diagnostic } from "./lib/compiler/types";

    let { source: initialSource, shared: wasShared } = getInitialSource();

    let src = $state(initialSource);
    let compiledSrc = $state<string | null>(null);
    let typstVersion = $state(DEFAULT_VERSION);
    let compiledVersion = $state<string | null>(null);
    let sharedSrc = $state<string | null>(wasShared ? initialSource : null);
    let busy = $state(false);
    let result = $state<Analysis | null>(null);
    let systemError = $state<string | null>(null);
    let activeTab = $state("ast");
    let activeLayoutIndex = $state(0);
    let stale = $derived(
        result !== null && (src !== compiledSrc || typstVersion !== compiledVersion)
    );
    let shareLabel = $state("Share");

    let editor: Editor;

    const repoUrl = "https://github.com/IgKh/typst-ray";
    const revision = __GIT_REVISION__;
    const sourceUrl = revision ? `${repoUrl}/tree/${revision}` : repoUrl;

    $effect(() => {
        const current = src;
        if (current === sharedSrc) {
            // This prevents merely opening a shared link from overriding
            // the local storage
            return;
        }
        if (sharedSrc !== null) {
            sharedSrc = null;
            clearSharingParam();
        }
        saveSource(current);
    });

    async function share() {
        try {
            await navigator.clipboard.writeText(makeSharingUrl(src));
            shareLabel = "Copied!";
        } catch {
            shareLabel = "Failed";
        }
        setTimeout(() => (shareLabel = "Share"), 1500);
    }

    async function compile() {
        if (busy) {
            return;
        }

        busy = true;
        const source = src;
        const version = typstVersion;
        try {
            result = await analyze(source, version);
            compiledSrc = source;
            compiledVersion = version;
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
    <a href="/" class="brand">
        <img src="/favicon.svg" alt="" width="22" height="22" />
        <h1><span style="color: #239dad">Typst</span>Ray</h1>
    </a>
    <label class="version">
        Typst version
        <select bind:value={typstVersion} title="Typst compiler version">
            {#each VERSIONS as v}
                <option value={v.version}>{v.description}</option>
            {/each}
        </select>
    </label>
    <a
        class="github"
        href={repoUrl}
        target="_blank"
        title="Project page on GitHub"
        aria-label="Project page on GitHub"
    >
        <svg width="22" height="22" aria-hidden="true">
            <use href="/icons.svg#github-icon" />
        </svg>
    </a>
</header>

<main>
    <section id="editor-section">
        <nav>
            <button
                class:stale={stale && !busy}
                onclick={compile}
                disabled={busy}
                title={stale
                    ? "Source or Typst version changed since last compile (Ctrl+Enter)"
                    : "Compile (Ctrl+Enter)"}
            >
                <svg width="12" height="12" aria-hidden="true">
                    <use href="/icons.svg#play-icon" />
                </svg>
                {busy ? "Compiling..." : "Compile"}
            </button>
            <button onclick={share} title="Copy a shareable link to the clipboard">
                {shareLabel}
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
    <span>
        TypstRay is free software, under the terms of the
        <a href="https://choosealicense.com/licenses/agpl-3.0/" target="_blank">AGPL 3.0</a>
        or later
    </span>
    <a href={sourceUrl}>
        Source{revision ? ` (${revision.slice(0, 7)})` : ""}
    </a>
    <a href="/licenses.html" target="_blank">Licenses</a>
    <span class="stretch"></span>
    <span>Not affiliated with Typst GmbH</span>
    <span>Made in 🇮🇱 with ❤️</span>
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

    .brand {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 22px;
        color: inherit;
        text-decoration: none;
    }

    .brand img {
        flex: none;
        transform: translateY(0.16em);
    }

    .version {
        display: flex;
        align-items: center;
        gap: 6px;
        margin-left: auto;
        color: var(--muted);
    }

    .version select {
        font: inherit;
        color: var(--fg);
        background: var(--bg);
        border: 1px solid var(--line);
        border-radius: 4px;
        padding: 3px 6px;
        cursor: pointer;
    }

    .version select:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 1px;
    }

    .github {
        display: flex;
        color: var(--fg);
    }

    header h1 {
        font-size: inherit;
        font-weight: 700;
        letter-spacing: -0.02em;
        line-height: 1;
        margin: 0;
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
        gap: 16px;
    }

    footer .stretch {
        flex: 1;
    }

    footer a {
        color: inherit;
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
