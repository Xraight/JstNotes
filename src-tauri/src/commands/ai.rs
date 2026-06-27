//! AI study commands: flashcard generation, Feynman technique, SM-2 reviews.
//!
//! Two-generation strategy:
//!   Via API (Groq/OpenAI/OpenCode Go): sends note content + PDF context + calendar
//!   events to an LLM, receives a JSON array of question-answer pairs.
//!   Offline fallback: extracts questions from Markdown structure (headings,
//!   bold text, bullet points, sentences) — no API call needed.
//!
//! Feynman commands require an API key — they need a model to evaluate explanations.
//!
//! Settings are read from AppSettingsState (persisted with other app settings).

use tauri::State;
use std::sync::Mutex;

use crate::ai::client::{self, AiConfig, ChatMessage, ModelInfo};
use crate::ai::study::{sm2, StudyItem, StudyQuestion};
use crate::commands::settings::AppSettingsState;
use crate::storage::hybrid::HybridStorage;
use crate::models::AppSettings;

fn build_study_prompt(note_title: &str, content: &str, pdf_context: &str, calendar_context: &str) -> Vec<ChatMessage> {
    let system = format!(
        "You are a study assistant. Generate active recall questions from the user's note.\n\
         Return ONLY a valid JSON array: [{{\"question\": \"...\", \"answer\": \"...\"}}]\n\
         Questions must require the learner to actively retrieve information, not just recognize it.\n\
         Generate exactly 4-6 questions.\n\
         RESPOND IN THE SAME LANGUAGE AS THE NOTE CONTENT."
    );

    let user = format!(
        "Note title: {}\n\nNote content:\n{}\n{}\n{}\n\nGenerate 4-6 active recall questions.",
        note_title, content, pdf_context, calendar_context
    );

    vec![
        ChatMessage { role: "system".to_string(), content: system },
        ChatMessage { role: "user".to_string(), content: user },
    ]
}

fn generate_rules(content: &str) -> Vec<StudyQuestion> {
    let mut questions: Vec<StudyQuestion> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim();
            if heading.len() > 3 && heading.len() < 100 {
                questions.push(StudyQuestion {
                    question: format!("What is \"{}\"?", heading),
                    answer: heading.to_string(),
                });
            }
        }
    }

    let mut idx = 0;
    while let Some(start) = content[idx..].find("**") {
        let abs_start = idx + start + 2;
        if let Some(end) = content[abs_start..].find("**") {
            let term = &content[abs_start..abs_start + end];
            if term.len() > 3 && term.len() < 60 && !term.contains('\n') {
                questions.push(StudyQuestion {
                    question: format!("Define: {}", term),
                    answer: format!("Key term from the note: \"{}\"", term),
                });
            }
            idx = abs_start + end + 2;
        } else {
            idx = abs_start;
        }
        if questions.len() >= 8 { break; }
    }

    if questions.is_empty() {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                let point = trimmed.trim_start_matches(['-', '*', ' ']).trim();
                if point.len() > 10 && point.len() < 200 {
                    questions.push(StudyQuestion {
                        question: format!("Explain: {}", &point[..point.len().min(80)]),
                        answer: point.to_string(),
                    });
                }
                if questions.len() >= 8 { break; }
            }
        }
    }

    if questions.is_empty() {
        let sentences: Vec<&str> = content.split(&['.', '!', '?'][..])
            .map(|s| s.trim())
            .filter(|s| s.len() > 20 && s.len() < 300)
            .collect();
        for sentence in sentences.iter().take(4) {
            let words: Vec<&str> = sentence.split_whitespace().collect();
            if words.len() >= 5 {
                let blank_idx = words.len() / 2;
                let masked: String = words.iter().enumerate()
                    .map(|(i, w)| if i == blank_idx { "___" } else { w })
                    .collect::<Vec<_>>()
                    .join(" ");
                questions.push(StudyQuestion {
                    question: format!("Fill in the blank: {}", masked),
                    answer: words[blank_idx].to_string(),
                });
            }
        }
    }

    questions
}

fn extract_json(content: &str) -> Result<Vec<StudyQuestion>, String> {
    let start = content.find('[').unwrap_or(0);
    let end = content.rfind(']').unwrap_or(content.len());
    let json_str = &content[start..=end];
    serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse AI response: {}", e))
}

fn load_config(settings: &AppSettings) -> AiConfig {
    let endpoint = if !settings.ai_endpoint.is_empty() {
        settings.ai_endpoint.clone()
    } else {
        match settings.ai_provider.as_str() {
            "groq" => "https://api.groq.com/openai/v1".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "opencode" => "https://opencode.ai/zen/go/v1".to_string(),
            _ => "https://api.groq.com/openai/v1".to_string(),
        }
    };

    AiConfig {
        provider: settings.ai_provider.clone(),
        endpoint,
        api_key: settings.ai_api_key.clone(),
        model: settings.ai_model.clone(),
        enabled: settings.ai_enabled,
    }
}

#[tauri::command]
pub async fn test_ai_connection(
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<String, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    if config.api_key.is_empty() {
        return Err("No API key configured".to_string());
    }
    let msgs = vec![ChatMessage {
        role: "user".to_string(),
        content: "Say 'OK' if you receive this message.".to_string(),
    }];
    let resp = client::chat(msgs, &config, 32).await?;
    Ok(format!("Connected: {}", resp.trim()))
}

#[tauri::command]
pub async fn fetch_ai_models(
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<Vec<ModelInfo>, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    if config.api_key.is_empty() {
        return Err("No API key configured".to_string());
    }
    client::list_models(&config).await
}

#[tauri::command]
pub async fn generate_study_questions(
    note_id: String,
    storage: State<'_, HybridStorage>,
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<Vec<StudyItem>, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    let note = storage.get_note(&note_id)?.ok_or("Note not found")?;
    let content = storage.get_note_content(&note_id)?.unwrap_or_default();
    if content.is_empty() {
        return Err("Note has no content".to_string());
    }

    let pdf_refs = storage.db.get_pdf_references_for_note(&note_id)
        .map_err(|e| format!("{}", e))?;
    let events = storage.db.get_events_for_note(&note_id)
        .map_err(|e| format!("{}", e))?;

    let mut pdf_ctx = String::new();
    if !pdf_refs.is_empty() {
        pdf_ctx.push_str("Linked PDF pages: ");
        for r in &pdf_refs {
            pdf_ctx.push_str(&format!("p.{} {}, ", r.page, r.label));
        }
    }

    let mut cal_ctx = String::new();
    if !events.is_empty() {
        cal_ctx.push_str("Upcoming dates: ");
        for e in &events {
            if !e.completed {
                cal_ctx.push_str(&format!("{} ({}) ", e.title, e.date));
            }
        }
    }

    let questions: Vec<StudyQuestion>;
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };

    if !config.api_key.is_empty() && config.enabled {
        let note_text: String = if content.len() > 2000 {
            format!("{}… ({} chars total)", &content[..2000], content.len())
        } else {
            content.clone()
        };
        let msgs = build_study_prompt(&note.title, &note_text, &pdf_ctx, &cal_ctx);
        let resp = client::chat(msgs, &config, 1024).await?;
        questions = extract_json(&resp)?;
    } else {
        questions = generate_rules(&content);
    }

    let now = chrono::Utc::now();
    let today = now.date_naive().to_string();
    let mut items = Vec::new();
    let source_page = pdf_refs.first().map(|r| r.page);

    for q in &questions {
        let id = uuid::Uuid::new_v4().to_string();
        items.push(StudyItem {
            id,
            note_id: note_id.clone(),
            question: q.question.clone(),
            answer: q.answer.clone(),
            created_at: now.to_rfc3339(),
            next_review: today.clone(),
            interval_days: 1,
            ease_factor: 2.5,
            repetitions: 0,
            reviewed_at: None,
            source_page: None,
            days_until_event: None,
        });
    }

    if items.is_empty() {
        return Err("No questions generated".to_string());
    }

    storage.save_study_items(&items)?;
    Ok(items)
}

/**
 * Generates "deep" questions using elaborative interrogation.
 * Instead of simple recall ("What is X?"), these ask "Why does X lead to Y?",
 * "How does X relate to Z?", forcing the learner to connect concepts.
 * Uses the same cloud/fallback dual path as generate_study_questions.
 */
#[tauri::command]
pub async fn generate_elaboration_questions(
    note_id: String,
    storage: State<'_, HybridStorage>,
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<Vec<StudyItem>, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    let note = storage.get_note(&note_id)?.ok_or("Note not found")?;
    let content = storage.get_note_content(&note_id)?.unwrap_or_default();
    if content.is_empty() {
        return Err("Note has no content".to_string());
    }

    let pdf_refs = storage.db.get_pdf_references_for_note(&note_id).map_err(|e| format!("{}", e))?;
    let mut pdf_ctx = String::new();
    if !pdf_refs.is_empty() {
        pdf_ctx.push_str("Linked PDF pages: ");
        for r in &pdf_refs { pdf_ctx.push_str(&format!("p.{} {}, ", r.page, r.label)); }
    }

    let questions: Vec<StudyQuestion>;
    if !config.api_key.is_empty() && config.enabled {
        let note_text: String = if content.len() > 2000 {
            format!("{}… ({} chars total)", &content[..2000], content.len())
        } else { content.clone() };

        let system = "You are a study coach using elaborative interrogation. Generate questions that force the learner to explain WHY and HOW concepts relate — not just recall facts. Return ONLY a JSON array: [{\"question\": \"...\", \"answer\": \"...\"}]. Generate 3-5 questions. RESPOND IN THE SAME LANGUAGE AS THE NOTE.";
        let user = format!("Note title: {}\n\nContent:\n{}\n{}\nGenerate 3-5 deep 'why/how' questions.", note.title, note_text, pdf_ctx);

        let msgs = vec![
            ChatMessage { role: "system".to_string(), content: system.to_string() },
            ChatMessage { role: "user".to_string(), content: user },
        ];
        let resp = client::chat(msgs, &config, 1024).await?;
        questions = {
            let start = resp.find('[').unwrap_or(0);
            let end = resp.rfind(']').unwrap_or(resp.len());
            serde_json::from_str(&resp[start..=end]).map_err(|e| format!("Parse error: {}", e))?
        };
    } else {
        // Rules-based deep questions: find sentences with cause/effect markers
        let mut qs = Vec::new();
        for sentence in content.split(&['.', '!', '?'][..]).map(|s| s.trim()).filter(|s| s.len() > 30 && s.len() < 400) {
            let lower = sentence.to_lowercase();
            if lower.contains("because") || lower.contains(" causes") || lower.contains(" leads to") || lower.contains(" since") || lower.contains(" therefore") || lower.contains(" thus") || lower.contains(" as a result") {
                qs.push(StudyQuestion {
                    question: format!("Why does this happen: \"{}\"", sentence),
                    answer: sentence.to_string(),
                });
                if qs.len() >= 5 { break; }
            }
        }
        if qs.is_empty() {
            return Err("Could not extract cause/effect patterns. Try adding more explanatory content to the note.".to_string());
        }
        questions = qs;
    }

    let now = chrono::Utc::now();
    let today = now.date_naive().to_string();
    let source_page = pdf_refs.first().map(|r| r.page);
    let mut items = Vec::new();
    for q in &questions {
        items.push(StudyItem {
            id: uuid::Uuid::new_v4().to_string(),
            note_id: note_id.clone(),
            question: q.question.clone(),
            answer: q.answer.clone(),
            created_at: now.to_rfc3339(),
            next_review: today.clone(),
            interval_days: 1, ease_factor: 2.5, repetitions: 0,
            reviewed_at: None,
            days_until_event: None, source_page,
        });
    }
    if items.is_empty() { return Err("No questions generated".to_string()); }
    storage.save_study_items(&items)?;
    Ok(items)
}

/**
 * Generates a concrete example or analogy for a concept in the note.
 * Uses the "concrete examples" learning strategy: abstract concepts become
 * memorable through relatable analogies.
 * Returns a single string (the example), not a StudyItem array like the
 * other generators — this is a one-time reference, not a spaced repetition card.
 */
#[tauri::command]
pub async fn generate_concrete_example(
    note_id: String,
    storage: State<'_, HybridStorage>,
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<String, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    let note = storage.get_note(&note_id)?.ok_or("Note not found")?;
    let content = storage.get_note_content(&note_id)?.unwrap_or_default();
    if content.is_empty() { return Err("Note has no content".to_string()); }

    let note_text: String = if content.len() > 1500 {
        format!("{}…", &content[..1500])
    } else { content.clone() };

    if !config.api_key.is_empty() && config.enabled {
        let msgs = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a study coach. Take an abstract concept from the note and explain it with a concrete, memorable analogy or real-world example. Keep it under 150 words. RESPOND IN THE SAME LANGUAGE AS THE NOTE.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Note \"{}\":\n\n{}\n\nGenerate a concrete example or analogy for one key concept.", note.title, note_text),
            },
        ];
        client::chat(msgs, &config, 256).await
    } else {
        // Fallback: find definitions/sentences that define something
        for line in content.lines() {
            let t = line.trim();
            if t.len() > 30 && t.len() < 300 && (t.contains(" is ") || t.contains(" means ") || t.contains(" refers to ")) {
                return Ok(format!("Think of it like this: imagine a real-world scenario where {} This is similar to how everyday situations work when you break them down to their core principles.", t[..80.min(t.len())].to_string()));
            }
        }
        Err("No definable concepts found. Add more explanatory content to the note.".to_string())
    }
}

/**
 * Returns study items due for review, sorted by calendar priority.
 * Items linked to notes with upcoming events (exams, deadlines) appear first.
 * Within the same priority group, items are sorted by next_review date.
 */
#[tauri::command]
pub fn get_due_reviews(
    storage: State<'_, HybridStorage>,
) -> Result<Vec<StudyItem>, String> {
    let mut items = storage.get_due_study_items()?;
    let today = chrono::Utc::now().date_naive();

    for item in &mut items {
        let events = storage.db.get_events_for_note(&item.note_id)
            .map_err(|e| format!("{}", e))?;
        if events.is_empty() { continue; }

        let mut closest: Option<i32> = None;
        for e in &events {
            if e.completed { continue; }
            if let Ok(ev_date) = chrono::NaiveDate::parse_from_str(&e.date, "%Y-%m-%d") {
                let days = (ev_date - today).num_days() as i32;
                if days >= 0 && (closest.is_none() || days < closest.unwrap()) {
                    closest = Some(days);
                }
            }
        }
        item.days_until_event = closest;
    }

    items.sort_by(|a, b| {
        match (a.days_until_event, b.days_until_event) {
            (Some(da), Some(db)) => da.cmp(&db),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.next_review.cmp(&b.next_review),
        }
    });

    Ok(items)
}

#[tauri::command]
pub fn rate_review(
    item_id: String,
    quality: i32,
    storage: State<'_, HybridStorage>,
) -> Result<(), String> {
    let due = storage.get_due_study_items()?;
    let item = due.into_iter().find(|i| i.id == item_id)
        .ok_or("Study item not found")?;
    let (interval, ef, reps) = sm2(quality, item.interval_days, item.ease_factor, item.repetitions);
    let next = (chrono::Utc::now() + chrono::Duration::days(interval as i64)).date_naive().to_string();
    storage.update_study_item_review(&item_id, interval, ef, reps, &next)?;
    storage.db.log_study_review(&item_id, quality).map_err(|e| format!("{}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_study_stats(
    storage: State<'_, HybridStorage>,
) -> Result<crate::storage::sqlite::StudyStats, String> {
    storage.db.get_study_stats().map_err(|e| format!("{}", e))
}

#[tauri::command]
pub fn search_notes(
    query: String,
    storage: State<'_, HybridStorage>,
) -> Result<Vec<crate::storage::sqlite::SearchResult>, String> {
    storage.db.search_notes(&query).map_err(|e| format!("{}", e))
}

#[tauri::command]
pub fn rebuild_fts(
    storage: State<'_, HybridStorage>,
) -> Result<(), String> {
    storage.db.rebuild_fts().map_err(|e| format!("{}", e))
}

#[tauri::command]
pub async fn generate_feynman_prompt(
    note_id: String,
    storage: State<'_, HybridStorage>,
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<String, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    if config.api_key.is_empty() || !config.enabled {
        return Err("Configure AI API key in Settings first".to_string());
    }

    let note = storage.get_note(&note_id)?.ok_or("Note not found")?;
    let content = storage.get_note_content(&note_id)?.unwrap_or_default();
    if content.is_empty() {
        return Err("Note has no content".to_string());
    }

    let note_text: String = if content.len() > 1000 {
        format!("{}…", &content[..1000])
    } else {
        content.clone()
    };

    let pdf_refs = storage.db.get_pdf_references_for_note(&note_id)
        .map_err(|e| format!("{}", e))?;
    let mut pdf_ctx = String::new();
    if !pdf_refs.is_empty() {
        pdf_ctx.push_str("\nLinked PDF pages: ");
        for r in &pdf_refs {
            pdf_ctx.push_str(&format!("p.{} {}, ", r.page, r.label));
        }
    }

    let msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a study coach. Write a single challenge question asking the learner to explain a key concept in their own words. Respond in the SAME LANGUAGE as the note content. Output ONLY the question.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
            "Note title: {}\n\nContent:\n{}{}\n\nWrite a Feynman-style challenge for this note.",
            note.title, note_text, pdf_ctx
        ),
        },
    ];

    client::chat(msgs, &config, 128).await
}

#[tauri::command]
pub async fn evaluate_feynman(
    note_id: String,
    user_explanation: String,
    storage: State<'_, HybridStorage>,
    settings_state: State<'_, crate::commands::settings::AppSettingsState>,
) -> Result<String, String> {
    let config = {
        let s = settings_state.settings.lock().map_err(|e| format!("{}", e))?;
        load_config(&s)
    };
    if config.api_key.is_empty() || !config.enabled {
        return Err("Configure AI API key in Settings first".to_string());
    }

    let note = storage.get_note(&note_id)?.ok_or("Note not found")?;
    let content = storage.get_note_content(&note_id)?.unwrap_or_default();
    let note_text: String = if content.len() > 800 {
        format!("{}…", &content[..800])
    } else {
        content.clone()
    };

    let pdf_refs = storage.db.get_pdf_references_for_note(&note_id)
        .map_err(|e| format!("{}", e))?;
    let mut pdf_ctx = String::new();
    if !pdf_refs.is_empty() {
        pdf_ctx.push_str("\nLinked PDF pages: ");
        for r in &pdf_refs {
            pdf_ctx.push_str(&format!("p.{} {}, ", r.page, r.label));
        }
    }

    let msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a supportive study coach using the Feynman technique. Give the user conversational feedback in 2-3 short paragraphs. Start by acknowledging what they understood correctly. Then gently suggest 1-2 improvements. End with encouragement. RESPOND IN THE SAME LANGUAGE AS THE USER'S EXPLANATION. Never use bullet points, numbered lists, or section headers.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
            "I'm practicing the Feynman technique. Here's the original note content:\n\n{}{}\n\nHere's my explanation:\n\n{}\n\nGive me feedback on my explanation.",
            note_text, pdf_ctx, user_explanation
        ),
        },
    ];

    client::chat(msgs, &config, 256).await
}

#[tauri::command]
pub fn get_study_items(
    note_id: String,
    storage: State<'_, HybridStorage>,
) -> Result<Vec<StudyItem>, String> {
    storage.get_study_items_for_note(&note_id)
}
