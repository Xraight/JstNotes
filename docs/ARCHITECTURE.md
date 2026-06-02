# Architecture

## Overview

JSTNotes is a hybrid desktop app: **Tauri v2** provides the native shell,
**Svelte 5 + TypeScript** runs the frontend in a webview, and **Rust**
handles all backend logic (storage, AI, PDF). Communication between frontend
and backend uses Tauri's `invoke` IPC mechanism.

```
┌─────────────────────────────────────┐
│           Tauri Webview             │
│  ┌───────────────────────────────┐  │
│  │     Svelte 5 App             │  │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ │  │
│  │  │Editor│ │Tree  │ │PdfView│ │  │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ │  │
│  │     │        │        │      │  │
│  │  ┌──┴────────┴────────┴───┐  │  │
│  │  │   Stores (.svelte.ts)  │  │  │
│  │  └──────────┬─────────────┘  │  │
│  └─────────────┼────────────────┘  │
└────────────────┼───────────────────┘
                 │ invoke() IPC
┌────────────────┼───────────────────┐
│  Tauri Rust    ▼                   │
│  ┌──────────────────────────────┐  │
│  │  Commands (notes, settings,  │  │
│  │  calendar, pdf, ai)          │  │
│  └──────┬───────────────────┬───┘  │
│         ▼                   ▼      │
│  ┌────────────┐    ┌────────────┐  │
│  │ Hybrid     │    │  PDF       │  │
│  │ Storage    │    │  Extractor │  │
│  ├────────────┤    └────────────┘  │
│  │ SQLite     │                    │
│  │ Markdown   │                    │
│  └────────────┘                    │
│  ┌────────────┐                    │
│  │  AI (stub) │ (deferred)        │
│  └────────────┘                    │
└────────────────────────────────────┘
```

---

## Phase status

| Phase | Status | Description |
|---|---|---|
| **Phase 0** | ✅ Complete | Scaffolding, UI, Editor, Calendar, Settings, Themes, Typography |
| **Phase 1** | ⏭️ Deferred | Local AI + RAG + Flashcards (plan in `docs/PHASE_1_AI.md`) |
| **Phase 2** | 🔄 In progress | PDF viewer + annotations |
| **Phase 3** | 📅 Planned | Graph view (D3.js / Canvas) |
| **Phase 4** | 📅 Planned | Sync / SaaS layer |

---

## Frontend (Svelte 5)

### Component tree (Phase 2)

```
App.svelte
├── Sidebar
│   ├── NoteTree.svelte
│   │   └── TreeItem.svelte (recursive)
│   ├── Calendar.svelte
│   └── resize handle (draggable)
├── Main
│   ├── Breadcrumbs.svelte
│   ├── Editor.svelte (split markdown)
│   │   ├── .toolbar
│   │   ├── .event-bar (linked calendar events)
│   │   ├── .title-input
│   │   ├── .content-input (markdown textarea)
│   │   └── .preview (rendered markdown)
│   └── PdfViewer.svelte (right panel, toggleable)
│       ├── PDF.js canvas renderer
│       └── Annotation overlay
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
| `calendarStore` | `src/lib/stores/calendar.svelte.ts` | Calendar events, note linking |
| `pdfStore` | `src/lib/stores/pdf.svelte.ts` | PDF viewer state, annotations |

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

### Module map (Phase 2)

```
src-tauri/src/
├── main.rs              # Entry point
├── lib.rs               # Tauri setup, plugin registration, command registration
├── models.rs            # Data structs + serde + load/persist settings
├── commands/
│   ├── mod.rs
│   ├── notes.rs         # 9 IPC commands for note CRUD
│   ├── settings.rs      # get_settings / save_settings
│   ├── calendar.rs      # 7 IPC commands for events + note linking
│   ├── ai.rs            # stub (deferred)
│   └── pdf.rs           # import_pdf, get_pdf_text, save_annotation, get_annotations
├── storage/
│   ├── mod.rs
│   ├── hybrid.rs        # Sync layer between SQLite + Markdown
│   ├── sqlite.rs        # SQLite CRUD, tree building, breadcrumbs, PDF tables
│   └── markdown.rs      # Read/write .md files
├── ai/                  # Deferred: embeddings, inference, RAG
│   ├── mod.rs
│   ├── models.rs
│   ├── embeddings.rs
│   ├── inference.rs
│   └── rag.rs
└── pdf/                 # PDF parsing, annotations
    ├── mod.rs
    ├── extract.rs       # Text extraction from PDF files
    └── annotations.rs   # Annotation CRUD helpers
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

CREATE TABLE calendar_events (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    date        TEXT NOT NULL,
    description TEXT DEFAULT '',
    completed   INTEGER DEFAULT 0,
    created_at  TEXT NOT NULL
);

CREATE TABLE event_notes (
    event_id TEXT NOT NULL,
    note_id  TEXT NOT NULL,
    PRIMARY KEY (event_id, note_id),
    FOREIGN KEY (event_id) REFERENCES calendar_events(id),
    FOREIGN KEY (note_id) REFERENCES notes(id)
);

CREATE TABLE pdfs (
    id          TEXT PRIMARY KEY,
    title       TEXT,
    file_path   TEXT NOT NULL,
    page_count  INTEGER DEFAULT 0,
    text        TEXT,
    created_at  TEXT NOT NULL
);

CREATE TABLE pdf_annotations (
    id          TEXT PRIMARY KEY,
    pdf_id      TEXT NOT NULL,
    page        INTEGER NOT NULL,
    type        TEXT NOT NULL,       -- 'highlight' | 'note' | 'underline'
    x           REAL,
    y           REAL,
    width       REAL,
    height      REAL,
    color       TEXT,
    content     TEXT,
    created_at  TEXT NOT NULL,
    FOREIGN KEY (pdf_id) REFERENCES pdfs(id)
);

CREATE TABLE note_pdfs (
    note_id TEXT NOT NULL,
    pdf_id  TEXT NOT NULL,
    PRIMARY KEY (note_id, pdf_id),
    FOREIGN KEY (note_id) REFERENCES notes(id),
    FOREIGN KEY (pdf_id) REFERENCES pdfs(id)
);
```

### Commands

All commands registered in `lib.rs`:

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
| `create_calendar_event` | Create calendar event |
| `get_calendar_events` | List events for a date range |
| `update_calendar_event` | Update event title/description/completed |
| `delete_calendar_event` | Delete event |
| `link_note_to_event` | Associate a note with an event |
| `unlink_note_from_event` | Remove note-event association |
| `get_events_for_note` | Get all events linked to a note |
| `import_pdf` | Copy PDF to app data, extract text, return metadata |
| `get_pdf_text` | Return extracted text for a PDF |
| `save_annotation` | Persist or update an annotation |
| `get_annotations` | Load all annotations for a PDF |
| `get_pdfs_for_note` | Get PDFs linked to a note |
| `link_pdf_to_note` | Associate PDF with a note |
| `unlink_pdf_from_note` | Remove PDF-note association |

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

### Opening a PDF

```
User clicks "Open PDF" in toolbar
  → File dialog (tauri-plugin-dialog)
    → invoke('import_pdf', { filePath })
      → Copy PDF to {app_data_dir}/pdfs/{uuid}.pdf
      → Extract text via pdf-extract
      → Insert into pdfs table
      → Return PdfMetadata
    → PdfViewer.svelte renders PDF.js
      → Load PDF from copied path via tauri-plugin-fs
      → Render canvases per page
```

### Saving an annotation

```
User highlights text or adds note
  → Annotation overlay captures position + type + content
    → invoke('save_annotation', { pdfId, page, type, x, y, w, h, color, content })
      → Upsert pdf_annotations table
      → Return annotation ID
    → Overlay renders the annotation shape
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
| PDF.js over custom renderer | Mature, handles all PDF features, Svelte overlay for annotations |
| pdf-extract for Rust extraction | Pure Rust, no system deps, extracts text + metadata |
| Annotation coords in relative % | Scales correctly with zoom / window resize |

---

## Future architecture (Phase 1 + 3)

### AI pipeline (deferred, see `docs/PHASE_1_AI.md`)

```
User query
  → Embedding → Vector search (cosine similarity)
    → Context retrieval → LLM prompt
      → Streamed response → ChatPanel / FlashCard generation
```

### Graph view (Phase 3)

```
D3.js / Canvas rendering
  → Nodes = notes
    → Edges = parent-child + backlinks + AI-suggested connections
      → Force-directed layout
```
