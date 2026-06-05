<script lang="ts">
  import { onDestroy } from 'svelte';
  import { pdfStore } from '../stores/pdf';
  import { noteStore } from '../stores/notes';

  let { pdfWidth = 50 }: { pdfWidth?: number } = $props();

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
  let selectionPreview: { page: number; x: number; y: number; w: number; h: number } | null = $state(null);

  let showNoteDialog = $state(false);
  let noteDialogData: { page: number; x: number; y: number } | null = $state(null);
  let noteDialogTitle = $state('');
  let noteDialogContent = $state('');
  let editingAnnotationId: string | null = $state(null);

  let tooltipContent = $state('');
  let tooltipX = $state(0);
  let tooltipY = $state(0);

  let searchOpen = $state(false);
  let searchQuery = $state('');
  let searchResults: Array<{ page: number; text: string; x: number; y: number; w: number; h: number }> = $state([]);
  let currentSearchIdx = $state(0);
  let isSearching = $state(false);

  let hThumbLeft = $state(0);
  let hThumbWidth = $state(100);
  let hBarRef = $state<HTMLDivElement>();

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
      const baseW = bodyEl ? Math.max(bodyEl.clientWidth - 24, 200) : 800;
      const effScale = (baseW / vp1.width) * scale;
      const viewport = page.getViewport({ scale: effScale });
      const vpw = viewport.width;
      const vph = viewport.height;

      if (viewerEl) {
        const curMinW = parseFloat(viewerEl.style.minWidth) || 0;
        if (vpw > curMinW) viewerEl.style.minWidth = vpw + 'px';
      }

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

    try {
      const pdfjsLib = await import('pdfjs-dist');
      const workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).href;
      pdfjsLib.GlobalWorkerOptions.workerSrc = workerSrc;
      const loadingTask = pdfjsLib.getDocument({ data: pdfStore.pdfData.slice(0) });
      pdfDoc = await loadingTask.promise;
      numPages = pdfDoc.numPages;
      scale = 1;

      viewerEl.innerHTML = '';
      for (let i = 1; i <= numPages; i++) {
        const el = document.createElement('div');
        el.dataset.page = String(i);
        el.style.minWidth = '100%';
        el.style.marginBottom = '8px';
        el.style.borderRadius = '4px';
        el.style.background = 'var(--bg-primary)';
        el.style.boxShadow = '0 1px 4px rgba(0,0,0,0.2)';
        el.style.minHeight = '200px';
        el.style.height = `${pageHeight}px`;
        viewerEl.appendChild(el);
      }

      await renderPage(1);

      const actualH = pageHeight;
      for (const el of viewerEl.querySelectorAll('[data-page]:not(.rendered)')) {
        (el as HTMLElement).style.height = `${actualH}px`;
      }

      requestAnimationFrame(() => setupObserver());
      noteStore.loadNoteTitles();

      if (pdfStore.targetPage) {
        const target = pdfStore.targetPage;
        requestAnimationFrame(() => {
          const el = viewerEl?.querySelector(`[data-page="${target}"]`) as HTMLElement | null;
          if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
          pdfStore.targetPage = null;
        });
      }
    } catch (err) {
      console.error('Failed to load PDF:', err);
    } finally {
      isLoading = false;
    }
  }

  let zoomGen = 0;

  async function changeScale(newScale: number) {
    const gen = ++zoomGen;
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
    if (viewerEl) viewerEl.style.minWidth = '';

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
      if (zoomGen !== gen) return;

      const oldScroll = bodyEl.scrollTop;
      const clientH = bodyEl.clientHeight;
      const margin = 600;
      const startPage = Math.max(1, Math.floor((oldScroll - clientH - margin) / (newEstHeight + gap)));
      const endPage = Math.min(numPages, Math.ceil((oldScroll + 2 * clientH + margin) / (newEstHeight + gap)));
      for (let i = startPage; i <= endPage; i++) {
        if (i !== 1) await renderPage(i);
      }
      if (zoomGen !== gen) return;

      await new Promise(r => requestAnimationFrame(r));
      if (zoomGen !== gen) return;
      const anchorEl = viewerEl.querySelector(`[data-page="${anchorPage}"]`) as HTMLElement | null;
      if (anchorEl) {
        const newRect = anchorEl.getBoundingClientRect();
        bodyEl.scrollTop += newRect.top - anchorOffset;
      }

      setupObserver();

      await new Promise(r => requestAnimationFrame(r));
      if (zoomGen !== gen) return;
      const anchorEl2 = viewerEl.querySelector(`[data-page="${anchorPage}"]`) as HTMLElement | null;
      if (anchorEl2) {
        const newRect2 = anchorEl2.getBoundingClientRect();
        bodyEl.scrollTop += newRect2.top - anchorOffset;
      }
    } catch (e) {
      console.error('Zoom failed:', e);
      setupObserver();
    }
    requestAnimationFrame(() => updateHScroll());
  }

  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      if (isLoading) return;
      changeScale(scale + (e.deltaY > 0 ? -0.1 : 0.1));
    }
  }

  function updateHScroll() {
    if (!bodyEl || !hBarRef) return;
    const sw = bodyEl.scrollWidth;
    const cw = bodyEl.clientWidth;
    if (sw <= cw) {
      hThumbLeft = 0;
      hThumbWidth = 100;
      return;
    }
    hThumbWidth = Math.max(10, (cw / sw) * 100);
    hThumbLeft = (bodyEl.scrollLeft / (sw - cw)) * (100 - hThumbWidth);
  }

  function handleHScrollClick(e: MouseEvent) {
    if (!bodyEl || !hBarRef || scale <= 1) return;
    const rect = hBarRef.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const ratio = clickX / rect.width;
    bodyEl.scrollLeft = ratio * (bodyEl.scrollWidth - bodyEl.clientWidth);
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
    if (selectionPreview && selectionPreview.page === pageNum) {
      const { x, y, w, h } = selectionPreview;
      ctx.save();
      ctx.globalAlpha = 0.25;
      ctx.fillStyle = activeTool === 'highlight' ? '#E94560' : '#E94560';
      ctx.fillRect(x * vpW, y * vpH, w * vpW, h * vpH);
      ctx.globalAlpha = 0.8;
      ctx.strokeStyle = '#E94560';
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 3]);
      ctx.strokeRect(x * vpW, y * vpH, w * vpW, h * vpH);
      ctx.setLineDash([]);
      ctx.restore();
    }
    for (let i = 0; i < searchResults.length; i++) {
      const m = searchResults[i];
      if (m.page !== pageNum) continue;
      const isActive = i === currentSearchIdx;
      const sx = m.x * vpW;
      const sy = m.y * vpH;
      const sw = m.w * vpW;
      const sh = m.h * vpH;
      if (isActive) {
        ctx.globalAlpha = 0.45;
        ctx.fillStyle = '#FF6B00';
        ctx.fillRect(sx, sy, sw, sh);
        ctx.globalAlpha = 1;
        ctx.strokeStyle = '#FF4500';
        ctx.lineWidth = 2.5;
        ctx.strokeRect(sx, sy, sw, sh);
      } else {
        ctx.globalAlpha = 0.3;
        ctx.fillStyle = '#FFD65A';
        ctx.fillRect(sx, sy, sw, sh);
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

  function redrawAllOverlays() {
    for (const [pageNum] of renderedPages) {
      redrawPageAnnotations(pageNum);
    }
  }

  function toggleSearch() {
    searchOpen = !searchOpen;
    if (!searchOpen) {
      searchQuery = '';
      searchResults = [];
      currentSearchIdx = 0;
      redrawAllOverlays();
    }
  }

  let searchGen = 0;

  async function runSearch() {
    const gen = ++searchGen;
    const q = searchQuery.trim();
    if (!q || !pdfDoc) {
      searchResults = [];
      redrawAllOverlays();
      return;
    }
    isSearching = true;
    const results: Array<{ page: number; text: string; x: number; y: number; w: number; h: number }> = [];
    currentSearchIdx = 0;

    const query = q.toLowerCase();
    for (let i = 1; i <= numPages; i++) {
      if (searchGen !== gen) return;
      try {
        const page = await pdfDoc.getPage(i);
        const textContent = await page.getTextContent();
        const vp1 = page.getViewport({ scale: 1 });
        const rx = 1 / vp1.width;
        const ry = 1 / vp1.height;

        for (const item of textContent.items as any[]) {
          const str: string = item.str;
          if (!str) continue;
          const idx = str.toLowerCase().indexOf(query);
          if (idx === -1) continue;
          const tx = item.transform[4] as number;
          const ty = item.transform[5] as number;
          results.push({
            page: i,
            text: str,
            x: tx * rx,
            y: ty * ry,
            w: (item.width as number) * rx,
            h: (item.height as number) * Math.abs(item.transform[3] as number) * ry,
          });
        }
      } catch { /* skip */ }
    }
    if (searchGen !== gen) return;
    searchResults = results;
    isSearching = false;
    redrawAllOverlays();
  }

  async function goToSearchMatch(dir: 1 | -1) {
    if (searchResults.length === 0) return;
    currentSearchIdx = (currentSearchIdx + dir + searchResults.length) % searchResults.length;
    const match = searchResults[currentSearchIdx];
    await renderPage(match.page);
    const el = viewerEl?.querySelector(`[data-page="${match.page}"]`) as HTMLElement | null;
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    redrawAllOverlays();
  }

  function findAnnotationAt(e: MouseEvent, pad = 0.015): { page: number; annotation: typeof pdfStore.annotations[0] } | null {
    const target = (e.target as HTMLElement).closest('[data-page]') as HTMLElement | null;
    if (!target) return null;
    const page = Number(target.dataset.page);
    const rect = target.getBoundingClientRect();
    const relX = (e.clientX - rect.left) / rect.width;
    const relY = (e.clientY - rect.top) / rect.height;

    for (const ann of pdfStore.annotations) {
      if (ann.page !== page) continue;
      if (ann.annotation_type === 'note') {
        if (relX >= ann.x - pad && relX <= ann.x + ann.width + pad &&
            relY >= ann.y - pad && relY <= ann.y + ann.height + pad) {
          return { page, annotation: ann };
        }
      } else if (ann.annotation_type === 'highlight') {
        if (relX >= ann.x && relX <= ann.x + ann.width &&
            relY >= ann.y && relY <= ann.y + ann.height) {
          return { page, annotation: ann };
        }
      }
    }
    return null;
  }

  function getNoteIdFromAnnotation(ann: typeof pdfStore.annotations[0]): string | null {
    if (ann.annotation_type !== 'note' || !ann.content || !ann.content.startsWith('note:')) return null;
    return ann.content.slice(5);
  }

  function getAnnotationTooltip(ann: typeof pdfStore.annotations[0]): string {
    if (ann.annotation_type === 'highlight') return ann.content || '';
    const noteId = getNoteIdFromAnnotation(ann);
    if (noteId) {
      const note = noteStore.notes.find(n => n.id === noteId);
      if (note) return note.title;
      return 'Note';
    }
    return ann.content || '';
  }

  function handleMouseMove(e: MouseEvent) {
    const found = findAnnotationAt(e);
    if (found) {
      const tip = getAnnotationTooltip(found.annotation);
      if (tip) {
        tooltipContent = tip;
        tooltipX = e.clientX + 12;
        tooltipY = e.clientY + 12;
      } else {
        tooltipContent = '';
      }
    } else {
      tooltipContent = '';
    }

    if (selectionStart && activeTool !== 'cursor') {
      const target = (e.target as HTMLElement).closest('[data-page]');
      if (!target) return;
      const rect = target.getBoundingClientRect();
      const page = Number((target as HTMLElement).dataset.page);
      const endX = (e.clientX - rect.left) / rect.width;
      const endY = (e.clientY - rect.top) / rect.height;
      const x = Math.min(selectionStart.x, endX);
      const y = Math.min(selectionStart.y, endY);
      const w = Math.abs(endX - selectionStart.x);
      const h = Math.abs(endY - selectionStart.y);
      selectionPreview = { page, x, y, w, h };
      redrawPageAnnotations(page);
    }
  }

  function handleDblClick(e: MouseEvent) {
    if (activeTool !== 'cursor') return;
    const found = findAnnotationAt(e, 0.025);
    if (!found) return;
    tooltipContent = '';
    const noteId = getNoteIdFromAnnotation(found.annotation);
    if (noteId) {
      noteStore.selectNote(noteId);
    }
  }

  async function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    const found = findAnnotationAt(e, 0.03);
    if (found) {
      if (confirm(`Delete this ${found.annotation.annotation_type}?`)) {
        await pdfStore.deleteAnnotation(found.annotation.id);
        redrawPageAnnotations(found.page);
      }
    } else {
      if (selectionStart) {
        selectionStart = null;
        selectionPreview = null;
        redrawAllOverlays();
      }
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && selectionStart) {
      selectionStart = null; selectionPreview = null;
      selectionPreview = null;
      redrawAllOverlays();
      tooltipContent = '';
    }
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
      if (activeTool !== 'note') { selectionStart = null; selectionPreview = null; return; }
      tooltipContent = '';
      noteDialogData = { page, x, y };
      noteDialogTitle = '';
      noteDialogContent = '';
      editingAnnotationId = null;
      showNoteDialog = true;
      selectionStart = null; selectionPreview = null;
      return;
    } else {
      let captured: string | null = null;
      if (activeTool === 'highlight' && pdfDoc) {
        try {
          const hlPage = await pdfDoc.getPage(page);
          const vp1 = hlPage.getViewport({ scale: 1 });
          const textContent = await hlPage.getTextContent();
          const pdfX = x * vp1.width;
          const pdfY = y * vp1.height;
          const pdfW = w * vp1.width;
          const pdfH = h * vp1.height;
          captured = (textContent.items as any[])
            .filter((item: any) => {
              const tx = item.transform[4] as number;
              const ty = item.transform[5] as number;
              const iw = (item.width as number) * (item.transform[0] as number);
              const ih = (item.height as number) * Math.abs(item.transform[3] as number);
              return tx + iw >= pdfX - 3 && tx <= pdfX + pdfW + 3 &&
                     ty + ih >= pdfY - 3 && ty <= pdfY + pdfH + 3;
            })
            .map((item: any) => item.str)
            .join(' ')
            .trim() || null;
        } catch { /* ignore text extraction errors */ }
      }
      await pdfStore.saveAnnotation({
        pdf_id: pdfStore.activePdf!.id,
        page, annotation_type: activeTool,
        x, y, width: w, height: h,
        color: '#E94560', content: captured,
      });
    }

    selectionStart = null; selectionPreview = null;
    redrawPageAnnotations(page);
  }

  async function saveNoteDialog() {
    if (!noteDialogData) return;
    const { page, x, y } = noteDialogData;

    const note = await noteStore.createNote({
      title: noteDialogTitle || 'PDF Note',
      content: noteDialogContent || '',
      parent_id: noteStore.selectedNote?.id ?? null,
    });

    if (note) {
      const ann = await pdfStore.saveAnnotation({
        pdf_id: pdfStore.activePdf!.id,
        page, annotation_type: 'note',
        x, y, width: 0.05, height: 0.05,
        color: '#E94560', content: `note:${note.id}`,
      });
      await pdfStore.linkToNote(note.id, pdfStore.activePdf!.id);
      if (ann) {
        await pdfStore.createPdfReference({
          note_id: note.id,
          pdf_id: pdfStore.activePdf!.id,
          page,
          page_end: null,
          label: noteDialogTitle || `Page ${page}`,
          annotation_id: ann.id,
        });
      }
      if (noteStore.selectedNote) {
        pdfStore.loadLinkedPdfs(noteStore.selectedNote.id);
      }
    }

    showNoteDialog = false;
    noteDialogData = null;
    noteDialogTitle = '';
    noteDialogContent = '';
    editingAnnotationId = null;
    redrawPageAnnotations(page);
    tooltipContent = '';
  }

  function cancelNoteDialog() {
    showNoteDialog = false;
    noteDialogData = null;
    noteDialogTitle = '';
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

<div class="pdf-viewer" class:open={pdfStore.isOpen} style={`width: ${pdfWidth}%`}>
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
      {#if scale > 1}
        <button class="tool-btn nav-btn" onclick={() => { if (bodyEl) bodyEl.scrollLeft -= 300; }} title="Pan left">◀</button>
        <button class="tool-btn nav-btn" onclick={() => { if (bodyEl) bodyEl.scrollLeft += 300; }} title="Pan right">▶</button>
      {/if}
    </div>
    <div class="pdf-toolbar-right">
      <button class="tool-btn" class:active={activeTool === 'cursor'} onclick={() => activeTool = 'cursor'} title="Cursor">↖</button>
      <button class="tool-btn" class:active={activeTool === 'highlight'} onclick={() => activeTool = 'highlight'} title="Highlight">⬛</button>
      <button class="tool-btn" class:active={activeTool === 'note'} onclick={() => activeTool = 'note'} title="Add note">📝</button>
      <button class="tool-btn" onclick={toggleSearch} title="Search PDF">🔍</button>
      {#if noteStore.selectedNote}
        {@const linked = pdfStore.activePdf && pdfStore.linkedPdfIds.includes(pdfStore.activePdf.id)}
        <button class="tool-btn link-btn" class:active={linked} onclick={async () => {
          if (linked) await pdfStore.unlinkFromNote(noteStore.selectedNote!.id, pdfStore.activePdf!.id);
          else await pdfStore.linkToNote(noteStore.selectedNote!.id, pdfStore.activePdf!.id);
          if (noteStore.selectedNote) pdfStore.loadLinkedPdfs(noteStore.selectedNote.id);
        }} title={linked ? 'Unlink from current note' : 'Link to current note'}>
          {linked ? '🔓' : '🔗'}
        </button>
      {/if}
    </div>
  </div>

  {#if searchOpen}
    <div class="search-bar">
      <input type="text" bind:value={searchQuery} oninput={runSearch} placeholder="Search in PDF..." />
      {#if searchResults.length > 0}
        <span class="search-count">{currentSearchIdx + 1}/{searchResults.length}</span>
        <button class="tool-btn" onclick={() => goToSearchMatch(-1)} title="Previous">▲</button>
        <button class="tool-btn" onclick={() => goToSearchMatch(1)} title="Next">▼</button>
      {/if}
      {#if isSearching}
        <span class="search-spinner">…</span>
      {/if}
      <button class="tool-btn" onclick={toggleSearch}>✕</button>
    </div>
  {/if}

  <div class="pdf-body" bind:this={bodyEl} onwheel={handleWheel} onscroll={updateHScroll} oncontextmenu={handleContextMenu}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="pdf-pages" bind:this={viewerEl} onmousedown={startAnnotation} onmouseup={endAnnotation} onmousemove={handleMouseMove} ondblclick={handleDblClick} onkeydown={handleKeyDown} tabindex="0"></div>
    {#if isLoading}
      <div class="pdf-loading">Loading PDF…</div>
    {/if}
  </div>

  {#if scale > 1}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="h-scrollbar" bind:this={hBarRef} onmousedown={handleHScrollClick} onclick={() => {}}>
      <div class="h-scrollbar-thumb" style="left: {hThumbLeft}%; width: {hThumbWidth}%;"></div>
    </div>
  {/if}
</div>

{#if showNoteDialog}
<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="note-dialog-overlay" onclick={cancelNoteDialog}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="note-dialog" onclick={(e) => e.stopPropagation()}>
    <div class="note-dialog-header">
      Add Note
    </div>
    <input
      type="text"
      bind:value={noteDialogTitle}
      placeholder="Note title"
      class="note-dialog-input"
    />
    <textarea
      bind:value={noteDialogContent}
      placeholder="Note content (optional)..."
      rows={3}
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
    overflow: auto;
    padding: 12px;
    position: relative;
  }
  .pdf-body::-webkit-scrollbar {
    width: 10px;
    height: 10px;
  }
  .pdf-body::-webkit-scrollbar-track {
    background: var(--bg-secondary);
    border-radius: 4px;
  }
  .pdf-body::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
    border: 2px solid var(--bg-secondary);
  }
  .pdf-body::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary);
  }
  .pdf-body::-webkit-scrollbar-corner {
    background: var(--bg-secondary);
  }
  .search-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-primary);
    flex-shrink: 0;
  }
  .search-bar input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 13px;
    color: var(--text-primary);
    font-family: var(--font-sans);
    outline: none;
    min-width: 0;
  }
  .search-bar input:focus {
    border-color: var(--highlight);
  }
  .search-count {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .search-spinner {
    font-size: 14px;
    color: var(--text-secondary);
    animation: pulse 0.8s ease-in-out infinite;
  }
  @keyframes pulse {
    50% { opacity: 0.2; }
  }
  .link-btn.active {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-color: var(--highlight);
  }
.pdf-pages {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .pdf-body {
    flex: 1;
    overflow: auto;
    padding: 12px;
    position: relative;
    scrollbar-width: auto;
    scrollbar-color: #999 var(--bg-primary);
  }
  .pdf-body::-webkit-scrollbar {
    width: 14px;
    height: 14px;
  }
  .pdf-body::-webkit-scrollbar-track {
    background: var(--bg-primary);
    border: 1px solid var(--border);
  }
  .pdf-body::-webkit-scrollbar-thumb {
    background: #888;
    border-radius: 7px;
    border: 3px solid var(--bg-primary);
    min-height: 40px;
  }
  .pdf-body::-webkit-scrollbar-thumb:hover {
    background: #aaa;
  }
  .pdf-body::-webkit-scrollbar-thumb:active {
    background: #ccc;
  }
  .pdf-body::-webkit-scrollbar-corner {
    background: var(--bg-primary);
  }
  .nav-btn {
    background: var(--bg-secondary);
    font-size: 11px;
  }
  .h-scrollbar {
    height: 10px;
    background: var(--bg-primary);
    border-top: 1px solid var(--border);
    cursor: pointer;
    flex-shrink: 0;
    position: relative;
  }
  .h-scrollbar-thumb {
    position: absolute;
    top: 1px;
    bottom: 1px;
    background: var(--highlight);
    border-radius: 4px;
    min-width: 20px;
    transition: left 0.05s linear;
  }
  .h-scrollbar:hover .h-scrollbar-thumb {
    background: var(--text-primary);
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
  .note-dialog-input {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 8px;
    font-size: 14px;
    font-weight: 600;
    font-family: var(--font-sans);
    color: var(--text-primary);
    box-sizing: border-box;
    margin-bottom: 8px;
  }
  .note-dialog-input:focus {
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
