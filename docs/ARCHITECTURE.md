# Architecture

## Overview

JSTNotes is a hybrid desktop app: **Tauri v2** provides the native shell, **Svelte 5 + TypeScript** runs the frontend in a webview, and **Rust** handles all backend logic (storage, AI, PDF). Communication between frontend and backend uses Tauri's `invoke` IPC mechanism.

```
┌─────────────────────────────────────┐
│           Tauri Webview             │
│  ┌───────────────────────────────┐  │
│  │     Svelte 5 App             │  │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ │  │
│  │  │Editor│ │Tree  │ │Settings│ │  │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ │  │
│  │     │        │        │      │  │
│  │  ┌──┴────────┴────────┴───┐  │  │
│  │  │     Stores (.svelte.ts) │  │  │
│  │  └──────────┬─────────────┘  │  │
│  └─────────────┼────────────────┘  │
└────────────────┼───────────────────┘
                 │ invoke() IPC
┌────────────────┼───────────────────┐
│  Tauri Rust    ▼                   │
│  ┌──────────────────────────────┐  │
│  │      Commands (notes,        │  │
│  │      settings, ai, pdf)      │  │
│  └──────┬────────────────┬──────┘  │
│         ▼                ▼         │
│  ┌──────────┐    ┌──────────────┐  │
│  │Hybrid    │    │  AI (stub)   │  │
│  │Storage   │    │  PDF (stub)  │  │
│  ├──────────┤    └──────────────┘  │
│  │SQLite    │                      │
│  │Markdown  │                      │
│  └──────────┘                      │
└────────────────────────────────────┘
```

---

## Frontend (Svelte 5)

### Component tree

```
App.svelte
├── Sidebar
│   ├── NoteTree.svelte
│   │   └── TreeItem.svelte (recursive)
│   └── resize handle (draggable)
├── Main
│   ├── Breadcrumbs.svelte
│   └── Editor.svelte (split markdown)
├── StatusBar
└── Settings.svelte (modal overlay)
    ├── Presets tab
    ├── Colors tab
    ├── Font tab
    ├── Layout tab
    └── CSS tab
```

### State management

Uses **Svelte 5 runes** (`$state`, `$derived`, `$effect`) with the `.svelte.ts` extension for module-level stores.

| Store | File | Purpose |
|---|---|---|
| `noteStore` | `src/lib/stores/notes.svelte.ts` | Note tree, selection, CRUD |
| `settingsStore` | `src/lib/stores/settings.svelte.ts` | Theme, colors, typography, layout, custom CSS |

**Critical pattern:** Always pass `JSON.parse(JSON.stringify())` copies to Tauri `invoke`. Svelte 5 `$state` proxies cannot be cloned by `structuredClone` (used internally by Tauri IPC), causing `DataCloneError`.

### Settings system

`settingsStore` applies CSS custom properties to `:root` on every change:

```
--bg-primary     --text-primary
--bg-secondary   --text-secondary
--accent         --border
--highlight      --font-sans
                  --font-mono
--sidebar-width  --editor-font-size
                  --editor-line-height
```

The `Presets` tab allows hover preview (via `applyThemeFrom()`) without saving, then applies permanently on click.

### Custom CSS

A `<style id="jstnotes-custom-css">` tag is injected into `<head>` and updated on every keystroke in the CSS editor tab. This overrides any CSS variable or component style.

---

## Backend (Rust)

### Module map

```
src-tauri/src/
├── main.rs              # Entry point
├── lib.rs               # Tauri setup, plugin registration, command registration
├── models.rs            # Data structs + serde + load/persist settings
├── commands/
│   ├── mod.rs
│   ├── notes.rs         # 9 IPC commands for note CRUD
│   ├── settings.rs      # get_settings / save_settings
│   ├── ai.rs            # stub
│   └── pdf.rs           # stub
├── storage/
│   ├── mod.rs
│   ├── hybrid.rs        # Sync layer between SQLite + Markdown
│   ├── sqlite.rs        # SQLite CRUD, tree building, breadcrumbs
│   └── markdown.rs      # Read/write .md files
├── ai/                  # Future: embeddings, inference, RAG
│   ├── mod.rs
│   ├── models.rs
│   ├── embeddings.rs
│   ├── inference.rs
│   └── rag.rs
└── pdf/                 # Future: PDF parsing, annotations
    ├── mod.rs
    └── extract.rs
```

### Hybrid storage

**Dual storage** — content in Markdown, metadata in SQLite:

| Aspect | SQLite | Markdown files |
|---|---|---|
| Note ID | Primary key | Filename (`{uuid}.md`) |
| Title | `title` column | First `# ` heading, or filename |
| Content | — | Full body |
| Parent ID | `parent_id` column | — |
| Timestamps | `created_at`, `updated_at` | Filesystem mtime |
| Path | Computed from hierarchy | `notes/{parent_tree}/{uuid}.md` |

`HybridStorage` ensures both layers stay in sync. Every read/write goes through it.

### SQLite schema

```sql
CREATE TABLE notes (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    parent_id   TEXT,
    sort_order  INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES notes(id)
);
```

### Commands

All 11 commands are registered in `lib.rs`:

| Command | Description |
|---|---|
| `create_note` | Create a note with optional parent |
| `get_note` | Fetch single note by ID |
| `update_note` | Update title and/or content |
| `delete_note` | Delete note and all descendants |
| `list_notes` | List all notes (flat) |
| `get_children` | Get direct children of a note |
| `get_breadcrumbs` | Path from root to note |
| `get_tree` | Full nested tree |
| `get_settings` | Load AppSettings from disk |
| `save_settings` | Persist AppSettings + apply theme |
| `export_note` | Export single note as Markdown |

### Settings persistence

Settings are stored as JSON at `{app_data_dir}/settings.json`. The `AppSettingsState` holds a `Mutex<PathBuf>` pointing to the data directory, initialized once at app startup.

---

## Data flow

### Opening a note

```
User clicks note in sidebar
  → NoteTree → selectNode(id)
    → noteStore.selectNote(id)
      → invoke('get_breadcrumbs', { id })
      → invoke('get_note', { id })
      → updates $state → Editor reacts
```

### Auto-save

```
User types in editor
  → 500ms debounce
    → noteStore.updateNote()
      → invoke('update_note', { id, title, content })
        → HybridStorage.save()
          → SQLite update + Markdown write
```

---

## Key design decisions

| Decision | Rationale |
|---|---|
| `.svelte.ts` for stores | Svelte 5 requires runes in module files to be `.svelte.ts` |
| `JSON.parse(JSON.stringify())` for deep copies | Avoids `DataCloneError` from `structuredClone` on `$state` proxies |
| Vite `target: 'esnext'` | Tauri's webview (modern Chromium) supports all ESNext features |
| Devtools auto-open in debug | `window.open_devtools()` in Rust for debug builds |
| `expanded = $state(true)` on TreeItem | Nodes start collapsed; saves rendering on large trees |

---

## Future architecture (Phase 1+)

### AI pipeline

```
User query
  → Embedding → Vector search (SQLite FTS5 + embeddings)
    → Context retrieval → LLM prompt
      → Streamed response → ChatPanel / FlashCard generation
```

### PDF pipeline

```
Open PDF
  → Render via PDF.js (frontend)
    → Text extraction (Rust)
      → Annotation overlay (Svelte)
        → Save annotations to SQLite
```

### Graph view

```
D3.js / Canvas rendering
  → Nodes = notes
    → Edges = parent-child + backlinks + AI-suggested connections
      → Force-directed layout
```
