# JSTNotes

A note-taking desktop app with local AI, PDF annotations, adaptive learning, and full UI customization. Built with Tauri v2 + Svelte 5 + TypeScript.

> **Status:** Phase 0 — Core scaffolding complete. Active development.

---

## Features

- **Markdown editor** with live split preview
- **Tree sidebar** with nested notes
- **Persistent settings** — custom colors, typography, layout
- **Theme presets** (Midnight, Dracula, Nord, Catppuccin, etc.)
- **Custom CSS** overlay
- Hybrid storage: Markdown files + SQLite metadata
- (Coming) Local AI with RAG, PDF viewer, flashcards, graph view

---

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- Conda environment `JstNotes` (for Python AI models later)

### Quick start

```bash
git clone <repo-url> && cd JstNotes

# Install Node dependencies
npm install

# Run in dev mode (Tauri window + hot reload)
npm run tauri dev

# Or just the web frontend
npm run dev
```

The Rust backend compiles on first run — subsequent launches are faster.

### Conda environment

```bash
source /home/xraight/anaconda3/bin/activate JstNotes
```

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
│   ├── App.svelte          # Root layout
│   └── main.ts             # Entry point
├── src-tauri/              # Backend (Rust)
│   └── src/
│       ├── commands/       # Tauri IPC commands
│       ├── storage/        # Hybrid storage (SQLite + Markdown)
│       ├── ai/             # AI module (stub)
│       ├── pdf/            # PDF module (stub)
│       ├── models.rs       # Data models + serialization
│       └── lib.rs          # Tauri entry + plugin registration
├── docs/                   # Architecture docs (`docs/ARCHITECTURE.md`)
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
| AI (future) | Local LLM via `llama.cpp` or similar |

---

## Architecture

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for full details on the project structure, data flow, backend modules, and design decisions.

## Contributing

1. Fork the repo
2. Create a branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run `npm run build` to verify no errors
5. Commit and push — PRs welcome

Keep it simple: one feature per PR, match existing code style.

---

## License

MIT
