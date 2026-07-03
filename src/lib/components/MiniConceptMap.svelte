<script lang="ts">
  import { tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { noteStore } from '../stores/notes';
  import * as d3 from 'd3';

  let svgEl = $state<SVGSVGElement>();
  let note = $derived(noteStore.selectedNote);
  let collapsed = $state(false);
  let nodeCount = $state(0);
  let status = $state('');

  let { context = false, compact = false }: { context?: boolean; compact?: boolean } = $props(); // '' = idle, 'loading', error msg

  const LINK_COLORS: Record<string, string> = { parent: '#888', mention: '#E9C46A', pdf: '#E94560' };

  async function buildGraph() {
    if (!note) return;
    status = 'loading';
    collapsed = false;
    await tick(); // Wait for SVG to enter DOM
    if (!svgEl) { status = 'SVG not ready'; return; }

    let all: { nodes: any[]; links: any[] };
    try { all = await invoke('get_graph_data'); } catch (e: any) {
      status = e?.toString() || 'Failed to load';
      return;
    }

    const connected = new Set<string>([note.id]);
    for (const l of all.links) {
      if (l.source === note.id) connected.add(l.target as string);
      if (l.target === note.id) connected.add(l.source as string);
    }

    const nodes = all.nodes.filter((n: any) => connected.has(n.id)).map((n: any) => ({ ...n }));
    const links = all.links
      .filter((l: any) => connected.has(l.source) && connected.has(l.target))
      .map((l: any) => ({ source: l.source, target: l.target, link_type: l.link_type }));

    nodeCount = nodes.length;
    status = '';
    if (nodes.length === 0) return;

    const w = svgEl.clientWidth || 300;
    const h = 160;
    d3.select(svgEl).selectAll('*').remove();
    const svg = d3.select(svgEl);

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(50))
      .force('charge', d3.forceManyBody().strength(-150))
      .force('center', d3.forceCenter(w / 2, h / 2));

    svg.append('g').selectAll('line').data(links).join('line')
      .attr('stroke', (d: any) => LINK_COLORS[d.link_type] || '#555')
      .attr('stroke-width', 1).attr('stroke-opacity', 0.4);

    const nodeG = svg.append('g').selectAll('g').data(nodes).join('g')
      .attr('cursor', 'pointer')
      .on('click', (_e: any, d: any) => noteStore.selectNote(d.id));

    nodeG.append('circle')
      .attr('r', (d: any) => d.id === note.id ? 10 : 6)
      .attr('fill', (d: any) => d.id === note.id ? '#E94560' : d.has_pdf ? '#ff6b6b' : d.child_count > 0 ? '#8B5CF6' : '#4A90D9')
      .attr('stroke', '#fff').attr('stroke-width', 1.5);

    nodeG.append('text')
      .text((d: any) => d.title.length > 10 ? d.title.slice(0, 8) + '…' : d.title)
      .attr('font-size', 8).attr('fill', 'var(--text-secondary)')
      .attr('text-anchor', 'middle').attr('dy', -14);

    simulation.on('tick', () => {
      nodeG.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
      svg.selectAll('line')
        .attr('x1', (d: any) => d.source.x).attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x).attr('y2', (d: any) => d.target.y);
    });
  }

  $effect(() => { if (note) buildGraph(); });
</script>

{#if note}
  <div class="cmap" class:context={context} class:compact={compact}>
    {#if !context}
    <div class="cmap-header" onclick={() => { collapsed = !collapsed; if (!collapsed) buildGraph(); }} onkeydown={(e) => e.key === 'Enter' && (collapsed = !collapsed)} role="button" tabindex="0">
      <span class="cmap-toggle">{collapsed ? '▶' : '▼'}</span>
      <span class="cmap-title">Concept Map {nodeCount > 0 ? `(${nodeCount})` : ''}</span>
    </div>
    {/if}
    {#if context || !collapsed}
      <svg bind:this={svgEl} class="cmap-svg" class:hidden={status === 'loading' || !!status}></svg>
      {#if status === 'loading'}
        <div class="cmap-msg overlay">Loading…</div>
      {:else if status}
        <div class="cmap-msg overlay">{status}</div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .cmap { border-top: 1px solid var(--border); font-family: var(--font-sans); position: relative; }
  .cmap.context { border-top: none; height: 100%; display: flex; flex-direction: column; }
  .cmap.context .cmap-svg { flex: 1; height: auto; }
  .cmap.compact { border-top: none; }
  .cmap.compact .cmap-svg { height: 120px; }
  .cmap-header { display: flex; align-items: center; padding: 5px 24px; gap: 6px;
    cursor: pointer; user-select: none; font-size: 11px; }
  .cmap-header:hover { background: var(--bg-secondary); }
  .cmap-toggle { font-size: 9px; color: var(--text-secondary); }
  .cmap-title { color: var(--text-secondary); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; }
  .cmap-svg { width: 100%; height: 160px; display: block; }
  .cmap-svg.hidden { display: none; }
  .cmap-msg { padding: 10px 24px; font-size: 13px; color: var(--text-secondary); }
  .cmap-msg.overlay { position: absolute; }
</style>