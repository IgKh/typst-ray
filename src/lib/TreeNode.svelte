<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import TreeNode from "./TreeNode.svelte";

    let {
        node,
        depth = 0
    } = $props();

    let open = $state(true);

    const hasChildren = $derived(!!node.children?.length);
</script>

{#if hasChildren}
    <details bind:open>
        <summary>
            <b>{node.kind}</b> <span class="label">{node.text}</span>
            <span class="range">{node.start}..{node.end}</span>
        </summary>
        {#if open}
            <div class="children">
                {#each node.children as child}
                    <TreeNode node={child} depth={depth + 1} />
                {/each}
            </div>
        {/if}
    </details>
{:else}
    <div class="leaf" role="presentation">
        <b>{node.kind}</b> <span class="label">{node.text}</span>
        <span class="range">{node.start}..{node.end}</span>
    </div>
{/if}

<style>
    .children {
        margin-left: 14px;
        border-left: 1px dotted var(--line);
        padding-left: 6px;
    }

    summary, .leaf {
        cursor: default;
        padding: 1px 2px;
    }

    .leaf {
        margin-left: 14px;
    }

    summary:hover, .leaf:hover {
        background: var(--hl);
    }

    .label {
        color: var(--accent);
    }

    .range {
        color: var(--muted);
        font-size: 11px;
        margin-left: 6px;
    }
</style>
