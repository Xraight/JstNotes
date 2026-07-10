import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types';

const FONT_GOOGLE_MAP: Record<string, string> = {
  "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif": 'Inter:wght@400;500;600;700',
  "'Merriweather', Georgia, 'Times New Roman', serif": 'Merriweather:wght@400;700',
  "'Space Mono', 'Courier New', monospace": 'Space+Mono:wght@400;700',
  "'Ubuntu', 'Segoe UI', sans-serif": 'Ubuntu:wght@400;500;700',
  "'Atkinson Hyperlegible', sans-serif": 'Atkinson+Hyperlegible:wght@400;700',
};

const MONO_GOOGLE_MAP: Record<string, string> = {
  "'JetBrains Mono', 'Fira Code', monospace": 'JetBrains+Mono:wght@400;500;700',
  "'Fira Code', 'JetBrains Mono', monospace": 'Fira+Code:wght@400;500;700',
  "'Space Mono', 'Courier New', monospace": 'Space+Mono:wght@400;700',
  "'Ubuntu Mono', 'Fira Code', monospace": 'Ubuntu+Mono:wght@400;700',
  "'IBM Plex Mono', 'JetBrains Mono', monospace": 'IBM+Plex+Mono:wght@400;500;700',
  "'Source Code Pro', 'Fira Code', monospace": 'Source+Code+Pro:wght@400;500;700',
};

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
  private loadedFonts = new Set<string>();

  private ensureFontLoaded(family: string, map: Record<string, string>) {
    const gf = map[family];
    if (!gf || this.loadedFonts.has(gf)) return;
    this.loadedFonts.add(gf);
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = `https://fonts.googleapis.com/css2?family=${gf}&display=swap`;
    document.head.appendChild(link);
  }

  async load() {
    try {
      const raw = await invoke<string>('get_settings_raw');
      const parsed = JSON.parse(raw) as AppSettings;
      this.settings = {
        ...DEFAULT_SETTINGS,
        theme: parsed.theme,
        colors: { ...DEFAULT_SETTINGS.colors, ...parsed.colors },
        typography: { ...DEFAULT_SETTINGS.typography, ...parsed.typography },
        layout: { ...DEFAULT_SETTINGS.layout, ...parsed.layout },
        ai_provider: parsed.ai_provider ?? DEFAULT_SETTINGS.ai_provider,
        ai_api_key: parsed.ai_api_key ?? DEFAULT_SETTINGS.ai_api_key,
        ai_model: parsed.ai_model ?? DEFAULT_SETTINGS.ai_model,
        ai_enabled: parsed.ai_enabled ?? DEFAULT_SETTINGS.ai_enabled,
        ai_endpoint: parsed.ai_endpoint ?? DEFAULT_SETTINGS.ai_endpoint,
      };
    } catch {
      this.settings = { ...DEFAULT_SETTINGS };
    }
    this.applyTheme();
  }

  async save() {
    this.applyTheme();
    const s = this.settings;
    const plain: AppSettings = {
      theme: s.theme,
      colors: { ...s.colors },
      typography: { ...s.typography },
      layout: { ...s.layout },
      ai_provider: s.ai_provider,
      ai_api_key: s.ai_api_key,
      ai_model: s.ai_model,
      ai_enabled: s.ai_enabled,
      ai_endpoint: s.ai_endpoint,
    };
    try {
      const saved = await invoke<AppSettings>('save_settings', {
        newSettings: plain,
      });
      this.settings = {
        ...DEFAULT_SETTINGS,
        theme: saved.theme,
        colors: { ...DEFAULT_SETTINGS.colors, ...saved.colors },
        typography: { ...DEFAULT_SETTINGS.typography, ...saved.typography },
        layout: { ...DEFAULT_SETTINGS.layout, ...saved.layout },
        ai_provider: saved.ai_provider ?? DEFAULT_SETTINGS.ai_provider,
        ai_api_key: saved.ai_api_key ?? DEFAULT_SETTINGS.ai_api_key,
        ai_model: saved.ai_model ?? DEFAULT_SETTINGS.ai_model,
        ai_enabled: saved.ai_enabled ?? DEFAULT_SETTINGS.ai_enabled,
        ai_endpoint: saved.ai_endpoint ?? DEFAULT_SETTINGS.ai_endpoint,
      };
    } catch {
      // keep local state even if save fails
    }
  }

  async saveRawJson(jsonContent: string) {
    this.applyTheme();
    try {
      const saved = await invoke<string>('save_settings_raw', {
        jsonContent: jsonContent,
      });
      const parsed = JSON.parse(saved);
      this.settings = {
        ...DEFAULT_SETTINGS,
        theme: parsed.theme,
        colors: { ...DEFAULT_SETTINGS.colors, ...parsed.colors },
        typography: { ...DEFAULT_SETTINGS.typography, ...parsed.typography },
        layout: { ...DEFAULT_SETTINGS.layout, ...parsed.layout },
        ai_provider: parsed.ai_provider ?? DEFAULT_SETTINGS.ai_provider,
        ai_api_key: parsed.ai_api_key ?? DEFAULT_SETTINGS.ai_api_key,
        ai_model: parsed.ai_model ?? DEFAULT_SETTINGS.ai_model,
        ai_enabled: parsed.ai_enabled ?? DEFAULT_SETTINGS.ai_enabled,
        ai_endpoint: parsed.ai_endpoint ?? DEFAULT_SETTINGS.ai_endpoint,
      };
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
    this.ensureFontLoaded(s.typography.font_family, FONT_GOOGLE_MAP);
    this.ensureFontLoaded(s.typography.font_family_mono, MONO_GOOGLE_MAP);
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
