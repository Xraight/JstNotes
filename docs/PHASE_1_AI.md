# Phase 1: AI Study Tools (Operational)

> Status: Operational with Groq/OpenAI/OpenCode Go. Cloud-first architecture.
> Originally designed for local Ollama → migrated to candle → settled on cloud APIs.

## Goal

An AI-powered study system that applies evidence-based learning techniques:
- Spaced Repetition (SM-2)
- Active Recall (flashcards)
- Feynman Technique
- Rule-based fallback when offline

## Architecture

```
User action
  → Frontend (Svelte) invoke()
    → Rust command (commands/ai.rs)
      → Read note content + PDF refs + calendar events
      → IF API key: ai/client.rs → POST to Groq/OpenAI → parse response
      → IF no key: rule-based extraction (headings, bold, bullets)
    → Save to SQLite (study_items table)
  → Frontend renders cards / dashboard
```

## Backend (Rust)

### Modules

| File | Purpose |
|---|---|
| `ai/client.rs` | HTTP client (OpenAI-compatible). `chat()`, `list_models()`. Auth via Bearer + x-api-key. |
| `ai/study.rs` | SM-2 algorithm (`sm2()`), `StudyItem`, `StudyQuestion` types |
| `ai/models.rs` | `FlashcardInput` type |

### Commands (8)

| Command | Description |
|---|---|
| `generate_study_questions(note_id)` | AI or rule-based → saves `StudyItem[]` |
| `get_due_reviews()` | Items with `next_review <= today` |
| `rate_review(item_id, quality)` | Update SM-2: interval, ease_factor, repetitions |
| `generate_feynman_prompt(note_id)` | AI generates challenge question |
| `evaluate_feynman(note_id, explanation)` | AI evaluates user's explanation |
| `get_study_items(note_id)` | All study items for a note |
| `test_ai_connection()` | Ping API with "Say OK" → verify auth + endpoint |
| `fetch_ai_models()` | GET /models → return available chat models |

### SM-2 Algorithm

```
quality < 3: reset (interval=1, reps=0)
quality >= 3:
  reps += 1
  interval: 1 → 1, 2 → 6, 3+ → interval * ease_factor
  ease_factor: ef + 0.1 - (5-q)*(0.08 + (5-q)*0.02), min 1.3
```

### SQLite: study_items

```sql
CREATE TABLE study_items (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TEXT NOT NULL,
    next_review TEXT NOT NULL,
    interval_days INTEGER DEFAULT 1,
    ease_factor REAL DEFAULT 2.5,
    repetitions INTEGER DEFAULT 0,
    reviewed_at TEXT,
    source_page INTEGER
);
```

### AI Provider Config (stored in AppSettings)

| Field | Default |
|---|---|
| `ai_provider` | `"groq"` |
| `ai_api_key` | `""` |
| `ai_model` | `"llama-3.3-70b-versatile"` |
| `ai_enabled` | `true` |
| `ai_endpoint` | auto-filled per provider |

Supported providers: Groq, OpenAI, OpenCode Go, Custom

---

## Frontend (Svelte)

### Components

| Component | Purpose |
|---|---|
| `FlashCard.svelte` | Embedded in Editor. Q&A cards with next/prev, reveal, self-rate 1-5 |
| `StudyPanel.svelte` | Right panel. Dashboard: due reviews, quiz mode, Feynman exercises |
| `Settings.svelte` (AI tab) | Provider cards, API key, Test Connection, dynamic model dropdown |

### Store: `aiStore`

| State | Purpose |
|---|---|
| `studyItems` | Current note's questions |
| `dueReviews` | All items due today |
| `isGenerating` | Loading state |
| `availableModels` | Populated by `fetchModels()` |
| `apiConfigured` | Derived: key exists + enabled |
| `lastError` | Latest error message |

### Rule-based fallback (offline)

When no API key is configured, `generate_study_questions` extracts questions from:
1. Markdown headings (`# Title` → "What is 'Title'?")
2. Bold text (`**term**` → "Define: term")
3. Bullet points (`- point` → "Explain: point")
4. Sentences → fill-in-the-blank

---

## Providers

| Provider | Price | Models | Auto-detect | Reasoning noise |
|---|---|---|---|---|
| **Groq** | Free | Llama 3.3 70B, Llama 3.1 8B, Qwen3 32B | ✅ | No |
| OpenAI | Pay per use | GPT-5.4 mini, GPT-5.4, GPT-4o | ✅ | No |
| OpenCode Go | $10/mo | DeepSeek V4 Pro/Flash, Kimi, Qwen | ❌ | ⚠️ DeepSeek |
| Custom | Varies | Any | ❌ | Depends |

---

## Future Enhancements

- **RAG** — semantic search over note embeddings (table + cosine similarity ready, needs embedding model)
- **Elaborative Interrogation** — AI asks "why" and "how" questions, not just factual recall
- **Interleaving** — mixed-topic review sessions based on forgetting curves
- **Concrete Examples** — AI generates analogies for abstract concepts
- **Study Statistics** — progress dashboard, streaks, topic mastery tracking
- **Calendar-aware prioritization** — upcoming exam dates influence what to review
