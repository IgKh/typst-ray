<!-- SPDX-FileCopyrightText: 2026 Igor Khanin -->
<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<script lang="ts">
    let {
        diagnostics,
        systemError,
        onclick
    } = $props();
</script>

{#if systemError || diagnostics?.length > 0}
    <ul class="diagnostics">
        {#if systemError}
            <li>
                <button type="button" disabled>
                    <span class="badge" aria-label="Error">!</span>
                    <span class="message">{systemError}</span>
                </button>
            </li>
        {/if}
        {#each diagnostics ?? [] as diag}
            <li>
                <button type="button" disabled={!diag.range} onclick={() => (onclick(diag))}>
                    <span class="badge" aria-label="Error">!</span>
                    <span class="message">{diag.message}</span>
                    {#if diag.range}
                        <span class="range">{diag.range.start}..{diag.range.end}</span>
                    {/if}
                </button>
            </li>
        {/each}
    </ul>
{/if}

<style>
    .diagnostics {
        list-style: none;
        margin: 0;
        padding: 0;
        border-top: 1px solid var(--line);
        font: 13px ui-monospace, monospace;
    }

    li {
        border-top: 1px solid var(--line);
    }

    button {
        display: flex;
        align-items: baseline;
        gap: 8px;
        width: 100%;
        box-sizing: border-box;
        padding: 6px 12px;
        background: none;
        border: 0;
        color: var(--fg);
        font: inherit;
        text-align: left;
        cursor: pointer;
    }

    button:hover:not(:disabled),
    button:focus-visible {
        background: color-mix(in srgb, var(--fg) 6%, var(--bg));
    }

    button:disabled {
        cursor: default;
    }

    .badge {
        flex: none;
        width: 14px;
        height: 14px;
        line-height: 14px;
        border-radius: 50%;
        background: var(--error);
        color: var(--bg);
        font-size: 11px;
        font-weight: 700;
        text-align: center;
    }

    .message {
        flex: 1;
        min-width: 0;
        white-space: pre-wrap;
        overflow-wrap: anywhere;
    }

    .range {
        color: var(--muted);
        font-size: 11px;
    }
</style>
