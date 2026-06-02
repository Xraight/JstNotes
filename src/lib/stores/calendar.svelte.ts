import { invoke } from '@tauri-apps/api/core';
import type { CalendarEvent } from '../types';

class CalendarStore {
  events = $state<CalendarEvent[]>([]);
  currentYear = $state(new Date().getFullYear());
  currentMonth = $state(new Date().getMonth() + 1);

  async loadEvents(year: number, month: number) {
    this.currentYear = year;
    this.currentMonth = month;
    this.events = await invoke<CalendarEvent[]>('get_calendar_events', { year, month });
  }

  async createEvent(date: string, title: string, description: string, noteIds: string[] = []) {
    const event = await invoke<CalendarEvent>('create_calendar_event', {
      date,
      title,
      description,
      noteIds,
    });
    await this.loadEvents(this.currentYear, this.currentMonth);
    return event;
  }

  async updateEvent(id: string, title: string, description: string, completed: boolean) {
    await invoke<void>('update_calendar_event', { id, title, description, completed });
    await this.loadEvents(this.currentYear, this.currentMonth);
  }

  async deleteEvent(id: string) {
    await invoke<void>('delete_calendar_event', { id });
    await this.loadEvents(this.currentYear, this.currentMonth);
  }

  async linkNote(eventId: string, noteId: string) {
    await invoke<void>('link_note_to_event', { eventId, noteId });
    await this.loadEvents(this.currentYear, this.currentMonth);
  }

  async unlinkNote(eventId: string, noteId: string) {
    await invoke<void>('unlink_note_from_event', { eventId, noteId });
    await this.loadEvents(this.currentYear, this.currentMonth);
  }

  async getEventsForNote(noteId: string): Promise<CalendarEvent[]> {
    return await invoke<CalendarEvent[]>('get_events_for_note', { noteId });
  }

  getEventsForDay(day: number): CalendarEvent[] {
    const date = `${this.currentYear}-${String(this.currentMonth).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    return this.events.filter(e => e.date === date);
  }
}

export const calendarStore = new CalendarStore();
