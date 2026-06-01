<script lang="ts">
  import { noteStore } from '../stores/notes';
  import { Marked } from 'marked';
  import katex from 'katex';
  import 'katex/dist/katex.min.css';

  let note = $derived(noteStore.selectedNote);
  let title = $state('');
  let content = $state('');
  let viewMode = $state<'edit' | 'preview' | 'split'>('split');
  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let mentionOpen = $state(false);
  let mentionQuery = $state('');
  let mentionIdx = $state(0);
  let noteTitles = $state<string[]>([]);
  let mentionFiltered = $derived(
    noteTitles.filter(t => t.toLowerCase().includes(mentionQuery.toLowerCase()))
  );

  const marked = new Marked({ breaks: true, gfm: true });

  marked.use({
    extensions: [{
      name: 'inlineMath',
      level: 'inline',
      start(src: string) { return src.indexOf('$'); },
      tokenizer(src: string) {
        const match = src.match(/^\$([^$\n]+?)\$/);
        if (match) {
          return { type: 'inlineMath', raw: match[0], text: match[1] };
        }
      },
      renderer(token: { text: string }) {
        try {
          return katex.renderToString(token.text, { throwOnError: false });
        } catch { return token.text; }
      },
    }, {
      name: 'displayMath',
      level: 'block',
      start(src: string) { return src.indexOf('$$'); },
      tokenizer(src: string) {
        const match = src.match(/^\$\$([\s\S]+?)\$\$/);
        if (match) {
          return { type: 'displayMath', raw: match[0], text: match[1] };
        }
      },
      renderer(token: { text: string }) {
        try {
          return katex.renderToString(token.text, { throwOnError: false, displayMode: true });
        } catch { return token.text; }
      },
    }],
  });

  function renderContent(raw: string): string {
    const map = noteStore.getNoteTitleMap();
    let result = raw;
    const sorted = [...map.entries()].sort((a, b) => b[0].length - a[0].length);
    for (const [titleLower, id] of sorted) {
      const regex = new RegExp(`(?<!\\w)@(${escapeRegex(titleLower)})(?!\\w)`, 'gi');
      result = result.replace(regex, (_, matched) => {
        return `<a href="note:${id}" class="mention" data-note-id="${id}">@${matched}</a>`;
      });
    }
    return result;
  }

  function escapeRegex(s: string) {
    return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  let renderedHTML = $derived(marked.parse(renderContent(content)));

  $effect(() => {
    if (note) {
      title = note.title;
      content = note.content;
    } else {
      title = '';
      content = '';
    }
  });

  $effect(() => {
    if (noteStore.notes.length === 0) {
      noteStore.loadNoteTitles();
    }
    noteTitles = noteStore.notes.map(n => n.title);
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

  function handleKeydown(e: KeyboardEvent) {
    if (mentionOpen) {
      if (e.key === 'ArrowDown') { e.preventDefault(); mentionIdx = (mentionIdx + 1) % mentionFiltered.length; }
      if (e.key === 'ArrowUp') { e.preventDefault(); mentionIdx = (mentionIdx - 1 + mentionFiltered.length) % mentionFiltered.length; }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        insertMention();
        return;
      }
      if (e.key === 'Escape') { mentionOpen = false; return; }
    }

    if ((e.ctrlKey || e.metaKey) && e.key === 'b') { e.preventDefault(); applyFormat('bold'); return; }
    if ((e.ctrlKey || e.metaKey) && e.key === 'i') { e.preventDefault(); applyFormat('italic'); return; }

    if (e.key === '@') {
      setTimeout(() => checkMentionTrigger(), 0);
    }
  }

  function handleInput(e: Event) {
    handleContentInput(e);
    checkMentionTrigger();
  }

  function checkMentionTrigger() {
    if (!textareaEl) return;
    const pos = textareaEl.selectionStart;
    const text = content.slice(0, pos);
    const atIdx = text.lastIndexOf('@');
    if (atIdx === -1 || text.slice(atIdx).includes(' ')) {
      mentionOpen = false;
      return;
    }
    const query = text.slice(atIdx + 1);
    mentionQuery = query;
    mentionIdx = 0;
    mentionOpen = query.length > 0;
  }

  function insertMention() {
    if (!textareaEl || mentionFiltered.length === 0) return;
    const selected = mentionFiltered[mentionIdx];
    const pos = textareaEl.selectionStart;
    const textBefore = content.slice(0, pos);
    const atIdx = textBefore.lastIndexOf('@');
    const before = content.slice(0, atIdx);
    const after = content.slice(pos);
    content = `${before}@${selected} ${after}`;
    mentionOpen = false;
    scheduleSave();
    requestAnimationFrame(() => {
      textareaEl!.focus();
      const newPos = before.length + selected.length + 2;
      textareaEl!.setSelectionRange(newPos, newPos);
    });
  }

  type FormatAction = {
    prefix: string;
    suffix?: string;
    block?: string;
    placeholder?: string;
  };

  const FORMAT_ACTIONS: Record<string, FormatAction> = {
    bold: { prefix: '**', suffix: '**', placeholder: 'bold text' },
    italic: { prefix: '*', suffix: '*', placeholder: 'italic text' },
    strikethrough: { prefix: '~~', suffix: '~~', placeholder: 'strikethrough' },
    h1: { prefix: '# ', block: 'line' },
    h2: { prefix: '## ', block: 'line' },
    h3: { prefix: '### ', block: 'line' },
    bullet: { prefix: '- ', block: 'line' },
    numbered: { prefix: '1. ', block: 'line' },
    code: { prefix: '`', suffix: '`', placeholder: 'code' },
    codeblock: { prefix: '```\n', suffix: '\n```', block: 'block', placeholder: 'code block' },
    quote: { prefix: '> ', block: 'line' },
    link: { prefix: '[', suffix: '](url)', placeholder: 'link text' },
    image: { prefix: '![', suffix: '](url)', placeholder: 'alt text' },
    math: { prefix: '$', suffix: '$', placeholder: 'equation' },
    mathblock: { prefix: '$$\n', suffix: '\n$$', block: 'block', placeholder: 'display equation' },
    hr: { prefix: '\n---\n', block: 'line' },
  };

  function applyFormat(type: string) {
    if (!textareaEl) return;
    const fmt = FORMAT_ACTIONS[type];
    if (!fmt) return;
    const start = textareaEl.selectionStart;
    const end = textareaEl.selectionEnd;
    const selected = content.slice(start, end);
    const before = content.slice(0, start);
    const after = content.slice(end);

    if (fmt.block === 'line') {
      const lineStart = content.lastIndexOf('\n', start - 1) + 1;
      const beforeLine = content.slice(0, lineStart);
      const line = content.slice(lineStart, content.indexOf('\n', start) === -1 ? content.length : content.indexOf('\n', start));
      const rest = content.slice(lineStart + line.length);
      content = `${beforeLine}${fmt.prefix}${line.trim()}${fmt.suffix || ''}${rest}`;
    } else if (selected) {
      content = `${before}${fmt.prefix}${selected}${fmt.suffix || ''}${after}`;
    } else {
      const ph = fmt.placeholder || '';
      content = `${before}${fmt.prefix}${ph}${fmt.suffix || ''}${after}`;
    }
    scheduleSave();
    textareaEl.focus();
  }

  function cycleView() {
    if (viewMode === 'edit') viewMode = 'preview';
    else if (viewMode === 'preview') viewMode = 'split';
    else viewMode = 'edit';
  }

  function handleMentionClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    const anchor = target.closest('a.mention');
    if (anchor) {
      const href = anchor.getAttribute('href');
      if (href?.startsWith('note:')) {
        e.preventDefault();
        const id = href.slice(5);
        noteStore.selectNote(id);
      }
    }
  }
</script>

<div class="editor">
  {#if note}
    <div class="toolbar">
      <button class="view-toggle" onclick={cycleView} title="Toggle view (edit / preview / split)">
        {viewMode === 'edit' ? '✏️' : viewMode === 'preview' ? '👁️' : '⇔'}
        {viewMode === 'edit' ? ' Edit' : viewMode === 'preview' ? ' Preview' : ' Split'}
      </button>
      <span class="toolbar-hint">
        LaTeX: <code>$$...$$</code> / <code>$...$</code> &nbsp;|&nbsp; Mention: <code>@Título</code>
      </span>
    </div>
    <input
      class="title-input"
      type="text"
      bind:value={title}
      oninput={handleTitleInput}
      placeholder="Note title..."
    />
    <div class="fmt-toolbar">
      <button class="fmt-btn" onclick={() => applyFormat('bold')} title="Bold (Ctrl+B)"><b>B</b></button>
      <button class="fmt-btn" onclick={() => applyFormat('italic')} title="Italic (Ctrl+I)"><i>I</i></button>
      <button class="fmt-btn" onclick={() => applyFormat('strikethrough')} title="Strikethrough"><s>S</s></button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('h1')} title="Heading 1">H1</button>
      <button class="fmt-btn" onclick={() => applyFormat('h2')} title="Heading 2">H2</button>
      <button class="fmt-btn" onclick={() => applyFormat('h3')} title="Heading 3">H3</button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('bullet')} title="Bullet list">≡</button>
      <button class="fmt-btn" onclick={() => applyFormat('numbered')} title="Numbered list">#</button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('code')} title="Inline code">&lt;/&gt;</button>
      <button class="fmt-btn" onclick={() => applyFormat('codeblock')} title="Code block">▣</button>
      <button class="fmt-btn" onclick={() => applyFormat('quote')} title="Blockquote">❝</button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('link')} title="Link">🔗</button>
      <button class="fmt-btn" onclick={() => applyFormat('image')} title="Image">🖼</button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('math')} title="Inline LaTeX">∑</button>
      <button class="fmt-btn" onclick={() => applyFormat('mathblock')} title="Display LaTeX">∫</button>
      <span class="fmt-sep"></span>
      <button class="fmt-btn" onclick={() => applyFormat('hr')} title="Horizontal rule">—</button>
    </div>
    <div class="editor-body">
      {#if viewMode === 'edit' || viewMode === 'split'}
        <div class="textarea-wrap" class:with-preview={viewMode === 'split'}>
          <textarea
            class="content-input"
            bind:this={textareaEl}
            bind:value={content}
            oninput={handleInput}
            onkeydown={handleKeydown}
            placeholder="Write in Markdown... (use @ para mencionar notas, $...$ para LaTeX)"
          ></textarea>
          {#if mentionOpen && mentionFiltered.length > 0}
            <div class="mention-dropdown">
              {#each mentionFiltered as t, i}
                <button class="mention-item" class:active={i === mentionIdx}
                  onmousedown={(e) => { e.preventDefault(); mentionIdx = i; insertMention(); }}>
                  @{t}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
      {#if viewMode === 'preview' || viewMode === 'split'}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="preview" class:with-editor={viewMode === 'split'} onclick={handleMentionClick}>
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
    gap: 12px;
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
  .toolbar-hint {
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.6;
  }
  .toolbar-hint code {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-secondary);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .fmt-toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-wrap: wrap;
  }
  .fmt-btn {
    background: none;
    border: none;
    border-radius: 4px;
    padding: 3px 8px;
    font-size: 13px;
    cursor: pointer;
    color: var(--text-secondary);
    line-height: 1.4;
  }
  .fmt-btn:hover {
    background: var(--accent);
    color: var(--text-primary);
  }
  .fmt-sep {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 4px;
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
  .textarea-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
  }
  .textarea-wrap.with-preview {
    width: 50%;
    border-right: 1px solid var(--border);
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
  .mention-dropdown {
    position: absolute;
    left: 20px;
    bottom: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
    max-height: 180px;
    overflow-y: auto;
    min-width: 180px;
    z-index: 100;
  }
  .mention-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 14px;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    cursor: pointer;
  }
  .mention-item.active,
  .mention-item:hover {
    background: var(--accent);
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
  .preview-content :global(a),
  .preview-content :global(a.mention) {
    color: var(--highlight);
    cursor: pointer;
  }
  .preview-content :global(a.mention) {
    text-decoration: none;
    border-bottom: 1px dotted var(--highlight);
    padding: 0 2px;
  }
  .preview-content :global(a.mention:hover) {
    background: var(--accent);
    border-radius: 3px;
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
