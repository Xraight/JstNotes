import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types';

const DEFAULT_SETTINGS: AppSettings = {
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
  layout: {
    sidebar_width: 280,
  },
  ai_provider: 'groq',
  ai_api_key: '',
  ai_model: 'llama-3.3-70b-versatile',
  ai_enabled: true,
  ai_endpoint: '',
};

class SettingsStore {
  settings = $state<AppSettings>(DEFAULT_SETTINGS);
  private styleTag: HTMLStyleElement | null = null;

  async load() {
    try {
      const s = await invoke<AppSettings>('get_settings');
      this.settings = { ...DEFAULT_SETTINGS, ...s };
    } catch {
      this.settings = DEFAULT_SETTINGS;
    }
    this.applyTheme();
  }

  async save() {
    this.applyTheme();
    try {
      const plain = JSON.parse(JSON.stringify(this.settings));
      this.settings = await invoke<AppSettings>('save_settings', {
        newSettings: plain,
      });
    } catch {
      // keep local state even if save fails
    }
  }

  applyThemeFrom(custom: AppSettings) {
    const s = custom;
    const root = document.documentElement;
    root.style.setProperty('--bg-primary', s.colors.bg_primary);
    root.style.setProperty('--bg-secondary', s.colors.bg_secondary);
    root.style.setProperty('--accent', s.colors.accent);
    root.style.setProperty('--highlight', s.colors.highlight);
    root.style.setProperty('--text-primary', s.colors.text_primary);
    root.style.setProperty('--text-secondary', s.colors.text_secondary);
    root.style.setProperty('--border', s.colors.border);
    root.style.setProperty('--font-sans', s.typography.font_family);
    root.style.setProperty('--font-mono', s.typography.font_family_mono);
    root.style.setProperty('--editor-font-size', `${s.typography.font_size}px`);
    root.style.setProperty('--editor-line-height', `${s.typography.line_height}`);
    root.style.setProperty('--sidebar-width', `${s.layout.sidebar_width}px`);
    root.setAttribute('data-theme', s.theme);
  }

  applyTheme() {
    this.applyThemeFrom(this.settings);
  }

  setCustomCSS(css: string) {
    if (!this.styleTag) {
      this.styleTag = document.createElement('style');
      this.styleTag.id = 'jstnotes-custom-css';
      document.head.appendChild(this.styleTag);
    }
    this.styleTag.textContent = css;
  }

  setSidebarWidth(width: number) {
    this.settings.layout.sidebar_width = width;
    document.documentElement.style.setProperty('--sidebar-width', `${width}px`);
  }

  async saveSidebarWidth(width: number) {
    this.settings.layout.sidebar_width = width;
    await this.save();
  }

  toggleTheme() {
    this.settings.theme = this.settings.theme === 'dark' ? 'light' : 'dark';
    this.save();
  }
}

export const settingsStore = new SettingsStore();
