//! Shared helpers of the algebra word-problem solver: the top of Python's `word_solver.py`.

use super::number_puzzle::Lin;
pub use super::q::{decline, Q};
use super::rx::{re, Re};
use std::sync::LazyLock;

/// What a family returns (Python `Result(family, values, equation, reading)`).
#[derive(Clone, Debug, PartialEq)]
pub struct SolveResult {
    pub family: &'static str,
    pub values: Vec<Q>,
    pub equation: String,
    pub reading: String,
}

/// A family: the problem text in, an answer or None (a decline) out.
pub type Family = fn(&str) -> Option<SolveResult>;

/// `Result(...)` wrapped in Some.
pub fn res(family: &'static str, values: Vec<Q>, equation: String, reading: String) -> Option<SolveResult> {
    Some(SolveResult { family, values, equation, reading })
}

pub const NUM: &str = r"(\d+(?:\.\d+)?)";
pub const KWORD: &str = r"(?:two|three|four|five|six|seven|eight|nine|ten|\d+)";

/// `_SMALL`
pub fn small(word: &str) -> Option<i64> {
    Some(match word {
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        _ => return None,
    })
}

/// `fr("7.50")` -> 15/2, `fr("three")` -> 3.
pub fn fr(text: &str) -> Q {
    if let Some(n) = small(text) {
        return Q::int(n);
    }
    Q::parse(text).unwrap_or_else(|| decline())
}

static THOUSANDS: LazyLock<Re> = LazyLock::new(|| re(r"(?<=\d),(?=\d{3}(?!\d))"));
static PERCENT_WORD: LazyLock<Re> = LazyLock::new(|| re(r"(?i)\bper\s?cent\b|\bpercent\b"));
static DIGIT_NUM: LazyLock<Re> = LazyLock::new(|| re(r"(\$)?\s?(\d+(?:\.\d+)?)"));

/// Lower-case text with thousands commas removed and 'percent' -> '%'.
pub fn clean(problem: &str) -> String {
    let t = problem.trim().replace('\u{2019}', "'").replace('\u{2212}', "-");
    let t = THOUSANDS.sub(&t, "");
    let t = PERCENT_WORD.sub(&t, "%");
    t.to_lowercase()
}

/// One number written in digits: (value, is_money, start, end) in the cleaned text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Num {
    pub v: Q,
    pub money: bool,
    pub start: usize,
    pub end: usize,
}

/// Every number written in digits, in order, taken from the cleaned text.
pub fn digit_numbers(problem: &str) -> Vec<Num> {
    let t = clean(problem);
    DIGIT_NUM
        .find_all(&t)
        .iter()
        .map(|m| Num { v: fr(m.g(2)), money: m.opt(1).is_some(), start: m.start(), end: m.end() })
        .collect()
}

pub fn n_numbers(problem: &str) -> usize {
    digit_numbers(problem).len()
}

/// `show`: a whole number as a whole number, otherwise up to four decimals.
pub fn show(v: Q) -> String {
    if v.is_int() {
        return v.numer().to_string();
    }
    let s = format!("{:.4}", v.to_f64());
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The value as an exact fraction, never a rounded decimal.
pub fn show_exact(v: Q) -> String {
    v.to_string()
}

/// Exact value for the reading.  Money that is not a whole number of cents also shows the nearest cent.
pub fn show_answer(v: Q, money: bool) -> String {
    let text = show_exact(v);
    if !money {
        return text;
    }
    let cents = Q::new((v * Q::int(100)).round(), 100);
    if cents == v {
        text
    } else {
        format!("{} ({} to the nearest cent)", text, show(cents))
    }
}

/// `lhs = rhs` where both are linear: the exact value of x, or None.
pub fn solve_linear(lhs: &Lin, rhs: &Lin) -> Option<Q> {
    let d = lhs.a - rhs.a;
    if d.is_zero() {
        return None;
    }
    Some((rhs.b - lhs.b) / d)
}

/// Build a Lin from (coefficient, constant) pairs added together.
pub fn x_plus(terms: &[(Q, Q)]) -> Lin {
    let mut out = Lin::new(Q::ZERO, Q::ZERO);
    for &(a, b) in terms {
        out = Lin::new(out.a + a, out.b + b);
    }
    out
}

pub use super::rx::{strip, take, take_before};

/// Python `str.split()` (split on any whitespace, dropping empty pieces).
pub fn words(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

pub fn answer_text(r: &SolveResult) -> String {
    r.values.iter().map(|v| show_exact(*v)).collect::<Vec<_>>().join(" and ")
}

/// `t[n.end:n.end + 2].strip().startswith("%")`: the number is followed by a percent sign.
pub fn is_pct(t: &str, n: &Num) -> bool {
    take(t, n.end, 2).trim().starts_with('%')
}

/// The last three words of `s` joined by spaces (`" ".join(s.split()[-3:])`).
pub fn last_words(s: &str, n: usize) -> String {
    let w = s.split_whitespace().collect::<Vec<_>>();
    w[w.len().saturating_sub(n)..].join(" ")
}

/// Python `re.split(r"(?<=[.!?])\s+", t)`.
pub fn sentences(t: &str) -> Vec<String> {
    use std::sync::LazyLock;
    static SPLIT: LazyLock<super::rx::Re> = LazyLock::new(|| super::rx::re(r"(?<=[.!?])\s+"));
    SPLIT.split(t)
}

/// The sentences of `t` that end with "?".
pub fn questions(t: &str) -> Vec<String> {
    sentences(t).into_iter().filter(|x| x.trim().ends_with('?')).collect()
}

pub use super::patterns::*;
