//! Spaced repetition system using the SM-2 algorithm.
//!
//! SM-2 (SuperMemo 2) is a scientifically-backed algorithm for scheduling reviews.
//! After each review, the user rates their recall quality (0-5), and the algorithm
//! computes a new interval, ease factor, and repetition count.
//!
//!  - Quality < 3: item is reset (forgotten, needs frequent review)
//!  - Quality >= 3: interval grows exponentially based on ease factor
//!  - Ease factor starts at 2.5 and adjusts based on rating quality
//!  - Minimum ease factor is 1.3 (hardest items get reviewed more often)

use serde::{Deserialize, Serialize};

/// A single study card with SM-2 scheduling metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyItem {
    pub id: String,
    pub note_id: String,
    pub question: String,
    pub answer: String,
    pub created_at: String,
    /// Review schedule. Compared against today's date to find due items.
    pub next_review: String,
    /// Days between reviews. Grows with successful repetitions.
    pub interval_days: i32,
    /// Starts at 2.5. Adjusted per review: higher = easier, lower = harder.
    /// Minimum 1.3.
    pub ease_factor: f64,
    /// Number of consecutive successful reviews (quality >= 3).
    pub repetitions: i32,
    pub reviewed_at: Option<String>,
    /// Optional PDF page this question was generated from.
    pub source_page: Option<i32>,
}

/// A question-answer pair before being saved as a StudyItem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyQuestion {
    pub question: String,
    pub answer: String,
}

/// SM-2 algorithm implementation.
///
/// Returns (new_interval_days, new_ease_factor, new_repetitions).
///
/// # Parameters
/// - `quality`: User's self-rating (0 = complete blackout, 5 = perfect recall)
/// - `interval`: Current interval between reviews
/// - `ease_factor`: Current ease factor (2.5 default)
/// - `repetitions`: Current consecutive successful reviews
pub fn sm2(quality: i32, interval: i32, ease_factor: f64, repetitions: i32) -> (i32, f64, i32) {
    let q = quality.clamp(0, 5);

    if q < 3 {
        return (1, ease_factor.max(1.3), 0);
    }

    let new_reps = repetitions + 1;
    let new_interval = match new_reps {
        0 | 1 => 1,
        2 => 6,
        _ => ((interval as f64) * ease_factor).round() as i32,
    };

    let new_ef = ease_factor + 0.1 - ((5 - q) as f64) * (0.08 + (5 - q) as f64 * 0.02);
    let new_ef = new_ef.max(1.3);

    (new_interval, new_ef, new_reps)
}
