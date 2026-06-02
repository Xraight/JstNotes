use crate::models::*;
use crate::storage::markdown::MarkdownStorage;
use crate::storage::sqlite::Database;

pub struct HybridStorage {
    pub db: Database,
    notes_dir: String,
}

impl HybridStorage {
    pub fn new(app_dir: &str) -> Result<Self, String> {
        let db_path = format!("{}/jstnotes.db", app_dir);
        let notes_dir = MarkdownStorage::get_notes_dir(app_dir);
        let db = Database::new(&db_path).map_err(|e| format!("DB error: {}", e))?;
        Ok(HybridStorage { db, notes_dir })
    }

    pub fn create_note(&self, req: CreateNoteRequest) -> Result<Note, String> {
        let note = Note::new(req, &self.notes_dir);

        MarkdownStorage::save_note(&note.path, &note.title, &note.content)
            .map_err(|e| format!("File error: {}", e))?;

        self.db
            .create_note(&note)
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(note)
    }

    pub fn get_note(&self, id: &str) -> Result<Option<Note>, String> {
        let mut note = match self.db.get_note(id).map_err(|e| format!("DB error: {}", e))? {
            Some(n) => n,
            None => return Ok(None),
        };

        let md_path = MarkdownStorage::get_md_path(&self.notes_dir, id);
        if MarkdownStorage::note_exists(&md_path) {
            note.content = MarkdownStorage::read_note(&md_path).unwrap_or_default();
        }

        Ok(Some(note))
    }

    pub fn update_note(&self, req: UpdateNoteRequest) -> Result<Note, String> {
        let mut note = self
            .db
            .get_note(&req.id)
            .map_err(|e| format!("DB error: {}", e))?
            .ok_or_else(|| "Note not found".to_string())?;

        if let Some(title) = req.title {
            note.title = title;
        }
        if let Some(content) = req.content {
            note.content = content;
        }
        if let Some(parent_id) = req.parent_id {
            note.parent_id = Some(parent_id);
        }
        if let Some(sort_order) = req.sort_order {
            note.sort_order = sort_order;
        }

        note.updated_at = chrono::Utc::now();

        let md_path = MarkdownStorage::get_md_path(&self.notes_dir, &note.id);
        MarkdownStorage::save_note(&md_path, &note.title, &note.content)
            .map_err(|e| format!("File error: {}", e))?;

        self.db
            .update_note(&note)
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(note)
    }

    pub fn delete_note(&self, id: &str) -> Result<(), String> {
        let md_path = MarkdownStorage::get_md_path(&self.notes_dir, id);
        MarkdownStorage::delete_note(&md_path).ok();

        self.db
            .delete_note(id)
            .map_err(|e| format!("DB error: {}", e))
    }

    pub fn list_notes(&self) -> Result<Vec<Note>, String> {
        self.db.list_notes().map_err(|e| format!("DB error: {}", e))
    }

    pub fn list_note_titles(&self) -> Result<Vec<(String, String)>, String> {
        self.db.list_note_titles().map_err(|e| format!("DB error: {}", e))
    }

    pub fn get_children(&self, parent_id: &str) -> Result<Vec<Note>, String> {
        self.db
            .get_children(parent_id)
            .map_err(|e| format!("DB error: {}", e))
    }

    pub fn get_breadcrumbs(&self, note_id: &str) -> Result<Vec<Breadcrumb>, String> {
        self.db
            .get_breadcrumbs(note_id)
            .map_err(|e| format!("DB error: {}", e))
    }

    pub fn build_tree(&self) -> Result<Vec<NoteTreeNode>, String> {
        self.db
            .build_tree()
            .map_err(|e| format!("DB error: {}", e))
    }
}
