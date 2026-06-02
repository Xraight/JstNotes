# Phase 2: PDF Integration

> Current phase. Status: in progress.

## Goal

Integrate PDF viewing and annotation into the note-taking workflow.
Users can open PDFs alongside their notes, highlight text, add margin notes,
and link PDFs to specific notes for cross-referencing.

## Architecture

```
Frontend (Svelte)
├── PdfViewer.svelte          ← PDF.js canvas renderer + overlay
├── PdfToolbar.svelte         ← Annotation tools (highlight, note, zoom)
├── PdfStore (pdf.svelte.ts)  ← Viewer state, open PDFs, annotations

Backend (Rust)
├── commands/pdf.rs           ← IPC commands for PDF CRUD + annotations
├── pdf/extract.rs            ← Text extraction via pdf-extract
└── storage/sqlite.rs         ← Tables: pdfs, pdf_annotations, note_pdfs
```

## Dependencies

### Frontend

```json
{
  "pdfjs-dist": "^4.0"
}
```

### Rust (Cargo.toml)

```toml
pdf-extract = "0.7"
lopdf = "0.32"        # for metadata (page count, title, etc.)
```

## SQLite schema

```sql
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
    FOREIGN KEY (pdf_id) REFERENCES pdfs(id) ON DELETE CASCADE
);

CREATE TABLE note_pdfs (
    note_id TEXT NOT NULL,
    pdf_id  TEXT NOT NULL,
    PRIMARY KEY (note_id, pdf_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (pdf_id) REFERENCES pdfs(id) ON DELETE CASCADE
);
```

## Rust commands

| Command | Input | Output |
|---|---|---|
| `import_pdf` | `file_path: String` | `PdfMetadata` |
| `get_pdf_text` | `pdf_id: String` | `String` |
| `save_annotation` | `AnnotationInput` | `PdfAnnotation` |
| `get_annotations` | `pdf_id: String` | `Vec<PdfAnnotation>` |
| `delete_annotation` | `id: String` | `()` |
| `get_pdfs_for_note` | `note_id: String` | `Vec<PdfMetadata>` |
| `link_pdf_to_note` | `note_id, pdf_id: String` | `()` |
| `unlink_pdf_from_note` | `note_id, pdf_id: String` | `()` |

## Frontend components

### PdfViewer.svelte

- Right-side panel, toggleable via toolbar button
- PDF.js renders each page to a `<canvas>`
- Virtual scrolling: only render visible pages
- Zoom controls (fit-width, 100%, custom)
- Annotation overlay: transparent `<div>` positioned over each canvas
- On page render: load annotations for that page, draw on overlay

### Annotation system

- **Highlight mode**: select text → captured as highlight annotation
- **Note mode**: click position → opens text input → saves as note annotation
- **Underline mode**: select text → underline annotation
- All annotations stored with relative coordinates (% of page dimensions)
- Overlay re-draws on zoom change using stored relative coords

### PDF-note linking

- Button in Editor toolbar: "Open PDF" → file dialog → imports → opens viewer
- In PdfViewer: "Link to current note" button
- In Editor: linked PDFs listed in event-bar area
- Click a linked PDF → opens it in viewer

## Layout

### Split modes

The app has three layout modes:

1. **Notes only** — current layout, editor full width
2. **PDF only** — PdfViewer fills the main area  
3. **Split** — Editor left / PdfViewer right (resizable divider)

Mode toggled via toolbar button or when opening a PDF.

### Implementation

```svelte
<!-- App.svelte: main area -->
<div class="main">
  <Breadcrumbs />
  <div class="editor-area" class:with-pdf={pdfOpen}>
    <Editor />
    {#if pdfOpen}
      <div class="pdf-divider" onmousedown={startPdfResize} />
      <PdfViewer />
    {/if}
  </div>
</div>
```

## Implementation order

1. ✅ Add Rust deps (pdf-extract, lopdf)
2. ✅ Add models (PdfMetadata, PdfAnnotation, PdfNoteLink)
3. ✅ Add SQLite tables
4. ✅ Implement extract.rs (text + metadata extraction)
5. ✅ Implement commands/pdf.rs
6. ✅ Register commands in lib.rs
7. ✅ Install pdfjs-dist
8. ✅ Create PdfStore
9. ✅ Create PdfViewer.svelte (basic render)
10. 🔄 Editor integration (Open PDF button, split layout)
11. 🔄 Annotation overlay (tools + persistence)
12. 🔄 PDF-note linking

## PDF.js in Tauri notes

- PDF.js loads from `Uint8Array` via `tauri-plugin-fs` `readFile()`
- Worker: configure `pdfjs.GlobalWorkerOptions.workerSrc` to point to bundled worker
- File scope in tauri.conf.json: allow reading `$APPDATA/pdfs/**`
- Copy PDFs to app data dir on import for isolation

## Key decisions

| Decision | Rationale |
|---|---|
| pdf-extract for text | Best pure-Rust PDF text extraction quality |
| Relative % coordinates for annotations | Survives zoom changes and window resize |
| Copy PDFs to app data | Isolation, survives file moves/deletes |
| Canvas per page + div overlay | Simpler than full canvas rendering of annotations |
| Right-side split | Consistent with future ChatPanel placement |
