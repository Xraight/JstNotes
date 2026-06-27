import { invoke } from '@tauri-apps/api/core';
import type { StudyItem } from '../types';
import { settingsStore } from './settings';

class AIStore {
  studyItems = $state<StudyItem[]>([]);
  dueReviews = $state<StudyItem[]>([]);
  isGenerating = $state(false);
  studyPanelOpen = $state(false);
  lastError = $state('');
  availableModels = $state<Array<{ id: string; name: string }>>([]);

  get apiConfigured(): boolean {
    const s = settingsStore.settings;
    return !!(s.ai_api_key && s.ai_enabled);
  }

  async fetchModels(): Promise<number> {
    try {
      this.availableModels = await invoke<Array<{ id: string; name: string }>>('fetch_ai_models');
      return this.availableModels.length;
    } catch {
      this.availableModels = [];
      return 0;
    }
  }

  async generateQuestions(noteId: string) {
    this.isGenerating = true;
    this.lastError = '';
    try {
      const items = await invoke<StudyItem[]>('generate_study_questions', { noteId });
      this.studyItems = items;
      await this.loadDueReviews();
      return items;
    } catch (e: any) {
      this.lastError = e?.toString() || 'Generation failed';
      return [];
    } finally {
      this.isGenerating = false;
    }
  }

  async generateElaboration(noteId: string) {
    this.isGenerating = true;
    this.lastError = '';
    try {
      const items = await invoke<StudyItem[]>('generate_elaboration_questions', { noteId });
      this.studyItems = items;
      await this.loadDueReviews();
      return items;
    } catch (e: any) {
      this.lastError = e?.toString() || 'Generation failed';
      return [];
    } finally {
      this.isGenerating = false;
    }
  }

  async generateExample(noteId: string): Promise<string> {
    this.isGenerating = true;
    this.lastError = '';
    try {
      return await invoke<string>('generate_concrete_example', { noteId });
    } catch (e: any) {
      this.lastError = e?.toString() || 'Generation failed';
      return '';
    } finally {
      this.isGenerating = false;
    }
  }

  async loadDueReviews() {
    try {
      this.dueReviews = await invoke<StudyItem[]>('get_due_reviews');
    } catch {
      this.dueReviews = [];
    }
  }

  async rateReview(itemId: string, quality: number) {
    try {
      await invoke('rate_review', { itemId, quality });
      await this.loadDueReviews();
    } catch (e) {
      console.error('Rate review failed:', e);
    }
  }

  async loadStudyItems(noteId: string) {
    try {
      this.studyItems = await invoke<StudyItem[]>('get_study_items', { noteId });
    } catch {
      this.studyItems = [];
    }
  }

  togglePanel() {
    this.studyPanelOpen = !this.studyPanelOpen;
    if (this.studyPanelOpen) {
      this.loadDueReviews();
    }
  }
}

export const aiStore = new AIStore();
