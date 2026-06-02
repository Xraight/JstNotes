export interface Note {
  id: string;
  title: string;
  content: string;
  parent_id: string | null;
  path: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface CreateNoteRequest {
  title: string;
  content: string;
  parent_id: string | null;
  sort_order?: number;
}

export interface UpdateNoteRequest {
  id: string;
  title?: string;
  content?: string;
  parent_id?: string | null;
  sort_order?: number;
}

export interface NoteTreeNode {
  id: string;
  title: string;
  path: string;
  children: NoteTreeNode[];
  created_at: string;
}

export interface Breadcrumb {
  id: string;
  title: string;
  path: string;
}

export interface ColorSettings {
  bg_primary: string;
  bg_secondary: string;
  accent: string;
  highlight: string;
  text_primary: string;
  text_secondary: string;
  border: string;
}

export interface TypographySettings {
  font_family: string;
  font_family_mono: string;
  font_size: number;
  line_height: number;
}

export interface LayoutSettings {
  sidebar_width: number;
}

export interface AppSettings {
  theme: string;
  colors: ColorSettings;
  typography: TypographySettings;
  layout: LayoutSettings;
}

export interface CalendarEvent {
  id: string;
  date: string;
  title: string;
  description: string;
  completed: boolean;
  note_ids: string[];
  created_at: string;
}
