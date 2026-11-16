<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import Editor from "./lib/Editor.svelte";
    import TreeNode from "./lib/TreeNode.svelte";

    import { analyze } from "./lib/compiler/client"
    import type { Analysis } from "./lib/compiler/types";

    let src = $state("FOO");
    let result = $state<Analysis | null>(null);
    let busy = $state(false);
    let activeTab = $state("ast");
    let activeLayoutIndex = $state(0);

    async function compile() {
        busy = true;
        try {
            result = await analyze(src);
            if (activeLayoutIndex >= result.layouts.length) {
                activeLayoutIndex = Math.max(0, result.layouts.length - 1);
            }
        }
        catch (err) {
            // TODO
        }
        finally {
            busy = false;
        }
    }
</script>

<header>
    <h1>TypstRay</h1>
</header>

<main>
    <section id="editor-section">
        <nav>
            <!-- TODO: Improve look, make Ctrl+Enter trigger it -->
            <button onclick={compile} disabled={busy}>{busy ? "Compiling..." : "Compile"}</button>
        </nav>
        <div id="editor-wrapper" style="flex: 1">
            <Editor bind:value={src} />
        </div>
    </section>

    <section id="results-section">
        <nav>
            <button class:active={activeTab === 'ast'}
                    onclick={() => (activeTab = 'ast')}>
                AST
            </button>
            <button class:active={activeTab === 'content'}
                    onclick={() => (activeTab = 'content')}>
                Content
            </button>
            {#each result?.layouts ?? [] as _, index}
                <button class:active={activeTab === 'layout' && activeLayoutIndex === index}
                        onclick={() => (activeTab = 'layout', activeLayoutIndex = index)}>
                    Layout #{index + 1}
                </button>
            {/each}
        </nav>
        <div class="result-panel">
            {#if result}
                {#if activeTab === 'ast'}
                    <TreeNode node={result.ast} />
                {:else if activeTab === 'content'}
                    <TreeNode node={result.content} />
                {:else if activeTab === 'layout'}
                    {#key activeLayoutIndex}
                        <TreeNode node={result.layouts[activeLayoutIndex]} />
                    {/key}
                {/if}
            {:else}
                <!-- TODO put pending message -->
            {/if}
        </div>
    </section>
</main>

<style>
    header {
        display: flex;
        gap: 16px;
        align-items: center;
        padding: 8px 16px;
        border-bottom: 1px solid var(--line);
    }

    header h1 {
        font-size: 16px;
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

    #editor-section {
        border-right: 1px solid var(--line);
        min-height: 0;
        min-width: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    #results-section {
        min-height: 0;
        min-width: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    #results-section nav {
        display: flex;
        border-bottom: 1px solid var(--line);
    }

    #results-section nav button {
        background: none;
        border: 0;
        border-bottom: 2px solid transparent;
        padding: 8px 14px;
        color: var(--fg);
        cursor: pointer;
        font: inherit;
    }

    #results-section nav button.active {
        border-bottom-color: var(--accent);
        color: var(--accent);
    }

    .result-panel {
        flex: 1;
        overflow: auto;
        padding: 8px 12px;
        font: 13px ui-monospace, monospace;
    }
</style>
