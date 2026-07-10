<script lang="ts">
  import { aiStore } from '../stores/ai';
  import { noteStore } from '../stores/notes';

  let revealed = $state(false);

  let note = $derived(noteStore.selectedNote);
  let items = $derived(aiStore.studyItems);
  let currentIdx = $state(0);
  let info = $state('');

  function next() {
    if (currentIdx < items.length - 1) { currentIdx++; revealed = false; }
  }

  function prev() {
    if (currentIdx > 0) { currentIdx--; revealed = false; }
  }

  function toggleReveal() { revealed = !revealed; }

async function generate() {
    if (!note) return;
    revealed = false;
    info = 'Generating…';
    try {
      const result = await aiStore.generateQuestions(note.id);
      if (result.length > 0) { currentIdx = 0; info = ''; }
      else { info = aiStore.lastError || 'No questions generated'; }
    } catch (e: any) { info = e?.message || 'Error'; }
  }

  async function generateDeep() {
    if (!note) return;
    revealed = false;
    info = 'Generating deep questions…';
    try {
      const result = await aiStore.generateElaboration(note.id);
      if (result.length > 0) { currentIdx = 0; info = ''; }
      else { info = aiStore.lastError || 'No questions generated'; }
    } catch (e: any) { info = e?.message || 'Error'; }
  }

  async function handleRate(quality: number) {
    const item = items[currentIdx];
    if (!item) return;
    await aiStore.rateReview(item.id, quality);
    revealed = false;
    if (currentIdx >= items.length - 1) currentIdx = 0;
    else currentIdx++;
  }

  $effect(() => {
    if (note) {
      aiStore.loadStudyItems(note.id);
      currentIdx = 0;
      revealed = false;
    }
  });
</script>

<div class="flashcards">
  <div class="fc-toolbar">
    <span class="fc-title">Flashcards</span>
    <button class="fc-btn" onclick={generate} disabled={aiStore.isGenerating}>
      {aiStore.isGenerating ? '…' : 'Generate'}
    </button>
    <button class="fc-btn" onclick={generateDeep} disabled={aiStore.isGenerating}>
      {aiStore.isGenerating ? '…' : 'Deep'}
    </button>
  </div>

  <div class="fc-scroll">  {#if items.length > 0}
    <div class="fc-card" onclick={toggleReveal}>
      <div class="fc-counter">{currentIdx + 1} / {items.length}</div>
      <div class="fc-question">{items[currentIdx].question}</div>
      {#if revealed}
        <div class="fc-divider"></div>
        <div class="fc-answer">{items[currentIdx].answer}</div>
      {/if}
    </div>
    {#if revealed}
      <div class="rate-row">
        <span class="rate-label">How well did you recall?</span>
        <div class="rate-buttons">
          {#each [1, 2, 3, 4, 5] as q}
            <button class="rate-btn" onclick={() => handleRate(q)}
              title={q === 1 ? 'Forgot' : q === 5 ? 'Perfect' : ''}>
              {q}
            </button>
          {/each}
        </div>
      </div>
    {/if}
    <div class="fc-nav">
      <button class="fc-btn" onclick={prev} disabled={currentIdx === 0}>← Prev</button>
      <button class="fc-btn" onclick={toggleReveal}>{revealed ? 'Hide' : 'Reveal'}</button>
      <button class="fc-btn" onclick={next} disabled={currentIdx >= items.length - 1}>Next →</button>
    </div>
  {:else}
    <div class="fc-empty">
      {#if aiStore.isGenerating}
        Generating…
      {:else if info}
        <span class="fc-error">{info}</span>
      {:else if aiStore.lastError}
        <span class="fc-error">{aiStore.lastError}</span>
      {:else if !aiStore.apiConfigured}
        Add Groq API key in Settings → AI
      {:else if note}
        Click Generate
      {:else}
        Select a note
      {/if}
    </div>
  {/if}
  </div>
</div>

<style>
  .flashcards { display: flex; flex-direction: column; font-family: var(--font-sans); }
  .fc-scroll { max-height: 50vh; overflow-y: auto; padding: 12px 0 12px 12px; }
  .fc-toolbar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .fc-title { font-size: 13px; font-weight: 600; color: var(--text-primary); }
  .fc-btn {
    background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 4px; padding: 3px 10px; font-size: 12px;
    cursor: pointer; color: var(--text-primary); font-family: inherit;
  }
  .fc-btn:hover { border-color: var(--highlight); }
  .fc-btn:disabled { opacity: 0.4; cursor: default; }
  .fc-card {
    background: var(--bg-secondary); border: 1px solid var(--border);
    border-radius: 8px; padding: 16px; cursor: pointer; min-height: 80px;
  }
  .fc-card:hover { border-color: var(--highlight); }
  .fc-counter { font-size: 11px; color: var(--text-secondary); margin-bottom: 8px; }
  .fc-question { font-size: 14px; font-weight: 600; color: var(--text-primary); line-height: 1.5; }
  .fc-divider { margin: 12px 0; border-top: 1px solid var(--border); }
  .fc-answer { font-size: 13px; color: var(--text-primary); line-height: 1.5; }
  .fc-nav { display: flex; justify-content: space-between; margin-top: 10px; }
  .fc-empty { text-align: center; color: var(--text-secondary); font-size: 13px; padding: 20px; }
  .rate-row { display: flex; flex-direction: column; align-items: center; gap: 6px; margin-top: 8px; }
  .rate-label { font-size: 12px; color: var(--text-secondary); }
  .rate-buttons { display: flex; gap: 6px; }
  .rate-btn {
    width: 30px; height: 30px; border: 1px solid var(--border);
    border-radius: 50%; background: var(--bg-secondary);
    color: var(--text-primary); font-size: 13px; font-weight: 600;
    cursor: pointer; font-family: inherit;
  }
  .rate-btn:hover { background: var(--highlight); color: #fff; border-color: var(--highlight); }
  .fc-error { color: var(--highlight); font-size: 12px; }
  .fc-link { color: var(--highlight); cursor: pointer; text-decoration: underline; }
</style>
