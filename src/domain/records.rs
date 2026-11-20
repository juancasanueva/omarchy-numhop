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
//! practice.times_tables = "2 4 0"
//! ```
//!
//! A practice line holds the topic's tier, recent misses and right answers in a row.

use std::collections::{BTreeMap, BTreeSet};

use super::math::{Question, Topic};
use super::practice::{self, Practice, RetryQueue, Skill};
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

#[derive(Debug, Clone, Default)]
pub struct Records {
    players: BTreeMap<String, Record>,
    /// What each player has been practising, by the same keys.
    practice: BTreeMap<String, Practice>,
    /// Topics answered in this run, by key. Only these keep this run's view when
    /// merging with a copy saved meanwhile; it is not saved.
    answered: BTreeSet<(String, Topic)>,
}

/// Two sets of records are equal when they would save the same file.
impl PartialEq for Records {
    fn eq(&self, other: &Self) -> bool {
        self.players == other.players && self.practice == other.practice
    }
}

impl Eq for Records {}

/// Whether `score` beats the best saved before the match. Without a saved best to
/// compare with, nothing is claimed.
pub fn is_new_best(score: u32, best_before: Option<u32>) -> bool {
    best_before.is_some_and(|best| score > best)
}

/// A record key as a name to show: each word capitalised again ("ana maría" becomes
/// "Ana María"). A seat key (`#1`) has no name; it gives the seat index instead.
pub fn display_name(key: &str) -> Result<String, usize> {
    if let Some(seat) = key.strip_prefix('#').and_then(|n| n.parse::<usize>().ok()) {
        return Err(seat.saturating_sub(1));
    }
    let mut out = String::with_capacity(key.len());
    let mut word_start = true;
    for c in key.chars() {
        if word_start {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
        word_start = matches!(c, ' ' | '-');
    }
    Ok(out)
}

/// The record key for the name typed into seat `seat` (0 or 1).
pub fn record_key(name: &str, seat: usize) -> String {
    let name = clean_name(name).to_lowercase();
    if name.is_empty() { format!("#{}", seat + 1) } else { name }
}

impl Records {
    /// What this player has been practising; nothing yet for a new player.
    pub fn practice(&self, key: &str) -> Practice {
        self.practice.get(key).cloned().unwrap_or_default()
    }

    /// Records this player's answer and remembers the topic as answered in this run.
    /// This is the only way practice changes, so `absorb` always knows what this run did.
    pub fn record_answer(&mut self, key: &str, retries: &mut RetryQueue, question: &Question, right: bool, was_retry: bool) {
        practice::record_answer(self.practice_mut(key), retries, question, right, was_retry);
        self.answered.insert((key.to_string(), question.topic));
    }

    /// Called once these records are on disk: what this run answered is saved, so from
    /// now on a newer copy saved by another window wins for those topics too, until
    /// this run answers them again.
    pub fn forget_answered(&mut self) {
        self.answered.clear();
    }

    /// Private on purpose: a change made here is not remembered as answered in this
    /// run, so `absorb` would let another copy overwrite it. Use `record_answer`.
    fn practice_mut(&mut self, key: &str) -> &mut Practice {
        self.practice.entry(key.to_string()).or_default()
    }

    pub fn get(&self, key: &str) -> Record {
        self.players.get(key).copied().unwrap_or_default()
    }

    /// Keeps the better of the saved record and this result, and says what improved.
    pub fn merge(&mut self, key: &str, score: u32, worlds_cleared: usize) -> Improved {
        let record = self.players.entry(key.to_string()).or_default();
        let improved = Improved {
            best_score: score > record.best_score,
            worlds_cleared: worlds_cleared > record.worlds_cleared,
        };
        record.best_score = record.best_score.max(score);
        record.worlds_cleared = record.worlds_cleared.max(worlds_cleared);
        improved
    }

    /// Takes in another set of records, keeping the better value of each, so progress
    /// saved meanwhile by another running copy of the game is not lost.
    pub fn absorb(&mut self, other: &Records) {
        for (key, theirs) in &other.players {
            let mine = self.players.entry(key.clone()).or_default();
            mine.best_score = mine.best_score.max(theirs.best_score);
            mine.worlds_cleared = mine.worlds_cleared.max(theirs.worlds_cleared);
        }
        // Practice has no "better" value: a topic answered in this run keeps this run's
        // view, and every other topic takes the other copy's, which may be newer than
        // what this run loaded at startup.
        for (key, theirs) in &other.practice {
            let mine = self.practice.entry(key.clone()).or_default();
            for (topic, skill) in &theirs.0 {
                if !self.answered.contains(&(key.clone(), *topic)) {
                    mine.0.insert(*topic, *skill);
                }
            }
        }
    }

    /// Players with any progress, best score first, then most worlds cleared.
    pub fn ranked(&self) -> Vec<(&str, Record)> {
        let mut list: Vec<(&str, Record)> = self
            .players
            .iter()
            .filter(|(_, r)| r.best_score > 0 || r.worlds_cleared > 0)
            .map(|(k, r)| (k.as_str(), *r))
            .collect();
        list.sort_by(|(ka, a), (kb, b)| {
            b.best_score.cmp(&a.best_score).then(b.worlds_cleared.cmp(&a.worlds_cleared)).then(ka.cmp(kb))
        });
        list
    }

    /// How many worlds this player may start from: the first, plus one for each world
    /// cleared, up to `total_worlds`.
    pub fn startable_worlds(&self, key: &str, total_worlds: usize) -> usize {
        self.get(key).worlds_cleared.saturating_add(1).min(total_worlds).max(1)
    }

    /// Reads records, skipping anything it does not understand, so a damaged file loses
    /// only the damaged part.
    pub fn parse(text: &str) -> Self {
        let mut records = Records::default();
        let mut current: Option<String> = None;
        for line in text.lines().map(str::trim) {
            if let Some(key) = line.strip_prefix("[\"").and_then(|l| l.strip_suffix("\"]")) {
                current = (!key.is_empty() && !key.contains(['"', '\\'])).then(|| key.to_string());
                continue;
            }
            let (Some(key), Some((name, value))) = (current.as_ref(), line.split_once('=')) else { continue };
            let (name, value) = (name.trim(), value.trim());
            if let Some(topic) = name.strip_prefix("practice.").and_then(Topic::from_id) {
                if let Some(skill) = Skill::parse(value) {
                    records.practice_mut(key).0.insert(topic, skill);
                }
                continue;
            }
            // A player entry exists once one of its fields is read; a header alone, or a
            // table with only practice, adds none.
            match name {
                "best_score" => {
                    let record = records.players.entry(key.clone()).or_default();
                    record.best_score = value.parse().unwrap_or(record.best_score);
                }
                "worlds_cleared" => {
                    let record = records.players.entry(key.clone()).or_default();
                    record.worlds_cleared = value.parse().unwrap_or(record.worlds_cleared);
                }
                _ => {}
            }
        }
        records
    }

    pub fn render(&self) -> String {
        let mut out = String::from("# numhop records\n");
        let keys: BTreeSet<&String> = self.players.keys().chain(self.practice.keys()).collect();
        for key in keys {
            out.push_str(&format!("\n[\"{key}\"]\n"));
            if let Some(record) = self.players.get(key) {
                out.push_str(&format!("best_score = {}\nworlds_cleared = {}\n", record.best_score, record.worlds_cleared));
            }
            for (topic, skill) in self.practice.get(key).map(|p| &p.0).into_iter().flatten() {
                out.push_str(&format!("practice.{} = \"{}\"\n", topic.id(), skill.render()));
            }
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
    fn absorbing_keeps_the_better_of_both_sides() {
        let mut mine = Records::default();
        mine.merge("lucía", 900, 1);
        mine.merge("ana", 50, 0);
        let mut theirs = Records::default();
        theirs.merge("lucía", 400, 3);
        theirs.merge("mateo", 120, 2);
        mine.absorb(&theirs);
        assert_eq!(mine.get("lucía"), Record { best_score: 900, worlds_cleared: 3 });
        assert_eq!(mine.get("mateo"), Record { best_score: 120, worlds_cleared: 2 });
        assert_eq!(mine.get("ana"), Record { best_score: 50, worlds_cleared: 0 });
    }

    #[test]
    fn ranking_puts_the_best_score_first_and_skips_empty_records() {
        let mut records = Records::default();
        records.merge("ana", 500, 1);
        records.merge("bo", 900, 0);
        records.merge("cy", 500, 3);
        records.merge("#2", 0, 0);
        let order: Vec<&str> = records.ranked().iter().map(|(k, _)| *k).collect();
        assert_eq!(order, ["bo", "cy", "ana"], "ties on score go to more worlds cleared");
    }

    #[test]
    fn keys_show_as_capitalised_names_or_seats() {
        assert_eq!(display_name("lucía"), Ok("Lucía".to_string()));
        assert_eq!(display_name("ana maría"), Ok("Ana María".to_string()));
        assert_eq!(display_name("jean-luc"), Ok("Jean-Luc".to_string()));
        assert_eq!(display_name("#1"), Err(0));
        assert_eq!(display_name("#2"), Err(1));
    }

    #[test]
    fn practice_round_trips_and_old_files_load_without_it() {
        let mut records = Records::default();
        records.merge("lucía", 900, 2);
        records.practice_mut("lucía").answered(Topic::TimesTables, 2, false);
        records.practice_mut("#2").answered(Topic::Gcd, 3, true);
        let text = records.render();
        assert!(text.contains("practice.times_tables = \"1 2 0\""), "{text}");
        assert_eq!(Records::parse(&text), records);

        let old = Records::parse("[\"ana\"]\nbest_score = 70\nworlds_cleared = 2\n");
        assert_eq!(old.get("ana"), Record { best_score: 70, worlds_cleared: 2 });
        assert_eq!(old.practice("ana"), Practice::default());
    }

    #[test]
    fn damaged_practice_lines_are_skipped() {
        let text = "[\"ana\"]\npractice.juggling = \"2 0 0\"\npractice.gcd = \"9 9 9\"\npractice.lcm = \"3 1 2\"\n";
        let practice = Records::parse(text).practice("ana");
        assert_eq!(practice.0.len(), 1, "{practice:?}");
        assert_eq!(practice.tier(Topic::Lcm, 1), 3);
    }

    #[test]
    fn absorbing_keeps_only_topics_answered_in_this_run() {
        use crate::domain::math::generate_topic;
        use crate::domain::rng::Rng;
        // This run loaded division (tier 2) and fractions-of (tier 2) at startup.
        let mut mine = Records::default();
        mine.practice_mut("ana").answered(Topic::Division, 2, true);
        mine.practice_mut("ana").answered(Topic::FractionOf, 2, true);
        // It then misses a fraction-of question itself.
        let question = generate_topic(Topic::FractionOf, 2, &mut Rng::new(1));
        mine.record_answer("ana", &mut RetryQueue::default(), &question, false, false);

        // Meanwhile another window saved newer division and fraction-of practice.
        let mut theirs = Records::default();
        theirs.practice_mut("ana").answered(Topic::Division, 4, true);
        theirs.practice_mut("ana").answered(Topic::FractionOf, 4, true);
        theirs.practice_mut("ana").answered(Topic::Decimals, 3, true);
        mine.absorb(&theirs);

        let practice = mine.practice("ana");
        assert_eq!(practice.tier(Topic::Division, 1), 4, "only loaded here, so the newer copy wins");
        assert_eq!(practice.tier(Topic::FractionOf, 1), 1, "answered here, so this run's view wins");
        assert_eq!(practice.tier(Topic::Decimals, 1), 3, "new to this run, taken from the other copy");
    }

    #[test]
    fn after_a_save_another_windows_newer_practice_wins_again() {
        use crate::domain::math::generate_topic;
        use crate::domain::rng::Rng;
        let question = generate_topic(Topic::Gcd, 2, &mut Rng::new(8));
        let mut mine = Records::default();
        mine.record_answer("ana", &mut RetryQueue::default(), &question, false, false);
        mine.forget_answered();
        // Another window practised GCD more after this one saved.
        let mut theirs = Records::default();
        theirs.practice_mut("ana").answered(Topic::Gcd, 4, true);
        mine.absorb(&theirs);
        assert_eq!(mine.practice("ana").tier(Topic::Gcd, 1), 4);
    }

    #[test]
    fn a_header_alone_adds_no_player_and_a_repeated_one_adds_to_it() {
        let records = Records::parse("[\"ana\"]\n[\"bo\"]\nbest_score = 5\n[\"bo\"]\nworlds_cleared = 2\n");
        assert!(records.ranked().iter().all(|(k, _)| *k != "ana"));
        assert_eq!(records.render().matches("[\"ana\"]").count(), 0, "nothing to save for ana");
        assert_eq!(records.get("bo"), Record { best_score: 5, worlds_cleared: 2 });
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
