//! Each player's best score and how far they got, kept between runs.
//!
//! Records are keyed by name, ignoring case, so "Lucía" and "lucía" share one. A blank
//! name falls back to its seat (`#1`, `#2`); names cannot contain `#`, so the two never
//! clash. The file is a few flat TOML tables, read line by line like the settings:
//!
//! ```toml
//! ["lucía"]
//! best_score = 2140
//! worlds_cleared = 3
//! ```

use std::collections::BTreeMap;

use super::rules::clean_name;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Record {
    pub best_score: u32,
    /// How many worlds the player has cleared, counting from the first. It is also the
    /// index of the furthest world they have reached.
    pub worlds_cleared: usize,
}

/// What a finished stretch of play improved on the saved record.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Improved {
    pub best_score: bool,
    pub worlds_cleared: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Records(BTreeMap<String, Record>);

/// Whether `score` beats the best saved before the match. Without a saved best to
/// compare with, nothing is claimed.
pub fn is_new_best(score: u32, best_before: Option<u32>) -> bool {
    best_before.is_some_and(|best| score > best)
}

/// The record key for the name typed into seat `seat` (0 or 1).
pub fn record_key(name: &str, seat: usize) -> String {
    let name = clean_name(name).to_lowercase();
    if name.is_empty() { format!("#{}", seat + 1) } else { name }
}

impl Records {
    pub fn get(&self, key: &str) -> Record {
        self.0.get(key).copied().unwrap_or_default()
    }

    /// Keeps the better of the saved record and this result, and says what improved.
    pub fn merge(&mut self, key: &str, score: u32, worlds_cleared: usize) -> Improved {
        let record = self.0.entry(key.to_string()).or_default();
        let improved = Improved {
            best_score: score > record.best_score,
            worlds_cleared: worlds_cleared > record.worlds_cleared,
        };
        record.best_score = record.best_score.max(score);
        record.worlds_cleared = record.worlds_cleared.max(worlds_cleared);
        improved
    }

    /// How many worlds this player may start from: the first, plus one for each world
    /// cleared, up to `total_worlds`.
    pub fn startable_worlds(&self, key: &str, total_worlds: usize) -> usize {
        self.get(key).worlds_cleared.saturating_add(1).min(total_worlds).max(1)
    }

    /// Reads records, skipping anything it does not understand, so a damaged file loses
    /// only the damaged part.
    pub fn parse(text: &str) -> Self {
        let mut records = BTreeMap::new();
        let mut current: Option<(String, Record)> = None;
        for line in text.lines().map(str::trim) {
            if let Some(key) = line.strip_prefix("[\"").and_then(|l| l.strip_suffix("\"]")) {
                if let Some((k, r)) = current.take() {
                    records.insert(k, r);
                }
                current = (!key.is_empty() && !key.contains(['"', '\\'])).then(|| (key.to_string(), Record::default()));
                continue;
            }
            let (Some((_, record)), Some((name, value))) = (current.as_mut(), line.split_once('=')) else { continue };
            match (name.trim(), value.trim()) {
                ("best_score", v) => record.best_score = v.parse().unwrap_or(record.best_score),
                ("worlds_cleared", v) => record.worlds_cleared = v.parse().unwrap_or(record.worlds_cleared),
                _ => {}
            }
        }
        if let Some((k, r)) = current {
            records.insert(k, r);
        }
        Self(records)
    }

    pub fn render(&self) -> String {
        let mut out = String::from("# numhop records\n");
        for (key, record) in &self.0 {
            out.push_str(&format!(
                "\n[\"{key}\"]\nbest_score = {}\nworlds_cleared = {}\n",
                record.best_score, record.worlds_cleared
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_differing_in_case_or_spacing_share_a_record() {
        assert_eq!(record_key("Lucía", 0), "lucía");
        assert_eq!(record_key("  LUCÍA ", 1), "lucía");
        assert_eq!(record_key("Ana   María", 0), "ana maría");
    }

    #[test]
    fn a_blank_name_uses_its_seat() {
        assert_eq!(record_key("", 0), "#1");
        assert_eq!(record_key("   ", 1), "#2");
        assert_ne!(record_key("#1", 0), "#1", "a typed name can never take a seat key");
    }

    #[test]
    fn merging_keeps_the_best_and_reports_improvements() {
        let mut records = Records::default();
        let first = records.merge("lucía", 900, 1);
        assert_eq!(first, Improved { best_score: true, worlds_cleared: true });
        let worse = records.merge("lucía", 400, 0);
        assert_eq!(worse, Improved::default());
        assert_eq!(records.get("lucía"), Record { best_score: 900, worlds_cleared: 1 });
        let further = records.merge("lucía", 900, 3);
        assert_eq!(further, Improved { best_score: false, worlds_cleared: true });
        assert_eq!(records.get("lucía"), Record { best_score: 900, worlds_cleared: 3 });
    }

    #[test]
    fn a_new_player_scoring_nothing_has_not_improved() {
        let mut records = Records::default();
        assert_eq!(records.merge("mateo", 0, 0), Improved::default());
    }

    #[test]
    fn startable_worlds_grow_with_progress_and_stop_at_the_last() {
        let mut records = Records::default();
        assert_eq!(records.startable_worlds("nobody", 5), 1, "everyone may start at world 1");
        records.merge("lucía", 0, 2);
        assert_eq!(records.startable_worlds("lucía", 5), 3);
        records.merge("lucía", 0, 5);
        assert_eq!(records.startable_worlds("lucía", 5), 5, "clearing everything unlocks all five");
    }

    #[test]
    fn records_round_trip_through_text() {
        let mut records = Records::default();
        records.merge("lucía", 2140, 3);
        records.merge("#2", 300, 0);
        assert_eq!(Records::parse(&records.render()), records);
    }

    #[test]
    fn damaged_text_keeps_what_it_can() {
        let text = "garbage\n[\"ana\"]\nbest_score = lots\nworlds_cleared = 2\n[\"bad\"key\"]\nbest_score = 5\n[\"bo\"]\nbest_score = 70\n";
        let records = Records::parse(text);
        assert_eq!(records.get("ana"), Record { best_score: 0, worlds_cleared: 2 });
        assert_eq!(records.get("bo"), Record { best_score: 70, worlds_cleared: 0 });
        assert_eq!(records.get("bad\"key"), Record::default(), "a key with a quote is dropped");
        assert_eq!(Records::parse(""), Records::default());
    }

    #[test]
    fn a_huge_world_count_is_held_to_the_last_world() {
        let records = Records::parse(&format!("[\"ana\"]\nworlds_cleared = {}\n", usize::MAX));
        assert_eq!(records.startable_worlds("ana", 5), 5);
    }

    #[test]
    fn a_new_best_must_beat_the_old_one() {
        assert!(is_new_best(901, Some(900)));
        assert!(!is_new_best(900, Some(900)), "a tie is not a new best");
        assert!(!is_new_best(0, Some(0)), "scoring nothing is never a best");
        assert!(is_new_best(20, Some(0)), "a first score counts");
        assert!(!is_new_best(5000, None), "no saved best to beat, no claim");
    }
}
