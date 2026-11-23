<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    import NodeAttributes from "./NodeAttributes.svelte";
    import TreeNode from "./TreeNode.svelte";

    let { node, onhover, onclick, depth = 0 } = $props();

    // svelte-ignore state_referenced_locally
    let open = $state(depth < 3);

    const hasChildren = $derived(!!node.children?.length);
</script>

{#if hasChildren}
    <details bind:open>
        <summary onmouseenter={() => onhover(node)} onmouseleave={() => onhover(null)}>
            <b class:error={node.error}>{node.kind}</b>
            {#if node.text}<bdi class="label" title={node.text}>{node.text}</bdi>{/if}
            <span class="range">{node.range?.start}..{node.range?.end}</span>
            <NodeAttributes attrs={node.attrs} />
        </summary>
        {#if open}
            <div class="children">
                {#each node.children as child}
                    <TreeNode node={child} {onhover} {onclick} depth={depth + 1} />
                {/each}
            </div>
        {/if}
    </details>
{:else}
    <div
        class="leaf"
        role="presentation"
        onmouseenter={() => onhover(node)}
        onmouseleave={() => onhover(null)}
        onclick={() => onclick(node)}
    >
        <b class:error={node.error}>{node.kind}</b>
        {#if node.text}<bdi class="label" title={node.text}>{node.text}</bdi>{/if}
        <span class="range">{node.range?.start}..{node.range?.end}</span>
        <NodeAttributes attrs={node.attrs} />
    </div>
{/if}

<style>
    .children {
        margin-left: 14px;
        border-left: 1px dotted var(--line);
        padding-left: 6px;
    }

    summary,
    .leaf {
        cursor: default;
        padding: 1px 2px;
    }

    .leaf {
        margin-left: 14px;
    }

    summary:hover,
    .leaf:hover {
        background: var(--hl);
    }

    .error {
        color: var(--error);
    }

    .label {
        color: var(--accent);
        display: inline-block;
        max-width: 40ch;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        vertical-align: bottom;
    }

    .range {
        color: var(--muted);
        font-size: 11px;
        margin-left: 6px;
    }
</style>
