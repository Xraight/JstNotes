<script lang="ts">
  import type { NoteTreeNode } from '../types';
  import { noteStore } from '../stores/notes';
  import TreeItem from './TreeItem.svelte';

  let { nodes = [] }: { nodes?: NoteTreeNode[] } = $props();

  function onTreeMove(e: MouseEvent) {
    const ghost = document.querySelector('.drag-ghost') as HTMLElement | null;
    if (ghost && ghost.style.display !== 'none') {
      ghost.style.left = (e.clientX + 12) + 'px';
      ghost.style.top = (e.clientY - 20) + 'px';
    }
  }

  function selectNode(id: string) {
    noteStore.selectNote(id);
  }

  function createChild(parentId: string | null) {
    const title = prompt('Note title:');
    if (title) {
      noteStore.createNote({ title, content: '', parent_id: parentId });
    }
  }

  async function deleteNode(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (confirm('Delete this note?')) {
      await noteStore.deleteNote(id);
    }
  }

  async function moveNode(id: string, targetParentId: string | null, e: MouseEvent) {
    e.stopPropagation();
    try {
      await noteStore.updateNote({ id, parent_id: targetParentId });
    } catch (err) {
      console.error('Move failed:', err);
    }
  }
</script>

<div class="tree">
  <div class="tree-header">
    <span class="title">Notes</span>
    <button class="icon-btn" onclick={() => createChild(null)} title="New root note">+</button>
  </div>
  <div class="tree-content" onmousemove={onTreeMove}>
    {#each nodes as node}
      <TreeItem {node} onselect={selectNode} ondelete={deleteNode} oncreate={createChild} onmove={moveNode} />
    {/each}
    {#if nodes.length === 0}
      <div class="empty">No notes yet. Click + to create one.</div>
    {/if}
  </div>
</div>

<style>
  .tree {
    display: flex; flex-direction: column; flex: 1; overflow: hidden;
    min-width: 0; width: 100%; font-family: var(--font-sans);
  }
  .tree-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 16px 6px; font-weight: 600; font-size: 13px;
    color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.5px;
  }
  .icon-btn { background: none; border: none; color: var(--text-primary);
    cursor: pointer; font-size: 18px; font-family: inherit; padding: 2px 8px; border-radius: 4px; }
  .icon-btn:hover { background: var(--accent); }
  .tree-content { flex: 1; overflow-y: auto; padding: 4px 0; }
  .empty { padding: 24px 16px; color: var(--text-secondary); font-size: 13px; text-align: center; }
</style>