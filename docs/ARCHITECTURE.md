# Architecture — JSTNotes

## Overview

JSTNotes is a hybrid desktop app. The Tauri webview (Svelte 5) communicates with the Rust backend via `invoke()` IPC. Storage is hybrid: SQLite for metadata, Markdown files for note content. AI uses cloud APIs (Groq/OpenAI) via `reqwest`.

```
┌──────────────────────────────────────────────────────┐
│  Svelte 5 Frontend (Tauri WebView)                    │
│  ┌────────┐  ┌──────────────┐  ┌───────────────────┐ │
│  │Sidebar │  │   Editor     │  │PDF Viewer/Study   │ │
│  └────────┘  └──────────────┘  └───────────────────┘ │
├──────────────────────────────────────────────────────┤
│  Tauri IPC (invoke)                                   │
├──────────────────────────────────────────────────────┤
│  Rust Backend                                         │
│  ┌──────────┐ ┌───────────┐ ┌──────┐ ┌────────────┐ │
│  │commands/ │ │ storage/  │ │ ai/  │ │ pdf/       │ │
│  │notes     │ │ sqlite    │ │client│ │ extract    │ │
│  │pdf       │ │ markdown  │ │study │ │            │ │
│  │ai        │ │ hybrid    │ │models│ │            │ │
│  │calendar  │ │           │ │      │ │            │ │
│  │settings  │ │           │ │      │ │            │ │
│  └──────────┘ └───────────┘ └──────┘ └────────────┘ │
├──────────────────────────────────────────────────────┤
│  File System                  │  Cloud APIs           │
│  Markdown files + SQLite DB   │  Groq / OpenAI / etc  │
└──────────────────────────────────────────────────────┘
```

---

## Phase Status

| Phase | Status | Description |
|---|---|---|
| **Phase 0** | ✅ Complete | Scaffolding, Editor, Calendar, Settings, Themes |
| **Phase 1** | ✅ Operational | AI study tools (flashcards, Feynman, SM-2) via cloud APIs |
| **Phase 2** | ✅ Complete | PDF viewer, annotations, note-PDF linking, text search |
| **Phase 3** | 📅 Planned | Graph view (D3.js / Canvas) |
| **Phase 4** | 📅 Planned | Sync / SaaS layer |

---

## Frontend (Svelte 5)

### Component Tree

```
App.svelte (Editor ↔ StudyHome toggle)
├── Sidebar
│   ├── NoteTree.svelte → TreeItem.svelte (recursive)
│   ├── Calendar.svelte
│   └── PdfLibrary.svelte
├── Main
│   ├── Breadcrumbs.svelte
│   ├── Editor.svelte
│   │   └── FlashCard.svelte
│   ├── PdfViewer.svelte
│   └── StudyHome.svelte (full-screen grid dashboard)
│       └── FlashCard.svelte (reused as widget)
├── StatusBar
└── Settings.svelte (modal, Tips tab)
```

### Stores (Svelte 5 Runes)

| Store | File | Purpose |
|---|---|---|
| `noteStore` | `stores/notes.svelte.ts` | Note CRUD, tree, selection, breadcrumbs |
| `settingsStore` | `stores/settings.svelte.ts` | Theme, colors, typography, layout, AI config, CSS |
| `calendarStore` | `stores/calendar.svelte.ts` | Calendar events, note linking |
| `pdfStore` | `stores/pdf.svelte.ts` | PDF import/list, annotations, references, linking |
| `aiStore` | `stores/ai.svelte.ts` | Study items, flashcards, model management |

### Key Design Decisions

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`) for all reactivity
- **`.svelte.ts` stores** — class-based stores with `$state` fields
- **PDF rendering** — pdfjs-dist with canvas-per-page + overlay canvas for annotations
- **IntersectionObserver** — virtual scrolling for PDF pages
- **Custom horizontal scrollbar** — DOM-based (WebKitGTK doesn't render `::-webkit-scrollbar`)

---

## Backend (Rust)

### Module Map

```
src-tauri/src/
├── commands/
│   ├── notes.rs      — create, get, update, delete, list, tree, breadcrumbs
│   ├── pdf.rs        — import, annotations, references, linking, delete
│   ├── ai.rs         — study questions, reviews, Feynman, model fetch
│   ├── calendar.rs   — events, note linking
│   └── settings.rs   — get/save settings
├── storage/
│   ├── sqlite.rs     — Database impl (20+ tables, 40+ methods)
│   ├── markdown.rs   — File-based note content
│   └── hybrid.rs     — HybridStorage (wraps DB + Markdown)
├── ai/
│   ├── client.rs     — HTTP client (OpenAI-compatible API)
│   ├── study.rs      — SM-2 algorithm, StudyItem type
│   └── models.rs     — FlashcardInput
├── pdf/
│   ├── extract.rs    — PDF text extraction
│   └── mod.rs
├── models.rs          — All data types + serialization
└── lib.rs             — Tauri entry, state management, command registration
```

### SQLite Schema (13 tables)

| Table | Purpose |
|---|---|
| `notes` | Note metadata (title, path, parent_id, timestamps) |
| `calendar_events` | Events (date, title, description, completed) |
| `event_notes` | Many-to-many: events ↔ notes |
| `pdfs` | Imported PDFs (title, file_path, page_count, text) |
| `pdf_annotations` | Highlights, notes on PDF pages |
| `note_pdfs` | Many-to-many: notes ↔ PDFs |
| `pdf_references` | Note → PDF page references (for navigation) |
| `study_items` | Spaced repetition items (question, answer, SM-2 params) |
| `study_log` | Review history (item_id, quality, timestamp) |
| `flashcards` | AI-generated Q&A |
| `note_embeddings` | Embedding vectors for future RAG |
| `notes_fts` | FTS5 full-text search index |

### IPC Commands (42 total)

**Notes (8):** `create_note`, `get_note`, `update_note`, `delete_note`, `list_notes`, `get_children`, `get_breadcrumbs`, `build_tree`

**PDF (14):** `list_pdfs`, `import_pdf`, `get_pdf_text`, `save_annotation`, `get_annotations`, `delete_annotation`, `update_annotation_content`, `get_pdfs_for_note`, `link_pdf_to_note`, `unlink_pdf_from_note`, `delete_pdf`, `get_linked_pdf_ids`, `create_pdf_reference`, `get_pdf_references_for_note`, `delete_pdf_reference`

**AI (14):** `generate_study_questions`, `generate_elaboration_questions`, `generate_concrete_example`, `get_due_reviews`, `rate_review`, `get_study_stats`, `search_notes`, `rebuild_fts`, `generate_feynman_prompt`, `evaluate_feynman`, `get_study_items`, `fetch_ai_models`, `test_ai_connection`

**Settings (2):** `get_settings`, `save_settings`

**Calendar (7):** `create_calendar_event`, `get_calendar_events`, `update_calendar_event`, `delete_calendar_event`, `link_note_to_event`, `unlink_note_from_event`, `get_events_for_note`

### Managed State

| State | Type | Purpose |
|---|---|---|
| `HybridStorage` | Single instance | Wraps Database + MarkdownStorage |
| `AppSettingsState` | `{app_dir, settings: Mutex<AppSettings>}` | Settings persistence |

---

## Data Flow

### Opening a note
1. User clicks note in tree → `noteStore.selectNote(id)`
2. `invoke('get_note', {id})` → Rust reads SQLite + Markdown file
3. Returns full `Note` → Editor renders title + content
4. Breadcrumbs loaded via `get_breadcrumbs`

### Auto-save
1. Editor `oninput` → clears 500ms timeout → sets new timeout
2. On timeout → `invoke('update_note', {req})`
3. Rust updates SQLite metadata + writes Markdown file

### Generating study questions (AI)
1. User clicks Generate → `invoke('generate_study_questions', {noteId})`
2. Rust reads note + PDF refs + calendar events
3. IF API key configured: sends prompt to Groq/OpenAI → parses JSON → saves to `study_items`
4. IF no API key: extracts from headings, bold text, bullet points (rule-based)
5. Returns `StudyItem[]` → frontend shows cards

### Feynman Technique
1. User clicks Start Feynman → Rust sends note content to API
2. API returns challenge question → user types explanation → submit
3. Rust sends note + explanation to API → returns conversational feedback
4. Language automatically matches user's content

### Opening a PDF
1. User clicks PDF in library → `pdfStore.openPdf(meta)`
2. `invoke('get_annotations')` + `readFile` (tauri-plugin-fs)
3. PDF rendered via pdfjs-dist with `IntersectionObserver` for lazy loading
4. Annotation overlay canvas drawn per page
5. Search highlights overlaid on search

### Rate a review (SM-2)
1. User rates 1-5 → `invoke('rate_review', {itemId, quality})`
2. SM-2 algorithm computes: new interval, ease factor, repetitions
3. `next_review` set based on quality score
4. Review logged to `study_log` for statistics

### Study Home (Grid Dashboard)
1. Toggles full-screen via `📚 Study` button in Editor toolbar
2. CSS Grid layout: Stats (top-left), Quick Actions (top-center), Search (top-right)
3. Quiz + Feynman/Examples (middle row), Cards (bottom row)
4. Mutually exclusive with Editor/PdfViewer: opening a PDF auto-closes Study Home

### Note Search (FTS5)
1. User types in Search widget → debounced 250ms
2. `search_notes(query)` → FTS5 full-text search on titles + content
3. Returns ranked results with `<mark>` highlighted snippets
4. Click result → opens note in Editor

### Calendar Priority
1. `get_due_reviews` fetches events for each note via `get_events_for_note`
2. Computes `days_until_event` for the nearest incomplete event
3. Items with upcoming events (exams, deadlines) sort first
4. Badge `📅 2d` shown on Quiz items with events ≤7 days away

### Study Statistics
1. `get_study_stats` queries `study_log` for reviews today, streak, total, 7-day activity
2. Streak algorithm: walk backward from today, count consecutive days with ≥1 review
3. Displayed in Stats widget and Stats tab
