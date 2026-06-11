# Phase 2: PDF Integration (Complete)

> Status: Complete. All planned features implemented + extensions.

## Architecture

```
PdfViewer.svelte (pdfjs-dist)
  ├── Canvas per page (PDF rendering)
  ├── Overlay canvas per page (annotations, search highlights, drag preview)
  ├── IntersectionObserver (lazy page loading, 600px margin)
  ├── Zoom 30-400% with anchor-based scroll preservation
  ├── Custom horizontal scrollbar (DOM-based, WebKitGTK compatible)
  ├── Search bar (text extraction + highlight navigation)
  └── Annotation tools (highlight drag, note click → dialog)

PdfLibrary.svelte (sidebar)
  └── List PDFs, filter by selected note, import/delete

PdfViewer integrated via App.svelte
  ├── Resizable split (drag divider, 30-70%)
  └── StudyPanel coexists as additional right panel
```

## Backend

### SQLite Tables

```sql
-- PDF metadata
CREATE TABLE pdfs (
    id TEXT PRIMARY KEY, title TEXT, file_path TEXT,
    page_count INTEGER, text TEXT, created_at TEXT
);

-- Visual annotations (positioned relative to page)
CREATE TABLE pdf_annotations (
    id TEXT PRIMARY KEY, pdf_id TEXT, page INTEGER,
    type TEXT, x REAL, y REAL, width REAL, height REAL,
    color TEXT, content TEXT, created_at TEXT
);

-- Note ↔ PDF general linking
CREATE TABLE note_pdfs (
    note_id TEXT, pdf_id TEXT, PRIMARY KEY (note_id, pdf_id)
);

-- Note → PDF page references (for bidirectional navigation)
CREATE TABLE pdf_references (
    id TEXT PRIMARY KEY, note_id TEXT, pdf_id TEXT,
    page INTEGER, page_end INTEGER, label TEXT,
    annotation_id TEXT, created_at TEXT
);
```

### Commands (13)

| Command | Description |
|---|---|
| `list_pdfs` | All imported PDFs |
| `import_pdf(file_path)` | Copy to app data, extract text, save metadata |
| `get_pdf_text(pdf_id)` | Extracted text content |
| `save_annotation(input)` | Create highlight or note annotation |
| `get_annotations(pdf_id)` | All annotations for a PDF |
| `delete_annotation(id)` | Remove annotation |
| `update_annotation_content(id, content)` | Update annotation text |
| `link_pdf_to_note(note_id, pdf_id)` | Link note ↔ PDF |
| `unlink_pdf_from_note(note_id, pdf_id)` | Remove link |
| `delete_pdf(pdf_id)` | Delete PDF + cascade annotations/links |
| `get_linked_pdf_ids(note_id)` | PDF IDs linked to a note |
| `create_pdf_reference(input)` | Note → PDF page reference |
| `get_pdf_references_for_note(note_id)` | References for a note |
| `delete_pdf_reference(id)` | Remove reference |

---

## Frontend Features

### PdfViewer.svelte
- **pdfjs-dist v6** rendering with Web Worker (`GlobalWorkerOptions.workerSrc`)
- **Canvas-per-page** with DPR-aware rendering
- **Overlay canvas** stacked on top for annotations/search/drag preview
- **Zoom**: 30-400%, Ctrl+scroll, +/- buttons, Fit button
- **Stable zoom**: anchor-based scroll preservation with generation counter for concurrent calls
- **Horizontal pan**: ◀ ▶ buttons appear at zoom > 100%
- **Custom scrollbar**: DOM-based bar at bottom (WebKitGTK doesn't render CSS scrollbars)

### Annotation System
- **Highlight tool**: drag rectangle with live dashed-line preview
- **Note tool**: click → dialog (title + content) → creates real Note in tree + annotation + PdfReference
- **Right-click** on any annotation → confirm delete
- **Escape** cancels active drag
- **Text capture**: highlights extract text from `page.getTextContent()` via coordinate intersection
- **Note-annotation linking**: `content` stores `note:<id>` for resolution

### Text Search
- Search bar with input field, match counter, prev/next arrows
- `page.getTextContent()` per page → substring match → position mapping
- Results highlighted on overlay: active match in orange with border, others in yellow
- Navigation scrolls to page + renders overlay with highlights

### PdfLibrary.svelte
- Sidebar section with collapse toggle
- List imported PDFs, click to open, ✕ to delete
- Filter by current note: `● note` toggle when note selected
- Highlight active PDF
- Import button (+)

### Editor Integration
- PDF reference badges below note title (📄 p.42) — clickable, opens PDF at page
- Double-click annotation marker → navigates to linked note in editor
- Link/Unlink button in PDF toolbar

---

## Dependencies

| Layer | Dependency |
|---|---|
| Frontend | `pdfjs-dist` ^6.0 |
| Rust | `pdf-extract` 0.10, `lopdf` 0.34 |

---

## Key Decisions

| Decision | Rationale |
|---|---|
| Canvas + overlay per page | Separates PDF rendering from annotations. Overlay can be cleared/redrawn independently. |
| Relative coordinates (0-1) | Zoom-independent. Convert to viewport pixels at render time. |
| Text extraction for highlights | `page.getTextContent()` maps coordinates → captures real text, not just bounding boxes. |
| Anchor-based zoom | Save first visible page + offset → re-render → restore position → avoids scroll jumping. |
| Custom scrollbar (DOM) | WebKitGTK in Tauri Linux doesn't render CSS `::-webkit-scrollbar`. DOM bar is always visible. |
| Union of note + annotation + reference | A note created from PDF is: Note (in tree) + Annotation (on canvas) + PdfReference (for navigation). |
