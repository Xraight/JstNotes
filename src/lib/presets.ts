import type { AppSettings } from './types';

export interface ThemePreset {
  id: string;
  name: string;
  description: string;
  settings: AppSettings;
}

export interface FontPreset {
  id: string;
  name: string;
  font_family: string;
  font_family_mono: string;
  google_font?: string;
  preview: string;
}

export const FONT_PRESETS: FontPreset[] = [
  {
    id: 'system',
    name: 'System',
    font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
    preview: 'Aa — System default sans-serif',
  },
  {
    id: 'inter',
    name: 'Inter',
    font_family: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
    google_font: 'Inter:wght@400;500;600;700',
    preview: 'Aa — Inter sans-serif',
  },
  {
    id: 'merriweather',
    name: 'Merriweather',
    font_family: "'Merriweather', Georgia, 'Times New Roman', serif",
    font_family_mono: "'Source Code Pro', 'Fira Code', monospace",
    google_font: 'Merriweather:wght@400;700',
    preview: 'Aa — Merriweather serif',
  },
  {
    id: 'space-mono',
    name: 'Space Mono',
    font_family: "'Space Mono', 'Courier New', monospace",
    font_family_mono: "'Space Mono', 'Courier New', monospace",
    google_font: 'Space+Mono:wght@400;700',
    preview: 'Aa — Space Mono (mono UI)',
  },
  {
    id: 'ubuntu',
    name: 'Ubuntu',
    font_family: "'Ubuntu', 'Segoe UI', sans-serif",
    font_family_mono: "'Ubuntu Mono', 'Fira Code', monospace",
    google_font: 'Ubuntu:wght@400;500;700',
    preview: 'Aa — Ubuntu sans-serif',
  },
  {
    id: 'atkinson',
    name: 'Atkinson',
    font_family: "'Atkinson Hyperlegible', sans-serif",
    font_family_mono: "'IBM Plex Mono', 'JetBrains Mono', monospace",
    google_font: 'Atkinson+Hyperlegible:wght@400;700',
    preview: 'Aa — Atkinson Hyperlegible',
  },
  {
    id: 'jetbrains',
    name: 'JetBrains Mono',
    font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
    google_font: 'JetBrains+Mono:wght@400;500;700',
    preview: 'Aa — Default UI + JetBrains Mono',
  },
  {
    id: 'fira-code',
    name: 'Fira Code',
    font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    font_family_mono: "'Fira Code', 'JetBrains Mono', monospace",
    google_font: 'Fira+Code:wght@400;500;700',
    preview: 'Aa — Default UI + Fira Code',
  },
];

export const THEME_PRESETS: ThemePreset[] = [
  {
    id: 'midnight',
    name: 'Midnight',
    description: 'Deep navy dark theme',
    settings: {
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
    },
  },
  {
    id: 'dracula',
    name: 'Dracula',
    description: 'Dark purple-pink palette',
    settings: {
      theme: 'dark',
      colors: {
        bg_primary: '#282a36',
        bg_secondary: '#21222c',
        accent: '#44475a',
        highlight: '#ff79c6',
        text_primary: '#f8f8f2',
        text_secondary: '#6272a4',
        border: '#44475a',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
    },
  },
  {
    id: 'nord',
    name: 'Nord',
    description: 'Arctic bluish theme',
    settings: {
      theme: 'dark',
      colors: {
        bg_primary: '#2e3440',
        bg_secondary: '#3b4252',
        accent: '#434c5e',
        highlight: '#88c0d0',
        text_primary: '#eceff4',
        text_secondary: '#81a1c1',
        border: '#4c566a',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
    },
  },
  {
    id: 'solarized',
    name: 'Solarized Light',
    description: 'Warm light theme',
    settings: {
      theme: 'light',
      colors: {
        bg_primary: '#fdf6e3',
        bg_secondary: '#eee8d5',
        accent: '#93a1a1',
        highlight: '#cb4b16',
        text_primary: '#073642',
        text_secondary: '#586e75',
        border: '#d5ccb8',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
    },
  },
  {
    id: 'catppuccin',
    name: 'Catppuccin',
    description: 'Warm pastel dark theme',
    settings: {
      theme: 'dark',
      colors: {
        bg_primary: '#1e1e2e',
        bg_secondary: '#181825',
        accent: '#313244',
        highlight: '#cba6f7',
        text_primary: '#cdd6f4',
        text_secondary: '#a6adc8',
        border: '#45475a',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
    },
  },
  {
    id: 'github-light',
    name: 'GitHub Light',
    description: 'Clean light theme',
    settings: {
      theme: 'light',
      colors: {
        bg_primary: '#ffffff',
        bg_secondary: '#f6f8fa',
        accent: '#e1e4e8',
        highlight: '#0969da',
        text_primary: '#1f2328',
        text_secondary: '#656d76',
        border: '#d0d7de',
      },
      typography: {
        font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', sans-serif",
        font_family_mono: "'JetBrains Mono', 'Fira Code', monospace",
        font_size: 15,
        line_height: 1.7,
      },
      layout: { sidebar_width: 280 },
    },
  },
];
