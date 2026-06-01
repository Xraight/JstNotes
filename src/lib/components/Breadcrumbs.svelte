<script lang="ts">
  import type { Breadcrumb } from '../types';

  let { items }: { items: Breadcrumb[] } = $props();
</script>

<nav class="breadcrumbs">
  <span class="crumb-root">📚</span>
  {#each items as crumb, i}
    <span class="crumb-sep">▸</span>
    <span class="crumb" class:crumb-last={i === items.length - 1}>
      {crumb.title}
    </span>
  {/each}
  {#if items.length === 0}
    <span class="crumb-empty">No note selected</span>
  {/if}
</nav>

<style>
  .breadcrumbs {
    display: flex;
    align-items: center;
    padding: 6px 20px;
    font-size: 13px;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    gap: 2px;
    min-height: 32px;
  }
  .crumb-root {
    font-size: 14px;
    opacity: 0.6;
    margin-right: 2px;
  }
  .crumb-sep {
    font-size: 11px;
    opacity: 0.4;
    margin: 0 4px;
  }
  .crumb {
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }
  .crumb-last {
    color: var(--text-primary);
    font-weight: 500;
  }
  .crumb-empty {
    color: var(--text-secondary);
    opacity: 0.5;
    font-style: italic;
  }
</style>
