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

  let showNoteDialog = $state(false);
  let noteDialogData: { page: number; x: number; y: number } | null = $state(null);
  let noteDialogContent = $state('');
  let editingAnnotationId: string | null = $state(null);

  let tooltipContent = $state('');
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  interface PageRender {
    canvas: HTMLCanvasElement;
    overlay: HTMLCanvasElement;
    viewport: { width: number; height: number };
  }

  const renderedPages = new Map<number, PageRender>();
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
      const vpw = viewport.width;
      const vph = viewport.height;

      const container = document.createElement('div');
      container.style.position = 'relative';
      container.style.width = vpw + 'px';
      container.style.margin = '0 auto';

      const canvas = document.createElement('canvas');
      canvas.width = vpw * dpr;
      canvas.height = vph * dpr;
      canvas.dataset.page = String(pageNum);
      canvas.style.width = '100%';
      canvas.style.height = 'auto';
      canvas.style.display = 'block';

      const overlay = document.createElement('canvas');
      overlay.width = vpw * dpr;
      overlay.height = vph * dpr;
      overlay.style.width = '100%';
      overlay.style.height = '100%';
      overlay.style.position = 'absolute';
      overlay.style.top = '0';
      overlay.style.left = '0';
      overlay.style.pointerEvents = 'none';

      container.appendChild(canvas);
      container.appendChild(overlay);

      const ctx = canvas.getContext('2d')!;
      ctx.scale(dpr, dpr);
      await page.render({ canvasContext: ctx, viewport }).promise;

      const octx = overlay.getContext('2d')!;
      octx.scale(dpr, dpr);
      drawAnnotations(pageNum, octx, vpw, vph);

      const placeholder = viewerEl?.querySelector(`[data-page="${pageNum}"]`) as HTMLElement | null;
      if (placeholder) {
        placeholder.innerHTML = '';
        placeholder.appendChild(container);
        placeholder.style.height = 'auto';
        placeholder.classList.add('rendered');
        placeholder.style.display = 'inline-flex';
        placeholder.style.minHeight = '';
        placeholder.style.background = 'none';
        placeholder.style.boxShadow = 'none';
      }

      renderedPages.set(pageNum, { canvas, overlay, viewport: { width: vpw, height: vph } });

      if (pageNum === 1) {
        pageHeight = vph;
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

    const actualH = pageHeight;
    for (const el of viewerEl.querySelectorAll('[data-page]:not(.rendered)')) {
      (el as HTMLElement).style.height = `${actualH}px`;
    }

    requestAnimationFrame(() => setupObserver());
  }

  async function changeScale(newScale: number) {
    if (!viewerEl || !bodyEl || isLoading) return;

    let anchorPage = 1;
    let anchorOffset = 0;
    for (const el of viewerEl.querySelectorAll('[data-page].rendered')) {
      const rect = el.getBoundingClientRect();
      if (rect.top < bodyEl.clientHeight && rect.bottom > 0) {
        anchorPage = Number((el as HTMLElement).dataset.page);
        anchorOffset = rect.top;
        break;
      }
    }

    const oldScaleVal = scale;
    const clampedScale = Math.max(0.3, Math.min(4, newScale));
    scale = clampedScale;

    renderedPages.clear();
    pendingPages.clear();

    const gap = 8;
    const newEstHeight = Math.round(pageHeight * clampedScale / (oldScaleVal || 1));

    const wraps = viewerEl.querySelectorAll('[data-page]');
    wraps.forEach((el) => {
      const elem = el as HTMLElement;
      elem.innerHTML = '';
      elem.classList.remove('rendered');
      elem.style.height = `${newEstHeight}px`;
    });

    try {
      await renderPage(1);

      const oldScroll = bodyEl.scrollTop;
      const clientH = bodyEl.clientHeight;
      const margin = 600;
      const startPage = Math.max(1, Math.floor((oldScroll - clientH - margin) / (newEstHeight + gap)));
      const endPage = Math.min(numPages, Math.ceil((oldScroll + 2 * clientH + margin) / (newEstHeight + gap)));
      for (let i = startPage; i <= endPage; i++) {
        if (i !== 1) await renderPage(i);
      }

      await new Promise(r => requestAnimationFrame(r));
      const anchorEl = viewerEl.querySelector(`[data-page="${anchorPage}"]`) as HTMLElement | null;
      if (anchorEl) {
        const newRect = anchorEl.getBoundingClientRect();
        bodyEl.scrollTop += newRect.top - anchorOffset;
      }

      setupObserver();

      await new Promise(r => requestAnimationFrame(r));
      const anchorEl2 = viewerEl.querySelector(`[data-page="${anchorPage}"]`) as HTMLElement | null;
      if (anchorEl2) {
        const newRect2 = anchorEl2.getBoundingClientRect();
        bodyEl.scrollTop += newRect2.top - anchorOffset;
      }
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

  function drawAnnotations(pageNum: number, ctx: CanvasRenderingContext2D, vpW: number, vpH: number) {
    const annotations = pdfStore.annotations.filter(a => a.page === pageNum);
    if (annotations.length === 0) return;

    ctx.save();
    for (const ann of annotations) {
      const x = ann.x * vpW;
      const y = ann.y * vpH;
      const w = ann.width * vpW;
      const h = ann.height * vpH;

      if (ann.annotation_type === 'highlight') {
        ctx.globalAlpha = 0.3;
        ctx.fillStyle = ann.color || '#E94560';
        ctx.fillRect(x, y, w, h);
      } else if (ann.annotation_type === 'note') {
        const cx = x + w / 2;
        const cy = y + h / 2;
        ctx.globalAlpha = 1;
        ctx.fillStyle = ann.color || '#E94560';
        ctx.beginPath();
        ctx.arc(cx, cy, 6, 0, 2 * Math.PI);
        ctx.fill();
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 1.5;
        ctx.stroke();
        ctx.fillStyle = '#fff';
        ctx.beginPath();
        ctx.arc(cx, cy, 2, 0, 2 * Math.PI);
        ctx.fill();
      }
    }
    ctx.restore();
  }

  function redrawPageAnnotations(pageNum: number) {
    const render = renderedPages.get(pageNum);
    if (!render) return;
    const dpr = window.devicePixelRatio || 1;
    render.overlay.width = render.overlay.width;
    const octx = render.overlay.getContext('2d')!;
    octx.scale(dpr, dpr);
    drawAnnotations(pageNum, octx, render.viewport.width, render.viewport.height);
  }

  function findAnnotationAt(e: MouseEvent, thresholdRel = 0.02): { page: number; annotation: typeof pdfStore.annotations[0] } | null {
    const target = (e.target as HTMLElement).closest('[data-page]') as HTMLElement | null;
    if (!target) return null;
    const page = Number(target.dataset.page);
    const rect = target.getBoundingClientRect();
    const relX = (e.clientX - rect.left) / rect.width;
    const relY = (e.clientY - rect.top) / rect.height;

    for (const ann of pdfStore.annotations) {
      if (ann.page !== page || ann.annotation_type !== 'note') continue;
      const cx = ann.x + ann.width / 2;
      const cy = ann.y + ann.height / 2;
      const dx = relX - cx;
      const dy = relY - cy;
      if (dx * dx + dy * dy < thresholdRel * thresholdRel) {
        return { page, annotation: ann };
      }
    }
    return null;
  }

  function handleMouseMove(e: MouseEvent) {
    const found = findAnnotationAt(e);
    if (found && found.annotation.content) {
      tooltipContent = found.annotation.content;
      tooltipX = e.clientX + 12;
      tooltipY = e.clientY + 12;
    } else {
      tooltipContent = '';
    }
  }

  function handleDblClick(e: MouseEvent) {
    if (activeTool !== 'cursor') return;
    const found = findAnnotationAt(e, 0.03);
    if (!found) return;
    tooltipContent = '';
    noteDialogData = { page: found.page, x: found.annotation.x, y: found.annotation.y };
    noteDialogContent = found.annotation.content || '';
    editingAnnotationId = found.annotation.id;
    showNoteDialog = true;
  }

  function startAnnotation(e: MouseEvent) {
    if (activeTool === 'cursor') return;
    const target = (e.target as HTMLElement).closest('[data-page]');
    if (!target) return;
    const rect = target.getBoundingClientRect();
    selectionStart = {
      x: (e.clientX - rect.left) / rect.width,
      y: (e.clientY - rect.top) / rect.height,
    };
  }

  async function endAnnotation(e: MouseEvent) {
    if (activeTool === 'cursor' || !selectionStart) return;
    const target = (e.target as HTMLElement).closest('[data-page]');
    if (!target) return;
    const rect = target.getBoundingClientRect();
    const endX = (e.clientX - rect.left) / rect.width;
    const endY = (e.clientY - rect.top) / rect.height;
    const page = Number((target as HTMLElement).dataset.page);

    const x = Math.min(selectionStart.x, endX);
    const y = Math.min(selectionStart.y, endY);
    const w = Math.abs(endX - selectionStart.x);
    const h = Math.abs(endY - selectionStart.y);

    if (w < 0.01 && h < 0.01) {
      if (activeTool !== 'note') { selectionStart = null; return; }
      tooltipContent = '';
      noteDialogData = { page, x, y };
      noteDialogContent = '';
      editingAnnotationId = null;
      showNoteDialog = true;
      selectionStart = null;
      return;
    } else {
      await pdfStore.saveAnnotation({
        pdf_id: pdfStore.activePdf!.id,
        page, annotation_type: activeTool,
        x, y, width: w, height: h,
        color: '#E94560', content: null,
      });
    }

    selectionStart = null;
    redrawPageAnnotations(page);
  }

  async function saveNoteDialog() {
    if (!noteDialogData) return;
    const { page, x, y } = noteDialogData;

    if (editingAnnotationId) {
      await pdfStore.updateAnnotationContent(editingAnnotationId, noteDialogContent || null);
    } else {
      await pdfStore.saveAnnotation({
        pdf_id: pdfStore.activePdf!.id,
        page, annotation_type: 'note',
        x, y, width: 0.05, height: 0.05,
        color: '#E94560', content: noteDialogContent || null,
      });
    }

    showNoteDialog = false;
    noteDialogData = null;
    noteDialogContent = '';
    editingAnnotationId = null;
    redrawPageAnnotations(page);
    tooltipContent = '';
  }

  function cancelNoteDialog() {
    showNoteDialog = false;
    noteDialogData = null;
    noteDialogContent = '';
    editingAnnotationId = null;
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
    <div class="pdf-pages" bind:this={viewerEl} onmousedown={startAnnotation} onmouseup={endAnnotation} onmousemove={handleMouseMove} ondblclick={handleDblClick}></div>
    {#if isLoading}
      <div class="pdf-loading">Loading PDF…</div>
    {/if}
  </div>
</div>

{#if showNoteDialog}
<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="note-dialog-overlay" onclick={cancelNoteDialog}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="note-dialog" onclick={(e) => e.stopPropagation()}>
    <div class="note-dialog-header">
      {editingAnnotationId ? 'Edit Note' : 'Add Note'}
    </div>
    <textarea
      bind:value={noteDialogContent}
      placeholder="Type your note..."
      rows={4}
    ></textarea>
    <div class="note-dialog-actions">
      <button class="tool-btn" onclick={cancelNoteDialog}>Cancel</button>
      <button class="tool-btn note-save-btn" onclick={saveNoteDialog}>Save</button>
    </div>
  </div>
</div>
{/if}

{#if tooltipContent}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="note-tooltip" style="left: {tooltipX}px; top: {tooltipY}px;">
  {tooltipContent}
</div>
{/if}

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

  .note-dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .note-dialog {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    min-width: 280px;
    max-width: 400px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.3);
  }
  .note-dialog-header {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 10px;
  }
  .note-dialog textarea {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 8px;
    font-size: 13px;
    font-family: var(--font-sans);
    color: var(--text-primary);
    resize: vertical;
    box-sizing: border-box;
  }
  .note-dialog textarea:focus {
    outline: none;
    border-color: var(--highlight);
  }
  .note-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 10px;
  }
  .note-save-btn {
    background: var(--highlight);
    color: #fff;
    border-color: var(--highlight);
  }
  .note-save-btn:hover {
    opacity: 0.9;
  }
  .note-tooltip {
    position: fixed;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 13px;
    color: var(--text-primary);
    max-width: 280px;
    word-wrap: break-word;
    box-shadow: 0 2px 10px rgba(0,0,0,0.2);
    z-index: 999;
    pointer-events: none;
    white-space: pre-wrap;
  }
</style>
