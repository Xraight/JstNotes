use std::fs;
use std::path::Path;

pub struct MarkdownStorage;

impl MarkdownStorage {
    pub fn save_note(path: &str, title: &str, content: &str) -> std::io::Result<()> {
        let dir = Path::new(path).parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;

        let md_content = format!("# {}\n\n{}", title, content);
        fs::write(path, md_content.as_bytes())?;
        Ok(())
    }

    pub fn read_note(path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }

    pub fn delete_note(path: &str) -> std::io::Result<()> {
        if Path::new(path).exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn note_exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    pub fn get_notes_dir(base_dir: &str) -> String {
        let dir = Path::new(base_dir).join("notes");
        fs::create_dir_all(&dir).ok();
        dir.to_string_lossy().to_string()
    }

    pub fn get_md_path(notes_dir: &str, note_id: &str) -> String {
        format!("{}/{}.md", notes_dir, note_id)
    }
}
