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
  ai_provider: string;
  ai_api_key: string;
  ai_model: string;
  ai_enabled: boolean;
  ai_endpoint: string;
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

export interface PdfMetadata {
  id: string;
  title: string | null;
  file_path: string;
  page_count: number;
  text: string | null;
  created_at: string;
}

export interface PdfAnnotation {
  id: string;
  pdf_id: string;
  page: number;
  annotation_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string | null;
  content: string | null;
  created_at: string;
}

export interface AnnotationInput {
  pdf_id: string;
  page: number;
  annotation_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string | null;
  content: string | null;
}

export interface PdfReference {
  id: string;
  note_id: string;
  pdf_id: string;
  page: number;
  page_end: number | null;
  label: string;
  annotation_id: string | null;
  created_at: string;
}

export interface CreatePdfReferenceInput {
  note_id: string;
  pdf_id: string;
  page: number;
  page_end: number | null;
  label: string;
  annotation_id: string | null;
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface Flashcard {
  id: string;
  note_id: string;
  question: string;
  answer: string;
  created_at: string;
  reviewed: number;
  difficulty: number;
}

export interface OllamaModel {
  name: string;
  modified_at: string | null;
  size: number | null;
}

export interface StudyItem {
  id: string;
  note_id: string;
  question: string;
  answer: string;
  created_at: string;
  next_review: string;
  interval_days: number;
  ease_factor: number;
  repetitions: number;
  reviewed_at: string | null;
  source_page: number | null;
}
