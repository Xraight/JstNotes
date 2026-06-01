#[tauri::command]
pub fn generate_flashcards(_note_content: String) -> Result<Vec<FlashcardStub>, String> {
    Ok(vec![])
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FlashcardStub {
    pub question: String,
    pub answer: String,
}
