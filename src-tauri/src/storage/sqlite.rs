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

            CREATE TABLE IF NOT EXISTS calendar_entries (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                title TEXT NOT NULL,
                note_id TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_calendar_date ON calendar_entries(date);
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

    pub fn create_calendar_entry(&self, entry: &CalendarEntry) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO calendar_entries (id, date, title, note_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry.id, entry.date, entry.title, entry.note_id, entry.created_at],
        )?;
        Ok(())
    }

    pub fn get_calendar_entries(&self, year: i32, month: i32) -> SqlResult<Vec<CalendarEntry>> {
        let conn = self.conn.lock().unwrap();
        let prefix = format!("{}-{:02}", year, month);
        let mut stmt = conn.prepare(
            "SELECT ce.id, ce.date, ce.title, ce.note_id, n.title, ce.created_at
             FROM calendar_entries ce
             LEFT JOIN notes n ON ce.note_id = n.id
             WHERE ce.date LIKE ?1
             ORDER BY ce.date, ce.created_at",
        )?;
        let rows = stmt.query_map(params![format!("{}%", prefix)], |row| {
            Ok(CalendarEntry {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                note_id: row.get(3)?,
                note_title: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_calendar_entry(&self, id: &str, title: &str, note_id: Option<&str>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE calendar_entries SET title = ?1, note_id = ?2 WHERE id = ?3",
            params![title, note_id, id],
        )?;
        Ok(())
    }

    pub fn delete_calendar_entry(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM calendar_entries WHERE id = ?1", params![id])?;
        Ok(())
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
}
