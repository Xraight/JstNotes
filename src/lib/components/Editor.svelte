<script lang="ts">
  import { noteStore } from '../stores/notes';
  import { Marked } from 'marked';

  let note = $derived(noteStore.selectedNote);
  let title = $state('');
  let content = $state('');
  let viewMode = $state<'edit' | 'preview' | 'split'>('split');
  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  const marked = new Marked({ breaks: true, gfm: true });

  let renderedHTML = $derived(marked.parse(content));

  $effect(() => {
    if (note) {
      title = note.title;
      content = note.content;
    } else {
      title = '';
      content = '';
    }
  });

  function scheduleSave() {
    if (!note) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      noteStore.updateNote({ id: note!.id, title, content });
    }, 500);
  }

  function handleTitleInput(e: Event) {
    title = (e.target as HTMLInputElement).value;
    scheduleSave();
  }

  function handleContentInput(e: Event) {
    content = (e.target as HTMLTextAreaElement).value;
    scheduleSave();
  }

  function cycleView() {
    if (viewMode === 'edit') viewMode = 'preview';
    else if (viewMode === 'preview') viewMode = 'split';
    else viewMode = 'edit';
  }
</script>

<div class="editor">
  {#if note}
    <div class="toolbar">
      <button class="view-toggle" onclick={cycleView} title="Toggle view (edit / preview / split)">
        {viewMode === 'edit' ? '✏️' : viewMode === 'preview' ? '👁️' : '⇔'}
        {viewMode === 'edit' ? ' Edit' : viewMode === 'preview' ? ' Preview' : ' Split'}
      </button>
    </div>
    <input
      class="title-input"
      type="text"
      bind:value={title}
      oninput={handleTitleInput}
      placeholder="Note title..."
    />
    <div class="editor-body">
      {#if viewMode === 'edit' || viewMode === 'split'}
        <textarea
          class="content-input"
          class:with-preview={viewMode === 'split'}
          bind:value={content}
          oninput={handleContentInput}
          placeholder="Start writing in Markdown..."
        ></textarea>
      {/if}
      {#if viewMode === 'preview' || viewMode === 'split'}
        <div class="preview" class:with-editor={viewMode === 'split'}>
          <div class="preview-content">{@html renderedHTML}</div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <h2>Welcome to JSTNotes</h2>
      <p>Select a note from the sidebar or create a new one.</p>
    </div>
  {/if}
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }
  .toolbar {
    display: flex;
    align-items: center;
    padding: 6px 20px;
    border-bottom: 1px solid var(--border);
    gap: 8px;
  }
  .view-toggle {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: pointer;
    color: var(--text-secondary);
  }
  .view-toggle:hover {
    color: var(--text-primary);
    border-color: var(--highlight);
  }
  .title-input {
    border: none;
    border-bottom: 1px solid var(--border);
    padding: 12px 24px;
    font-size: 22px;
    font-weight: 700;
    background: transparent;
    color: var(--text-primary);
    outline: none;
    width: 100%;
  }
  .title-input::placeholder {
    color: var(--text-secondary);
  }
  .editor-body {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }
  .content-input {
    flex: 1;
    border: none;
    padding: 16px 24px;
    font-size: var(--editor-font-size, 15px);
    line-height: var(--editor-line-height, 1.7);
    background: transparent;
    color: var(--text-primary);
    resize: none;
    outline: none;
    font-family: var(--font-mono);
    tab-size: 2;
  }
  .content-input::placeholder {
    color: var(--text-secondary);
  }
  .content-input.with-preview {
    border-right: 1px solid var(--border);
    width: 50%;
  }
  .preview {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
    line-height: var(--editor-line-height, 1.7);
  }
  .preview.with-editor {
    width: 50%;
    border-left: 1px solid var(--border);
  }
  .preview-content {
    max-width: 720px;
  }
  .preview-content :global(h1),
  .preview-content :global(h2),
  .preview-content :global(h3),
  .preview-content :global(h4) {
    margin: 0.6em 0 0.3em;
    color: var(--text-primary);
  }
  .preview-content :global(p) {
    margin: 0.5em 0;
    color: var(--text-primary);
  }
  .preview-content :global(code) {
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 0.9em;
  }
  .preview-content :global(pre) {
    background: var(--bg-secondary);
    padding: 12px 16px;
    border-radius: 8px;
    overflow-x: auto;
  }
  .preview-content :global(pre code) {
    background: none;
    padding: 0;
  }
  .preview-content :global(blockquote) {
    border-left: 3px solid var(--highlight);
    margin: 0.5em 0;
    padding: 4px 16px;
    color: var(--text-secondary);
  }
  .preview-content :global(ul),
  .preview-content :global(ol) {
    padding-left: 24px;
    margin: 0.4em 0;
  }
  .preview-content :global(li) {
    margin: 0.2em 0;
  }
  .preview-content :global(a) {
    color: var(--highlight);
  }
  .preview-content :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 1.5em 0;
  }
  .preview-content :global(img) {
    max-width: 100%;
    border-radius: 8px;
  }
  .preview-content :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
  }
  .preview-content :global(th),
  .preview-content :global(td) {
    border: 1px solid var(--border);
    padding: 6px 12px;
    text-align: left;
  }
  .preview-content :global(th) {
    background: var(--bg-secondary);
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }
  .empty-state h2 {
    font-size: 24px;
    margin-bottom: 8px;
    color: var(--text-primary);
  }
</style>
