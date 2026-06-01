<script lang="ts">
  import type { NoteTreeNode } from '../types';
  import TreeItem from './TreeItem.svelte';

  let {
    node,
    onselect,
    ondelete,
    oncreate,
  }: {
    node: NoteTreeNode;
    onselect: (id: string) => void;
    ondelete: (id: string, e: MouseEvent) => void;
    oncreate: (parentId: string | null) => void;
  } = $props();

  let expanded = $state(false);
  let hasChildren = $derived(node.children.length > 0);
</script>

<div class="tree-item">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="item-row" onclick={() => onselect(node.id)} role="button" tabindex="-1">
    {#if hasChildren}
      <button class="toggle" onclick={(e) => { e.stopPropagation(); expanded = !expanded; }}>
        {expanded ? '▾' : '▸'}
      </button>
    {:else}
      <span class="toggle spacer"></span>
    {/if}
    <span class="item-title">{node.title || 'Untitled'}</span>
    <span class="item-actions">
      <button class="action" onclick={(e) => { e.stopPropagation(); oncreate(node.id); }} title="Add child">+</button>
      <button class="action" onclick={(e) => ondelete(node.id, e)} title="Delete">×</button>
    </span>
  </div>
  {#if expanded && hasChildren}
    <div class="children">
      {#each node.children as child}
        <TreeItem node={child} {onselect} {ondelete} {oncreate} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-item {
    user-select: none;
  }
  .item-row {
    display: flex;
    align-items: center;
    padding: 4px 8px 4px 16px;
    cursor: pointer;
    font-size: 13px;
    border-radius: 4px;
    margin: 1px 8px;
    gap: 4px;
  }
  .item-row:hover {
    background: var(--accent);
  }
  .toggle {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 11px;
    padding: 0;
    width: 16px;
    flex-shrink: 0;
  }
  .toggle:hover {
    color: var(--highlight);
  }
  .spacer {
    width: 16px;
  }
  .item-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .item-actions {
    display: none;
    gap: 2px;
  }
  .item-row:hover .item-actions {
    display: flex;
  }
  .action {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
    line-height: 1;
  }
  .action:hover {
    color: var(--text-primary);
  }
  .children {
    padding-left: 8px;
    border-left: 1px solid var(--border);
    margin-left: 23px;
  }
</style>
