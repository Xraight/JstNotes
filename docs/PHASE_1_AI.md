# Phase 1: Local AI + RAG (Deferred)

> This phase is deferred until after PDF integration (Phase 2).
> Revisit once Phase 2 is stable.

## Goal

Build a fully offline AI assistant with RAG (Retrieval-Augmented Generation)
that understands the user's notes, generates flashcards, answers questions,
and adapts to available hardware (0.5B–7B models).

## Architecture

```
User query
  → Embedding (Ollama /api/embeddings)
    → Vector search (cosine similarity over SQLite)
      → Context retrieval (top K notes)
        → Prompt assembly
          → LLM inference (Ollama /api/generate, streaming)
            → Response → ChatPanel / Flashcards
```

## Backend (Rust)

### Dependencies (Cargo.toml)

```toml
reqwest = { version = "0.12", features = ["json"] }
tokio-tungstenite = "0.21"  # for streaming chat
```

### Commands

| Command | Description |
|---|---|
| `check_ollama` | Health check: verify Ollama is running |
| `list_ollama_models` | Fetch available models from `ollama list` |
| `generate_embedding(text)` | Call `/api/embeddings` for a single string |
| `generate_flashcards(note_id)` | Generate Q&A pairs from note content |
| `chat_stream(message, note_id, opts)` | Streamed chat with RAG context |
| `reindex_embeddings` | Rebuild all note embeddings |

### Embeddings storage

```sql
CREATE TABLE note_embeddings (
    note_id     TEXT PRIMARY KEY,
    embedding   BLOB NOT NULL,       -- f32 vector as raw bytes
    updated_at  TEXT NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id)
);
```

Search is done in Rust: load all embeddings, compute cosine similarity,
return top K note IDs.

### Flashcard SQLite schema

```sql
CREATE TABLE flashcards (
    id          TEXT PRIMARY KEY,
    note_id     TEXT NOT NULL,
    question    TEXT NOT NULL,
    answer      TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    reviewed    INTEGER DEFAULT 0,
    difficulty  INTEGER DEFAULT 3,   -- 1-5
    FOREIGN KEY (note_id) REFERENCES notes(id)
);
```

## Frontend (Svelte)

### ChatPanel.svelte

- Right sidebar panel (toggleable)
- Message history (user + assistant)
- Streaming markdown rendering
- "Attach current note" toggle
- RAG context indicator

### Flashcards.svelte

- Quiz mode: question → reveal answer → rate difficulty
- List mode: browse all flashcards for a note
- Generate button in Editor toolbar

### AI Settings tab

- Ollama connection status
- Model selector (populated from `list_ollama_models`)
- Sliders: temperature, top-k RAG, max context length

## Prompt design

### System prompt (chat)

```
You are JST, an AI assistant integrated into the user's note-taking app.
You have access to the following notes from their vault to provide context:

{context_notes}

Answer based on the user's notes when relevant.
If asked about something not in the notes, say so and offer general help.
Keep answers concise.
```

### Flashcard generation prompt

```
Generate flashcards from the following note content.
Each flashcard should be a question-answer pair covering key concepts.
Return as a JSON array: [{"question": "...", "answer": "..."}]

Note content:
{content}
```

## Data flow

### On note save:
1. Debounce 2s
2. Call `generate_embedding(note.content)`
3. Upsert into `note_embeddings`

### On chat message:
1. Get current note content (if attached)
2. Call `generate_embedding(message)`
3. Search top-K similar notes
4. Assemble prompt with context
5. Stream response via Ollama `/api/generate`

## Key decisions

| Decision | Rationale |
|---|---|
| Ollama over embedded llama.cpp | Smaller binary, flexible model choice, no C++ build deps |
| Embeddings via Ollama API | Consistent with inference, no extra dependencies |
| Cosine similarity in Rust | Simple, no need for vector db yet; SQLite + f32 math |
| Streaming via Server-Sent Events | Tauri IPC supports streaming callback patterns |
| JSON prompt for flashcards | Structured output for reliable parsing |
