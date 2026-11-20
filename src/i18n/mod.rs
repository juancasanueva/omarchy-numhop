//! Every string the player sees, in each supported language.
//!
//! A catalog is one `Strings` value per language. Because it is a struct, a language
//! that misses a string does not compile. Templates use numbered placeholders (`{0}`),
//! filled with [`fill`], so each language can order the parts its own way.

use std::fmt::Display;

use crate::domain::math::{
    self, AgeLevel, AnswerValue, ColumnOp, ColumnStep, Decimal, FracOp, Prompt, Question, Topic,
    common_denominator, factor_string, multiplication_steps, subtraction_steps,
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
            pub worlds: [&'static str; 5],
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
    language, back, settings_hint, name_placeholder, name_hint,
    about, about_hint, about_version, about_source, about_license, about_fonts, about_inspired,
    age_six, age_eight, age_ten, age_twelve,
    topic_addition, topic_subtraction, topic_subtraction_borrow, topic_times_tables,
    topic_multiplication, topic_division, topic_decimals, topic_fraction_of,
    topic_gcd, topic_lcm, topic_fractions,
    gcd_prompt, lcm_prompt, division_prompt, fraction_of_prompt, fraction_of_tail,
    thousands_short, hundreds_short, tens_short, units_short, decimal_point,
    crystal_question, gate_question, answer_hint, correct, not_quite, press_space_continue,
    time_up_answer, seconds,
    explain_add_single, explain_add_columns, explain_sub_single, explain_sub_columns,
    col_units, col_tens, col_hundreds, col_thousands,
    explain_col_sub, explain_col_sub_borrow, explain_col_down, explain_col_mul, explain_col_mul_carry,
    explain_answer, explain_times, explain_division, explain_decimal,
    explain_fraction_of, explain_fraction_of_unit,
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
            AgeLevel::Eight => self.age_eight,
            AgeLevel::Ten => self.age_ten,
            AgeLevel::Twelve => self.age_twelve,
        }
    }

    pub fn topic(&self, topic: Topic) -> &'static str {
        match topic {
            Topic::Addition => self.topic_addition,
            Topic::Subtraction => self.topic_subtraction,
            Topic::SubtractionBorrow => self.topic_subtraction_borrow,
            Topic::TimesTables => self.topic_times_tables,
            Topic::Multiplication => self.topic_multiplication,
            Topic::Division => self.topic_division,
            Topic::Decimals => self.topic_decimals,
            Topic::FractionOf => self.topic_fraction_of,
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

    /// The single-line question, with the local notation: abbreviations, the division
    /// sign, and the decimal separator.
    pub fn prompt_line(&self, prompt: &Prompt) -> String {
        match prompt {
            Prompt::Gcd { a, b } => fill(self.gcd_prompt, &[a, b]),
            Prompt::Lcm { a, b } => fill(self.lcm_prompt, &[a, b]),
            Prompt::Division { dividend, divisor } => fill(self.division_prompt, &[dividend, divisor]),
            Prompt::Decimal { left, right, plus } => format!(
                "{} {} {} = ?",
                self.decimal(*left),
                if *plus { '+' } else { '−' },
                self.decimal(*right)
            ),
            Prompt::FractionOf { num, den, whole } => fill(self.fraction_of_prompt, &[num, den, whole]),
            other => other.one_line(),
        }
    }

    /// An answer as the player reads it, with the local decimal separator.
    pub fn value(&self, value: AnswerValue) -> String {
        match value {
            AnswerValue::Decimal(d) => self.decimal(d),
            other => other.to_string(),
        }
    }

    pub fn decimal(&self, d: Decimal) -> String {
        d.to_string().replace('.', self.decimal_point)
    }

    /// The short column headers, units first: U, T, H, Th in English; U, D, C, UM in Spanish.
    pub fn place_short(&self, place: usize) -> &'static str {
        [self.units_short, self.tens_short, self.hundreds_short, self.thousands_short][place.min(3)]
    }

    fn place_name(&self, place: usize) -> &'static str {
        [self.col_units, self.col_tens, self.col_hundreds, self.col_thousands][place.min(3)]
    }

    /// Written subtraction column by column; a carried 1 is added to the next bottom digit.
    fn explain_subtraction_steps(&self, steps: &[ColumnStep], answer: i64) -> String {
        let mut lines: Vec<String> = steps
            .iter()
            .map(|s| {
                let name = self.place_name(s.place);
                let bottom = match (s.bottom, s.carry_in) {
                    (b, 0) => b.to_string(),
                    (0, c) => c.to_string(),
                    (b, c) => format!("({b} + {c})"),
                };
                if s.borrowed {
                    fill(self.explain_col_sub_borrow, &[&name, &s.top, &bottom, &s.value, &(s.top + 10)])
                } else if s.bottom == 0 && s.carry_in == 0 {
                    fill(self.explain_col_down, &[&name, &s.top])
                } else {
                    fill(self.explain_col_sub, &[&name, &s.top, &bottom, &s.value])
                }
            })
            .collect();
        lines.push(fill(self.explain_answer, &[&answer]));
        lines.join(" ")
    }

    /// Written multiplication by one digit, column by column, with the carries.
    fn explain_multiplication_steps(&self, steps: &[ColumnStep], answer: i64) -> String {
        let mut lines: Vec<String> = steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let name = self.place_name(s.place);
                let plus = if s.carry_in > 0 { format!(" + {}", s.carry_in) } else { String::new() };
                if i + 1 < steps.len() && s.value >= 10 {
                    fill(
                        self.explain_col_mul_carry,
                        &[&name, &s.top, &s.bottom, &plus, &s.value, &(s.value % 10), &s.carry_out],
                    )
                } else {
                    fill(self.explain_col_mul, &[&name, &s.top, &s.bottom, &plus, &s.value])
                }
            })
            .collect();
        lines.push(fill(self.explain_answer, &[&answer]));
        lines.join(" ")
    }

    /// The worked solution shown after an answer.
    pub fn explain(&self, q: &Question) -> String {
        match &q.prompt {
            // Any borrow needs the column-by-column walk-through; the short form below
            // only fits subtractions without borrowing.
            Prompt::Vertical { top, bottom, op: ColumnOp::Sub }
                if subtraction_steps(*top, *bottom).iter().any(|s| s.borrowed) =>
            {
                self.explain_subtraction_steps(&subtraction_steps(*top, *bottom), top - bottom)
            }
            Prompt::Vertical { top, bottom, op: ColumnOp::Mul } => {
                self.explain_multiplication_steps(&multiplication_steps(*top, *bottom), top * bottom)
            }
            Prompt::Vertical { top, bottom, op: ColumnOp::Add } => {
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
            Prompt::Vertical { top, bottom, op: ColumnOp::Sub } => {
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
            Prompt::Times { a, b } => {
                let counted: Vec<String> = (1..=*b).map(|k| (a * k).to_string()).collect();
                fill(self.explain_times, &[a, b, &(a * b), &counted.join(", ")])
            }
            Prompt::Division { dividend, divisor } => {
                fill(self.explain_division, &[dividend, divisor, &(dividend / divisor)])
            }
            Prompt::Decimal { left, right, plus } => {
                // Line the points up by writing every number with the same decimal places.
                let places = left.places.max(right.places);
                let h = if *plus { left.hundredths + right.hundredths } else { left.hundredths - right.hundredths };
                fill(
                    self.explain_decimal,
                    &[
                        &self.decimal(left.with_places(places)),
                        &if *plus { '+' } else { '−' },
                        &self.decimal(right.with_places(places)),
                        &self.decimal(Decimal::shortest(h).with_places(places)),
                    ],
                )
            }
            Prompt::FractionOf { num: 1, den, whole } => {
                fill(self.explain_fraction_of_unit, &[&1, den, whole, &(whole / den)])
            }
            Prompt::FractionOf { num, den, whole } => {
                let part = whole / den;
                fill(self.explain_fraction_of, &[num, den, whole, &part, &(part * num)])
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
            for topic in Topic::ALL {
                for tier in 1..=4 {
                    for _ in 0..50 {
                        let q = generate_topic(topic, tier, &mut rng);
                        let text = t.explain(&q);
                        assert!(!text.contains('{'), "{lang:?}: {text}");
                        assert!(text.contains(&t.value(q.correct_value())), "{lang:?}: {text}");
                    }
                }
            }
        }
    }

    #[test]
    fn spanish_uses_school_notation() {
        let t = Lang::Es.strings();
        assert_eq!(t.prompt_line(&Prompt::Division { dividend: 84, divisor: 7 }), "84 : 7 = ?");
        let left = Decimal::new(350, 1);
        let right = Decimal::new(125, 2);
        assert_eq!(t.prompt_line(&Prompt::Decimal { left, right, plus: true }), "3,5 + 1,25 = ?");
        assert_eq!(t.value(AnswerValue::Decimal(Decimal::shortest(475))), "4,75");
        assert_eq!(Lang::En.strings().value(AnswerValue::Decimal(Decimal::shortest(475))), "4.75");
        assert_eq!(
            [t.place_short(0), t.place_short(1), t.place_short(2), t.place_short(3)],
            ["U", "D", "C", "UM"]
        );
    }

    #[test]
    fn any_borrowing_subtraction_gets_the_column_walk_through() {
        // The explanation follows the numbers, not the topic: a borrow anywhere is explained.
        let t = Lang::En.strings();
        let q = Question {
            topic: Topic::Subtraction,
            tier: 1,
            prompt: Prompt::Vertical { top: 52, bottom: 27, op: ColumnOp::Sub },
            options: [AnswerValue::Whole(25), AnswerValue::Whole(1), AnswerValue::Whole(2), AnswerValue::Whole(3)],
            correct: 0,
        };
        assert_eq!(
            t.explain(&q),
            "Units: 2 is less than 7, so 12 − 7 = 5 and carry 1. Tens: 5 − (2 + 1) = 2. Answer: 25."
        );
    }

    #[test]
    fn column_explanations_walk_every_digit() {
        let t = Lang::En.strings();
        let sub = Question {
            topic: Topic::SubtractionBorrow,
            tier: 4,
            prompt: Prompt::Vertical { top: 432, bottom: 175, op: ColumnOp::Sub },
            options: [AnswerValue::Whole(257), AnswerValue::Whole(1), AnswerValue::Whole(2), AnswerValue::Whole(3)],
            correct: 0,
        };
        assert_eq!(
            t.explain(&sub),
            "Units: 2 is less than 5, so 12 − 5 = 7 and carry 1. \
             Tens: 3 is less than (7 + 1), so 13 − (7 + 1) = 5 and carry 1. \
             Hundreds: 4 − (1 + 1) = 2. Answer: 257."
        );
        let mul = Question {
            topic: Topic::Multiplication,
            tier: 4,
            prompt: Prompt::Vertical { top: 347, bottom: 6, op: ColumnOp::Mul },
            options: [AnswerValue::Whole(2082), AnswerValue::Whole(1), AnswerValue::Whole(2), AnswerValue::Whole(3)],
            correct: 0,
        };
        assert_eq!(
            t.explain(&mul),
            "Units: 7 × 6 = 42, write 2 and carry 4. Tens: 4 × 6 + 4 = 28, write 8 and carry 2. \
             Hundreds: 3 × 6 + 2 = 20. Answer: 2082."
        );
    }

    #[test]
    fn spanish_uses_local_abbreviations() {
        let t = Lang::Es.strings();
        assert_eq!(t.prompt_line(&Prompt::Gcd { a: 12, b: 18 }), "MCD(12, 18) = ?");
        assert_eq!(t.prompt_line(&Prompt::Lcm { a: 4, b: 6 }), "mcm(4, 6) = ?");
    }
}
