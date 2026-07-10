use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardInput {
    pub note_id: String,
    pub question: String,
    pub answer: String,
}
