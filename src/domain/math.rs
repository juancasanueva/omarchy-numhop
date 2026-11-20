//! Math questions by age level and difficulty tier.
//!
//! Age 6 practices addition with carrying and subtraction without borrowing, shown in
//! the vertical school layout. Age 12 practices GCD, LCM, and fraction arithmetic with
//! results simplified to lowest terms. Every question has four distinct options.

use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use super::rng::Rng;

pub const MIN_TIER: u8 = 1;
pub const MAX_TIER: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgeLevel {
    Six,
    Twelve,
}

impl AgeLevel {
    pub fn toggle(self) -> Self {
        match self {
            AgeLevel::Six => AgeLevel::Twelve,
            AgeLevel::Twelve => AgeLevel::Six,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Addition,
    Subtraction,
    Gcd,
    Lcm,
    Fractions,
}

impl Topic {
    pub fn for_age(age: AgeLevel) -> &'static [Topic] {
        match age {
            AgeLevel::Six => &[Topic::Addition, Topic::Subtraction],
            AgeLevel::Twelve => &[Topic::Gcd, Topic::Lcm, Topic::Fractions],
        }
    }
}

pub fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a / gcd(a, b) * b).abs()
}

/// A fraction always stored in lowest terms with a positive denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fraction {
    num: i64,
    den: i64,
}

impl Fraction {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "denominator must not be zero");
        let sign = if den < 0 { -1 } else { 1 };
        let g = gcd(num, den).max(1);
        Self {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    pub fn num(self) -> i64 {
        self.num
    }

    pub fn den(self) -> i64 {
        self.den
    }

}

impl Add for Fraction {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
}

impl Sub for Fraction {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(self.num * o.den - o.num * self.den, self.den * o.den)
    }
}

impl Mul for Fraction {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(self.num * o.num, self.den * o.den)
    }
}

impl Div for Fraction {
    type Output = Self;
    fn div(self, o: Self) -> Self {
        Self::new(self.num * o.den, self.den * o.num)
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// A fraction operand as written in the question, which need not be in lowest terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawFraction {
    pub num: i64,
    pub den: i64,
}

impl RawFraction {
    pub fn value(self) -> Fraction {
        Fraction::new(self.num, self.den)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FracOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl FracOp {
    pub fn symbol(self) -> char {
        match self {
            FracOp::Add => '+',
            FracOp::Sub => '−',
            FracOp::Mul => '×',
            FracOp::Div => '÷',
        }
    }

    pub fn apply(self, a: Fraction, b: Fraction) -> Fraction {
        match self {
            FracOp::Add => a.add(b),
            FracOp::Sub => a.sub(b),
            FracOp::Mul => a.mul(b),
            FracOp::Div => a.div(b),
        }
    }
}

/// The steps of adding or subtracting fractions over a common denominator:
/// `ln/common op rn/common = raw/common`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonDenominator {
    pub common: i64,
    pub left_num: i64,
    pub right_num: i64,
    pub raw: i64,
}

pub fn common_denominator(left: RawFraction, op: FracOp, right: RawFraction) -> CommonDenominator {
    let common = lcm(left.den, right.den);
    let left_num = left.num * (common / left.den);
    let right_num = right.num * (common / right.den);
    let raw = if op == FracOp::Sub { left_num - right_num } else { left_num + right_num };
    CommonDenominator { common, left_num, right_num, raw }
}

/// How the question is presented. The UI decides the exact layout for each shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prompt {
    /// Column arithmetic for young players: tens and units stacked.
    Vertical { top: i64, bottom: i64, plus: bool },
    Gcd { a: i64, b: i64 },
    Lcm { a: i64, b: i64 },
    Fraction {
        left: RawFraction,
        op: FracOp,
        right: RawFraction,
    },
}

impl Prompt {
    /// A single-line rendering, used by the UI and by tests.
    pub fn one_line(&self) -> String {
        match self {
            Prompt::Vertical { top, bottom, plus } => {
                format!("{top} {} {bottom} = ?", if *plus { '+' } else { '−' })
            }
            Prompt::Gcd { a, b } => format!("GCD({a}, {b}) = ?"),
            Prompt::Lcm { a, b } => format!("LCM({a}, {b}) = ?"),
            Prompt::Fraction { left, op, right } => format!(
                "{}/{} {} {}/{} = ?",
                left.num,
                left.den,
                op.symbol(),
                right.num,
                right.den
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnswerValue {
    Whole(i64),
    Frac(Fraction),
}

impl AnswerValue {
    fn as_fraction(self) -> Fraction {
        match self {
            AnswerValue::Whole(n) => Fraction::new(n, 1),
            AnswerValue::Frac(f) => f,
        }
    }
}

impl fmt::Display for AnswerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnswerValue::Whole(n) => write!(f, "{n}"),
            AnswerValue::Frac(x) => write!(f, "{x}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Question {
    pub topic: Topic,
    pub tier: u8,
    pub prompt: Prompt,
    pub options: [AnswerValue; 4],
    pub correct: usize,
}

impl Question {
    pub fn correct_value(&self) -> AnswerValue {
        self.options[self.correct]
    }

    pub fn is_correct(&self, choice: usize) -> bool {
        choice == self.correct
    }
}

pub fn clamp_tier(tier: u8) -> u8 {
    tier.clamp(MIN_TIER, MAX_TIER)
}

pub fn tier_stars(tier: u8) -> String {
    "★".repeat(clamp_tier(tier) as usize)
}

/// Picks a topic for the player's age and generates a question at that tier.
pub fn generate(age: AgeLevel, tier: u8, rng: &mut Rng) -> Question {
    let topic = *rng.pick(Topic::for_age(age));
    generate_topic(topic, tier, rng)
}

pub fn generate_topic(topic: Topic, tier: u8, rng: &mut Rng) -> Question {
    let tier = clamp_tier(tier);
    match topic {
        Topic::Addition => addition(tier, rng),
        Topic::Subtraction => subtraction(tier, rng),
        Topic::Gcd => gcd_question(tier, rng),
        Topic::Lcm => lcm_question(tier, rng),
        Topic::Fractions => fraction_question(tier, rng),
    }
}

/// Builds four distinct whole-number options around the answer from plausible mistakes,
/// then pads with nearby values if the candidates collapse.
fn whole_options(answer: i64, candidates: &[i64], min: i64, rng: &mut Rng) -> ([AnswerValue; 4], usize) {
    let mut picked = vec![answer];
    for &c in candidates {
        if picked.len() == 4 {
            break;
        }
        if c >= min && !picked.contains(&c) {
            picked.push(c);
        }
    }
    let mut step = 1;
    while picked.len() < 4 {
        for c in [answer + step, answer - step] {
            if picked.len() < 4 && c >= min && !picked.contains(&c) {
                picked.push(c);
            }
        }
        step += 1;
    }
    rng.shuffle(&mut picked);
    let correct = picked.iter().position(|&v| v == answer).unwrap();
    let options = [
        AnswerValue::Whole(picked[0]),
        AnswerValue::Whole(picked[1]),
        AnswerValue::Whole(picked[2]),
        AnswerValue::Whole(picked[3]),
    ];
    (options, correct)
}

fn addition(tier: u8, rng: &mut Rng) -> Question {
    // Every tier forces a carry in the units column, the skill being practiced.
    let (top, bottom) = match tier {
        1 => {
            let u1 = rng.range(4, 9);
            let u2 = rng.range((10 - u1).max(2), 9);
            (u1, u2)
        }
        2 => {
            let d1 = rng.range(2, 5);
            let u1 = rng.range(3, 9);
            let u2 = rng.range((10 - u1).max(2), 9);
            (d1 * 10 + u1, u2)
        }
        3 => {
            let (d1, d2) = (rng.range(1, 3), rng.range(1, 3));
            let u1 = rng.range(3, 9);
            let u2 = rng.range((10 - u1).max(2), 9);
            (d1 * 10 + u1, d2 * 10 + u2)
        }
        _ => {
            let d1 = rng.range(3, 5);
            let d2 = rng.range(2, 8 - d1);
            let u1 = rng.range(4, 9);
            let u2 = rng.range((10 - u1).max(3), 9);
            (d1 * 10 + u1, d2 * 10 + u2)
        }
    };
    let answer = top + bottom;
    // Forgetting the carry is the classic mistake, so it is always an option.
    let mut candidates = vec![answer - 10, answer + 10, answer + 1, answer - 1, answer + 2];
    rng.shuffle(&mut candidates[1..]);
    let (options, correct) = whole_options(answer, &candidates, 0, rng);
    Question {
        topic: Topic::Addition,
        tier,
        prompt: Prompt::Vertical { top, bottom, plus: true },
        options,
        correct,
    }
}

fn subtraction(tier: u8, rng: &mut Rng) -> Question {
    // No borrowing: each column of the bottom number is at most the top's.
    let (top, bottom) = match tier {
        1 => {
            let u1 = rng.range(3, 9);
            (u1, rng.range(1, u1 - 1))
        }
        2 => {
            let d1 = rng.range(2, 4);
            let u1 = rng.range(2, 9);
            (d1 * 10 + u1, rng.range(1, u1))
        }
        3 => {
            let d1 = rng.range(5, 7);
            let d2 = rng.range(2, d1 - 1);
            let u1 = rng.range(2, 9);
            (d1 * 10 + u1, d2 * 10 + rng.range(0, u1))
        }
        _ => {
            let d1 = rng.range(7, 9);
            let d2 = rng.range(3, d1 - 1);
            let u1 = rng.range(3, 9);
            (d1 * 10 + u1, d2 * 10 + rng.range(1, u1 - 1))
        }
    };
    let answer = top - bottom;
    let mut candidates = vec![top + bottom, answer + 1, answer - 1, answer + 10, answer - 10, answer + 2];
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(answer, &candidates, 0, rng);
    Question {
        topic: Topic::Subtraction,
        tier,
        prompt: Prompt::Vertical { top, bottom, plus: false },
        options,
        correct,
    }
}

/// Two distinct multipliers with no common factor, so `g * m` and `g * n` have GCD `g`.
fn coprime_pair(lo: i64, hi: i64, rng: &mut Rng) -> (i64, i64) {
    loop {
        let m = rng.range(lo, hi);
        let n = rng.range(lo, hi);
        if m != n && gcd(m, n) == 1 {
            return (m, n);
        }
    }
}

pub fn prime_factors(mut n: i64) -> Vec<(i64, u32)> {
    let mut out = Vec::new();
    let mut p = 2;
    while p * p <= n {
        let mut count = 0;
        while n % p == 0 {
            n /= p;
            count += 1;
        }
        if count > 0 {
            out.push((p, count));
        }
        p += 1;
    }
    if n > 1 {
        out.push((n, 1));
    }
    out
}

/// Prime factorization written as `2^2·3`, the same in every language.
pub fn factor_string(n: i64) -> String {
    let parts: Vec<String> = prime_factors(n)
        .into_iter()
        .map(|(p, e)| if e == 1 { p.to_string() } else { format!("{p}^{e}") })
        .collect();
    if parts.is_empty() { n.to_string() } else { parts.join("·") }
}

fn gcd_question(tier: u8, rng: &mut Rng) -> Question {
    let (g, (m, n)) = match tier {
        1 => (rng.range(2, 5), coprime_pair(2, 5, rng)),
        2 => (rng.range(2, 6), coprime_pair(2, 7, rng)),
        3 => (rng.range(3, 9), coprime_pair(2, 9, rng)),
        _ => (rng.range(4, 12), coprime_pair(3, 11, rng)),
    };
    let (a, b) = (g * m, g * n);
    let answer = g;
    let mut candidates = vec![g * m.min(n), 1, g + 1, g - 1, g * 2, a.min(b)];
    if g % 2 == 0 {
        candidates.push(g / 2);
    }
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(answer, &candidates, 1, rng);
    Question {
        topic: Topic::Gcd,
        tier,
        prompt: Prompt::Gcd { a, b },
        options,
        correct,
    }
}

fn lcm_question(tier: u8, rng: &mut Rng) -> Question {
    let (lo, hi, cap) = match tier {
        1 => (2, 6, 30),
        2 => (2, 10, 60),
        3 => (4, 15, 120),
        _ => (6, 20, 240),
    };
    let (a, b) = loop {
        let a = rng.range(lo, hi);
        let b = rng.range(lo, hi);
        if a != b && lcm(a, b) <= cap {
            break (a, b);
        }
    };
    let answer = lcm(a, b);
    let mut candidates = vec![a * b, answer * 2, gcd(a, b), answer + a.min(b), answer - a.min(b), a.max(b)];
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(answer, &candidates, 2, rng);
    Question {
        topic: Topic::Lcm,
        tier,
        prompt: Prompt::Lcm { a, b },
        options,
        correct,
    }
}

fn fraction_question(tier: u8, rng: &mut Rng) -> Question {
    let op = match tier {
        1 | 2 => *rng.pick(&[FracOp::Add, FracOp::Sub]),
        3 => *rng.pick(&[FracOp::Add, FracOp::Sub, FracOp::Mul]),
        _ => *rng.pick(&[FracOp::Add, FracOp::Sub, FracOp::Mul, FracOp::Div]),
    };
    let (b, d) = match tier {
        1 => {
            let den = rng.range(3, 9);
            (den, den)
        }
        2 => {
            let pairs = [(2, 4), (3, 6), (2, 6), (4, 8), (5, 10), (3, 9), (2, 8)];
            let (x, y) = *rng.pick(&pairs);
            if rng.chance(0.5) { (x, y) } else { (y, x) }
        }
        3 => (rng.range(2, 8), rng.range(2, 8)),
        _ => (rng.range(2, 10), rng.range(2, 10)),
    };
    let max_num = |den: i64| if tier == 4 { den + 2 } else { den - 1 };
    let mut left = RawFraction { num: rng.range(1, max_num(b).max(1)), den: b };
    let mut right = RawFraction { num: rng.range(1, max_num(d).max(1)), den: d };
    // Keep subtraction non-negative; negative fractions are not part of the curriculum.
    if op == FracOp::Sub && left.value().sub(right.value()).num() < 0 {
        std::mem::swap(&mut left, &mut right);
    }
    // An answer of zero makes a weak question, so nudge the left numerator.
    if op == FracOp::Sub && left.value() == right.value() {
        left.num += 1;
    }
    let answer = op.apply(left.value(), right.value());

    // Typical mistakes: adding across, using the wrong operation, forgetting to flip.
    let across = |x: i64, y: i64| if y != 0 { Some(Fraction::new(x, y)) } else { None };
    let mut candidates: Vec<Fraction> = [
        across(left.num + right.num, left.den + right.den),
        across(left.num * right.num, left.den * right.den),
        across(left.num * right.den, left.den * right.num),
        across((left.num - right.num).abs().max(1), (left.den - right.den).abs().max(1)),
        Some(left.value().add(right.value())),
        Some(left.value().sub(right.value())),
        Some(answer.add(Fraction::new(1, answer.den()))),
        across(answer.den(), answer.num().max(1)),
    ]
    .into_iter()
    .flatten()
    .filter(|f| f.num() > 0)
    .collect();
    rng.shuffle(&mut candidates);

    let mut picked = vec![answer];
    for c in candidates {
        if picked.len() < 4 && !picked.contains(&c) {
            picked.push(c);
        }
    }
    let mut k = 1;
    while picked.len() < 4 {
        let c = answer.add(Fraction::new(k, answer.den() * 2));
        if !picked.contains(&c) {
            picked.push(c);
        }
        k += 1;
    }
    rng.shuffle(&mut picked);
    let correct = picked.iter().position(|&v| v == answer).unwrap();
    let options = [
        AnswerValue::Frac(picked[0]),
        AnswerValue::Frac(picked[1]),
        AnswerValue::Frac(picked[2]),
        AnswerValue::Frac(picked[3]),
    ];

    Question {
        topic: Topic::Fractions,
        tier,
        prompt: Prompt::Fraction { left, op, right },
        options,
        correct,
    }
}

/// Checks the invariants every question must hold. Used by tests and debug builds.
pub fn validate(q: &Question) -> Result<(), String> {
    if q.correct >= 4 {
        return Err("correct index out of range".into());
    }
    for i in 0..4 {
        for j in (i + 1)..4 {
            if q.options[i].as_fraction() == q.options[j].as_fraction() {
                return Err(format!("duplicate options in {}", q.prompt.one_line()));
            }
        }
    }
    let expected = match &q.prompt {
        Prompt::Vertical { top, bottom, plus } => {
            AnswerValue::Whole(if *plus { top + bottom } else { top - bottom })
        }
        Prompt::Gcd { a, b } => AnswerValue::Whole(gcd(*a, *b)),
        Prompt::Lcm { a, b } => AnswerValue::Whole(lcm(*a, *b)),
        Prompt::Fraction { left, op, right } => AnswerValue::Frac(op.apply(left.value(), right.value())),
    };
    if q.correct_value().as_fraction() != expected.as_fraction() {
        return Err(format!(
            "wrong answer for {}: {} vs {}",
            q.prompt.one_line(),
            q.correct_value(),
            expected
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Topic; 5] = [Topic::Addition, Topic::Subtraction, Topic::Gcd, Topic::Lcm, Topic::Fractions];

    #[test]
    fn every_topic_and_tier_produces_valid_questions() {
        let mut rng = Rng::new(2026);
        for topic in ALL {
            for tier in 1..=4 {
                for _ in 0..500 {
                    let q = generate_topic(topic, tier, &mut rng);
                    validate(&q).unwrap();
                    assert_eq!(q.tier, tier);
                }
            }
        }
    }

    #[test]
    fn addition_always_carries() {
        let mut rng = Rng::new(1);
        for tier in 1..=4 {
            for _ in 0..300 {
                let q = generate_topic(Topic::Addition, tier, &mut rng);
                let Prompt::Vertical { top, bottom, .. } = q.prompt else { panic!() };
                assert!(top % 10 + bottom % 10 >= 10, "{top} + {bottom} has no carry");
                assert!(top + bottom < 100);
            }
        }
    }

    #[test]
    fn subtraction_never_borrows() {
        let mut rng = Rng::new(2);
        for tier in 1..=4 {
            for _ in 0..300 {
                let q = generate_topic(Topic::Subtraction, tier, &mut rng);
                let Prompt::Vertical { top, bottom, .. } = q.prompt else { panic!() };
                assert!(bottom % 10 <= top % 10, "{top} − {bottom} borrows");
                assert!(bottom / 10 <= top / 10);
                assert!(top - bottom >= 0);
            }
        }
    }

    #[test]
    fn fraction_answers_are_positive_and_simplified() {
        let mut rng = Rng::new(3);
        for tier in 1..=4 {
            for _ in 0..500 {
                let q = generate_topic(Topic::Fractions, tier, &mut rng);
                let AnswerValue::Frac(f) = q.correct_value() else { panic!() };
                assert!(f.num() > 0, "{}", q.prompt.one_line());
                assert_eq!(gcd(f.num(), f.den()), 1);
            }
        }
    }

    #[test]
    fn tiers_unlock_operations_gradually() {
        let mut rng = Rng::new(4);
        for _ in 0..500 {
            let q = generate_topic(Topic::Fractions, 1, &mut rng);
            let Prompt::Fraction { left, right, op } = q.prompt else { panic!() };
            assert_eq!(left.den, right.den, "tier 1 uses like denominators");
            assert!(matches!(op, FracOp::Add | FracOp::Sub));
        }
    }

    #[test]
    fn fraction_arithmetic_is_exact() {
        let half = Fraction::new(1, 2);
        let third = Fraction::new(1, 3);
        assert_eq!(half.add(third), Fraction::new(5, 6));
        assert_eq!(half.sub(third), Fraction::new(1, 6));
        assert_eq!(half.mul(third), Fraction::new(1, 6));
        assert_eq!(half.div(third), Fraction::new(3, 2));
        assert_eq!(Fraction::new(4, 8).to_string(), "1/2");
        assert_eq!(Fraction::new(6, 3).to_string(), "2");
    }

    #[test]
    fn gcd_and_lcm_basics() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(prime_factors(72), vec![(2, 3), (3, 2)]);
    }

    #[test]
    fn age_picks_matching_topics() {
        let mut rng = Rng::new(5);
        for _ in 0..200 {
            assert!(matches!(generate(AgeLevel::Six, 2, &mut rng).topic, Topic::Addition | Topic::Subtraction));
            assert!(matches!(
                generate(AgeLevel::Twelve, 2, &mut rng).topic,
                Topic::Gcd | Topic::Lcm | Topic::Fractions
            ));
        }
    }
}
