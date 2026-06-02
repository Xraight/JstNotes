use std::collections::HashMap;
use rusqlite::{params, Connection, Result as SqlResult};
use std::sync::Mutex;

use crate::models::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content_hash TEXT NOT NULL DEFAULT '',
                parent_id TEXT,
                path TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_parent_id ON notes(parent_id);
            CREATE INDEX IF NOT EXISTS idx_notes_path ON notes(path);

            CREATE TABLE IF NOT EXISTS calendar_events (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                completed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS event_notes (
                event_id TEXT NOT NULL,
                note_id TEXT NOT NULL,
                PRIMARY KEY (event_id, note_id),
                FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_calendar_date ON calendar_events(date);
            CREATE INDEX IF NOT EXISTS idx_event_notes_event ON event_notes(event_id);
            CREATE INDEX IF NOT EXISTS idx_event_notes_note ON event_notes(note_id);

            -- migrate old entries if upgrading
            CREATE TABLE IF NOT EXISTS calendar_entries (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                title TEXT NOT NULL,
                note_id TEXT,
                created_at TEXT NOT NULL
            );
            INSERT OR IGNORE INTO calendar_events (id, date, title, description, completed, created_at)
                SELECT id, date, title, '', 0, created_at FROM calendar_entries;
            INSERT OR IGNORE INTO event_notes (event_id, note_id)
                SELECT id, note_id FROM calendar_entries WHERE note_id IS NOT NULL;
            ",
        )?;
        // drop old table schema after migrating data
        conn.execute_batch("DROP TABLE IF EXISTS calendar_entries;").ok();

        // PDF tables
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS pdfs (
                id          TEXT PRIMARY KEY,
                title       TEXT,
                file_path   TEXT NOT NULL,
                page_count  INTEGER DEFAULT 0,
                text        TEXT,
                created_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pdf_annotations (
                id          TEXT PRIMARY KEY,
                pdf_id      TEXT NOT NULL,
                page        INTEGER NOT NULL,
                type        TEXT NOT NULL,
                x           REAL NOT NULL DEFAULT 0,
                y           REAL NOT NULL DEFAULT 0,
                width       REAL NOT NULL DEFAULT 0,
                height      REAL NOT NULL DEFAULT 0,
                color       TEXT,
                content     TEXT,
                created_at  TEXT NOT NULL,
                FOREIGN KEY (pdf_id) REFERENCES pdfs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS note_pdfs (
                note_id TEXT NOT NULL,
                pdf_id  TEXT NOT NULL,
                PRIMARY KEY (note_id, pdf_id),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (pdf_id) REFERENCES pdfs(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_pdf_annotations_pdf ON pdf_annotations(pdf_id);
            CREATE INDEX IF NOT EXISTS idx_note_pdfs_note ON note_pdfs(note_id);
            CREATE INDEX IF NOT EXISTS idx_note_pdfs_pdf ON note_pdfs(pdf_id);
            ",
        )?;
        Ok(())
    }

    pub fn create_note(&self, note: &Note) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notes (id, title, content_hash, parent_id, path, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                note.id,
                note.title,
                "",
                note.parent_id,
                note.path,
                note.sort_order,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_note(&self, id: &str) -> SqlResult<Option<Note>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, '', parent_id, path, sort_order, created_at, updated_at FROM notes WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: String::new(),
                parent_id: row.get(3)?,
                path: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or_default(),
                updated_at: row.get::<_, String>(7)?.parse().unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_note_titles(&self) -> SqlResult<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title FROM notes ORDER BY title")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub fn list_notes(&self) -> SqlResult<Vec<Note>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, '', parent_id, path, sort_order, created_at, updated_at FROM notes ORDER BY sort_order, title",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: String::new(),
                parent_id: row.get(3)?,
                path: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or_default(),
                updated_at: row.get::<_, String>(7)?.parse().unwrap_or_default(),
            })
        })?;
        rows.collect()
    }

    pub fn update_note(&self, note: &Note) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET title = ?1, content_hash = ?2, parent_id = ?3, path = ?4, sort_order = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                note.title,
                "",
                note.parent_id,
                note.path,
                note.sort_order,
                note.updated_at.to_rfc3339(),
                note.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_note(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_children(&self, parent_id: &str) -> SqlResult<Vec<Note>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, '', parent_id, path, sort_order, created_at, updated_at FROM notes WHERE parent_id = ?1 ORDER BY sort_order, title",
        )?;
        let rows = stmt.query_map(params![parent_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                content: String::new(),
                parent_id: row.get(3)?,
                path: row.get(4)?,
                sort_order: row.get(5)?,
                created_at: row.get::<_, String>(6)?.parse().unwrap_or_default(),
                updated_at: row.get::<_, String>(7)?.parse().unwrap_or_default(),
            })
        })?;
        rows.collect()
    }

    pub fn get_breadcrumbs(&self, note_id: &str) -> SqlResult<Vec<Breadcrumb>> {
        let conn = self.conn.lock().unwrap();
        let mut crumbs = Vec::new();
        let mut current_id = Some(note_id.to_string());

        while let Some(cid) = current_id {
            let mut stmt = conn.prepare(
                "SELECT id, title, path FROM notes WHERE id = ?1",
            )?;
            let result: Option<(String, String, String)> = stmt
                .query_row(params![cid], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .ok();

            if let Some((id, title, path)) = result {
                crumbs.push(Breadcrumb { id, title, path });
                let mut parent_stmt = conn.prepare("SELECT parent_id FROM notes WHERE id = ?1")?;
                current_id = parent_stmt
                    .query_row(params![cid], |row| row.get(0))
                    .ok()
                    .flatten();
            } else {
                break;
            }
        }

        crumbs.reverse();
        Ok(crumbs)
    }

    pub fn build_tree(&self) -> SqlResult<Vec<NoteTreeNode>> {
        let notes = self.list_notes()?;
        let mut map: HashMap<Option<String>, Vec<Note>> = HashMap::new();
        for note in notes {
            map.entry(note.parent_id.clone()).or_default().push(note);
        }
        fn build_node(parent_id: Option<&str>, map: &HashMap<Option<String>, Vec<Note>>) -> Vec<NoteTreeNode> {
            let Some(children) = map.get(&parent_id.map(|s| s.to_string())) else {
                return Vec::new();
            };
            children.iter().map(|c| {
                NoteTreeNode {
                    id: c.id.clone(),
                    title: c.title.clone(),
                    path: c.path.clone(),
                    children: build_node(Some(&c.id), map),
                    created_at: c.created_at,
                }
            }).collect()
        }
        Ok(build_node(None, &map))
    }

    pub fn create_calendar_event(&self, event: &CalendarEvent, note_ids: &[String]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO calendar_events (id, date, title, description, completed, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![event.id, event.date, event.title, event.description, event.completed as i32, event.created_at],
        )?;
        for nid in note_ids {
            conn.execute(
                "INSERT OR IGNORE INTO event_notes (event_id, note_id) VALUES (?1, ?2)",
                params![event.id, nid],
            )?;
        }
        Ok(())
    }

    pub fn get_calendar_events(&self, year: i32, month: i32) -> SqlResult<Vec<CalendarEvent>> {
        let conn = self.conn.lock().unwrap();
        let prefix = format!("{}-{:02}%", year, month);
        let mut stmt = conn.prepare(
            "SELECT id, date, title, description, completed, created_at
             FROM calendar_events
             WHERE date LIKE ?1
             ORDER BY date, created_at",
        )?;
        let mut events: Vec<CalendarEvent> = stmt.query_map(params![prefix], |row| {
            Ok(CalendarEvent {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                note_ids: Vec::new(),
                created_at: row.get(5)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        for ev in &mut events {
            let mut nstmt = conn.prepare("SELECT note_id FROM event_notes WHERE event_id = ?1")?;
            let ids: Vec<String> = nstmt.query_map(params![ev.id], |row| row.get(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            ev.note_ids = ids;
        }
        Ok(events)
    }

    pub fn update_calendar_event(&self, id: &str, title: &str, description: &str, completed: bool) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE calendar_events SET title = ?1, description = ?2, completed = ?3 WHERE id = ?4",
            params![title, description, completed as i32, id],
        )?;
        Ok(())
    }

    pub fn delete_calendar_event(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM event_notes WHERE event_id = ?1", params![id])?;
        conn.execute("DELETE FROM calendar_events WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn link_note_to_event(&self, event_id: &str, note_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO event_notes (event_id, note_id) VALUES (?1, ?2)",
            params![event_id, note_id],
        )?;
        Ok(())
    }

    pub fn unlink_note_from_event(&self, event_id: &str, note_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM event_notes WHERE event_id = ?1 AND note_id = ?2",
            params![event_id, note_id],
        )?;
        Ok(())
    }

    pub fn get_events_for_note(&self, note_id: &str) -> SqlResult<Vec<CalendarEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ce.id, ce.date, ce.title, ce.description, ce.completed, ce.created_at
             FROM calendar_events ce
             JOIN event_notes en ON en.event_id = ce.id
             WHERE en.note_id = ?1
             ORDER BY ce.date",
        )?;
        let mut events: Vec<CalendarEvent> = stmt.query_map(params![note_id], |row| {
            Ok(CalendarEvent {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                note_ids: Vec::new(),
                created_at: row.get(5)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        for ev in &mut events {
            let mut nstmt = conn.prepare("SELECT note_id FROM event_notes WHERE event_id = ?1")?;
            let ids: Vec<String> = nstmt.query_map(params![ev.id], |row| row.get(0))?
                .collect::<SqlResult<Vec<_>>>()?;
            ev.note_ids = ids;
        }
        Ok(events)
    }

    fn build_node(&self, note: &Note, all: &[Note]) -> NoteTreeNode {
        let children: Vec<Note> = all.iter().filter(|n| n.parent_id.as_deref() == Some(&note.id)).cloned().collect();
        NoteTreeNode {
            id: note.id.clone(),
            title: note.title.clone(),
            path: note.path.clone(),
            children: children.iter().map(|c| self.build_node(c, all)).collect(),
            created_at: note.created_at,
        }
    }

    // ── PDF methods ──

    pub fn insert_pdf(&self, pdf: &PdfMetadata) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pdfs (id, title, file_path, page_count, text, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![pdf.id, pdf.title, pdf.file_path, pdf.page_count, pdf.text, pdf.created_at],
        )?;
        Ok(())
    }

    pub fn get_pdf(&self, pdf_id: &str) -> SqlResult<Option<PdfMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, page_count, text, created_at FROM pdfs WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![pdf_id], |row| {
            Ok(PdfMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                page_count: row.get(3)?,
                text: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(Ok(pdf)) => Ok(Some(pdf)),
            _ => Ok(None),
        }
    }

    pub fn get_pdf_text(&self, pdf_id: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT text FROM pdfs WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![pdf_id], |row| row.get::<_, Option<String>>(0))?;
        match rows.next() {
            Some(Ok(text)) => Ok(text),
            _ => Ok(None),
        }
    }

    pub fn get_all_pdfs(&self) -> SqlResult<Vec<PdfMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, file_path, page_count, text, created_at FROM pdfs ORDER BY created_at DESC",
        )?;
        let pdfs = stmt.query_map([], |row| {
            Ok(PdfMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                page_count: row.get(3)?,
                text: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        Ok(pdfs)
    }

    pub fn save_annotation(&self, input: &AnnotationInput) -> SqlResult<PdfAnnotation> {
        use uuid::Uuid;
        let conn = self.conn.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO pdf_annotations (id, pdf_id, page, type, x, y, width, height, color, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id, input.pdf_id, input.page, input.annotation_type,
                input.x, input.y, input.width, input.height,
                input.color, input.content, now
            ],
        )?;
        Ok(PdfAnnotation {
            id,
            pdf_id: input.pdf_id.clone(),
            page: input.page,
            annotation_type: input.annotation_type.clone(),
            x: input.x,
            y: input.y,
            width: input.width,
            height: input.height,
            color: input.color.clone(),
            content: input.content.clone(),
            created_at: now,
        })
    }

    pub fn get_annotations(&self, pdf_id: &str) -> SqlResult<Vec<PdfAnnotation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pdf_id, page, type, x, y, width, height, color, content, created_at
             FROM pdf_annotations WHERE pdf_id = ?1 ORDER BY page, created_at",
        )?;
        let annotations = stmt.query_map(params![pdf_id], |row| {
            Ok(PdfAnnotation {
                id: row.get(0)?,
                pdf_id: row.get(1)?,
                page: row.get(2)?,
                annotation_type: row.get(3)?,
                x: row.get(4)?,
                y: row.get(5)?,
                width: row.get(6)?,
                height: row.get(7)?,
                color: row.get(8)?,
                content: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        Ok(annotations)
    }

    pub fn delete_annotation(&self, annotation_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdf_annotations WHERE id = ?1", params![annotation_id])?;
        Ok(())
    }

    pub fn link_pdf_to_note(&self, note_id: &str, pdf_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO note_pdfs (note_id, pdf_id) VALUES (?1, ?2)",
            params![note_id, pdf_id],
        )?;
        Ok(())
    }

    pub fn unlink_pdf_from_note(&self, note_id: &str, pdf_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM note_pdfs WHERE note_id = ?1 AND pdf_id = ?2",
            params![note_id, pdf_id],
        )?;
        Ok(())
    }

    pub fn get_pdfs_for_note(&self, note_id: &str) -> SqlResult<Vec<PdfMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.file_path, p.page_count, p.text, p.created_at
             FROM pdfs p
             JOIN note_pdfs np ON np.pdf_id = p.id
             WHERE np.note_id = ?1
             ORDER BY p.created_at DESC",
        )?;
        let pdfs = stmt.query_map(params![note_id], |row| {
            Ok(PdfMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                page_count: row.get(3)?,
                text: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;
        Ok(pdfs)
    }

    pub fn delete_pdf(&self, pdf_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pdf_annotations WHERE pdf_id = ?1", params![pdf_id])?;
        conn.execute("DELETE FROM note_pdfs WHERE pdf_id = ?1", params![pdf_id])?;
        conn.execute("DELETE FROM pdfs WHERE id = ?1", params![pdf_id])?;
        Ok(())
    }
}
