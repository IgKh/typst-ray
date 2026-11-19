<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    const ATTR_LIMIT = 3;

    let { attrs } = $props();

    let expanded = $state(false);

    const entries = $derived(Object.entries(attrs ?? {}));
    const shown = $derived(expanded ? entries : entries.slice(0, ATTR_LIMIT));
    const hidden = $derived(entries.length - shown.length);

    function toggle(event: MouseEvent) {
        // Prevent toggling the parent <details> when inside a <summary>
        event.preventDefault();
        event.stopPropagation();
        expanded = !expanded;
    }
</script>

{#if entries.length}
    <span class="attrs">
        {#each shown as [name, value] (name)}
            <span class="pill" title="{name}: {value}">
                <span class="name">{name}</span><bdi class="value">{value}</bdi>
            </span>
        {/each}
        {#if hidden > 0}
            <button class="pill more" type="button" onclick={toggle}>+{hidden}</button>
        {:else if expanded && entries.length > ATTR_LIMIT}
            <button class="pill more" type="button" onclick={toggle}>less</button>
        {/if}
    </span>
{/if}

<style>
    .attrs {
        display: inline-flex;
        flex-wrap: wrap;
        gap: 3px;
        margin-left: 6px;
        vertical-align: baseline;
    }

    .pill {
        display: inline-flex;
        gap: 4px;
        padding: 0 5px;
        border: 1px solid var(--line);
        border-radius: 8px;
        font-size: 11px;
        font-family: ui-monospace, monospace;
        line-height: 16px;
        white-space: nowrap;
    }

    .name {
        color: var(--muted);
    }

    .name::after {
        content: ": ";
    }

    .value {
        display: inline-block;
        max-width: 24ch;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        vertical-align: bottom;
    }

    .more {
        background: none;
        color: var(--muted);
        cursor: pointer;
    }

    .more:hover {
        background: var(--hl);
        color: var(--fg);
    }
</style>
