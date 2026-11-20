//! Practice that follows each player: topics they miss come up more often, missed
//! questions come back a little later, and each topic's difficulty follows their
//! answers instead of only how far into the match they are.

use std::collections::{BTreeMap, VecDeque};

use super::math::{AgeLevel, MAX_TIER, MIN_TIER, Question, Topic, clamp_tier, generate_topic};
use super::rng::Rng;

/// The most `misses` can grow to, so one bad day cannot bury the other topics.
const MAX_MISSES: u8 = 6;
/// What one wrong answer (or running out of time) adds to `misses`.
const MISS_WEIGHT: u8 = 2;
/// Right answers in a row that raise a topic's tier.
const STREAK_TO_LEVEL_UP: u8 = 3;
/// Other questions asked before a missed one comes back.
const RETRY_GAP: u8 = 2;

/// How a player is doing in one topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skill {
    /// The difficulty tier this topic is asked at, `MIN_TIER..=MAX_TIER`.
    pub tier: u8,
    /// Recent mistakes; each one makes the topic come up more often.
    pub misses: u8,
    /// Right answers in a row since the last tier change or mistake.
    pub streak: u8,
}

impl Skill {
    fn starting_at(tier: u8) -> Self {
        Self { tier: clamp_tier(tier), misses: 0, streak: 0 }
    }

    fn answered(&mut self, right: bool) {
        if right {
            self.misses = self.misses.saturating_sub(1);
            self.streak += 1;
            if self.streak >= STREAK_TO_LEVEL_UP {
                self.tier = (self.tier + 1).min(MAX_TIER);
                self.streak = 0;
            }
        } else {
            self.misses = (self.misses + MISS_WEIGHT).min(MAX_MISSES);
            self.streak = 0;
            self.tier = self.tier.saturating_sub(1).max(MIN_TIER);
        }
    }

    /// The saved form: `"tier misses streak"`.
    pub fn render(&self) -> String {
        format!("{} {} {}", self.tier, self.misses, self.streak)
    }

    /// Reads the saved form, refusing anything out of range.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().trim_matches('"').split_whitespace().map(str::parse::<u8>);
        let (Some(Ok(tier)), Some(Ok(misses)), Some(Ok(streak)), None) = (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let valid = (MIN_TIER..=MAX_TIER).contains(&tier) && misses <= MAX_MISSES && streak < STREAK_TO_LEVEL_UP;
        valid.then_some(Self { tier, misses, streak })
    }

    /// How likely this topic is to be picked, relative to the others.
    fn weight(&self) -> u64 {
        1 + self.misses as u64
    }
}

/// One player's skills, by topic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Practice(pub BTreeMap<Topic, Skill>);

impl Practice {
    /// Records an answer to a question on `topic` asked at `tier`. A topic answered for
    /// the first time starts at the tier it was asked at.
    pub fn answered(&mut self, topic: Topic, tier: u8, right: bool) {
        self.0.entry(topic).or_insert_with(|| Skill::starting_at(tier)).answered(right);
    }

    /// The tier to ask `topic` at: its own once the player has answered it, otherwise
    /// the tier the match's progress suggests.
    pub fn tier(&self, topic: Topic, progress_tier: u8) -> u8 {
        self.0.get(&topic).map_or(clamp_tier(progress_tier), |s| s.tier)
    }

    /// Picks a topic for `age`, favouring the ones this player has been missing. A topic
    /// never answered weighs the same as one answered without mistakes.
    pub fn pick_topic(&self, age: AgeLevel, rng: &mut Rng) -> Topic {
        let topics = Topic::for_age(age);
        let weight = |t: &Topic| self.0.get(t).map_or(1, Skill::weight);
        let total: u64 = topics.iter().map(weight).sum();
        let mut roll = rng.next_u64() % total;
        for topic in topics {
            let w = weight(topic);
            if roll < w {
                return *topic;
            }
            roll -= w;
        }
        topics[topics.len() - 1]
    }
}

/// Missed questions waiting to be asked again.
#[derive(Debug, Clone, Default)]
pub struct RetryQueue(VecDeque<(Question, u8)>);

impl RetryQueue {
    pub fn push(&mut self, question: Question) {
        self.0.push_back((question, 0));
    }

    /// Call once for every question about to be asked. Returns a missed question once
    /// `RETRY_GAP` others have been asked since, with its options in a new order.
    pub fn next(&mut self, rng: &mut Rng) -> Option<Question> {
        let due = self.0.front().is_some_and(|(_, waited)| *waited >= RETRY_GAP);
        let question = if due { self.0.pop_front().map(|(q, _)| q) } else { None };
        for (_, waited) in &mut self.0 {
            *waited += 1;
        }
        question.map(|q| reshuffled(q, rng))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The next question for a player: a missed one that is due again, otherwise a new one
/// on a weighted topic at that topic's tier. The flag is true for a retry.
pub fn next_question(
    practice: &Practice,
    retries: &mut RetryQueue,
    age: AgeLevel,
    progress_tier: u8,
    rng: &mut Rng,
) -> (Question, bool) {
    if let Some(again) = retries.next(rng) {
        return (again, true);
    }
    let topic = practice.pick_topic(age, rng);
    (generate_topic(topic, practice.tier(topic, progress_tier), rng), false)
}

/// Records an answer. A question missed for the first time is queued to be asked
/// again; one missed again on its retry is not, so it cannot keep coming back.
pub fn record_answer(practice: &mut Practice, retries: &mut RetryQueue, question: &Question, right: bool, was_retry: bool) {
    practice.answered(question.topic, question.tier, right);
    if !right && !was_retry {
        retries.push(question.clone());
    }
}

/// The same question with its options in a new order, so the answer is not
/// remembered by where it was.
fn reshuffled(mut question: Question, rng: &mut Rng) -> Question {
    let right = question.correct_value();
    rng.shuffle(&mut question.options);
    question.correct = question.options.iter().position(|o| *o == right).unwrap_or(question.correct);
    question
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::math::generate_topic;

    #[test]
    fn three_right_in_a_row_raise_the_tier_and_a_miss_lowers_it() {
        let mut p = Practice::default();
        for _ in 0..3 {
            p.answered(Topic::TimesTables, 2, true);
        }
        assert_eq!(p.tier(Topic::TimesTables, 1), 3);
        p.answered(Topic::TimesTables, 3, false);
        assert_eq!(p.0[&Topic::TimesTables], Skill { tier: 2, misses: 2, streak: 0 });
    }

    #[test]
    fn tiers_stay_between_one_and_four() {
        let mut p = Practice::default();
        for _ in 0..5 {
            p.answered(Topic::Addition, 1, false);
        }
        assert_eq!(p.tier(Topic::Addition, 1), MIN_TIER);
        assert_eq!(p.0[&Topic::Addition].misses, MAX_MISSES, "misses are capped");
        for _ in 0..30 {
            p.answered(Topic::Addition, 1, true);
        }
        assert_eq!(p.tier(Topic::Addition, 1), MAX_TIER);
        assert_eq!(p.0[&Topic::Addition].misses, 0);
    }

    #[test]
    fn an_unanswered_topic_follows_the_match_progress() {
        let p = Practice::default();
        assert_eq!(p.tier(Topic::Gcd, 3), 3);
        assert_eq!(p.tier(Topic::Gcd, 9), MAX_TIER);
    }

    #[test]
    fn missed_topics_come_up_more_but_others_still_do() {
        let mut p = Practice::default();
        p.answered(Topic::SubtractionBorrow, 1, false);
        p.answered(Topic::SubtractionBorrow, 1, false);
        let mut rng = Rng::new(7);
        let mut counts = BTreeMap::new();
        for _ in 0..3000 {
            *counts.entry(p.pick_topic(AgeLevel::Eight, &mut rng)).or_insert(0) += 1;
        }
        // Weights 5 : 1 : 1, so borrowing gets about 5/7 of the picks.
        let borrow = counts[&Topic::SubtractionBorrow];
        assert!((1900..2400).contains(&borrow), "{counts:?}");
        assert!(counts[&Topic::TimesTables] > 250 && counts[&Topic::Multiplication] > 250, "{counts:?}");
        assert_eq!(counts.len(), 3, "only the age group's topics: {counts:?}");
    }

    #[test]
    fn a_missed_question_returns_after_two_others_with_new_option_order() {
        let mut rng = Rng::new(3);
        let missed = generate_topic(Topic::Addition, 2, &mut rng);
        let mut queue = RetryQueue::default();
        queue.push(missed.clone());
        assert!(queue.next(&mut rng).is_none(), "not right after the miss");
        assert!(queue.next(&mut rng).is_none(), "nor one question later");
        let again = queue.next(&mut rng).expect("back after two others");
        assert_eq!(again.correct_value(), missed.correct_value());
        let mut before = missed.options.to_vec();
        let mut after = again.options.to_vec();
        before.sort_by_key(|o| format!("{o:?}"));
        after.sort_by_key(|o| format!("{o:?}"));
        assert_eq!(before, after, "the same four options");
        assert!(queue.is_empty(), "asked again only once");
    }

    #[test]
    fn a_due_retry_comes_before_a_new_question_and_is_missed_only_once() {
        let mut rng = Rng::new(5);
        let mut practice = Practice::default();
        let mut retries = RetryQueue::default();
        let (first, retry) = next_question(&practice, &mut retries, AgeLevel::Ten, 1, &mut rng);
        assert!(!retry);
        assert!(Topic::for_age(AgeLevel::Ten).contains(&first.topic));
        record_answer(&mut practice, &mut retries, &first, false, false);
        assert_eq!(retries.len(), 1);

        let mut asked = Vec::new();
        for _ in 0..3 {
            let (q, retry) = next_question(&practice, &mut retries, AgeLevel::Ten, 1, &mut rng);
            asked.push(retry);
            if retry {
                assert_eq!(q.correct_value(), first.correct_value());
                // Missed again on the retry: recorded, but not queued a third time.
                record_answer(&mut practice, &mut retries, &q, false, true);
            }
        }
        assert_eq!(asked, [false, false, true]);
        assert!(retries.is_empty());
        assert_eq!(practice.0[&first.topic].misses, 4, "both misses count toward the topic");
    }

    #[test]
    fn new_questions_use_the_topic_tier_once_known() {
        let mut practice = Practice::default();
        practice.0.insert(Topic::Addition, Skill { tier: 4, misses: 0, streak: 0 });
        practice.0.insert(Topic::Subtraction, Skill { tier: 4, misses: 0, streak: 0 });
        let mut rng = Rng::new(9);
        let (q, _) = next_question(&practice, &mut RetryQueue::default(), AgeLevel::Six, 1, &mut rng);
        assert_eq!(q.tier, 4, "the topic's own tier, not the progress tier of 1");
    }

    #[test]
    fn reshuffling_keeps_the_right_answer_marked() {
        let mut rng = Rng::new(11);
        for seed in 0..50 {
            let q = generate_topic(Topic::Division, 2, &mut Rng::new(seed));
            let right = q.correct_value();
            let r = reshuffled(q, &mut rng);
            assert_eq!(r.correct_value(), right);
        }
    }

    #[test]
    fn skills_are_saved_as_three_numbers_and_bad_ones_refused() {
        let skill = Skill { tier: 3, misses: 4, streak: 1 };
        assert_eq!(Skill::parse(&skill.render()), Some(skill));
        assert_eq!(Skill::parse("\"2 0 0\""), Some(Skill { tier: 2, misses: 0, streak: 0 }));
        for bad in ["0 0 0", "5 0 0", "2 9 0", "2 0 3", "2 0", "2 0 0 0", "a b c", ""] {
            assert_eq!(Skill::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn topic_ids_round_trip() {
        for t in Topic::ALL {
            assert_eq!(Topic::from_id(t.id()), Some(t));
        }
        assert_eq!(Topic::from_id("juggling"), None);
    }
}
