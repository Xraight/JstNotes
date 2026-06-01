<script lang="ts">
  import { settingsStore } from '../stores/settings';
  import { THEME_PRESETS, type ThemePreset } from '../presets';
  import type { ColorSettings, TypographySettings, LayoutSettings } from '../types';

  let { onclose }: { onclose: () => void } = $props();

  let s = $derived(settingsStore.settings);
  let activeTab = $state<'presets' | 'colors' | 'typography' | 'layout' | 'css'>('presets');

  let tempColors = $state<ColorSettings>({ bg_primary: '', bg_secondary: '', accent: '', highlight: '', text_primary: '', text_secondary: '', border: '' });
  let tempTypo = $state<TypographySettings>({ font_family: '', font_family_mono: '', font_size: 15, line_height: 1.7 });
  let tempLayout = $state<LayoutSettings>({ sidebar_width: 280 });
  let customCSS = $state('');
  let previewPreset = $state<ThemePreset | null>(null);

  $effect(() => {
    tempColors = deepCopy(s.colors);
    tempTypo = deepCopy(s.typography);
    tempLayout = deepCopy(s.layout);
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
    settingsStore.save();
  }

  function applyLayout() {
    settingsStore.settings.layout = deepCopy(tempLayout);
    settingsStore.save();
  }

  function applyCustomCSS() {
    settingsStore.setCustomCSS(customCSS);
  }

  function close() {
    previewPreset = null;
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
    </div>

    <div class="panel-body">
      {#if activeTab === 'presets'}
        <div class="preset-grid">
          {#each THEME_PRESETS as preset}
            <button class="preset-card"
              class:selected={previewPreset?.id === preset.id}
              onmouseenter={() => previewPresetTheme(preset)}
              onmouseleave={() => previewPresetTheme(null)}
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
          <h3>Interface Font</h3>
          <input type="text" value={tempTypo.font_family} class="text-input"
            onchange={(e) => { tempTypo.font_family = e.currentTarget.value; applyTypo(); }} />
        </div>

        <div class="section">
          <h3>Editor Font</h3>
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
    width: 580px;
    max-height: 85vh;
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
</style>
