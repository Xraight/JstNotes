# JSTNotes

Aplicación de notas con IA local, anotaciones sobre PDF, y aprendizaje adaptativo.

## Stack

| Capa | Tecnología |
|---|---|
| Desktop | **Tauri v2** (Rust) |
| Frontend | **Svelte 5 + TypeScript + Vite** |
| Editor | CodeMirror 6 |
| Almacenamiento | SQLite (metadatos) + Markdown plano (contenido) |
| IA local | llama.cpp (inferencia) + fastembed (embeddings) + sqlite-vss (vectores) |
| PDF | pdf.js (visor) + lopdf (backend) |

## Requisitos

- **Node.js** >= 20
- **Rust** >= 1.77
- **Tauri CLI** >= 2.0
- **Conda** environment `JstNotes` (opcional pero recomendado)

## Setup

```bash
git clone <repo> && cd JstNotes

# Instalar dependencias Node
npm install

# Inicializar conda env (si no existe)
conda create -n JstNotes nodejs rust -y
conda activate JstNotes

# Compilar Rust (primer build descarga dependencias)
npm run tauri build -- --debug
# O en modo desarrollo:
npm run tauri dev
```

## Arquitectura

```
JstNotes/
├── src/                          # Frontend Svelte
│   ├── App.svelte                # Layout principal
│   ├── app.css                   # Estilos globales + tema
│   └── lib/
│       ├── components/           # Componentes UI
│       │   ├── Breadcrumbs.svelte    # Navegación jerárquica
│       │   ├── NoteTree.svelte       # Árbol de notas (sidebar)
│       │   ├── TreeItem.svelte       # Nodo recursivo del árbol
│       │   ├── Editor.svelte         # Editor Markdown
│       │   ├── FlashCard.svelte      # (Fase 2) Repaso spaced repetition
│       │   ├── ChatPanel.svelte      # (Fase 1) Chat IA local
│       │   ├── GraphView.svelte      # (Fase 6) Grafo de conocimiento
│       │   ├── PDFViewer.svelte      # (Fase 3) Visor PDF
│       │   └── Settings.svelte       # (Fase 6) Configuración
│       ├── stores/               # Estado global (runes)
│       │   ├── notes.ts              # Estado de notas
│       │   ├── settings.ts           # Tema, preferencias
│       │   └── ai.ts                 # Estado del chat IA
│       └── types.ts              # Tipos compartidos
├── src-tauri/                    # Backend Rust
│   └── src/
│       ├── main.rs / lib.rs          # Entry point Tauri
│       ├── models.rs                 # Modelos de datos
│       ├── commands/                 # Comandos IPC
│       │   ├── notes.rs              # CRUD de notas
│       │   ├── ai.rs                 # (Fase 1) IA
│       │   └── pdf.rs                # (Fase 3) PDF
│       ├── storage/                  # Capa de persistencia
│       │   ├── sqlite.rs             # SQLite (metadatos)
│       │   ├── markdown.rs           # Archivos .md (contenido)
│       │   └── hybrid.rs             # Sincronización SQLite ↔ MD
│       ├── ai/                       # (Fase 1) Motor IA
│       └── pdf/                      # (Fase 3) Extracción PDF
├── models/                       # Modelos IA descargables (gitignored)
└── static/                       # Assets estáticos
```

## Hoja de ruta

| Fase | Duración | Entregable |
|---|---|---|
| **0 — Fundación** | Semana 1-4 | App base: editor MD, árbol de notas, breadcrumbs, híbrido SQLite+MD |
| **1 — IA ligera** | Semana 5-8 | RAG + chat local + selección adaptativa de modelo |
| **2 — Flashcards** | Semana 9-12 | SM-2, generación IA, active recall, export Anki |
| **3 — PDF v1** | Semana 13-14 | Biblioteca PDF, enlace nota→página |
| **4 — PDF v2** | Semana 15-18 | Visor PDF integrado (pdf.js), panel dividido |
| **5 — PDF v3** | Semana 19-22 | Anotaciones, subrayado, margin notes |
| **6 — Polishing** | Semana 23-26 | Graph view, temas, layout customizable, plugins |

## Comandos

```bash
npm run dev            # Servidor de desarrollo Vite
npm run build          # Build frontend
npm run check          # Type-check (svelte-check)
npm run lint           # Linter
npm run tauri dev      # Desarrollo Tauri (app + frontend)
npm run tauri build    # Build producción
```

## Modelo de datos

Toda nota tiene:
- `id` (UUID v4)
- `title`
- `content` (Markdown)
- `parent_id` → jerarquía (nullable para raíces)
- `path` (ruta materializada, ej: `matematicas/calculo/derivadas`)
- `sort_order`
- `created_at` / `updated_at`

Los breadcrumbs se generan recorriendo `parent_id` hacia arriba hasta la raíz.

## Filosofía de diseño

1. **Dificultad adaptable**: El usuario descubre funciones progresivamente, sin saturación inicial.
2. **IA local y offline**: Todo el procesamiento de IA corre en el equipo del usuario. Sin dependencia de cloud.
3. **Simbiosis con PDF**: Las notas se enlazan a libros/páginas/párrafos específicos.
4. **Estructura clara**: No existen notas huérfanas. Todo tiene un rastro (breadcrumbs).
5. **Personalización completa**: Tema visual, layout, CSS custom.
