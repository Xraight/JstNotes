<script lang="ts">
  /**
   * StudyHome — full-screen study dashboard with CSS grid layout.
   *
   * Replaces the old StudyPanel (340px sidebar with tabs). All study features
   * are visible at once in a responsive grid: Stats, Quick Actions, Search,
   * Quiz, Feynman/Examples, and Cards. A single vertical scroll flows through
   * the entire dashboard.
   *
   * Mutually exclusive with the Editor/PdfViewer. Opening Study Home hides
   * the editor area. Opening a PDF auto-closes Study Home.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { aiStore } from '../stores/ai';
  import { noteStore } from '../stores/notes';
  import FlashCard from './FlashCard.svelte';

  // --- Quiz ---
  let quizIdx = $state(0);
  let revealed = $state(false);

  let quizList = $derived.by(() => {
    const source = aiStore.dueReviews;
    if (source.length < 2) return source;
    const groups = new Map<string, typeof source>();
    for (const item of source) {
      const g = groups.get(item.note_id) || [];
      g.push(item);
      groups.set(item.note_id, g);
    }
    const keys = Array.from(groups.keys());
    const result: typeof source = [];
    let i = 0;
    while (result.length < source.length) {
      for (const key of keys) {
        const g = groups.get(key)!;
        if (i < g.length) result.push(g[i]);
      }
      i++;
    }
    return result;
  });

  let quizItem = $derived(quizList[quizIdx] ?? null);

  async function rateQuiz(q: number) {
    if (quizItem) await aiStore.rateReview(quizItem.id, q);
    revealed = false;
    if (quizIdx >= quizList.length - 1) quizIdx = 0; else quizIdx++;
  }

  // --- Feynman ---
  let showFeynman = $state(false);
  let feynmanPrompt = $state('');
  let feynmanAnswer = $state('');
  let feynmanFeedback = $state('');
  let feynmanGen = $state(false);

  async function startFeynman() {
    if (!noteStore.selectedNote) return;
    showFeynman = true;
    feynmanPrompt = 'Loading…';
    feynmanAnswer = '';
    feynmanFeedback = '';
    try {
      feynmanPrompt = await invoke('generate_feynman_prompt', { noteId: noteStore.selectedNote!.id });
    } catch {
      feynmanPrompt = 'Explain a key concept from this note in your own words.';
    }
  }

  async function submitFeynman() {
    if (!noteStore.selectedNote || !feynmanAnswer.trim()) return;
    feynmanFeedback = '';
    feynmanGen = true;
    try {
      const unlisten = await listen<string>('ai-chunk', (ev) => {
        feynmanFeedback += ev.payload;
      });
      await listen('ai-done', () => {
        unlisten();
        feynmanGen = false;
      });
      await invoke('evaluate_feynman_stream', {
        noteId: noteStore.selectedNote!.id,
        userExplanation: feynmanAnswer,
      });
    } catch (e: any) {
      feynmanFeedback = e?.toString() || 'Failed';
      feynmanGen = false;
    }
  }

  // --- Examples ---
  let exampleText = $state('');
  let exampleGen = $state(false);

  async function genExample() {
    if (!noteStore.selectedNote) return;
    exampleGen = true;
    exampleText = '';
    try {
      exampleText = await aiStore.generateExample(noteStore.selectedNote.id);
    } catch { exampleText = 'Failed.'; }
    finally { exampleGen = false; }
  }

  // --- Search ---
  let searchInput = $state('');
  let searchTimeout: ReturnType<typeof setTimeout> | undefined;

  function onSearch() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => aiStore.searchNotes(searchInput), 250);
  }

  function openNote(id: string) { noteStore.selectNote(id); }

  // --- Init ---
  $effect(() => {
    if (aiStore.studyPanelOpen) {
      aiStore.loadDueReviews();
      aiStore.loadStats();
    }
  });
</script>

<div class="sh-outer" class:open={aiStore.studyPanelOpen}>
  <div class="sh-toolbar">
    <span class="sh-title">Study</span>
    <button class="sh-close" onclick={() => aiStore.togglePanel()}>✕</button>
  </div>

  <div class="sh-body">
    {#if !aiStore.apiConfigured}
      <div class="sh-status">AI not configured — Add Groq API key in Settings → AI</div>
    {:else}
      <!-- Row 1: Stats + Actions + Search -->
      <div class="sh-row-top">
        <div class="sh-widget sh-stats">
          <div class="sh-section-title">Stats</div>
          {#if aiStore.stats}
            <div class="sh-stats-grid">
              <div class="sh-stat"><span class="sh-stat-num">{aiStore.stats.reviews_today}</span><span class="sh-stat-label">Today</span></div>
              <div class="sh-stat"><span class="sh-stat-num">{aiStore.stats.streak_days}🔥</span><span class="sh-stat-label">Streak</span></div>
              <div class="sh-stat"><span class="sh-stat-num">{aiStore.stats.total_reviews}</span><span class="sh-stat-label">Total</span></div>
              <div class="sh-stat"><span class="sh-stat-num">{aiStore.stats.due_count}</span><span class="sh-stat-label">Due</span></div>
            </div>
          {:else}
            <p class="sh-muted">Complete a review</p>
          {/if}
        </div>

        <div class="sh-widget sh-actions">
          <div class="sh-section-title">Quick Actions</div>
          <p class="sh-note-name">{noteStore.selectedNote?.title || 'No note selected'}</p>
          <div class="sh-action-btns">
            <button class="sh-btn sh-btn-primary" onclick={() => noteStore.selectedNote && aiStore.generateQuestions(noteStore.selectedNote.id)} disabled={aiStore.isGenerating || !noteStore.selectedNote}>
              {aiStore.isGenerating ? '…' : 'Generate'}
            </button>
            <button class="sh-btn" onclick={() => noteStore.selectedNote && aiStore.generateElaboration(noteStore.selectedNote.id)} disabled={aiStore.isGenerating || !noteStore.selectedNote}>Deep</button>
            <button class="sh-btn" onclick={startFeynman} disabled={!noteStore.selectedNote}>Feynman</button>
            <button class="sh-btn" onclick={genExample} disabled={!noteStore.selectedNote}>Example</button>
          </div>
        </div>

        <div class="sh-widget sh-search">
          <div class="sh-section-title">Search</div>
          <input type="text" class="sh-search-input" bind:value={searchInput} oninput={onSearch} placeholder="Search notes…" />
          <div class="sh-search-results">
            {#each aiStore.searchResults as r}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="sh-search-item" onclick={() => openNote(r.id)} onkeydown={() => {}}>
                <div class="sh-search-title">{r.title}</div>
                <div class="sh-search-snippet">{@html r.snippet}</div>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Row 2: Quiz + Feynman/Examples -->
      <div class="sh-row-mid">
        <div class="sh-widget sh-quiz">
          <div class="sh-section-title">Quiz {quizList.length > 0 ? `${quizIdx + 1}/${quizList.length}` : ''}</div>
          {#if quizList.length > 0}
            <div class="sh-quiz-card">
              <div class="sh-quiz-meta">
                <span>from "{noteStore.notes.find(n => n.id === quizItem?.note_id)?.title || 'Note'}"</span>
                {#if quizItem?.days_until_event != null && quizItem!.days_until_event! <= 7}
                  <span class="sh-cal-badge">📅 {quizItem!.days_until_event}d</span>
                {/if}
              </div>
              <div class="sh-quiz-q">{quizItem?.question}</div>
              {#if revealed}
                <div class="sh-quiz-div"></div>
                <div class="sh-quiz-a">{quizItem?.answer}</div>
              {/if}
            </div>
            <div class="sh-quiz-btns">
              {#if !revealed}
                <button class="sh-btn sh-btn-primary" onclick={() => revealed = true}>Reveal</button>
              {:else}
                <span class="sh-rate-label">Recall?</span>
                {#each [1,2,3,4,5] as q}
                  <button class="sh-rate-btn" onclick={() => rateQuiz(q)}>{q}</button>
                {/each}
              {/if}
            </div>
          {:else}
            <p class="sh-muted">No reviews due</p>
          {/if}
        </div>

        <div class="sh-widget sh-feynman">
          {#if showFeynman}
            <div class="sh-section-title">Feynman</div>
            <div class="sh-fey-prompt">{feynmanPrompt}</div>
            <textarea class="sh-fey-input" bind:value={feynmanAnswer} placeholder="Explain in your own words…" rows={2} disabled={feynmanGen}></textarea>
            <div class="sh-fey-btns">
              <button class="sh-btn" onclick={() => showFeynman = false}>Cancel</button>
              <button class="sh-btn sh-btn-primary" onclick={submitFeynman} disabled={feynmanGen || !feynmanAnswer.trim()}>{feynmanGen ? '…' : 'Submit'}</button>
            </div>
            {#if feynmanFeedback}
              <div class="sh-fey-fb">{feynmanFeedback}</div>
            {/if}
          {:else if exampleText}
            <div class="sh-section-title">Example</div>
            <div class="sh-fey-fb">{exampleText}</div>
          {:else}
            <div class="sh-section-title">Feynman / Examples</div>
            <p class="sh-muted">Start a Feynman exercise or generate a concrete example.</p>
          {/if}
        </div>
      </div>

      <!-- Row 3: Cards -->
      <div class="sh-widget sh-cards">
        <FlashCard />
      </div>
    {/if}
  </div>
</div>

<style>
  .sh-outer { display: none; flex-direction: column; background: var(--bg-primary);
    font-family: var(--font-sans); flex: 1; overflow: hidden; }
  .sh-outer.open { display: flex; }
  .sh-toolbar { display: flex; align-items: center; justify-content: space-between;
    padding: 8px 16px; border-bottom: 1px solid var(--border);
    background: var(--bg-secondary); flex-shrink: 0; }
  .sh-title { font-size: 14px; font-weight: 700; color: var(--text-primary); }
  .sh-close { background: none; border: none; cursor: pointer; font-size: 16px;
    padding: 2px 8px; border-radius: 4px; color: var(--text-secondary); font-family: inherit; }
  .sh-close:hover { background: var(--accent); color: var(--text-primary); }
  .sh-body { flex: 1; overflow-y: auto; padding: 16px;
    display: flex; flex-direction: column; gap: 14px; }
  .sh-status { font-size: 13px; padding: 12px; border-radius: 6px;
    background: var(--bg-secondary); color: var(--text-secondary); text-align: center; }
  .sh-row-top { display: grid; grid-template-columns: 1fr 1.2fr 1.4fr; gap: 14px; }
  .sh-row-mid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .sh-widget { background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 8px; padding: 14px; }
  .sh-section-title { font-size: 11px; font-weight: 600; color: var(--text-secondary);
    text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px; }
  .sh-muted { font-size: 12px; color: var(--text-secondary); margin: 0; }

  /* Stats */
  .sh-stats-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .sh-stat { text-align: center; }
  .sh-stat-num { display: block; font-size: 20px; font-weight: 700; color: var(--text-primary); }
  .sh-stat-label { font-size: 10px; color: var(--text-secondary); }

  /* Actions */
  .sh-note-name { font-size: 12px; color: var(--text-primary); margin: 0 0 8px;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 600; }
  .sh-action-btns { display: flex; gap: 6px; flex-wrap: wrap; }

  /* Buttons */
  .sh-btn { background: var(--bg-primary); border: 1px solid var(--border);
    border-radius: 4px; padding: 5px 12px; font-size: 12px; cursor: pointer;
    color: var(--text-primary); font-family: inherit; }
  .sh-btn:hover { border-color: var(--highlight); }
  .sh-btn:disabled { opacity: 0.4; cursor: default; }
  .sh-btn-primary { background: var(--highlight); color: #fff; border-color: var(--highlight); }

  /* Search */
  .sh-search-input { width: 100%; background: var(--bg-primary); border: 1px solid var(--border);
    border-radius: 4px; padding: 6px 8px; font-size: 12px; color: var(--text-primary);
    font-family: inherit; outline: none; box-sizing: border-box; }
  .sh-search-input:focus { border-color: var(--highlight); }
  .sh-search-results { max-height: 160px; overflow-y: auto; margin-top: 6px; }
  .sh-search-item { padding: 6px; border-radius: 4px; cursor: pointer; margin-bottom: 2px; }
  .sh-search-item:hover { background: var(--bg-primary); }
  .sh-search-title { font-size: 12px; font-weight: 600; color: var(--text-primary); }
  .sh-search-snippet { font-size: 11px; color: var(--text-secondary); line-height: 1.3; margin-top: 1px; }
  .sh-search-snippet :global(mark) { background: #ffd65a; padding: 0 2px; border-radius: 2px; }

  /* Quiz */
  .sh-quiz-card { padding: 4px 0; }
  .sh-quiz-meta { display: flex; justify-content: space-between; font-size: 11px; color: var(--text-secondary); font-style: italic; margin-bottom: 8px; }
  .sh-quiz-q { font-size: 14px; font-weight: 600; color: var(--text-primary); line-height: 1.5; }
  .sh-quiz-div { margin: 10px 0; border-top: 1px solid var(--border); }
  .sh-quiz-a { font-size: 13px; color: var(--text-primary); line-height: 1.5; }
  .sh-quiz-btns { margin-top: 10px; display: flex; align-items: center; gap: 8px; }
  .sh-rate-label { font-size: 11px; color: var(--text-secondary); }
  .sh-rate-btn { width: 28px; height: 28px; border: 1px solid var(--border);
    border-radius: 50%; background: var(--bg-primary); color: var(--text-primary);
    font-size: 13px; font-weight: 600; cursor: pointer; font-family: inherit; }
  .sh-rate-btn:hover { background: var(--highlight); color: #fff; border-color: var(--highlight); }
  .sh-cal-badge { font-style: normal; background: var(--highlight); color: #fff;
    font-size: 10px; padding: 1px 5px; border-radius: 8px; }

  /* Feynman / Examples */
  .sh-fey-prompt { font-size: 12px; color: var(--text-primary); line-height: 1.5; margin-bottom: 8px;
    background: var(--bg-primary); padding: 8px; border-radius: 4px; }
  .sh-fey-input { width: 100%; background: var(--bg-primary); border: 1px solid var(--border);
    border-radius: 4px; padding: 6px 8px; font-size: 12px; color: var(--text-primary);
    font-family: inherit; resize: vertical; box-sizing: border-box; }
  .sh-fey-input:focus { outline: none; border-color: var(--highlight); }
  .sh-fey-btns { display: flex; gap: 6px; justify-content: flex-end; margin-top: 6px; }
  .sh-fey-fb { font-size: 12px; color: var(--text-primary); line-height: 1.5; margin-top: 8px;
    background: var(--bg-primary); padding: 8px; border-radius: 4px; border-left: 3px solid var(--highlight); }
</style>