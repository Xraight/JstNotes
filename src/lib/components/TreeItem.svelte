<script lang="ts" module>
  /**
   * Module-level drag state — shared across all TreeItem instances.
   * When the user mousedowns on an item, the drag note ID and start position
   * are stored here. On mouseup over any other item, the drop handler checks
   * if movement exceeded the threshold (4px) and moves the note.
   * This avoids the HTML5 Drag API which doesn't work in WebKitGTK (Tauri Linux).
   */
  let dragNoteId = '';
  let dragStartX = 0;
  let dragStartY = 0;
  let ghost: HTMLDivElement | null = null;

  export function showGhost(x: number, y: number, title: string) {
    if (!ghost) {
      ghost = document.createElement('div');
      ghost.className = 'drag-ghost';
      document.body.appendChild(ghost);
    }
    ghost.textContent = title;
    ghost.style.left = (x + 12) + 'px';
    ghost.style.top = (y - 20) + 'px';
    ghost.style.display = 'block';
  }

  function hideGhost() {
    if (ghost) {
      ghost.style.display = 'none';
    }
  }
</script>

<script lang="ts">
  import type { NoteTreeNode } from '../types';
  import TreeItem from './TreeItem.svelte';

  let {
    node,
    onselect,
    ondelete,
    oncreate,
    onmove,
  }: {
    node: NoteTreeNode;
    onselect: (id: string) => void;
    ondelete: (id: string, e: MouseEvent) => void;
    oncreate: (parentId: string | null) => void;
    onmove: (id: string, targetParentId: string | null, e: MouseEvent) => void;
  } = $props();

  let expanded = $state(false);
  let hasChildren = $derived(node.children.length > 0);
  let hoverTarget = $state(false);

  function handleMouseDown(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.action')) return; // Don't start drag from action buttons
    dragNoteId = node.id;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    showGhost(e.clientX, e.clientY, node.title);
  }

  function handleMouseUp(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.action')) return;
    const dx = Math.abs(e.clientX - dragStartX);
    const dy = Math.abs(e.clientY - dragStartY);
    if (dragNoteId && dragNoteId !== node.id && (dx > 4 || dy > 4)) {
      onmove(dragNoteId, node.id, e);
      dragNoteId = '';
      hideGhost();
      return; // Don't clear dragNoteId yet, we want onclick to know it was a drag
    }
    dragNoteId = '';
    hideGhost();
  }

  // onclick: only select if no drag happened (dragNoteId was just cleared by mouseup without a move)
  function handleClick() {
    onselect(node.id);
  }
</script>

<div class="tree-item">
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div
    class="item-row"
    class:drag-target={hoverTarget}
    data-note-id={node.id}
    onmousedown={handleMouseDown}
    onmouseup={handleMouseUp}
    onmouseenter={() => { if (dragNoteId && dragNoteId !== node.id) hoverTarget = true; }}
    onmouseleave={() => { hoverTarget = false; }}
    onclick={handleClick}
    role="button"
    tabindex="-1"
  >
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
        <TreeItem node={child} {onselect} {ondelete} {oncreate} {onmove} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-item { user-select: none; }
  .item-row {
    display: flex; align-items: center; padding: 4px 8px 4px 16px;
    font-size: 13px; border-radius: 4px; margin: 1px 8px;
    gap: 4px; transition: background 0.1s; user-select: none; -webkit-user-select: none;
    cursor: default;
  }
  .item-row:hover { background: var(--accent); }
  .item-row.drag-target {
    background: var(--highlight); color: #fff;
  }
  .toggle {
    background: none; border: none; color: var(--text-secondary);
    cursor: pointer; font-size: 11px; font-family: inherit;
    padding: 0; width: 16px; flex-shrink: 0;
  }
  .toggle:hover { color: var(--highlight); }
  .spacer { width: 16px; }
  .item-title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .item-actions { display: none; gap: 2px; }
  .item-row:hover .item-actions, .item-row.drag-target .item-actions { display: flex; }
  .action {
    background: none; border: none; color: var(--text-secondary);
    cursor: pointer; font-size: 14px; font-family: inherit; padding: 0 4px; line-height: 1;
  }
  .action:hover { color: var(--text-primary); }
  .children { padding-left: 8px; border-left: 1px solid var(--border); margin-left: 23px; }
  :global(.drag-ghost) {
    position: fixed; z-index: 9999; pointer-events: none;
    background: var(--highlight); color: #fff; font-size: 12px;
    font-family: var(--font-sans); padding: 4px 10px; border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3); display: none;
    white-space: nowrap;
  }
</style>