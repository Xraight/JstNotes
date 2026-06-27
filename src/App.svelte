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
