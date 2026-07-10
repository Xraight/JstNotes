# JSTNotes

A note-taking desktop app with cloud AI study tools, PDF annotations, adaptive learning (SM-2), and full UI customization. Built with Tauri v2 + Svelte 5 + TypeScript + Rust.

> **Status:** Phase 0-3 complete. All 6 evidence-based study techniques implemented.

---

## Features

### Notes & Organization
- **Markdown editor** with live split preview, LaTeX math, @-mentions with breadcrumb preview
- **Tree sidebar** with nested notes, **drag-and-drop** reorganization
- **Calendar** with events linked to notes
- **Breadcrumbs** navigation

### PDF Integration (Phase 2 ✅)
- **PDF viewer** with zoom (30-400%), Ctrl+scroll, horizontal pan buttons
- **PDF library** in sidebar: import, delete, filter by current note
- **Highlights** with configurable **color picker**, live preview rectangle, captured text, right-click delete
- **PDF notes** integrated as real Notes in the tree — double-click opens in editor
- **PDF Outline** (ToC navigation) in context panel
- **Bidirectional navigation**: note badges link to PDF pages, markers link to notes
- **Text search** with highlighted matches and prev/next navigation
- **Resizable** editor/PDF split

### AI Study Tools (Phase 1 ✅)
- **Study Home** — full-screen grid dashboard replacing old sidebar panel
- **Flashcards** — AI-generated active recall questions from notes + PDF context + calendar
- **Deep Questions** — elaborative interrogation: "why/how" questions that connect concepts
- **Spaced Repetition (SM-2)** — Study Dashboard with daily reviews, self-rating 1-5
- **Feynman Technique** — AI challenges you to explain a concept, then evaluates with **streaming** responses
- **Concrete Examples** — AI generates analogies and real-world examples for abstract concepts
- **Interleaving** — Quiz mode always mixes topics for better learning transfer
- **Calendar Priority** — questions linked to upcoming events (exams) appear first
- **Study Statistics** — daily streak, total reviews, 7-day activity chart
- **Note Search** — substring search across all notes with highlighted snippets
- **Multi-provider**: Groq (free), OpenAI, OpenCode Go
- **Auto-detect models**: fetches available models from provider API
- **Offline fallback**: rule-based question extraction from headings, bold text, bullet points
- **Language-aware**: responds in the note's language
- **Study Tips** in Settings — explains all 6 techniques + SM-2 algorithm

### Graph View (Phase 3 ✅)
- **Interactive force-directed graph** of all notes (D3.js)
- **Color-coded links**: parent (gray), @-mention (yellow), PDF reference (red)
- **Concept Map** — per-note mini-graph in the context panel (Dual Coding, technique 6/6)
- Click node → open note. Drag node. Zoom/pan.

### Customization
- **Persistent settings** — colors, typography, layout, custom CSS, AI provider
- **Theme presets** (Midnight, Dracula, Nord, Catppuccin, etc.)
- **Custom CSS** overlay with live preview
- **Resizable sidebar**

### Storage
- Hybrid: Markdown files (content) + SQLite (metadata + study data + logs)

---

## Download & Install

Pre-built installers are available for Windows and Linux.

### Windows

1. Download `JSTNotes_x.y.z_x64-setup.exe` from [Releases](https://github.com/Xraight/JstNotes/releases)
2. Double-click the `.exe` → Next → Next → Install
3. Launch JSTNotes from the Start Menu or desktop shortcut

> No additional dependencies. WebView2 is included in Windows 10 and later.

### Linux

**AppImage** (universal, no install needed):
```bash
chmod +x JSTNotes*.AppImage
./JSTNotes*.AppImage
```

**Debian/Ubuntu** (.deb):
```bash
sudo dpkg -i jstnotes*.deb
```

### Build from source

See [Development Setup](#development-setup) below.

---

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18

### Quick start

```bash
git clone <repo-url> && cd JstNotes
npm install
npm run tauri dev
```

### AI Setup (optional)

1. Go to [console.groq.com](https://console.groq.com) — create free account (no credit card)
2. Copy your API key
3. In JstNotes: Settings → AI → Groq → paste key → Test

---

## Project structure

```
JstNotes/
├── src/                    # Frontend (Svelte 5 + TS)
│   ├── lib/
│   │   ├── components/     # UI components
│   │   ├── stores/         # Svelte 5 runes stores
│   │   ├── types.ts        # Shared TS interfaces
│   │   └── presets.ts      # Theme presets
│   ├── App.svelte          # Root layout (editor / study home / graph toggle)
│   └── main.ts             # Entry point
├── src-tauri/              # Backend (Rust)
│   └── src/
│       ├── commands/       # Tauri IPC commands (notes, pdf, ai, calendar, graph, settings)
│       ├── storage/        # Hybrid storage (SQLite + Markdown)
│       ├── ai/             # AI module (client, study, models)
│       ├── pdf/            # PDF extraction
│       ├── models.rs       # Data models + serialization
│       └── lib.rs          # Tauri entry + command registration
├── docs/                   # Architecture + phase docs
├── package.json
├── vite.config.ts
└── opencode.json           # OpenCode agents config
```

---

## Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri v2](https://v2.tauri.app/) |
| Frontend | [Svelte 5](https://svelte.dev/), TypeScript, Vite |
| Backend | Rust, [rusqlite](https://github.com/rusqlite/rusqlite) |
| Storage | SQLite (metadata) + Markdown (content) |
| Markdown | `pulldown-cmark` (Rust), `marked` (JS preview) |
| AI | Groq / OpenAI / OpenCode Go via `reqwest` (OpenAI-compatible + SSE streaming) |
| PDF | `pdfjs-dist` v6, `pdf-extract` + `lopdf` (Rust) |
| Graph View | D3.js v7 (force simulation) |

---

## Architecture

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for full details.

## Contributing

1. Fork the repo
2. Create a branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run `npm run build` to verify no errors
5. Commit and push — PRs welcome

---

## License

PolyForm Noncommercial License 1.0.0 — see [`LICENSE`](LICENSE) for details.
