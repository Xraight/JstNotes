import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteTreeNode, Breadcrumb, CreateNoteRequest, UpdateNoteRequest } from '../types';

class NoteStore {
  notes = $state<Note[]>([]);
  tree = $state<NoteTreeNode[]>([]);
  selectedNote = $state<Note | null>(null);
  breadcrumbs = $state<Breadcrumb[]>([]);
  loading = $state(false);

  async loadTree() {
    this.tree = await invoke<NoteTreeNode[]>('build_tree');
  }

  async loadNotes() {
    this.notes = await invoke<Note[]>('list_notes');
  }

  async selectNote(id: string) {
    this.selectedNote = await invoke<Note | null>('get_note', { id });
    if (id) {
      this.breadcrumbs = await invoke<Breadcrumb[]>('get_breadcrumbs', { noteId: id });
    } else {
      this.breadcrumbs = [];
    }
  }

  async createNote(req: CreateNoteRequest) {
    const note = await invoke<Note>('create_note', { req });
    await this.loadTree();
    await this.selectNote(note.id);
    return note;
  }

  async updateNote(req: UpdateNoteRequest) {
    const note = await invoke<Note>('update_note', { req });
    await this.loadTree();
    if (this.selectedNote?.id === note.id) {
      this.selectedNote = note;
    }
    return note;
  }

  async deleteNote(id: string) {
    await invoke<void>('delete_note', { id });
    if (this.selectedNote?.id === id) {
      this.selectedNote = null;
      this.breadcrumbs = [];
    }
    await this.loadTree();
  }

  async getChildren(parentId: string) {
    return await invoke<Note[]>('get_children', { parentId });
  }
}

export const noteStore = new NoteStore();
