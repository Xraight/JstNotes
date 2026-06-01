import { invoke } from '@tauri-apps/api/core';
import type { CalendarEntry } from '../types';

class CalendarStore {
  entries = $state<CalendarEntry[]>([]);
  currentYear = $state(new Date().getFullYear());
  currentMonth = $state(new Date().getMonth() + 1);

  async loadEntries(year: number, month: number) {
    this.currentYear = year;
    this.currentMonth = month;
    this.entries = await invoke<CalendarEntry[]>('get_calendar_entries', { year, month });
  }

  async createEntry(date: string, title: string, noteId?: string) {
    const entry = await invoke<CalendarEntry>('create_calendar_entry', {
      date,
      title,
      noteId: noteId ?? null,
    });
    await this.loadEntries(this.currentYear, this.currentMonth);
    return entry;
  }

  async deleteEntry(id: string) {
    await invoke<void>('delete_calendar_entry', { id });
    await this.loadEntries(this.currentYear, this.currentMonth);
  }

  async updateEntry(id: string, title: string, noteId?: string) {
    await invoke<void>('update_calendar_entry', {
      id,
      title,
      noteId: noteId ?? null,
    });
    await this.loadEntries(this.currentYear, this.currentMonth);
  }

  getEntriesForDay(day: number): CalendarEntry[] {
    const date = `${this.currentYear}-${String(this.currentMonth).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    return this.entries.filter(e => e.date === date);
  }
}

export const calendarStore = new CalendarStore();
