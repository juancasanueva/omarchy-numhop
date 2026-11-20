//! Every string the player sees, in each supported language.
//!
//! A catalog is one `Strings` value per language. Because it is a struct, a language
//! that misses a string does not compile. Templates use numbered placeholders (`{0}`),
//! filled with [`fill`], so each language can order the parts its own way.

use std::fmt::Display;

use crate::domain::math::{
    self, AgeLevel, FracOp, Prompt, Question, Topic, common_denominator, factor_string,
};

mod en;
mod es;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Lang {
    #[default]
    En,
    Es,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Es];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    /// The language's own name, shown the same whatever the current language is.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Es => "Español",
        }
    }

    /// Reads codes like `es`, `es-AR`, or `es_ES.UTF-8`.
    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.trim().split(['-', '_', '.', '@']).next()?.to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Lang::En),
            "es" => Some(Lang::Es),
            _ => None,
        }
    }

    pub fn next(self) -> Lang {
        let i = Lang::ALL.iter().position(|l| *l == self).unwrap_or(0);
        Lang::ALL[(i + 1) % Lang::ALL.len()]
    }

    pub fn previous(self) -> Lang {
        let i = Lang::ALL.iter().position(|l| *l == self).unwrap_or(0);
        Lang::ALL[(i + Lang::ALL.len() - 1) % Lang::ALL.len()]
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &en::STRINGS,
            Lang::Es => &es::STRINGS,
        }
    }
}

/// Declares `Strings` and a list of its plain templates, so tests can check every one.
macro_rules! catalog {
    ($($field:ident),* $(,)?) => {
        pub struct Strings {
            $(pub $field: &'static str,)*
            /// Help overlay rows: keys, then what they do.
            pub shortcuts: [(&'static str, &'static str); 7],
            /// Places in the results: first, second, third.
            pub ordinals: [&'static str; 3],
            /// World names, in world order.
            pub worlds: [&'static str; 3],
        }

        impl Strings {
            pub fn templates(&self) -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($field), self.$field)),*]
            }
        }
    };
}

catalog! {
    loading, load_failed,
    tagline, players, player_n, player_level, player_hero, start, settings, menu_hint,
    language, back, settings_hint,
    age_six, age_twelve,
    topic_addition, topic_subtraction, topic_gcd, topic_lcm, topic_fractions,
    gcd_prompt, lcm_prompt, tens_short, units_short,
    crystal_question, gate_question, answer_hint, correct, not_quite, press_space_continue,
    time_up_answer, seconds,
    explain_add_single, explain_add_columns, explain_sub_single, explain_sub_columns,
    explain_gcd, explain_lcm, explain_frac_common, explain_frac_mul, explain_frac_div,
    ready, world_line, lives_none, lives_one, lives_many, lost_life, over_to, game_over,
    see_scores, world_cleared, champion, cleared_all_over_to, cleared_all,
    press_space_play, press_space_results,
    fell, grumble, wrong_answer, time_up,
    hud_stats, status_cleared, status_out, hud_world, difficulty, hud_hint, checkpoint,
    crystal_reward,
    shortcuts_title, close_help, quit_title, quit_lost, quit_keys,
    results, solo_cleared, great_run, tie, wins, result_line, results_hint,
}

/// Replaces `{0}`, `{1}`, … in `template` with `args`.
pub fn fill(template: &str, args: &[&dyn Display]) -> String {
    let mut out = template.to_string();
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), &arg.to_string());
    }
    out
}

impl Strings {
    pub fn age(&self, age: AgeLevel) -> &'static str {
        match age {
            AgeLevel::Six => self.age_six,
            AgeLevel::Twelve => self.age_twelve,
        }
    }

    pub fn topic(&self, topic: Topic) -> &'static str {
        match topic {
            Topic::Addition => self.topic_addition,
            Topic::Subtraction => self.topic_subtraction,
            Topic::Gcd => self.topic_gcd,
            Topic::Lcm => self.topic_lcm,
            Topic::Fractions => self.topic_fractions,
        }
    }

    pub fn world(&self, index: usize) -> &'static str {
        self.worlds[index.min(self.worlds.len() - 1)]
    }

    pub fn player(&self, index: usize) -> String {
        fill(self.player_n, &[&(index + 1)])
    }

    pub fn lives(&self, lives: u8) -> String {
        match lives {
            0 => self.lives_none.to_string(),
            1 => self.lives_one.to_string(),
            n => fill(self.lives_many, &[&n]),
        }
    }

    pub fn ordinal(&self, place: usize) -> &'static str {
        self.ordinals[place.min(self.ordinals.len() - 1)]
    }

    /// The single-line question for GCD and LCM, with the local abbreviation.
    pub fn prompt_line(&self, prompt: &Prompt) -> String {
        match prompt {
            Prompt::Gcd { a, b } => fill(self.gcd_prompt, &[a, b]),
            Prompt::Lcm { a, b } => fill(self.lcm_prompt, &[a, b]),
            other => other.one_line(),
        }
    }

    /// The worked solution shown after an answer.
    pub fn explain(&self, q: &Question) -> String {
        match &q.prompt {
            Prompt::Vertical { top, bottom, plus: true } => {
                let answer = top + bottom;
                if *top < 10 && *bottom < 10 {
                    fill(self.explain_add_single, &[top, bottom, &answer, &(answer % 10)])
                } else {
                    let units = top % 10 + bottom % 10;
                    fill(
                        self.explain_add_columns,
                        &[
                            &(top % 10),
                            &(bottom % 10),
                            &units,
                            &(units % 10),
                            &(top / 10),
                            &(bottom / 10),
                            &(top / 10 + bottom / 10 + 1),
                            &answer,
                        ],
                    )
                }
            }
            Prompt::Vertical { top, bottom, plus: false } => {
                let answer = top - bottom;
                if *top < 10 {
                    fill(self.explain_sub_single, &[top, bottom, &answer])
                } else {
                    fill(
                        self.explain_sub_columns,
                        &[
                            &(top % 10),
                            &(bottom % 10),
                            &(top % 10 - bottom % 10),
                            &(top / 10),
                            &(bottom / 10),
                            &(top / 10 - bottom / 10),
                            &answer,
                        ],
                    )
                }
            }
            Prompt::Gcd { a, b } => fill(
                self.explain_gcd,
                &[a, &factor_string(*a), b, &factor_string(*b), &math::gcd(*a, *b)],
            ),
            Prompt::Lcm { a, b } => fill(
                self.explain_lcm,
                &[a, &factor_string(*a), b, &factor_string(*b), &math::lcm(*a, *b)],
            ),
            Prompt::Fraction { left, op, right } => {
                let answer = op.apply(left.value(), right.value());
                match op {
                    FracOp::Add | FracOp::Sub => {
                        let s = common_denominator(*left, *op, *right);
                        fill(
                            self.explain_frac_common,
                            &[&s.common, &s.left_num, &op.symbol(), &s.right_num, &s.raw, &answer],
                        )
                    }
                    FracOp::Mul => fill(
                        self.explain_frac_mul,
                        &[
                            &left.num,
                            &right.num,
                            &left.den,
                            &right.den,
                            &(left.num * right.num),
                            &(left.den * right.den),
                            &answer,
                        ],
                    ),
                    FracOp::Div => fill(
                        self.explain_frac_div,
                        &[&left.num, &left.den, &right.den, &right.num, &answer],
                    ),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::domain::math::generate_topic;
    use crate::domain::rng::Rng;

    fn placeholders(template: &str) -> BTreeSet<usize> {
        let mut found = BTreeSet::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            rest = &rest[start + 1..];
            if let Some(end) = rest.find('}')
                && let Ok(n) = rest[..end].parse()
            {
                found.insert(n);
            }
        }
        found
    }

    #[test]
    fn every_language_uses_the_same_placeholders() {
        let en = Lang::En.strings().templates();
        for lang in Lang::ALL {
            let other = lang.strings().templates();
            assert_eq!(en.len(), other.len());
            for ((name, a), (_, b)) in en.iter().zip(other.iter()) {
                assert!(!b.is_empty(), "{lang:?}.{name} is empty");
                assert_eq!(placeholders(a), placeholders(b), "{lang:?}.{name} placeholders differ");
            }
        }
    }

    #[test]
    fn every_world_has_a_name() {
        let worlds = crate::domain::levels::worlds();
        for lang in Lang::ALL {
            assert_eq!(lang.strings().worlds.len(), worlds.len(), "{lang:?} world names");
        }
        // Level names are English, so the English catalog must list them in world order.
        let en = Lang::En.strings();
        for (i, world) in worlds.iter().enumerate() {
            assert_eq!(en.world(i), world.name, "world {i} name is out of order");
        }
    }

    #[test]
    fn fill_replaces_numbered_placeholders() {
        assert_eq!(fill("{1} then {0}", &[&"a", &2]), "2 then a");
    }

    #[test]
    fn codes_parse_with_regions_and_encodings() {
        assert_eq!(Lang::from_code("es_AR.UTF-8"), Some(Lang::Es));
        assert_eq!(Lang::from_code("es-ES"), Some(Lang::Es));
        assert_eq!(Lang::from_code("en_US"), Some(Lang::En));
        assert_eq!(Lang::from_code("fr"), None);
        assert_eq!(Lang::En.next(), Lang::Es);
        assert_eq!(Lang::Es.next(), Lang::En);
    }

    #[test]
    fn explanations_leave_no_placeholder_behind() {
        let mut rng = Rng::new(9);
        for lang in Lang::ALL {
            let t = lang.strings();
            for topic in [Topic::Addition, Topic::Subtraction, Topic::Gcd, Topic::Lcm, Topic::Fractions] {
                for tier in 1..=4 {
                    for _ in 0..50 {
                        let q = generate_topic(topic, tier, &mut rng);
                        let text = t.explain(&q);
                        assert!(!text.contains('{'), "{lang:?}: {text}");
                        assert!(text.contains(&q.correct_value().to_string()), "{lang:?}: {text}");
                    }
                }
            }
        }
    }

    #[test]
    fn spanish_uses_local_abbreviations() {
        let t = Lang::Es.strings();
        assert_eq!(t.prompt_line(&Prompt::Gcd { a: 12, b: 18 }), "MCD(12, 18) = ?");
        assert_eq!(t.prompt_line(&Prompt::Lcm { a: 4, b: 6 }), "mcm(4, 6) = ?");
    }
}
