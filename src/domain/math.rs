//! Math questions by age level and difficulty tier.
//!
//! The ages follow the Spanish primary curriculum (Primaria):
//! - Age 6 (1º): addition with carrying and subtraction without borrowing.
//! - Age 8 (2º–3º): subtraction with borrowing, times tables, multiplying by one digit.
//! - Age 10 (4º–5º): exact division by one digit, adding and subtracting decimals, and
//!   a fraction of a number.
//! - Age 12 (6º): GCD, LCM, and fraction arithmetic simplified to lowest terms.
//!
//! Column arithmetic uses the vertical school layout. Every question has four distinct
//! options.

use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use super::rng::Rng;

pub const MIN_TIER: u8 = 1;
pub const MAX_TIER: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgeLevel {
    Six,
    Eight,
    Ten,
    Twelve,
}

impl AgeLevel {
    pub const ALL: [AgeLevel; 4] = [AgeLevel::Six, AgeLevel::Eight, AgeLevel::Ten, AgeLevel::Twelve];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Addition,
    Subtraction,
    SubtractionBorrow,
    TimesTables,
    Multiplication,
    Division,
    Decimals,
    FractionOf,
    Gcd,
    Lcm,
    Fractions,
}

impl Topic {
    pub const ALL: [Topic; 11] = [
        Topic::Addition,
        Topic::Subtraction,
        Topic::SubtractionBorrow,
        Topic::TimesTables,
        Topic::Multiplication,
        Topic::Division,
        Topic::Decimals,
        Topic::FractionOf,
        Topic::Gcd,
        Topic::Lcm,
        Topic::Fractions,
    ];

    pub fn for_age(age: AgeLevel) -> &'static [Topic] {
        match age {
            AgeLevel::Six => &[Topic::Addition, Topic::Subtraction],
            AgeLevel::Eight => &[Topic::SubtractionBorrow, Topic::TimesTables, Topic::Multiplication],
            AgeLevel::Ten => &[Topic::Division, Topic::Decimals, Topic::FractionOf],
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

/// The operation in a column (written) calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnOp {
    Add,
    Sub,
    Mul,
}

impl ColumnOp {
    pub fn symbol(self) -> char {
        match self {
            ColumnOp::Add => '+',
            ColumnOp::Sub => '−',
            ColumnOp::Mul => '×',
        }
    }

    pub fn apply(self, a: i64, b: i64) -> i64 {
        match self {
            ColumnOp::Add => a + b,
            ColumnOp::Sub => a - b,
            ColumnOp::Mul => a * b,
        }
    }
}

/// One column of a written subtraction or multiplication, worked right to left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnStep {
    /// 0 for units, 1 for tens, 2 for hundreds, 3 for thousands.
    pub place: usize,
    /// The top number's digit in this column.
    pub top: i64,
    /// The bottom number's digit (subtraction) or the one-digit multiplier.
    pub bottom: i64,
    /// What the previous column carried over.
    pub carry_in: i64,
    /// Subtraction: the digit written. Multiplication: `top × bottom + carry_in`.
    pub value: i64,
    /// What this column carries to the next one.
    pub carry_out: i64,
    /// Subtraction only: the top digit was too small, so 10 was added to it.
    pub borrowed: bool,
}

fn digit(n: i64, place: usize) -> i64 {
    n / 10_i64.pow(place as u32) % 10
}

fn digit_count(n: i64) -> usize {
    n.abs().to_string().len()
}

/// Written subtraction as taught in Spain: when a top digit is too small, add 10 to it
/// and carry 1 onto the next bottom digit ("me llevo una").
pub fn subtraction_steps(top: i64, bottom: i64) -> Vec<ColumnStep> {
    let mut carry = 0;
    (0..digit_count(top))
        .map(|place| {
            let (t, b) = (digit(top, place), digit(bottom, place));
            let borrowed = t < b + carry;
            let value = t + if borrowed { 10 } else { 0 } - b - carry;
            let step = ColumnStep { place, top: t, bottom: b, carry_in: carry, value, carry_out: borrowed as i64, borrowed };
            carry = step.carry_out;
            step
        })
        .collect()
}

/// Written multiplication by one digit. The last column keeps its whole value.
pub fn multiplication_steps(top: i64, multiplier: i64) -> Vec<ColumnStep> {
    let columns = digit_count(top);
    let mut carry = 0;
    (0..columns)
        .map(|place| {
            let t = digit(top, place);
            let value = t * multiplier + carry;
            let carry_out = if place + 1 < columns { value / 10 } else { 0 };
            let step = ColumnStep { place, top: t, bottom: multiplier, carry_in: carry, value, carry_out, borrowed: false };
            carry = carry_out;
            step
        })
        .collect()
}

/// A decimal number stored exactly in hundredths, written with `places` digits after
/// the point (0, 1 or 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Decimal {
    pub hundredths: i64,
    pub places: u8,
}

impl Decimal {
    pub fn new(hundredths: i64, places: u8) -> Self {
        let d = Self { hundredths, places };
        assert!(places <= 2 && places >= Self::shortest(hundredths).places, "{hundredths} needs more places");
        d
    }

    /// The shortest way to write the value: 4.70 becomes 4.7 and 5.00 becomes 5.
    pub fn shortest(hundredths: i64) -> Self {
        let places = if hundredths % 100 == 0 {
            0
        } else if hundredths % 10 == 0 {
            1
        } else {
            2
        };
        Self { hundredths, places }
    }

    pub fn with_places(self, places: u8) -> Self {
        Self::new(self.hundredths, places)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (whole, frac) = (self.hundredths / 100, self.hundredths % 100);
        match self.places {
            0 => write!(f, "{whole}"),
            1 => write!(f, "{whole}.{}", frac / 10),
            _ => write!(f, "{whole}.{frac:02}"),
        }
    }
}

/// How the question is presented. The UI decides the exact layout for each shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prompt {
    /// Column arithmetic: hundreds, tens and units stacked.
    Vertical { top: i64, bottom: i64, op: ColumnOp },
    Times { a: i64, b: i64 },
    Division { dividend: i64, divisor: i64 },
    Decimal { left: Decimal, right: Decimal, plus: bool },
    FractionOf { num: i64, den: i64, whole: i64 },
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
            Prompt::Vertical { top, bottom, op } => format!("{top} {} {bottom} = ?", op.symbol()),
            Prompt::Times { a, b } => format!("{a} × {b} = ?"),
            Prompt::Division { dividend, divisor } => format!("{dividend} ÷ {divisor} = ?"),
            Prompt::Decimal { left, right, plus } => {
                format!("{left} {} {right} = ?", if *plus { '+' } else { '−' })
            }
            Prompt::FractionOf { num, den, whole } => format!("{num}/{den} of {whole} = ?"),
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
    Decimal(Decimal),
}

impl AnswerValue {
    fn as_fraction(self) -> Fraction {
        match self {
            AnswerValue::Whole(n) => Fraction::new(n, 1),
            AnswerValue::Frac(f) => f,
            AnswerValue::Decimal(d) => Fraction::new(d.hundredths, 100),
        }
    }
}

impl fmt::Display for AnswerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnswerValue::Whole(n) => write!(f, "{n}"),
            AnswerValue::Frac(x) => write!(f, "{x}"),
            AnswerValue::Decimal(d) => write!(f, "{d}"),
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
        Topic::SubtractionBorrow => subtraction_borrow(tier, rng),
        Topic::TimesTables => times_tables(tier, rng),
        Topic::Multiplication => multiplication(tier, rng),
        Topic::Division => division(tier, rng),
        Topic::Decimals => decimals(tier, rng),
        Topic::FractionOf => fraction_of(tier, rng),
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
        prompt: Prompt::Vertical { top, bottom, op: ColumnOp::Add },
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
        prompt: Prompt::Vertical { top, bottom, op: ColumnOp::Sub },
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

fn subtraction_borrow(tier: u8, rng: &mut Rng) -> Question {
    // The units column always borrows; later tiers add a hundreds column.
    let (top, bottom) = loop {
        let (top, bottom) = match tier {
            1 => (rng.range(21, 98), rng.range(2, 9)),
            2 => (rng.range(31, 98), rng.range(12, 89)),
            3 => (rng.range(120, 989), rng.range(12, 98)),
            _ => (rng.range(210, 989), rng.range(102, 899)),
        };
        if top % 10 < bottom % 10 && top > bottom {
            break (top, bottom);
        }
    };
    let answer = top - bottom;
    // Taking the small digit from the big one in every column is the classic mistake.
    let no_borrow: i64 = (0..digit_count(top))
        .map(|p| (digit(top, p) - digit(bottom, p)).abs() * 10_i64.pow(p as u32))
        .sum();
    let mut candidates = vec![no_borrow, answer + 10, answer - 10, answer + 1, answer - 1, answer + 100];
    rng.shuffle(&mut candidates[1..]);
    let (options, correct) = whole_options(answer, &candidates, 0, rng);
    Question {
        topic: Topic::SubtractionBorrow,
        tier,
        prompt: Prompt::Vertical { top, bottom, op: ColumnOp::Sub },
        options,
        correct,
    }
}

fn times_tables(tier: u8, rng: &mut Rng) -> Question {
    // Tables unlock in the order schools teach them: 2, 5 and 10 first, the 6–9 last.
    let table = match tier {
        1 => *rng.pick(&[2, 5, 10]),
        2 => *rng.pick(&[2, 3, 4, 5, 10]),
        3 => rng.range(2, 9),
        _ => rng.range(6, 9),
    };
    let times = if tier == 4 { rng.range(6, 9) } else { rng.range(2, 10) };
    let answer = table * times;
    // One step off in the table, a neighbouring table, or adding instead of multiplying.
    let mut candidates = vec![
        table * (times + 1),
        table * (times - 1),
        (table + 1) * times,
        (table - 1) * times,
        table + times,
        answer + 10,
        answer - 10,
    ];
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(answer, &candidates, 1, rng);
    Question {
        topic: Topic::TimesTables,
        tier,
        prompt: Prompt::Times { a: table, b: times },
        options,
        correct,
    }
}

fn multiplication(tier: u8, rng: &mut Rng) -> Question {
    // From tier 2 on, the units column always carries.
    let (top, multiplier) = loop {
        let pair = match tier {
            1 => (rng.range(11, 49), rng.range(2, 3)),
            2 => (rng.range(12, 99), rng.range(2, 5)),
            3 => (rng.range(12, 99), rng.range(6, 9)),
            _ => (rng.range(102, 399), rng.range(2, 6)),
        };
        if tier == 1 || pair.0 % 10 * pair.1 >= 10 {
            break pair;
        }
    };
    let answer = top * multiplier;
    // Forgetting the carries: each column keeps only the last digit of its product.
    let no_carry: i64 = (0..digit_count(top))
        .map(|p| digit(top, p) * multiplier % 10 * 10_i64.pow(p as u32))
        .sum();
    let mut candidates =
        vec![no_carry, answer + 10, answer - 10, answer + multiplier, answer - multiplier, answer + 100];
    rng.shuffle(&mut candidates[1..]);
    let (options, correct) = whole_options(answer, &candidates, 0, rng);
    Question {
        topic: Topic::Multiplication,
        tier,
        prompt: Prompt::Vertical { top, bottom: multiplier, op: ColumnOp::Mul },
        options,
        correct,
    }
}

fn division(tier: u8, rng: &mut Rng) -> Question {
    // Exact divisions only: the dividend is built from the quotient.
    let (divisor, quotient) = match tier {
        1 => (rng.range(2, 9), rng.range(2, 9)),
        2 => (rng.range(2, 5), rng.range(11, 30)),
        3 => (rng.range(3, 9), rng.range(12, 49)),
        _ => (rng.range(6, 9), rng.range(21, 99)),
    };
    let dividend = divisor * quotient;
    let mut candidates = vec![quotient + 1, quotient - 1, quotient + 10, quotient - 10, quotient + 2, quotient - 2];
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(quotient, &candidates, 1, rng);
    Question {
        topic: Topic::Division,
        tier,
        prompt: Prompt::Division { dividend, divisor },
        options,
        correct,
    }
}

/// A number with a real tenths digit, so it never prints as 3.0.
fn tenths(lo: i64, hi: i64, rng: &mut Rng) -> Decimal {
    loop {
        let t = rng.range(lo, hi);
        if t % 10 != 0 {
            return Decimal::new(t * 10, 1);
        }
    }
}

/// A number with a real hundredths digit, so it never prints as 2.50.
fn hundredths(lo: i64, hi: i64, rng: &mut Rng) -> Decimal {
    loop {
        let h = rng.range(lo, hi);
        if h % 10 != 0 {
            return Decimal::new(h, 2);
        }
    }
}

fn decimals(tier: u8, rng: &mut Rng) -> Question {
    let plus = tier == 1 || rng.chance(0.5);
    let (mut left, mut right) = match tier {
        // Tenths that add up without carrying.
        1 => {
            let (lt, ones) = (rng.range(1, 8), rng.range(1, 9));
            let rt = rng.range(1, 9 - lt);
            (Decimal::new((ones * 10 + lt) * 10, 1), Decimal::new((rng.range(0, 5) * 10 + rt) * 10, 1))
        }
        2 => (tenths(11, 99, rng), tenths(11, 99, rng)),
        // Mixed tenths and hundredths, so the points must be lined up.
        3 => (tenths(11, 99, rng), hundredths(101, 999, rng)),
        _ => (tenths(101, 499, rng), hundredths(101, 999, rng)),
    };
    if tier >= 3 && rng.chance(0.5) {
        std::mem::swap(&mut left, &mut right);
    }
    // Keep subtraction positive and non-zero.
    if !plus && left.hundredths < right.hundredths {
        std::mem::swap(&mut left, &mut right);
    }
    if !plus && left.hundredths == right.hundredths {
        // Skip whole numbers, which would print as 3.0.
        let nudged = left.hundredths + if left.hundredths % 100 == 90 { 20 } else { 10 };
        left = Decimal::new(nudged, left.places.max(1));
    }
    let apply = |a: i64, b: i64| if plus { a + b } else { a - b };
    let answer = apply(left.hundredths, right.hundredths);

    // Reading 3.5 as 3.05 next to a number with hundredths: the points were not lined up.
    let misread = |d: Decimal| if d.places == 1 { d.hundredths / 100 * 100 + d.hundredths % 100 / 10 } else { d.hundredths };
    let mut candidates = vec![
        apply(misread(left), misread(right)),
        apply(left.hundredths, misread(right)),
        apply(misread(left), right.hundredths),
        answer + 10,
        answer - 10,
        answer + 100,
        answer - 100,
        if plus { left.hundredths - right.hundredths } else { left.hundredths + right.hundredths },
    ];
    if tier >= 3 {
        candidates.extend([answer + 1, answer - 1]);
    }
    rng.shuffle(&mut candidates);
    let mut picked = vec![answer];
    for c in candidates {
        if picked.len() < 4 && c > 0 && !picked.contains(&c) {
            picked.push(c);
        }
    }
    let mut k = 1;
    while picked.len() < 4 {
        let c = answer + k * 10;
        if !picked.contains(&c) {
            picked.push(c);
        }
        k += 1;
    }
    rng.shuffle(&mut picked);
    let correct = picked.iter().position(|&v| v == answer).unwrap();
    let options = [picked[0], picked[1], picked[2], picked[3]].map(|h| AnswerValue::Decimal(Decimal::shortest(h)));
    Question {
        topic: Topic::Decimals,
        tier,
        prompt: Prompt::Decimal { left, right, plus },
        options,
        correct,
    }
}

fn fraction_of(tier: u8, rng: &mut Rng) -> Question {
    let (num, den) = match tier {
        1 => (1, *rng.pick(&[2, 3, 4, 5, 10])),
        2 => (1, rng.range(2, 10)),
        3 => {
            let den = rng.range(3, 6);
            (rng.range(2, den - 1), den)
        }
        _ => {
            let den = rng.range(4, 10);
            (rng.range(2, den - 1), den)
        }
    };
    let part = rng.range(2, if tier == 4 { 20 } else { 10 + tier as i64 });
    let whole = den * part;
    let answer = part * num;
    // Dividing but forgetting to multiply, or taking the rest of the whole.
    let mut candidates = vec![part, whole - answer, answer + part, answer - part, answer + 1, answer - 1, answer + den];
    rng.shuffle(&mut candidates);
    let (options, correct) = whole_options(answer, &candidates, 1, rng);
    Question {
        topic: Topic::FractionOf,
        tier,
        prompt: Prompt::FractionOf { num, den, whole },
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
        Prompt::Vertical { top, bottom, op } => AnswerValue::Whole(op.apply(*top, *bottom)),
        Prompt::Times { a, b } => AnswerValue::Whole(a * b),
        Prompt::Division { dividend, divisor } => {
            if dividend % divisor != 0 {
                return Err(format!("{dividend} ÷ {divisor} is not exact"));
            }
            AnswerValue::Whole(dividend / divisor)
        }
        Prompt::Decimal { left, right, plus } => {
            let h = if *plus { left.hundredths + right.hundredths } else { left.hundredths - right.hundredths };
            AnswerValue::Decimal(Decimal::shortest(h))
        }
        Prompt::FractionOf { num, den, whole } => {
            if whole % den != 0 {
                return Err(format!("{whole} does not split into {den} parts"));
            }
            AnswerValue::Whole(whole / den * num)
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

    #[test]
    fn every_topic_and_tier_produces_valid_questions() {
        let mut rng = Rng::new(2026);
        for topic in Topic::ALL {
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
                generate(AgeLevel::Eight, 2, &mut rng).topic,
                Topic::SubtractionBorrow | Topic::TimesTables | Topic::Multiplication
            ));
            assert!(matches!(
                generate(AgeLevel::Ten, 2, &mut rng).topic,
                Topic::Division | Topic::Decimals | Topic::FractionOf
            ));
            assert!(matches!(
                generate(AgeLevel::Twelve, 2, &mut rng).topic,
                Topic::Gcd | Topic::Lcm | Topic::Fractions
            ));
        }
    }

    #[test]
    fn every_topic_belongs_to_exactly_one_age() {
        for topic in Topic::ALL {
            let ages = AgeLevel::ALL.iter().filter(|a| Topic::for_age(**a).contains(&topic)).count();
            assert_eq!(ages, 1, "{topic:?}");
        }
    }

    #[test]
    fn ages_cycle_both_ways() {
        assert_eq!(AgeLevel::Six.next(), AgeLevel::Eight);
        assert_eq!(AgeLevel::Twelve.next(), AgeLevel::Six);
        assert_eq!(AgeLevel::Six.previous(), AgeLevel::Twelve);
        for age in AgeLevel::ALL {
            assert_eq!(age.next().previous(), age);
        }
    }

    fn vertical(q: &Question) -> (i64, i64) {
        let Prompt::Vertical { top, bottom, .. } = q.prompt else { panic!("{:?}", q.prompt) };
        (top, bottom)
    }

    #[test]
    fn subtraction_with_borrowing_always_borrows() {
        let mut rng = Rng::new(6);
        for tier in 1..=4 {
            for _ in 0..300 {
                let (top, bottom) = vertical(&generate_topic(Topic::SubtractionBorrow, tier, &mut rng));
                assert!(top % 10 < bottom % 10, "{top} − {bottom} does not borrow");
                assert!(top > bottom);
                assert!(top < 1000);
            }
        }
    }

    #[test]
    fn subtraction_steps_carry_onto_the_next_bottom_digit() {
        // 432 − 175: 12 − 5 = 7, 13 − (7 + 1) = 5, 4 − (1 + 1) = 2.
        let steps = subtraction_steps(432, 175);
        let values: Vec<i64> = steps.iter().map(|s| s.value).collect();
        assert_eq!(values, vec![7, 5, 2]);
        assert_eq!(steps.iter().map(|s| s.borrowed).collect::<Vec<_>>(), vec![true, true, false]);
        assert_eq!(steps[2].carry_in, 1);
        let mut rng = Rng::new(7);
        for _ in 0..500 {
            let (top, bottom) = (rng.range(100, 999), rng.range(1, 99));
            let rebuilt: i64 = subtraction_steps(top, bottom).iter().map(|s| s.value * 10_i64.pow(s.place as u32)).sum();
            assert_eq!(rebuilt, top - bottom, "{top} − {bottom}");
        }
    }

    #[test]
    fn multiplication_steps_rebuild_the_product() {
        // 347 × 6: 42 write 2 carry 4, 24 + 4 = 28 write 8 carry 2, 18 + 2 = 20.
        let values: Vec<i64> = multiplication_steps(347, 6).iter().map(|s| s.value).collect();
        assert_eq!(values, vec![42, 28, 20]);
        let mut rng = Rng::new(8);
        for _ in 0..500 {
            let (top, m) = (rng.range(10, 999), rng.range(2, 9));
            let steps = multiplication_steps(top, m);
            let last = steps.last().unwrap();
            let low: i64 = steps[..steps.len() - 1].iter().map(|s| s.value % 10 * 10_i64.pow(s.place as u32)).sum();
            assert_eq!(last.value * 10_i64.pow(last.place as u32) + low, top * m, "{top} × {m}");
        }
    }

    #[test]
    fn times_tables_follow_the_school_order() {
        let mut rng = Rng::new(9);
        for _ in 0..300 {
            let q = generate_topic(Topic::TimesTables, 1, &mut rng);
            let Prompt::Times { a, b } = q.prompt else { panic!() };
            assert!([2, 5, 10].contains(&a), "{a} is not an early table");
            assert!((2..=10).contains(&b));
        }
    }

    #[test]
    fn multiplying_by_one_digit_carries_from_tier_two() {
        let mut rng = Rng::new(10);
        for tier in 1..=4 {
            for _ in 0..300 {
                let (top, m) = vertical(&generate_topic(Topic::Multiplication, tier, &mut rng));
                assert!((2..=9).contains(&m));
                if tier >= 2 {
                    assert!(top % 10 * m >= 10, "{top} × {m} does not carry");
                }
            }
        }
    }

    #[test]
    fn divisions_are_exact_by_one_digit() {
        let mut rng = Rng::new(11);
        for tier in 1..=4 {
            for _ in 0..300 {
                let q = generate_topic(Topic::Division, tier, &mut rng);
                let Prompt::Division { dividend, divisor } = q.prompt else { panic!() };
                assert!((2..=9).contains(&divisor));
                assert_eq!(dividend % divisor, 0);
            }
        }
    }

    #[test]
    fn decimal_answers_are_positive_and_written_shortest() {
        let mut rng = Rng::new(12);
        for tier in 1..=4 {
            for _ in 0..500 {
                let q = generate_topic(Topic::Decimals, tier, &mut rng);
                for option in q.options {
                    let AnswerValue::Decimal(d) = option else { panic!() };
                    assert!(d.hundredths > 0, "{}", q.prompt.one_line());
                    assert_eq!(d, Decimal::shortest(d.hundredths));
                }
                let Prompt::Decimal { left, right, .. } = q.prompt else { panic!() };
                for operand in [left, right] {
                    assert_eq!(operand, Decimal::shortest(operand.hundredths), "{}", q.prompt.one_line());
                }
                if tier <= 2 {
                    assert_eq!((left.places, right.places), (1, 1), "tiers 1 and 2 use tenths");
                }
            }
        }
    }

    #[test]
    fn decimals_print_like_school() {
        assert_eq!(Decimal::shortest(470).to_string(), "4.7");
        assert_eq!(Decimal::shortest(500).to_string(), "5");
        assert_eq!(Decimal::shortest(425).to_string(), "4.25");
        assert_eq!(Decimal::shortest(305).to_string(), "3.05");
        assert_eq!(Decimal::new(350, 2).to_string(), "3.50");
    }

    #[test]
    fn fraction_of_a_number_splits_evenly() {
        let mut rng = Rng::new(13);
        for tier in 1..=4 {
            for _ in 0..300 {
                let q = generate_topic(Topic::FractionOf, tier, &mut rng);
                let Prompt::FractionOf { num, den, whole } = q.prompt else { panic!() };
                assert!(num >= 1 && num < den);
                assert_eq!(whole % den, 0);
                if tier <= 2 {
                    assert_eq!(num, 1, "tiers 1 and 2 use unit fractions");
                }
            }
        }
    }
}
