# Phase 3: Graph View + Dual Coding (Complete)

> Status: Complete. Interactive force-directed graph + per-note concept maps.

## Architecture

```
Rust command (get_graph_data)
  → Reads all notes + relationships
  → Builds nodes (id, title, link_count, has_pdf, child_count)
  → Builds links from 3 sources:
      1. parent_id (parent-child hierarchy)
      2. @-mentions in note content (regex)
      3. pdf_references (note ↔ PDF page)
  → Returns {nodes, links}

Frontend
  → GraphView.svelte: full-screen D3.js force simulation
  → MiniConceptMap.svelte: per-note mini-graph in context panel
  → Dual Coding: text + visual representation (technique 6/6)
```

## Backend

### Command: get_graph_data

| File | Purpose |
|---|---|
| `commands/graph.rs` | Builds GraphData from all notes |
| `lib.rs` | Registered as `commands::graph::get_graph_data` |

### Dependencies

| Crate | Purpose |
|---|---|
| `regex` `^1` | Parse `@-mentions` in note content |

### Link Types

| Type | Source | Color |
|---|---|---|
| `parent` | `parent_id` field | Gray (#888) |
| `mention` | `@Titulo` in markdown | Yellow (#E9C46A) |
| `pdf` | `pdf_references` table | Red (#E94560) |

### Node Fields

| Field | Purpose |
|---|---|
| `id` | Note ID |
| `title` | Display name |
| `parent_id` | For hierarchy |
| `child_count` | Number of children (large nodes = sections) |
| `link_count` | Total connections (larger radius) |
| `has_pdf` | Node color (red if linked to PDF) |

## Frontend

### Components

| Component | Purpose |
|---|---|
| `GraphView.svelte` | Full-screen D3 force graph. All notes visible. Zoom/pan/drag. Click → open note. |
| `MiniConceptMap.svelte` | Per-note mini-graph (Dual Coding). Shows only connections for selected note. Collapsible in editor / permanent in context panel. |

### Dependencies

| Package | Purpose |
|---|---|
| `d3` `^7` | forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide, zoom |
| `@types/d3` | TypeScript definitions |

### GraphView.svelte Details

- **SVG canvas** — full viewport, no scrolling
- **Force simulation** — charge (-300), link distance (100), center, collision (radius 30)
- **Zoom** — d3.zoom with scale extent [0.2, 4]
- **Drag** — nodes can be dragged to rearrange
- **Colors**: root (green #4CAF50), branch (purple #8B5CF6), leaf (blue #4A90D9), PDF note (red #E94560)
- **Radius**: 6 + min(link_count * 2, 20) — hub notes are larger
- **Labels**: truncated to 18 chars
- **Tooltip**: hover shows full title

### MiniConceptMap.svelte Details

- **Smaller**: 160px height, weaker forces (-150 charge, 50 link distance)
- **Context panel mode**: `clamp(200px, 20%, 320px)` responsive width
- **Compact mode**: 120px height when sharing space with PDF Outline
- **Header**: collapsible (▼/▶) with node count
- **Auto-rebuilds** when selected note changes via `$effect`

### Dual Coding (Learning Technique 6/6)

The Concept Map implements Dual Coding (Paivio 1971, Mayer 2009): combining textual content (the note) with a visual representation (the graph of connections). Evidence shows this doubles retention compared to text alone.

## Data Flow

### Graph View
1. User clicks `📊 Graph` → `aiStore.toggleGraph()` sets `graphOpen = true`
2. App.svelte renders `<GraphView />` full-screen
3. `onMount` → `invoke('get_graph_data')`
4. D3 renders SVG with force simulation
5. Click node → `noteStore.selectNote(id)` → opens in Editor
6. Close button → `aiStore.toggleGraph()` → returns to Editor

### Concept Map
1. User selects a note → `$effect` fires → `buildGraph()`
2. Fetches ALL graph data, filters for connected nodes only
3. Renders mini SVG with same D3 pattern (smaller scale)
4. Click connected node → opens that note
5. Collapse/expand via ▼/▶ toggle
