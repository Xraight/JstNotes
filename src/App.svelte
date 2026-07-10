<script lang="ts">
  import { onMount } from 'svelte';
  import Breadcrumbs from './lib/components/Breadcrumbs.svelte';
  import NoteTree from './lib/components/NoteTree.svelte';
  import Calendar from './lib/components/Calendar.svelte';
  import Editor from './lib/components/Editor.svelte';
  import PdfViewer from './lib/components/PdfViewer.svelte';
  import Settings from './lib/components/Settings.svelte';
  import PdfLibrary from './lib/components/PdfLibrary.svelte';
  import StudyHome from './lib/components/StudyHome.svelte';
  import GraphView from './lib/components/GraphView.svelte';
  import MiniConceptMap from './lib/components/MiniConceptMap.svelte';
  import { noteStore } from './lib/stores/notes';
  import { settingsStore } from './lib/stores/settings';
  import { pdfStore } from './lib/stores/pdf';
  import { aiStore } from './lib/stores/ai';

  let tree = $derived(noteStore.tree);
  let breadcrumbs = $derived(noteStore.breadcrumbs);
  let selectedNote = $derived(noteStore.selectedNote);
  let sidebarWidth = $derived(settingsStore.settings.layout.sidebar_width);
  let sidebarRef = $state<HTMLDivElement>();
  let calRef = $state<HTMLElement | null>(null);
  let settingsOpen = $state(false);
  let error = $state<string | null>(null);
  let dragging = $state(false);
  let pdfWidth = $state(50);
  let editorAreaRef = $state<HTMLDivElement>();

  onMount(async () => {
    await settingsStore.load();
    try {
      await noteStore.loadTree();
    } catch (e: unknown) {
      error = `Error: ${e instanceof Error ? e.message : String(e)}`;
    }
    await pdfStore.loadPdfs();
    if (sidebarRef) calRef = sidebarRef.querySelector<HTMLElement>('[data-calendar]');
  });

  $effect(() => {
    if (selectedNote) {
      pdfStore.loadLinkedPdfs(selectedNote.id);
    } else {
      pdfStore.linkedPdfIds = [];
    }
  });

  // Close Study Home when PDF is opened (mutually exclusive views)
  $effect(() => {
    if (pdfStore.isOpen) {
      aiStore.studyPanelOpen = false;
      aiStore.graphOpen = false;
    }
  });

  function startDrag(e: MouseEvent) {
    dragging = true;
    const startX = e.clientX;
    const startWidth = sidebarWidth;
    if (!calRef && sidebarRef) {
      calRef = sidebarRef.querySelector<HTMLElement>('[data-calendar]');
    }

    function onMouseMove(ev: MouseEvent) {
      const newWidth = Math.max(180, Math.min(500, startWidth + ev.clientX - startX));
      settingsStore.setSidebarWidth(newWidth);
      if (sidebarRef) {
        sidebarRef.style.width = newWidth + 'px';
        if (calRef) calRef.style.width = newWidth + 'px';
      }
    }

    function onMouseUp() {
      dragging = false;
      settingsStore.saveSidebarWidth(settingsStore.settings.layout.sidebar_width);
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
    }

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  }

  function startPdfResize(e: MouseEvent) {
    dragging = true;
    const startX = e.clientX;
    const startWidth = pdfWidth;

    function onMouseMove(ev: MouseEvent) {
      const areaW = editorAreaRef?.clientWidth || 1;
      const delta = ((startX - ev.clientX) / areaW) * 100;
      pdfWidth = Math.max(30, Math.min(70, startWidth + delta));
    }

    function onMouseUp() {
      dragging = false;
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
    }

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  }
</script>

{#if error}
  <div class="error-overlay">
    <h2>Connection Error</h2>
    <p>{error}</p>
    <p style="font-size:13px;color:var(--text-secondary);">Make sure you're running inside Tauri (npm run tauri dev)</p>
  </div>
{/if}

<div class="app-shell" class:dragging>
  <div class="sidebar" bind:this={sidebarRef} style="width: {sidebarWidth}px;">
    <div class="sidebar-header">
      <span class="logo">JSTNotes</span>
    </div>
    <NoteTree nodes={tree} />
    <Calendar />
    <PdfLibrary />
    <button class="sidebar-resize-handle"
      onmousedown={startDrag}
      aria-label="Resize sidebar"
      onkeydown={(e) => {
        if (e.key === 'ArrowLeft') settingsStore.setSidebarWidth(sidebarWidth - 20);
        if (e.key === 'ArrowRight') settingsStore.setSidebarWidth(sidebarWidth + 20);
      }}>
    </button>
  </div>
  <div class="main">
    <Breadcrumbs items={breadcrumbs} />
    {#if aiStore.studyPanelOpen}
      <StudyHome />
    {:else if aiStore.graphOpen}
      <GraphView />
    {:else}
    <div class="editor-area" class:with-pdf={pdfStore.isOpen} bind:this={editorAreaRef}>
      <div class="editor-wrap">
        <Editor />
      </div>
      {#if pdfStore.isOpen}
        <button class="pdf-divider"
          onmousedown={startPdfResize}
          aria-label="Resize PDF panel"
          onkeydown={(e) => {
            if (e.key === 'ArrowLeft') pdfWidth = Math.max(30, pdfWidth - 5);
            if (e.key === 'ArrowRight') pdfWidth = Math.min(70, pdfWidth + 5);
          }}>
        </button>
      {/if}
      <PdfViewer pdfWidth={pdfWidth} />
      {#if selectedNote}
        <div class="context-panel">
          {#if pdfStore.isOpen && pdfStore.showOutline && pdfStore.pdfOutline.length > 0}
            <div class="outline-view">
              <div class="outline-view-header">
                Outline
                <button class="outline-close" onclick={() => pdfStore.showOutline = false}>✕</button>
              </div>
              <div class="outline-view-body">
                {#each pdfStore.pdfOutline as item}
                  <button class="outline-view-item" style="padding-left: {item.depth * 14 + 12}px"
                    onclick={() => {
                      const el = document.querySelector(`[data-page="${item.page + 1}"]`) as HTMLElement | null;
                      if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
                    }}>
                    {item.title}
                  </button>
                {/each}
              </div>
            </div>
          {/if}
          <div class="context-fill">
            <MiniConceptMap context={pdfStore.isOpen} compact={pdfStore.isOpen} />
          </div>
        </div>
      {/if}
    </div>
    {/if}
  </div>
  <div class="status-bar">
    <span class="status-left">
      {#if selectedNote}
        {selectedNote.title}
      {:else}
        No note selected
      {/if}
    </span>
    <span class="status-right">
      <button class="icon-button" onclick={() => settingsOpen = !settingsOpen} title="Settings">⚙</button>
    </span>
  </div>
</div>

{#if settingsOpen}
  <Settings onclose={() => settingsOpen = false} />
{/if}

<style>
  .error-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    z-index: 1000;
    gap: 12px;
  }
  .error-overlay h2 {
    color: var(--highlight);
  }
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-sans);
  }
  .sidebar {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    position: relative;
    user-select: none;
    overflow: hidden;
    min-width: 0;
  }
  .dragging .sidebar {
    transition: none;
  }
  .sidebar-header {
    padding: 14px 20px;
    font-weight: 700;
    font-size: 16px;
    letter-spacing: 1px;
    border-bottom: 1px solid var(--border);
    color: var(--highlight);
  }
  .sidebar-resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 8px;
    cursor: col-resize;
    z-index: 10;
    background: transparent;
    border: none;
    padding: 0;
    outline: none;
  }
  .sidebar-resize-handle:hover,
  .dragging .sidebar-resize-handle {
    background: var(--highlight);
    opacity: 0.3;
  }
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
  .editor-area {
    flex: 1;
    overflow-y: auto;
    display: flex;
  }
  .editor-area.with-pdf {
    overflow-y: hidden;
  }
  .pdf-divider {
    width: 6px;
    cursor: col-resize;
    background: transparent;
    border: none;
    padding: 0;
    outline: none;
    flex-shrink: 0;
    z-index: 5;
  }
  .pdf-divider:hover,
  .dragging .pdf-divider {
    background: var(--highlight);
    opacity: 0.3;
  }
  .editor-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .context-panel {
    width: clamp(200px, 20%, 320px); flex-shrink: 0;
    display: flex; flex-direction: column;
    border-left: 1px solid var(--border);
    background: var(--bg-primary); overflow: hidden;
  }
  .outline-view {
    display: flex; flex-direction: column; height: 100%;
  }
  .outline-view-header {
    padding: 8px 12px; font-size: 11px; font-weight: 600;
    color: var(--text-secondary); text-transform: uppercase;
    letter-spacing: 0.5px; border-bottom: 1px solid var(--border);
    background: var(--bg-secondary); flex-shrink: 0;
    display: flex; align-items: center; justify-content: space-between;
  }
  .outline-close {
    background: none; border: none; cursor: pointer;
    font-size: 13px; color: var(--text-secondary); font-family: inherit;
    padding: 2px 4px; border-radius: 2px;
  }
  .outline-close:hover { background: var(--accent); color: var(--text-primary); }
  .outline-view-body {
    flex: 1; overflow-y: auto; padding: 4px 0;
  }
  .outline-view-item {
    display: block; width: 100%; text-align: left;
    background: none; border: none; cursor: pointer;
    font-size: 12px; color: var(--text-primary); font-family: inherit;
    padding: 3px 12px; line-height: 1.6;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .outline-view-item:hover { background: var(--accent); }
  .context-fill { flex: 1; overflow: hidden; min-height: 0; }
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 16px;
    font-size: 12px;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    height: 28px;
    flex-shrink: 0;
  }
  .icon-button {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    padding: 2px 8px;
    border-radius: 4px;
    color: var(--text-secondary);
  }
  .icon-button:hover {
    background: var(--accent);
    color: var(--text-primary);
  }
</style>
