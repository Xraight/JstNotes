<script lang="ts">
  import { pdfStore } from '../stores/pdf';
  import { noteStore } from '../stores/notes';

  let collapsed = $state(false);

  let noteFilterOn = $state(true);
  let displayPdfs = $derived(noteFilterOn && noteStore.selectedNote
    ? pdfStore.pdfs.filter(p => pdfStore.linkedPdfIds.includes(p.id))
    : pdfStore.pdfs);

  function toggle() {
    collapsed = !collapsed;
  }

  async function handleOpen(id: string) {
    const pdf = pdfStore.pdfs.find(p => p.id === id);
    if (pdf) {
      await pdfStore.openPdf(pdf);
    }
  }

  async function handleImport() {
    await pdfStore.importPdf();
  }

  async function handleDelete(id: string, e: MouseEvent) {
    e.stopPropagation();
    if (confirm('Delete this PDF?')) {
      await pdfStore.deletePdf(id);
    }
  }

  function toggleFilter() {
    noteFilterOn = !noteFilterOn;
  }
</script>

<div class="pdf-library">
  <div class="pdf-header">
    <button class="toggle-btn" onclick={toggle} title={collapsed ? 'Expand' : 'Collapse'}>
      {collapsed ? '▶' : '▼'}
    </button>
    <span class="title">PDFs</span>
    {#if noteStore.selectedNote && pdfStore.linkedPdfIds.length > 0}
      <button class="filter-toggle" class:active={noteFilterOn} onclick={toggleFilter} title="Filter by current note">
        {noteFilterOn ? '●' : '○'} note
      </button>
    {/if}
    <button class="icon-btn" onclick={handleImport} title="Import PDF">+</button>
  </div>
  {#if !collapsed}
    <div class="pdf-list">
      {#each displayPdfs as pdf (pdf.id)}
        <div class="pdf-item" class:active={pdfStore.activePdf?.id === pdf.id} onclick={() => handleOpen(pdf.id)}>
          <span class="pdf-icon">📄</span>
          <span class="pdf-title">{pdf.title || 'Untitled'}</span>
          <button class="pdf-delete" onclick={(e) => handleDelete(pdf.id, e)} title="Delete">✕</button>
        </div>
      {/each}
      {#if displayPdfs.length === 0}
        <div class="empty">
          {noteFilterOn && noteStore.selectedNote ? 'No PDFs linked to this note' : 'No PDFs imported'}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .pdf-library {
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--border);
    font-family: var(--font-sans);
  }
  .pdf-header {
    display: flex;
    align-items: center;
    padding: 10px 16px 6px;
    font-weight: 600;
    font-size: 13px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    gap: 4px;
  }
  .toggle-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 4px;
    font-family: inherit;
  }
  .toggle-btn:hover {
    color: var(--text-primary);
  }
  .title {
    flex: 1;
  }
  .filter-toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 10px;
    padding: 1px 5px;
    font-family: inherit;
    opacity: 0.7;
  }
  .filter-toggle:hover {
    opacity: 1;
    border-color: var(--highlight);
  }
  .filter-toggle.active {
    opacity: 1;
    color: var(--highlight);
    border-color: var(--highlight);
  }
  .icon-btn {
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 18px;
    font-family: inherit;
    padding: 2px 8px;
    border-radius: 4px;
  }
  .icon-btn:hover {
    background: var(--accent);
  }
  .pdf-list {
    overflow-y: auto;
    max-height: 200px;
    padding: 2px 0;
  }
  .pdf-item {
    display: flex;
    align-items: center;
    padding: 5px 16px 5px 28px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
    gap: 6px;
  }
  .pdf-item:hover {
    background: var(--accent);
  }
  .pdf-item.active {
    background: var(--highlight);
    color: #fff;
  }
  .pdf-item.active .pdf-delete {
    color: #fff;
  }
  .pdf-icon {
    font-size: 14px;
    flex-shrink: 0;
  }
  .pdf-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pdf-delete {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 4px;
    border-radius: 3px;
    font-family: inherit;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .pdf-item:hover .pdf-delete {
    opacity: 1;
  }
  .pdf-delete:hover {
    color: var(--highlight);
    background: var(--bg-primary);
  }
  .empty {
    padding: 12px 16px 12px 28px;
    color: var(--text-secondary);
    font-size: 13px;
  }
</style>