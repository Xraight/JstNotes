use std::panic;
use lopdf::Document;
use crate::models::PdfMetadata;

pub struct PdfExtractor;

impl PdfExtractor {
    pub fn new() -> Self {
        PdfExtractor
    }

    pub fn extract_metadata(&self, file_path: &str) -> Result<PdfMetadata, String> {
        let doc = Document::load(file_path).map_err(|e| format!("Failed to load PDF: {}", e))?;

        let pages = doc.get_pages();
        let page_count = pages.len() as i32;

        let title = doc.trailer.get(b"Info").ok().and_then(|info| {
            let info = info.as_reference().ok()?;
            let info_dict = doc.get_object(info).ok()?;
            let info_dict = info_dict.as_dict().ok()?;
            info_dict.get(b"Title").ok().and_then(|t| t.as_string().ok().map(|s| s.to_string()))
        });

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        Ok(PdfMetadata {
            id,
            title,
            file_path: file_path.to_string(),
            page_count,
            text: None,
            created_at: now,
        })
    }

    pub fn extract_text(&self, file_path: &str) -> Result<String, String> {
        let result = panic::catch_unwind(|| {
            pdf_extract::extract_text(file_path)
        });

        match result {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(format!("Failed to extract PDF text: {}", e)),
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic during PDF text extraction".to_string()
                };
                Err(format!("PDF text extraction panicked: {}", msg))
            }
        }
    }

    pub fn import(&self, file_path: &str) -> Result<PdfMetadata, String> {
        self.extract_metadata(file_path)
    }
}
