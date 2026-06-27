<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { aiStore } from '../stores/ai';
  import { noteStore } from '../stores/notes';
  import FlashCard from './FlashCard.svelte';

  let tab = $state<'quiz' | 'cards' | 'feynman' | 'examples'>('quiz');
  let currentIdx = $state(0);
  let revealed = $state(false);
  let showFeynman = $state(false);
  let feynmanPrompt = $state('');
  let feynmanAnswer = $state('');
  let feynmanFeedback = $state('');
  let feynmanGenerating = $state(false);
  let exampleResult = $state('');
  let exampleGenerating = $state(false);

  async function generateExample() {
    if (!noteStore.selectedNote) return;
    exampleGenerating = true;
    exampleResult = '';
    try {
      exampleResult = await aiStore.generateExample(noteStore.selectedNote.id);
    } catch {
      exampleResult = 'Failed to generate example.';
    } finally {
      exampleGenerating = false;
    }
  }

  async function startFeynman() {
    if (!noteStore.selectedNote) return;
    showFeynman = true;
    feynmanPrompt = 'Loading…';
    feynmanAnswer = '';
    feynmanFeedback = '';
    try {
      feynmanPrompt = await invoke('generate_feynman_prompt', { noteId: noteStore.selectedNote!.id });
      if (!feynmanPrompt) feynmanPrompt = 'Explain a key concept from this note in your own words.';
    } catch {
      feynmanPrompt = 'Explain a key concept from this note in your own words.';
    }
  }

  async function submitFeynman() {
    if (!noteStore.selectedNote || !feynmanAnswer.trim()) return;
    feynmanGenerating = true;
    try {
      feynmanFeedback = await invoke('evaluate_feynman', { noteId: noteStore.selectedNote!.id, userExplanation: feynmanAnswer });
    } catch {
      feynmanFeedback = 'Failed to get AI feedback.';
    } finally {
      feynmanGenerating = false;
    }
  }

  // Round-robin interleaving: groups items by note, takes one from each in rotation
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

  let quizItem = $derived(quizList[currentIdx] ?? null);

  $effect(() => {
    if (aiStore.studyPanelOpen) {
      aiStore.loadDueReviews();
    }
  });
</script>

<div class="study-panel" class:open={aiStore.studyPanelOpen}>
  <div class="study-toolbar">
    <span class="study-title">Study</span>
    <button class="study-tool-btn" onclick={() => aiStore.togglePanel()} title="Close">✕</button>
  </div>

  <div class="study-tabs">
    <button class="study-tab" class:active={tab === 'quiz'} onclick={() => tab = 'quiz'}>Quiz</button>
    <button class="study-tab" class:active={tab === 'cards'} onclick={() => tab = 'cards'}>Cards</button>
    <button class="study-tab" class:active={tab === 'feynman'} onclick={() => tab = 'feynman'}>Feynman</button>
    <button class="study-tab" class:active={tab === 'examples'} onclick={() => tab = 'examples'}>Examples</button>
  </div>

  <div class="study-body">
    {#if !aiStore.apiConfigured}
      <div class="study-status">
        <span>AI not configured</span>
        <span class="study-hint">Add Groq API key in Settings → AI</span>
      </div>
    {:else if tab === 'quiz'}
      {#if quizList.length > 0}
        <div class="study-section">
          <div class="study-section-header">Due for Review — {quizList.length} items</div>
          <div class="quiz-card">
            <div class="quiz-progress">
              <span class="quiz-counter">{currentIdx + 1} / {quizList.length}</span>
              <span class="quiz-note">from "{noteStore.notes.find(n => n.id === quizItem?.note_id)?.title || 'Note'}"
                {#if quizItem?.days_until_event != null && quizItem!.days_until_event! <= 7}
                  <span class="cal-badge" title="Event in {quizItem!.days_until_event} days">📅 {quizItem!.days_until_event}d</span>
                {/if}
              </span>
            </div>
            <div class="quiz-question">{quizItem?.question}</div>
            {#if revealed}
              <div class="quiz-divider"></div>
              <div class="quiz-answer">{quizItem?.answer}</div>
            {/if}
          </div>
          <div class="quiz-actions">
            {#if !revealed}
              <button class="study-btn study-primary" onclick={() => revealed = true}>Reveal Answer</button>
            {:else}
              <span class="rate-label">How well did you recall?</span>
              <div class="rate-buttons">
                {#each [1, 2, 3, 4, 5] as q}
                  <button class="rate-btn"
                    title={q === 1 ? 'Forgot' : q === 5 ? 'Perfect' : ''}
                    onclick={async () => {
                      if (quizItem) await aiStore.rateReview(quizItem.id, q);
                      revealed = false;
                      if (currentIdx >= quizList.length - 1) currentIdx = 0;
                      else currentIdx++;
                    }}>{q}</button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="study-empty">
          <p>No reviews due. Click ⚡ Generate in the editor toolbar to create study questions.</p>
        </div>
      {/if}

    {:else if tab === 'cards'}
      <FlashCard />

    {:else if tab === 'feynman'}
      <div class="study-section">
        <div class="study-section-header">Feynman Technique</div>
        {#if !showFeynman}
          <button class="study-btn" onclick={startFeynman} disabled={!noteStore.selectedNote || !aiStore.apiConfigured}>
            Start Feynman Exercise
          </button>
        {:else}
          <div class="feynman-prompt">{feynmanPrompt}</div>
          <textarea class="feynman-input" bind:value={feynmanAnswer} placeholder="Explain in your own words..." rows={4}
            disabled={feynmanGenerating}></textarea>
          <div class="feynman-actions">
            <button class="study-btn" onclick={() => showFeynman = false}>Cancel</button>
            <button class="study-btn study-primary" onclick={submitFeynman} disabled={!feynmanAnswer.trim() || feynmanGenerating}>
              {feynmanGenerating ? '…' : 'Submit'}
            </button>
          </div>
          {#if feynmanFeedback}
            <div class="feynman-feedback">{feynmanFeedback}</div>
          {/if}
        {/if}
      </div>
    {:else if tab === 'examples'}
      <div class="study-section">
        <div class="study-section-header">Concrete Examples</div>
        <p style="font-size:12px;color:var(--text-secondary);margin:0">
          Generates a relatable analogy or real-world example for a concept in your note.
        </p>
        <button class="study-btn" onclick={generateExample}
          disabled={!noteStore.selectedNote || exampleGenerating || !aiStore.apiConfigured}>
          {exampleGenerating ? '…' : 'Generate Example'}
        </button>
        {#if exampleResult}
          <div class="feynman-feedback">{exampleResult}</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .study-panel { display: none; flex-direction: column; width: 340px;
    border-left: 1px solid var(--border); background: var(--bg-primary);
    font-family: var(--font-sans); overflow: hidden; flex-shrink: 0; }
  .study-panel.open { display: flex; }
  .study-toolbar { display: flex; align-items: center; justify-content: space-between;
    padding: 8px 12px; border-bottom: 1px solid var(--border);
    background: var(--bg-secondary); flex-shrink: 0; }
  .study-title { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .study-tool-btn { background: none; border: none; cursor: pointer; font-size: 16px;
    padding: 2px 8px; border-radius: 4px; color: var(--text-secondary); font-family: inherit; }
  .study-tool-btn:hover { background: var(--accent); color: var(--text-primary); }
  .study-tabs { display: flex; border-bottom: 1px solid var(--border);
    background: var(--bg-secondary); flex-shrink: 0; }
  .study-tab { flex:1; background: none; border: none; padding: 8px 0;
    font-size: 12px; font-weight: 600; color: var(--text-secondary);
    cursor: pointer; font-family: inherit; border-bottom: 2px solid transparent; }
  .study-tab:hover { color: var(--text-primary); }
  .study-tab.active { color: var(--highlight); border-bottom-color: var(--highlight); }
  .study-body { flex: 1; overflow-y: auto; padding: 12px;
    display: flex; flex-direction: column; gap: 12px; }
  .study-status { font-size: 12px; padding: 6px 10px; border-radius: 6px;
    background: var(--bg-secondary); color: var(--text-secondary);
    display: flex; align-items: center; gap: 8px; }
  .study-empty { text-align: center; color: var(--text-secondary); font-size: 13px; padding: 20px; }
  .study-btn { background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 4px; padding: 4px 12px; font-size: 12px;
    cursor: pointer; color: var(--text-primary); font-family: inherit; }
  .study-btn:hover { border-color: var(--highlight); }
  .study-btn:disabled { opacity: 0.4; cursor: default; }
  .study-primary { background: var(--highlight); color: #fff; border-color: var(--highlight); }
  .study-section { display: flex; flex-direction: column; gap: 8px; }
  .study-section-header { font-size: 11px; font-weight: 600; color: var(--text-secondary);
    text-transform: uppercase; letter-spacing: 0.5px; }
  .quiz-card { background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 8px; padding: 16px; }
  .quiz-progress { display: flex; justify-content: space-between; margin-bottom: 10px; }
  .quiz-counter { font-size: 11px; color: var(--text-secondary); }
  .quiz-note { font-size: 11px; color: var(--text-secondary); font-style: italic;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 60%; }
  .cal-badge { font-style: normal; background: var(--highlight); color: #fff;
    font-size: 10px; padding: 1px 5px; border-radius: 8px; margin-left: 6px; }
  .quiz-question { font-size: 14px; font-weight: 600; color: var(--text-primary); line-height: 1.5; }
  .quiz-divider { margin: 12px 0; border-top: 1px solid var(--border); }
  .quiz-answer { font-size: 13px; color: var(--text-primary); line-height: 1.5; }
  .quiz-actions { display: flex; flex-direction: column; align-items: center; gap: 8px; margin-top: 4px; }
  .rate-label { font-size: 12px; color: var(--text-secondary); }
  .rate-buttons { display: flex; gap: 6px; }
  .rate-btn { width: 34px; height: 34px; border: 1px solid var(--border);
    border-radius: 50%; background: var(--bg-secondary); color: var(--text-primary);
    font-size: 14px; font-weight: 600; cursor: pointer; font-family: inherit; }
  .rate-btn:hover { background: var(--highlight); color: #fff; border-color: var(--highlight); }
  .feynman-prompt { font-size: 13px; color: var(--text-primary);
    background: var(--bg-secondary); padding: 10px; border-radius: 6px; line-height: 1.5; }
  .feynman-input { width: 100%; background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 6px; padding: 8px; font-size: 13px; font-family: var(--font-sans);
    color: var(--text-primary); resize: vertical; box-sizing: border-box; }
  .feynman-input:focus { outline: none; border-color: var(--highlight); }
  .feynman-actions { display: flex; gap: 6px; justify-content: flex-end; }
  .feynman-feedback { font-size: 13px; color: var(--text-primary);
    background: var(--bg-secondary); padding: 10px; border-radius: 6px;
    line-height: 1.5; border-left: 3px solid var(--highlight); }
</style>