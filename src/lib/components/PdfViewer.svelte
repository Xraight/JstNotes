<script lang="ts">
  import { onDestroy } from 'svelte';
  import { pdfStore } from '../stores/pdf';
  import workerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

  let viewerEl = $state<HTMLDivElement>();
  let bodyEl = $state<HTMLDivElement>();
  let currentPage = $state(1);
  let numPages = $state(0);
  let scale = $state(1);
  let isLoading = $state(false);
  let pageHeight = $state(800);
  let pdfDoc: any = null;

  let activeTool = $state<'highlight' | 'note' | 'cursor'>('cursor');
  let selectionStart: { x: number; y: number } | null = null;

  const renderedPages = new Map<number, HTMLCanvasElement>();
  const pendingPages = new Set<number>();
  let observer: IntersectionObserver | null = null;

  function resetState() {
    if (observer) { observer.disconnect(); observer = null; }
    renderedPages.clear();
    pendingPages.clear();
    pdfDoc = null;
    numPages = 0;
    currentPage = 1;
    pageHeight = 800;
  }

  async function renderPage(pageNum: number) {
    if (renderedPages.has(pageNum) || pendingPages.has(pageNum) || !pdfDoc || !viewerEl) return;
    pendingPages.add(pageNum);

    try {
      const dpr = window.devicePixelRatio || 1;
      const page = await pdfDoc.getPage(pageNum);
      const vp1 = page.getViewport({ scale: 1 });
      const effScale = (viewerEl.clientWidth / vp1.width) * scale;
      const viewport = page.getViewport({ scale: effScale });

      const canvas = document.createElement('canvas');
      canvas.width = viewport.width * dpr;
      canvas.height = viewport.height * dpr;
      canvas.dataset.page = String(pageNum);
      canvas.style.width = viewport.width + 'px';
      canvas.style.height = 'auto';
      canvas.style.display = 'block';
      canvas.style.margin = '0 auto';

      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      await page.render({ canvasContext: ctx, viewport }).promise;

      const placeholder = viewerEl?.querySelector(`[data-page="${pageNum}"]`) as HTMLElement | null;
      if (placeholder) {
        placeholder.innerHTML = '';
        placeholder.appendChild(canvas);
        placeholder.style.height = 'auto';
        placeholder.classList.add('rendered');
      }

      placeholder.style.display = 'inline-flex';
      placeholder.style.minHeight = '';
      placeholder.style.background = 'none';
      placeholder.style.boxShadow = 'none';

      renderedPages.set(pageNum, canvas);

      if (pageNum === 1) {
        pageHeight = viewport.height;
      }
    } finally {
      pendingPages.delete(pageNum);
    }
  }

  function setupObserver() {
    if (observer) observer.disconnect();
    if (!bodyEl || !viewerEl) return;

    observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          renderPage(Number((entry.target as HTMLElement).dataset.page));
        }
      }
    }, { root: bodyEl, rootMargin: '600px 0px' });

    for (const el of viewerEl.querySelectorAll('[data-page]')) {
      observer.observe(el);
    }
  }

  async function loadPdf() {
    if (!pdfStore.pdfData || !viewerEl) return;
    resetState();
    isLoading = true;

    const pdfjsLib = await import('pdfjs-dist');
    pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl;
    const loadingTask = pdfjsLib.getDocument({ data: pdfStore.pdfData.slice(0) });
    pdfDoc = await loadingTask.promise;
    numPages = pdfDoc.numPages;
    scale = 1;

    viewerEl.innerHTML = '';
    for (let i = 1; i <= numPages; i++) {
      const el = document.createElement('div');
      el.dataset.page = String(i);
      el.style.width = '100%';
      el.style.marginBottom = '8px';
      el.style.borderRadius = '4px';
      el.style.background = 'var(--bg-primary)';
      el.style.boxShadow = '0 1px 4px rgba(0,0,0,0.2)';
      el.style.minHeight = '200px';
      el.style.height = `${pageHeight}px`;
      viewerEl.appendChild(el);
    }

    await renderPage(1);
    isLoading = false;

    // Sync all remaining placeholders with actual page 1 height
    const actualH = pageHeight;
    for (const el of viewerEl.querySelectorAll('[data-page]:not(.rendered)')) {
      (el as HTMLElement).style.height = `${actualH}px`;
    }

    requestAnimationFrame(() => setupObserver());
  }

  async function changeScale(newScale: number) {
    if (!viewerEl || !bodyEl || isLoading) return;
    const oldScroll = bodyEl.scrollTop;
    const oldScaleVal = scale;
    const clampedScale = Math.max(0.3, Math.min(4, newScale));
    scale = clampedScale;

    renderedPages.clear();
    pendingPages.clear();

    const newEstHeight = Math.round(pageHeight * clampedScale / (oldScaleVal || 1));

    const wraps = viewerEl.querySelectorAll('[data-page]');
    wraps.forEach((el) => {
      const elem = el as HTMLElement;
      elem.innerHTML = '';
      elem.classList.remove('rendered');
      elem.style.height = `${newEstHeight}px`;
    });

    bodyEl.scrollTop = Math.round(oldScroll * clampedScale / (oldScaleVal || 1));

    try {
      await renderPage(1);
      setupObserver();
    } catch (e) {
      console.error('Zoom failed:', e);
      setupObserver();
    }
  }

  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      if (isLoading) return;
      changeScale(scale + (e.deltaY > 0 ? -0.1 : 0.1));
    }
  }

  function startAnnotation(e: MouseEvent) {
    if (activeTool === 'cursor') return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    selectionStart = {
      x: (e.clientX - rect.left) / rect.width,
      y: (e.clientY - rect.top) / rect.height,
    };
  }

  function endAnnotation(e: MouseEvent) {
    if (activeTool === 'cursor' || !selectionStart) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const endX = (e.clientX - rect.left) / rect.width;
    const endY = (e.clientY - rect.top) / rect.height;
    const page = Number(
      (e.currentTarget as HTMLElement).closest('[data-page]')?.getAttribute('data-page') || currentPage
    );

    const x = Math.min(selectionStart.x, endX);
    const y = Math.min(selectionStart.y, endY);
    const w = Math.abs(endX - selectionStart.x);
    const h = Math.abs(endY - selectionStart.y);

    if (w < 0.01 && h < 0.01) {
      if (activeTool === 'note') {
        pdfStore.saveAnnotation({
          pdf_id: pdfStore.activePdf!.id,
          page, annotation_type: 'note',
          x, y, width: 0.05, height: 0.05,
          color: '#E94560', content: 'Note',
        });
      }
    } else {
      pdfStore.saveAnnotation({
        pdf_id: pdfStore.activePdf!.id,
        page, annotation_type: activeTool,
        x, y, width: w, height: h,
        color: '#E94560', content: null,
      });
    }

    selectionStart = null;
  }

  $effect(() => {
    if (pdfStore.pdfData) {
      loadPdf();
    } else {
      resetState();
    }
  });

  onDestroy(() => {
    if (observer) observer.disconnect();
    resetState();
  });
</script>

<div class="pdf-viewer" class:open={pdfStore.isOpen}>
  <div class="pdf-toolbar">
    <div class="pdf-toolbar-left">
      <button class="tool-btn" onclick={() => pdfStore.closePdf()} title="Close PDF">✕</button>
      <span class="pdf-title">{pdfStore.activePdf?.title || 'PDF'}</span>
    </div>
    <div class="pdf-toolbar-center">
      <button class="tool-btn" onclick={() => changeScale(scale - 0.1)} disabled={scale <= 0.3 || isLoading}>−</button>
      <span class="zoom-label">{Math.round(scale * 100)}%</span>
      <button class="tool-btn" onclick={() => changeScale(scale + 0.1)} disabled={scale >= 4 || isLoading}>+</button>
      <button class="tool-btn" onclick={() => changeScale(1)} disabled={isLoading}>Fit</button>
    </div>
    <div class="pdf-toolbar-right">
      <button class="tool-btn" class:active={activeTool === 'cursor'} onclick={() => activeTool = 'cursor'} title="Cursor">↖</button>
      <button class="tool-btn" class:active={activeTool === 'highlight'} onclick={() => activeTool = 'highlight'} title="Highlight">⬛</button>
      <button class="tool-btn" class:active={activeTool === 'note'} onclick={() => activeTool = 'note'} title="Add note">📝</button>
    </div>
  </div>

  <div class="pdf-body" bind:this={bodyEl} onwheel={handleWheel}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="pdf-pages" bind:this={viewerEl} onmousedown={startAnnotation} onmouseup={endAnnotation}></div>
    {#if isLoading}
      <div class="pdf-loading">Loading PDF…</div>
    {/if}
  </div>
</div>

<style>
  .pdf-viewer {
    display: none;
    flex-direction: column;
    width: 50%;
    min-width: 300px;
    border-left: 1px solid var(--border);
    background: var(--bg-secondary);
    overflow: hidden;
  }
  .pdf-viewer.open {
    display: flex;
  }
  .pdf-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-primary);
    gap: 8px;
    flex-shrink: 0;
    font-family: var(--font-sans);
  }
  .pdf-toolbar-left,
  .pdf-toolbar-center,
  .pdf-toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .pdf-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }
  .tool-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 13px;
    cursor: pointer;
    color: var(--text-secondary);
    font-family: inherit;
  }
  .tool-btn:hover {
    color: var(--text-primary);
    border-color: var(--highlight);
  }
  .tool-btn.active {
    background: var(--highlight);
    color: #fff;
    border-color: var(--highlight);
  }
  .tool-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .zoom-label {
    font-size: 12px;
    color: var(--text-secondary);
    min-width: 40px;
    text-align: center;
  }
  .pdf-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    position: relative;
  }
  .pdf-pages {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .pdf-loading {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-size: 14px;
    z-index: 10;
  }
</style>
