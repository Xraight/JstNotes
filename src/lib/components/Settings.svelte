<script lang="ts">
  import { settingsStore } from '../stores/settings';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { aiStore } from '../stores/ai';
  import { THEME_PRESETS, FONT_PRESETS, type ThemePreset } from '../presets';
  import type { ColorSettings, TypographySettings, LayoutSettings } from '../types';

  let { onclose }: { onclose: () => void } = $props();

  let s = $derived(settingsStore.settings);
  let activeTab = $state<'general' | 'presets' | 'colors' | 'typography' | 'layout' | 'css' | 'ai' | 'tips'>('general');

  let tempColors = $state<ColorSettings>({ bg_primary: '', bg_secondary: '', accent: '', highlight: '', text_primary: '', text_secondary: '', border: '' });
  let tempTypo = $state<TypographySettings>({ font_family: '', font_family_mono: '', font_size: 15, line_height: 1.7 });
  let tempLayout = $state<LayoutSettings>({ sidebar_width: 280 });
  let customCSS = $state('');
  let aiKey = $state('');
  let aiProvider = $state('groq');
  let aiModel = $state('llama-3.3-70b-versatile');
  let aiEnabled = $state(true);
  let aiEndpoint = $state('');
  let testResult = $state('');
  let testing = $state(false);

  const PROVIDERS = [
    { id: 'groq', name: 'Groq', price: 'Free', url: 'console.groq.com', desc: 'Llama 3.3 70B. No credit card.', defaultModel: 'llama-3.3-70b-versatile', endpoint: 'https://api.groq.com/openai/v1', badge: '⭐ Best for study' },
    { id: 'opencode', name: 'OpenCode Go', price: '$10/mo', url: 'opencode.ai/go', desc: '12 open source models.', defaultModel: 'deepseek-v4-flash', endpoint: 'https://opencode.ai/zen/go/v1', badge: '' },
    { id: 'openai', name: 'OpenAI', price: 'Pay per use', url: 'platform.openai.com', desc: 'GPT-4o mini, GPT-4o.', defaultModel: 'gpt-4o-mini', endpoint: 'https://api.openai.com/v1', badge: '' },
  ];

  let previewPreset = $state<ThemePreset | null>(null);
  let previewFont = $state<string | null>(null);
  let activeFontId = $derived(FONT_PRESETS.find(fp =>
    s.typography.font_family === fp.font_family &&
    s.typography.font_family_mono === fp.font_family_mono
  )?.id ?? null);
  let loadedFonts = new Set<string>();

  function loadGoogleFont(family: string): void {
    if (loadedFonts.has(family)) return;
    loadedFonts.add(family);
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = `https://fonts.googleapis.com/css2?family=${family}&display=swap`;
    document.head.appendChild(link);
  }

  $effect(() => {
    tempColors = deepCopy(s.colors);
    tempTypo = deepCopy(s.typography);
    tempLayout = deepCopy(s.layout);
    aiKey = s.ai_api_key || '';
    aiProvider = s.ai_provider || 'groq';
    aiModel = s.ai_model || 'llama-3.3-70b-versatile';
    aiEnabled = s.ai_enabled ?? true;
    aiEndpoint = s.ai_endpoint || '';
  });

  function deepCopy<T>(obj: T): T {
    return JSON.parse(JSON.stringify(obj));
  }

  function applyPreset(preset: ThemePreset) {
    settingsStore.settings = deepCopy(preset.settings);
    settingsStore.save();
    previewPreset = null;
  }

  function previewPresetTheme(preset: ThemePreset | null) {
    previewPreset = preset;
    if (preset) {
      settingsStore.applyThemeFrom(preset.settings);
    } else {
      settingsStore.applyTheme();
    }
  }

  function applyColors() {
    settingsStore.settings.colors = deepCopy(tempColors);
    settingsStore.save();
  }

  function applyTypo() {
    settingsStore.settings.typography = deepCopy(tempTypo);
    // build JSON directly from local tempTypo to avoid $state proxy issues
    const fullJson = JSON.stringify({
      theme: settingsStore.settings.theme,
      colors: settingsStore.settings.colors,
      typography: {
        font_family: tempTypo.font_family,
        font_family_mono: tempTypo.font_family_mono,
        font_size: tempTypo.font_size,
        line_height: tempTypo.line_height,
      },
      layout: settingsStore.settings.layout,
      ai_provider: settingsStore.settings.ai_provider,
      ai_api_key: settingsStore.settings.ai_api_key,
      ai_model: settingsStore.settings.ai_model,
      ai_enabled: settingsStore.settings.ai_enabled,
      ai_endpoint: settingsStore.settings.ai_endpoint,
    }, null, 2);
    settingsStore.saveRawJson(fullJson);
  }

  function applyTypoWith(font: string, mono: string) {
    settingsStore.settings.typography.font_family = font;
    settingsStore.settings.typography.font_family_mono = mono;
    settingsStore.applyTheme();
  }

  function applyLayout() {
    settingsStore.settings.layout = deepCopy(tempLayout);
    settingsStore.save();
  }

  function applyCustomCSS() {
    settingsStore.setCustomCSS(customCSS);
  }

  function saveAiSettings() {
    settingsStore.settings.ai_provider = aiProvider;
    settingsStore.settings.ai_api_key = aiKey;
    settingsStore.settings.ai_model = aiModel;
    settingsStore.settings.ai_enabled = aiEnabled;
    settingsStore.settings.ai_endpoint = aiEndpoint;
    settingsStore.save();
  }

  function selectProvider(id: string) {
    aiProvider = id;
    const p = PROVIDERS.find(pp => pp.id === id);
    if (p) {
      aiEndpoint = p.endpoint;
      aiModel = p.defaultModel;
    }
  }

  async function testConnection() {
    testing = true;
    testResult = 'Testing…';
    try {
      saveAiSettings();
      const r = await invoke('test_ai_connection');
      testResult = `✅ Connected: ${r.trim()}`;
    } catch (e: any) {
      testResult = 'Error: ' + (e?.toString() || 'Unknown');
    } finally {
      testing = false;
    }
  }

  async function fetchModels() {
    testing = true;
    testResult = 'Fetching models…';
    try {
      saveAiSettings();
      const count = await aiStore.fetchModels();
      if (aiStore.availableModels.length > 0) {
        // Populate model field with first found model
        aiModel = aiStore.availableModels[0].id;
      }
      testResult = `✅ ${count} models found`;
    } catch (e: any) {
      testResult = 'Error: ' + (e?.toString() || 'Unknown');
    } finally {
      testing = false;
    }
  }

  function providerEndpoint(): string {
    const p = PROVIDERS.find(pp => pp.id === aiProvider);
    return p ? p.endpoint : 'https://api.openai.com/v1';
  }

  let exporting = $state(false);

  async function exportAll() {
    const dest = await open({
      directory: true,
      multiple: false,
      title: 'Select export folder',
    });
    if (!dest) return;
    exporting = true;
    try {
      const r = await invoke('export_all', { destPath: dest });
      alert(`Exported ${(r as any).notes_count} notes and ${(r as any).pdfs_count} PDFs to:\n${(r as any).dest_path}`);
    } catch (e: any) {
      alert('Export failed: ' + (e?.toString() || 'Unknown error'));
    } finally {
      exporting = false;
    }
  }

  function close() {
    previewPreset = null;
    settingsStore.save();
    settingsStore.applyTheme();
    onclose();
  }

  function handleOverlayKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function resetDefaults() {
    settingsStore.settings = {
      theme: 'dark',
      colors: {
        bg_primary: '#1A1A2E',
        bg_secondary: '#16213E',
        accent: '#0F3460',
        highlight: '#E94560',
        text_primary: '#E0E0E0',
        text_secondary: '#A0A0A0',
        border: '#2a2a4a',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
      ai_provider: 'groq',
      ai_api_key: '',
      ai_model: 'llama-3.3-70b-versatile',
      ai_enabled: true,
      ai_endpoint: '',
    };
    settingsStore.save();
  }

  const COLOR_LABELS: Record<string, string> = {
    bg_primary: 'Background',
    bg_secondary: 'Sidebar BG',
    accent: 'Accent',
    highlight: 'Highlight',
    text_primary: 'Text',
    text_secondary: 'Text (secondary)',
    border: 'Border',
  };
</script>

<div class="settings-overlay" onclick={close} onkeydown={handleOverlayKeydown} role="dialog" tabindex="-1">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="settings-panel" role="presentation" onclick={(e) => e.stopPropagation()}>
    <div class="panel-header">
      <h2>Settings</h2>
      <button class="close-btn" onclick={close}>×</button>
    </div>

    <div class="tabs">
      <button class="tab" class:active={activeTab === 'general'} onclick={() => activeTab = 'general'}>
        General
      </button>
      <button class="tab" class:active={activeTab === 'presets'} onclick={() => activeTab = 'presets'}>
        Presets
      </button>
      <button class="tab" class:active={activeTab === 'colors'} onclick={() => activeTab = 'colors'}>
        Colors
      </button>
      <button class="tab" class:active={activeTab === 'typography'} onclick={() => activeTab = 'typography'}>
        Font
      </button>
      <button class="tab" class:active={activeTab === 'layout'} onclick={() => activeTab = 'layout'}>
        Layout
      </button>
      <button class="tab" class:active={activeTab === 'css'} onclick={() => activeTab = 'css'}>
        CSS
      </button>
      <button class="tab" class:active={activeTab === 'ai'} onclick={() => activeTab = 'ai'}>
        AI
      </button>
      <button class="tab" class:active={activeTab === 'tips'} onclick={() => activeTab = 'tips'}>
        Tips
      </button>
    </div>

    <div class="panel-body">
      {#if activeTab === 'general'}
        <div class="section">
          <h3>Export</h3>
          <p class="hint">Export all your notes and PDFs to a folder of your choice. Notes are saved as individual .md files with YAML frontmatter. PDFs are copied to a <code>pdfs/</code> subfolder.</p>
          <button class="btn btn-primary" style="margin-top:8px;" onclick={exportAll} disabled={exporting}>
            {exporting ? 'Exporting…' : '📦 Export All'}
          </button>
        </div>
      {/if}

      {#if activeTab === 'presets'}
        <div class="preset-grid" onmouseleave={() => previewPresetTheme(null)}>
          {#each THEME_PRESETS as preset}
            <button class="preset-card"
              class:selected={previewPreset?.id === preset.id}
              onmouseenter={() => previewPresetTheme(preset)}
              onclick={() => applyPreset(preset)}>
              <div class="preset-swatches">
                {#each Object.values(preset.settings.colors) as color}
                  <span class="swatch" style="background: {color};"></span>
                {/each}
              </div>
              <div class="preset-name">{preset.name}</div>
              <div class="preset-desc">{preset.description}</div>
            </button>
          {/each}
        </div>
      {/if}

      {#if activeTab === 'colors'}
        <div class="section">
          <h3>Theme</h3>
          <label class="toggle-row">
            <span>Dark Mode</span>
            <input type="checkbox" checked={s.theme === 'dark'}
              onchange={(e) => { settingsStore.settings.theme = e.currentTarget.checked ? 'dark' : 'light'; settingsStore.save(); }} />
          </label>
        </div>

        <div class="section">
          <h3>Colors</h3>
          <div class="color-grid">
            {#each Object.entries(COLOR_LABELS) as [key, label]}
              <div class="color-row">
                <label for="c-{key}">{label}</label>
                <div class="color-input-group">
                  <input type="color" id="c-{key}" value={tempColors[key as keyof typeof tempColors]}
                    onchange={(e) => {
                      tempColors[key as keyof typeof tempColors] = e.currentTarget.value;
                      applyColors();
                    }} />
                  <input type="text" value={tempColors[key as keyof typeof tempColors]}
                    onchange={(e) => {
                      tempColors[key as keyof typeof tempColors] = e.currentTarget.value;
                      applyColors();
                    }} />
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if activeTab === 'typography'}
        <div class="section">
          <h3>Font Presets</h3>
          <p class="hint">Click a preset to preview, click again to apply. Custom values below are always available.</p>
          <div class="font-grid" onmouseleave={() => { previewFont = null; applyTypo(); }}>
            {#each FONT_PRESETS as fp}
              <button class="font-card"
                class:selected={previewFont === fp.id || activeFontId === fp.id}
                onmouseenter={() => {
                  previewFont = fp.id;
                  if (fp.google_font) loadGoogleFont(fp.google_font);
                  applyTypoWith(fp.font_family, fp.font_family_mono);
                }}
                onclick={() => {
                  tempTypo.font_family = fp.font_family;
                  tempTypo.font_family_mono = fp.font_family_mono;
                  if (fp.google_font) loadGoogleFont(fp.google_font);
                  previewFont = null;
                  applyTypo();
                }}>
                <span class="font-card-name">{fp.name}</span>
                <span class="font-preview" style="font-family: {fp.font_family};">{fp.preview}</span>
                <span class="font-mono-preview" style="font-family: {fp.font_family_mono};">console.log("hi")</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="section font-preview-section">
          <h3>Live Preview</h3>
          <div class="preview-sans" style="font-family: {tempTypo.font_family}; font-size: {tempTypo.font_size}px; line-height: {tempTypo.line_height};">
            The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.
          </div>
          <div class="preview-mono" style="font-family: {tempTypo.font_family_mono}; font-size: {tempTypo.font_size}px; line-height: {tempTypo.line_height};">
            const hello = "world";  // monospace preview
          </div>
        </div>

        <div class="section">
          <h3>Interface Font (custom)</h3>
          <input type="text" value={tempTypo.font_family} class="text-input"
            onchange={(e) => { tempTypo.font_family = e.currentTarget.value; applyTypo(); }} />
        </div>

        <div class="section">
          <h3>Editor Font (custom)</h3>
          <input type="text" value={tempTypo.font_family_mono} class="text-input"
            onchange={(e) => { tempTypo.font_family_mono = e.currentTarget.value; applyTypo(); }} />
        </div>

        <div class="section">
          <h3>Editor Size</h3>
          <div class="slider-row">
            <input type="range" min="10" max="30" value={tempTypo.font_size}
              oninput={(e) => { tempTypo.font_size = Number(e.currentTarget.value); }}
              onchange={applyTypo} />
            <span class="value">{tempTypo.font_size}px</span>
          </div>
        </div>

        <div class="section">
          <h3>Line Height</h3>
          <div class="slider-row">
            <input type="range" min="1" max="2.5" step="0.1" value={tempTypo.line_height}
              oninput={(e) => { tempTypo.line_height = Number(e.currentTarget.value); }}
              onchange={applyTypo} />
            <span class="value">{tempTypo.line_height}</span>
          </div>
        </div>
      {/if}

      {#if activeTab === 'layout'}
        <div class="section">
          <h3>Sidebar Width</h3>
          <p class="hint">Drag the sidebar border directly in the UI</p>
          <div class="slider-row">
            <input type="range" min="180" max="500" value={tempLayout.sidebar_width}
              oninput={(e) => { tempLayout.sidebar_width = Number(e.currentTarget.value); }}
              onchange={applyLayout} />
            <span class="value">{tempLayout.sidebar_width}px</span>
          </div>
        </div>
      {/if}

      {#if activeTab === 'css'}
        <div class="section">
          <h3>Custom CSS</h3>
          <p class="hint">Override any style with your own CSS. Changes apply immediately.</p>
          <textarea class="css-editor" placeholder={"/* Your custom CSS here */\n.sidebar { background: red; }"} bind:value={customCSS}
            oninput={() => settingsStore.setCustomCSS(customCSS)}></textarea>
        </div>
      {/if}

      {#if activeTab === 'ai'}
        <div class="section">
          <h3>Provider</h3>

          <div class="provider-cards">
            {#each PROVIDERS as p}
              <button
                class="provider-card"
                class:selected={aiProvider === p.id}
                onclick={() => selectProvider(p.id)}
              >
                <div class="provider-name">{p.name}</div>
                <div class="provider-price">{p.price}</div>
                <div class="provider-desc">{p.desc}</div>
                <div class="provider-url">🔑 {p.url}</div>
                {#if p.badge}
                  <div class="provider-badge">{p.badge}</div>
                {/if}
              </button>
            {/each}
          </div>

          <h3>API Key</h3>
          <div class="form-row">
            <input type="password" bind:value={aiKey} placeholder="Paste your API key here…" />
          </div>

          <h3>Model</h3>
          <div class="form-row" style="display:flex;gap:8px;align-items:start;">
            <input type="text" bind:value={aiModel} class="flex-1" style="flex:1;" placeholder={aiProvider === 'groq' ? 'llama-3.3-70b-versatile' : 'model-id'} />
            <button class="study-btn" onclick={fetchModels} disabled={!aiKey}>Fetch</button>
          </div>

          <h3>Endpoint</h3>
          <div class="form-row">
            <input type="text" bind:value={aiEndpoint} placeholder={providerEndpoint()} />
          </div>

          <div class="test-row" style="margin-top:12px;">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={aiEnabled} />
              Enable AI
            </label>
            <button class="study-btn" onclick={testConnection} disabled={testing || !aiKey}>
              {testing ? '…' : 'Test'}
            </button>
            {#if testResult}
              <span class="test-result" class:ok={testResult.startsWith('✅ Connected')}>{testResult}</span>
            {/if}
          </div>

          <button class="btn btn-primary" style="margin-top:14px;width:100%;" onclick={saveAiSettings}>Save AI Config</button>
        </div>
      {/if}

      {#if activeTab === 'tips'}
        <div class="section">
          <h3>Study Techniques</h3>
          <p class="hint">Evidence-based learning methods available in the Study panel.</p>

          <div class="tip-card">
            <div class="tip-icon">📝</div>
            <div class="tip-body">
              <strong>Flashcards (Generate)</strong>
              <p>Active recall Q&A cards generated from your notes. Best for factual knowledge. Cards use spaced repetition (SM-2): the more you recall correctly, the longer the interval until next review.</p>
            </div>
          </div>

          <div class="tip-card">
            <div class="tip-icon">🔍</div>
            <div class="tip-body">
              <strong>Deep Questions</strong>
              <p>Elaborative interrogation: "why" and "how" questions that connect concepts instead of just recalling facts. Forces deeper thinking. Uses SM-2 spaced repetition.</p>
            </div>
          </div>

          <div class="tip-card">
            <div class="tip-icon">🧠</div>
            <div class="tip-body">
              <strong>Feynman Technique</strong>
              <p>The AI challenges you to explain a concept in your own words, then evaluates your explanation. One-time exercise — no spaced repetition. Best for testing true understanding.</p>
            </div>
          </div>

          <div class="tip-card">
            <div class="tip-icon">💡</div>
            <div class="tip-body">
              <strong>Concrete Examples</strong>
              <p>Generates relatable analogies or real-world examples for abstract concepts in your notes. Makes complex ideas memorable through familiar comparisons.</p>
            </div>
          </div>

          <div class="tip-card">
            <div class="tip-icon">📊</div>
            <div class="tip-body">
              <strong>Quiz Dashboard</strong>
              <p>Daily review of items due via SM-2 spaced repetition. Rate your recall 1-5. Topics are automatically interleaved (mixed). Questions linked to notes with upcoming calendar events (exams, deadlines) are prioritized and shown with a 📅 badge.</p>
            </div>
          </div>

          <h3 style="margin-top:20px">How SM-2 works</h3>
          <p class="hint">
            After each review, rate your recall from 1 (forgot) to 5 (perfect). The algorithm adjusts the review schedule:<br/>
            <strong>1-2:</strong> Reset — review again tomorrow.<br/>
            <strong>3:</strong> Review in 1-6 days.<br/>
            <strong>4-5:</strong> Interval grows exponentially. You'll see the card again in days, weeks, then months.
          </p>
        </div>
      {/if}
    </div>

    <div class="panel-footer">
      <button class="btn btn-danger" onclick={resetDefaults}>Reset to Defaults</button>
    </div>
  </div>
</div>

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
  }
  .settings-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: 680px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,0.5);
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 12px;
  }
  .panel-header h2 {
    font-size: 18px;
    font-weight: 700;
    margin: 0;
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 22px;
    cursor: pointer;
    padding: 0 4px;
  }
  .close-btn:hover {
    color: var(--text-primary);
  }
  .tabs {
    display: flex;
    gap: 0;
    padding: 0 24px;
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
  }
  .tab {
    padding: 10px 16px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
  }
  .tab.active {
    color: var(--text-primary);
    border-bottom-color: var(--highlight);
  }
  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
  }
  .section {
    margin-bottom: 20px;
  }
  .section h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }
  .hint {
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    cursor: pointer;
  }

  .preset-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .font-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
  }
  .font-card {
    background: var(--bg-primary);
    border: 2px solid var(--border);
    border-radius: 8px;
    padding: 6px 8px;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s;
    overflow: hidden;
  }
  .font-card:hover, .font-card.selected {
    border-color: var(--highlight);
  }
  .font-card-name {
    display: block;
    font-size: 10px;
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    margin-bottom: 2px;
  }
  .font-preview {
    display: block;
    font-size: 13px;
    line-height: 1.3;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 1px;
  }
  .font-mono-preview {
    display: block;
    font-size: 10px;
    line-height: 1.3;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .font-preview-section {
    background: var(--bg-primary);
    border-radius: 8px;
    padding: 12px 16px;
  }
  .font-preview-section h3 {
    margin-bottom: 8px;
  }
  .preview-sans {
    margin-bottom: 8px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-radius: 6px;
    color: var(--text-primary);
  }
  .preview-mono {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-radius: 6px;
    color: var(--text-secondary);
  }
  .preset-card {
    background: var(--bg-primary);
    border: 2px solid var(--border);
    border-radius: 10px;
    padding: 16px;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s;
  }
  .preset-card:hover, .preset-card.selected {
    border-color: var(--highlight);
  }
  .preset-swatches {
    display: flex;
    gap: 4px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .swatch {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: 1px solid rgba(255,255,255,0.1);
  }
  .preset-name {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 2px;
  }
  .preset-desc {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .color-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .color-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .color-row label {
    font-size: 13px;
    min-width: 120px;
  }
  .color-input-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .color-input-group input[type="color"] {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px;
    background: none;
    cursor: pointer;
  }
  .color-input-group input[type="text"] {
    width: 100px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
  }
  .text-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 12px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-mono);
  }
  .slider-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .slider-row input[type="range"] {
    flex: 1;
    accent-color: var(--highlight);
  }
  .slider-row .value {
    min-width: 48px;
    font-size: 13px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }
  .css-editor {
    width: 100%;
    height: 200px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    resize: vertical;
    tab-size: 2;
  }
  .panel-footer {
    padding: 12px 24px 20px;
    border-top: 1px solid var(--border);
  }
  .btn {
    padding: 8px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  .btn-danger {
    color: var(--highlight);
    border-color: var(--highlight);
  }
  .btn-danger:hover {
    background: var(--highlight);
    color: #fff;
  }
  .form-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }
  .form-row label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .form-row select, .form-row input {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 13px;
    color: var(--text-primary);
    font-family: var(--font-sans);
    outline: none;
    color-scheme: dark;
  }
  .form-row select option {
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  .form-row select:focus, .form-row input:focus {
    border-color: var(--highlight);
  }
  .checkbox-label {
    display: flex; align-items: center; gap: 6px;
    cursor: pointer; text-transform: none; font-size: 13px;
    letter-spacing: 0; font-weight: 400; color: var(--text-primary);
  }
  .test-row {
    display: flex; align-items: center; gap: 10px; margin-top: 8px;
  }
  .test-result {
    font-size: 12px; color: var(--highlight);
  }
  .test-result.ok { color: #4CAF50; }
  .provider-cards {
    display: flex; gap: 10px; flex-wrap: wrap;
  }
  .provider-card {
    flex: 1; min-width: 180px; max-width: 220px;
    background: var(--bg-primary); border: 1px solid var(--border);
    border-radius: 8px; padding: 14px;
    cursor: pointer; text-align: left;
    color: var(--text-primary); font-family: inherit;
    transition: border-color 0.15s; position: relative;
  }
  .provider-card:hover { border-color: var(--highlight); }
  .provider-card.selected {
    border-color: var(--highlight);
    box-shadow: 0 0 0 2px var(--highlight);
  }
  .provider-name { font-size: 15px; font-weight: 700; margin-bottom: 2px; }
  .provider-price { font-size: 12px; color: var(--text-secondary); margin-bottom: 6px; }
  .provider-desc { font-size: 12px; color: var(--text-secondary); margin-bottom: 6px; }
  .provider-url { font-size: 11px; color: var(--text-secondary); opacity: 0.7; }
  .provider-badge {
    position: absolute; top: -8px; right: 8px;
    background: var(--highlight); color: #fff;
    font-size: 10px; font-weight: 600;
    padding: 2px 8px; border-radius: 10px;
  }
  .advanced-toggle {
    font-size: 12px; color: var(--text-secondary);
    cursor: pointer; padding: 4px 0; user-select: none;
  }
  .advanced-toggle:hover { color: var(--text-primary); }
  .tip-card { display: flex; gap: 12px; padding: 12px;
    background: var(--bg-primary); border: 1px solid var(--border);
    border-radius: 8px; margin-bottom: 10px; }
  .tip-icon { font-size: 20px; flex-shrink: 0; width: 28px; text-align: center; }
  .tip-body strong { font-size: 13px; color: var(--text-primary); }
  .tip-body p { font-size: 12px; color: var(--text-secondary); margin: 4px 0 0; line-height: 1.5; }
</style>
