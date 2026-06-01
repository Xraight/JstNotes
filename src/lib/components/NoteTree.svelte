<script lang="ts">
  import type { NoteTreeNode } from '../types';
  import { noteStore } from '../stores/notes';
  import TreeItem from './TreeItem.svelte';

  let { nodes = [] }: { nodes?: NoteTreeNode[] } = $props();

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
</script>

<div class="tree">
  <div class="tree-header">
    <span class="title">Notes</span>
    <button class="icon-btn" onclick={() => createChild(null)} title="New root note">+</button>
  </div>
  <div class="tree-content">
    {#each nodes as node}
      <TreeItem {node} onselect={selectNode} ondelete={deleteNode} oncreate={createChild} />
    {/each}
    {#if nodes.length === 0}
      <div class="empty">No notes yet. Click + to create one.</div>
    {/if}
  </div>
</div>

<style>
  .tree {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }
  .tree-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px 6px;
    font-weight: 600;
    font-size: 13px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .icon-btn {
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 18px;
    padding: 2px 8px;
    border-radius: 4px;
  }
  .icon-btn:hover {
    background: var(--accent);
  }
  .tree-content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .empty {
    padding: 24px 16px;
    color: var(--text-secondary);
    font-size: 13px;
    text-align: center;
  }
</style>
