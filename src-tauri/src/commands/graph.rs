use serde::{Deserialize, Serialize};
use tauri::State;
use crate::storage::hybrid::HybridStorage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub child_count: usize,
    pub link_count: usize,
    pub has_pdf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub link_type: String, // "parent", "pdf", "mention"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[tauri::command]
pub fn get_graph_data(
    storage: State<'_, HybridStorage>,
) -> Result<GraphData, String> {
    let notes = storage.list_notes().map_err(|e| format!("{}", e))?;
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    let mut child_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut link_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Count children per parent
    for note in &notes {
        if let Some(ref pid) = note.parent_id {
            *child_counts.entry(pid.clone()).or_default() += 1;
        }
    }

    // Check PDF references
    let mut note_has_pdf = std::collections::HashSet::new();
    for note in &notes {
        if let Ok(refs) = storage.db.get_pdf_references_for_note(&note.id) {
            if !refs.is_empty() {
                note_has_pdf.insert(note.id.clone());
            }
        }
    }

    // Build nodes
    for note in &notes {
        let child_count = child_counts.get(&note.id).copied().unwrap_or(0);
        let has_pdf = note_has_pdf.contains(&note.id);
        nodes.push(GraphNode {
            id: note.id.clone(),
            title: note.title.clone(),
            parent_id: note.parent_id.clone(),
            child_count,
            link_count: 0, // computed below
            has_pdf,
        });
    }

    // Build links: parent relationships
    for note in &notes {
        if let Some(ref pid) = note.parent_id {
            links.push(GraphLink {
                source: pid.clone(),
                target: note.id.clone(),
                link_type: "parent".to_string(),
            });
            *link_counts.entry(pid.clone()).or_default() += 1;
            *link_counts.entry(note.id.clone()).or_default() += 1;
        }
    }

    // Build links: mentions
    let mention_re = regex::Regex::new(r"@([^\s@]+)").unwrap();
    let mut seen_mentions = std::collections::HashSet::new();
    for note in &notes {
        if let Ok(Some(content)) = storage.get_note_content(&note.id) {
            for cap in mention_re.captures_iter(&content) {
                let mentioned_title = &cap[1];
                if let Some(target) = notes.iter().find(|n| n.title.eq_ignore_ascii_case(mentioned_title)) {
                    let key = (note.id.clone(), target.id.clone());
                    if seen_mentions.insert(key.clone()) {
                        links.push(GraphLink {
                            source: note.id.clone(),
                            target: target.id.clone(),
                            link_type: "mention".to_string(),
                        });
                        *link_counts.entry(note.id.clone()).or_default() += 1;
                        *link_counts.entry(target.id.clone()).or_default() += 1;
                    }
                }
            }
        }
    }

    // Build links: PDF references (note → PDF note)
    for note in &notes {
        if let Ok(refs) = storage.db.get_pdf_references_for_note(&note.id) {
            for r in refs {
                if let Ok(Some(linked_note)) = storage.get_note(&r.note_id) {
                    let key = (note.id.clone(), linked_note.id.clone());
                    if seen_mentions.insert(key) {
                        links.push(GraphLink {
                            source: note.id.clone(),
                            target: linked_note.id.clone(),
                            link_type: "pdf".to_string(),
                        });
                        *link_counts.entry(note.id.clone()).or_default() += 1;
                        *link_counts.entry(linked_note.id.clone()).or_default() += 1;
                    }
                }
            }
        }
    }

    // Fill link_counts into nodes
    for node in &mut nodes {
        node.link_count = link_counts.get(&node.id).copied().unwrap_or(0);
    }

    Ok(GraphData { nodes, links })
}
