<script lang="ts">
  /**
   * GraphView — full-screen interactive force-directed graph of all notes.
   * Uses D3.js v7 (forceSimulation, zoom, drag). Nodes = notes, links = relationships.
   * Link types: parent (gray), @-mention (yellow), PDF reference (red).
   * Click a node to open the note. Drag nodes to rearrange. Scroll to zoom.
   * Data comes from the `get_graph_data` Rust command.
   */
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { noteStore } from '../stores/notes';
  import { aiStore } from '../stores/ai';
  import * as d3 from 'd3';

  let svgEl = $state<SVGSVGElement>();
  let tooltip = $state<{ x: number; y: number; text: string } | null>(null);
  let loading = $state(true);
  let error = $state('');
  let width = $state(800);
  let height = $state(600);
  let nodeCount = $state(0);

  interface GraphNode { id: string; title: string; parent_id: string | null; child_count: number; link_count: number; has_pdf: boolean; }

  const LINK_COLORS: Record<string, string> = { parent: '#888', mention: '#E9C46A', pdf: '#E94560' };

  onMount(async () => {
    let data: { nodes: GraphNode[]; links: any[] };
    try {
      data = await invoke('get_graph_data');
    } catch (e: any) {
      error = e?.toString() || 'Failed to load graph';
      loading = false;
      return;
    }
    loading = false;
    nodeCount = data.nodes.length;

    if (data.nodes.length === 0) { error = 'No notes to graph. Create some notes first.'; return; }
    await tick();
    if (!svgEl) { error = 'SVG not ready'; return; }

    const svg = d3.select(svgEl);
    svg.selectAll('*').remove();
    width = svgEl.clientWidth;
    height = svgEl.clientHeight;

    const nodeMap = new Map(data.nodes.map(n => [n.id, n]));
    const links = data.links
      .filter(l => typeof l.source === 'string' && typeof l.target === 'string')
      .map(l => ({ source: l.source as string, target: l.target as string, link_type: l.link_type }))
      .filter(l => nodeMap.has(l.source) && nodeMap.has(l.target));

    const nodes = data.nodes.map(n => ({ ...n }));

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(100))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    const g = svg.append('g');
    svg.call(d3.zoom<any, any>().scaleExtent([0.2, 4]).on('zoom', (ev: any) => g.attr('transform', ev.transform)));

    const link = g.append('g').selectAll('line').data(links).join('line')
      .attr('stroke', (d: any) => LINK_COLORS[d.link_type] || '#555')
      .attr('stroke-width', (d: any) => d.link_type === 'parent' ? 1 : 2)
      .attr('stroke-opacity', 0.5);

    const node = g.append('g').selectAll('circle').data(nodes).join('circle')
      .attr('r', (d: any) => 6 + Math.min(d.link_count * 2, 20))
      .attr('fill', (d: any) => d.has_pdf ? '#E94560' : d.parent_id === null ? '#4CAF50' : d.child_count > 0 ? '#8B5CF6' : '#4A90D9')
      .attr('stroke', '#fff').attr('stroke-width', 1.5).attr('cursor', 'pointer')
      .on('click', (_e: any, d: any) => noteStore.selectNote(d.id))
      .on('mouseenter', (e: any, d: any) => {
        tooltip = { x: e.clientX + 10, y: e.clientY - 10, text: d.title };
        d3.select(e.currentTarget).attr('stroke', '#fff').attr('stroke-width', 3);
      })
      .on('mouseleave', (e: any) => { tooltip = null; d3.select(e.currentTarget).attr('stroke', '#fff').attr('stroke-width', 1.5); })
      .call(d3.drag<any, any>()
        .on('start', (ev: any, d: any) => { if (!ev.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
        .on('drag', (ev: any, d: any) => { d.fx = ev.x; d.fy = ev.y; })
        .on('end', (ev: any, d: any) => { if (!ev.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; }));

    const label = g.append('g').selectAll('text').data(nodes).join('text')
      .text((d: any) => d.title.length > 20 ? d.title.slice(0, 18) + '…' : d.title)
      .attr('font-size', 10).attr('fill', 'var(--text-secondary)')
      .attr('text-anchor', 'middle').attr('dy', (d: any) => -10 - (6 + Math.min(d.link_count * 2, 20)))
      .attr('pointer-events', 'none');

    simulation.on('tick', () => {
      link.attr('x1', (d: any) => d.source.x).attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x).attr('y2', (d: any) => d.target.y);
      node.attr('cx', (d: any) => d.x).attr('cy', (d: any) => d.y);
      label.attr('x', (d: any) => d.x).attr('y', (d: any) => d.y);
    });
  });
</script>

<div class="graph-view">
  <div class="graph-toolbar">
    <span class="graph-title">Graph View {nodeCount > 0 ? `(${nodeCount} notes)` : ''}</span>
    <div class="graph-legend">
      <span class="legend-item"><span class="legend-swatch hl-parent"></span>Parent</span>
      <span class="legend-item"><span class="legend-swatch hl-mention"></span>Mention</span>
      <span class="legend-item"><span class="legend-swatch hl-pdf"></span>PDF</span>
    </div>
    <button class="graph-close" onclick={() => aiStore.toggleGraph()}>✕</button>
  </div>

  <div class="graph-container">
  {#if loading}
    <div class="graph-loading">Building graph…</div>
  {:else if error}
    <div class="graph-loading">{error}</div>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <svg bind:this={svgEl} class="graph-svg" onmousemove={() => { tooltip = null; }}></svg>
    {#if tooltip}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="graph-tooltip" style="left: {tooltip.x}px; top: {tooltip.y}px;">{tooltip.text}</div>
    {/if}
{/if}
  </div>
</div>

<style>
  .graph-view { display: flex; flex-direction: column; flex: 1;
    background: var(--bg-primary); font-family: var(--font-sans); }
  .graph-toolbar { display: flex; align-items: center; justify-content: space-between;
    padding: 8px 16px; border-bottom: 1px solid var(--border);
    background: var(--bg-secondary); flex-shrink: 0; }
  .graph-title { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .graph-close { background: none; border: none; cursor: pointer; font-size: 16px;
    padding: 2px 8px; border-radius: 4px; color: var(--text-secondary); font-family: inherit; }
  .graph-close:hover { background: var(--accent); color: var(--text-primary); }
  .graph-legend { display: flex; gap: 12px; font-size: 11px; color: var(--text-secondary); }
  .legend-item { display: flex; align-items: center; gap: 4px; }
  .legend-swatch { width: 10px; height: 10px; border-radius: 2px; flex-shrink: 0; }
  .hl-parent { background: #888; }
  .hl-mention { background: #E9C46A; }
  .hl-pdf { background: #E94560; }
  .graph-container { flex: 1; position: relative; overflow: hidden; }
  .graph-svg { width: 100%; height: 100%; }
  .graph-loading { color: var(--text-secondary); font-size: 14px; font-family: var(--font-sans); }
  .graph-tooltip { position: fixed; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; font-size: 12px; color: var(--text-primary); font-family: var(--font-sans); pointer-events: none; z-index: 100; box-shadow: 0 2px 8px rgba(0,0,0,0.2); white-space: nowrap; }
</style>