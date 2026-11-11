//! Reads "number puzzles" directly: "Five less than twice a number is 31."  Port of `number_puzzle.py`.
//!
//! A bag-of-words matcher cannot tell "5 less than x" (x - 5) from "x less than 5" (5 - x).  This reader keeps the order.  It understands one unknown
//! number, whole-number / decimal / fraction constants, and a closed list of phrases, builds `a*x + b = c*x + d` and solves it exactly.  It returns None
//! whenever it is not completely sure it understood every word of the equation sentence.

use super::q::Q;
use super::rx::{re, strip, Re};
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq)]
pub struct Puzzle {
    pub answer: Q,
    pub equation: String,
    pub reading: String,
}

/// Raised inside the reader for anything it does not fully understand (also for a division by zero or a bad number).
#[derive(Debug)]
pub struct NotAPuzzle;

type Pr<T> = std::result::Result<T, NotAPuzzle>;

/// A linear expression `a*x + b` with exact fractions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lin {
    pub a: Q,
    pub b: Q,
}

impl Lin {
    pub fn new(a: Q, b: Q) -> Lin {
        Lin { a, b }
    }

    fn add(self, o: Lin) -> Lin {
        Lin::new(self.a + o.a, self.b + o.b)
    }

    fn sub(self, o: Lin) -> Lin {
        Lin::new(self.a - o.a, self.b - o.b)
    }

    fn mul(self, o: Lin) -> Pr<Lin> {
        if !self.a.is_zero() && !o.a.is_zero() {
            return Err(NotAPuzzle); // x times x
        }
        Ok(Lin::new(self.a * o.b + self.b * o.a, self.b * o.b))
    }

    fn div(self, o: Lin) -> Pr<Lin> {
        if !o.a.is_zero() || o.b.is_zero() {
            return Err(NotAPuzzle); // division by x or by zero
        }
        Ok(Lin::new(self.a / o.b, self.b / o.b))
    }
}

const X: Lin = Lin { a: Q::ONE, b: Q::ZERO };

fn constant(v: Q) -> Lin {
    Lin::new(Q::ZERO, v)
}

pub fn show_fraction(f: Q) -> String {
    f.to_string()
}

/// `2x - 5`  /  `x/4 + 3`  /  `7`
pub fn show(e: Lin) -> String {
    let mut text = String::new();
    if !e.a.is_zero() {
        let c = e.a.abs();
        let body = if c.is_int() {
            format!("{}x", if c == Q::ONE { String::new() } else { c.numer().to_string() })
        } else {
            format!("{}x/{}", if c.numer() == 1 { String::new() } else { c.numer().to_string() }, c.denom())
        };
        text = format!("{}{}", if e.a < Q::ZERO { "-" } else { "" }, body);
    }
    if !e.b.is_zero() || text.is_empty() {
        let mag = show_fraction(e.b.abs());
        text = if text.is_empty() {
            format!("{}{}", if e.b < Q::ZERO { "-" } else { "" }, mag)
        } else {
            format!("{}{}{}", text, if e.b < Q::ZERO { " - " } else { " + " }, mag)
        };
    }
    text
}

// ---------------------------------------------------------------------------------------------------------
// Words -> numbers
// ---------------------------------------------------------------------------------------------------------
const ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen",
    "eighteen", "nineteen",
];
const TENS: [(&str, i64); 8] = [("twenty", 20), ("thirty", 30), ("forty", 40), ("fifty", 50), ("sixty", 60), ("seventy", 70), ("eighty", 80), ("ninety", 90)];

fn word_value(w: &str) -> Option<i64> {
    if let Some(i) = ONES.iter().position(|o| *o == w) {
        return Some(i as i64);
    }
    if let Some((_, v)) = TENS.iter().find(|(n, _)| *n == w) {
        return Some(*v);
    }
    if w == "hundred" {
        return Some(100);
    }
    None
}

fn fraction_word(w: &str) -> Option<i64> {
    Some(match w {
        "half" | "halves" => 2,
        "third" | "thirds" => 3,
        "fourth" | "fourths" | "quarter" | "quarters" => 4,
        "fifth" | "fifths" => 5,
        "sixth" | "sixths" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        "tenth" => 10,
        _ => return None,
    })
}

const NUM_PAT: &str = r"-?\d+(?:\.\d+)?(?:/\d+)?";

static WD_FRACTION: LazyLock<Re> = LazyLock::new(|| {
    re(r"\b(one|two|three|four|five|six|seven|eight|nine|ten)[- ](half|halves|third|thirds|fourth|fourths|quarter|quarters|fifth|fifths|sixth|sixths|seventh|eighth|ninth|tenth)\b")
});
static WD_A_FRACTION: LazyLock<Re> = LazyLock::new(|| re(r"\b(a|one)[- ](half|third|fourth|quarter|fifth|sixth|seventh|eighth|ninth|tenth)\b"));
static WD_HALF: LazyLock<Re> = LazyLock::new(|| re(r"\bhalf\b"));
static WD_TENS_ONES: LazyLock<Re> = LazyLock::new(|| re(r"\b(twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)[- ](one|two|three|four|five|six|seven|eight|nine)\b"));
static WD_ALL: LazyLock<Re> = LazyLock::new(|| {
    let mut ws: Vec<&str> = ONES.to_vec();
    ws.extend(TENS.iter().map(|(n, _)| *n));
    ws.push("hundred");
    ws.sort_by_key(|w| std::cmp::Reverse(w.len())); // stable, longest first, like Python's sorted(..., key=len, reverse=True)
    re(&format!(r"\b({})\b", ws.join("|")))
});

/// "one-fourth" -> "1/4", "twenty-one" -> "21", "five" -> "5", "half" -> "1/2".  Lower-case text in and out.
pub fn words_to_digits(s: &str) -> String {
    let s = s.replace('\u{2212}', "-").replace('\u{2013}', "-").replace('\u{00d7}', " times ").replace('\u{00f7}', " divided by ");
    let s = WD_FRACTION.sub_with(&s, |m| format!("{}/{}", word_value(m.g(1)).unwrap(), fraction_word(m.g(2)).unwrap()));
    let s = WD_A_FRACTION.sub_with(&s, |m| format!("1/{}", fraction_word(m.g(2)).unwrap()));
    let s = WD_HALF.sub(&s, "1/2");
    let s = WD_TENS_ONES.sub_with(&s, |m| (word_value(m.g(1)).unwrap() + word_value(m.g(2)).unwrap()).to_string());
    WD_ALL.sub_with(&s, |m| word_value(m.g(1)).unwrap().to_string())
}

fn to_fraction(tok: &str) -> Pr<Q> {
    if let Some((a, b)) = tok.split_once('/') {
        let a: i128 = a.parse().map_err(|_| NotAPuzzle)?;
        let b: i128 = b.parse().map_err(|_| NotAPuzzle)?;
        if b == 0 {
            return Err(NotAPuzzle);
        }
        return Ok(Q::new(a, b));
    }
    Q::parse(tok).ok_or(NotAPuzzle)
}

// ---------------------------------------------------------------------------------------------------------
// Phrase grammar (everything is matched on lower-case text with words already turned into digits)
// ---------------------------------------------------------------------------------------------------------
const UNKNOWN: &str = r"(?:the same number|a certain number|a number|the number|my number|this number|that number|some number|x)";
const VERB: &str = r"(?:add|subtract|take away|multiply|divide|increase|decrease|reduce|double|triple|halve|minus|plus|increased|decreased|reduced|multiplied|divided|subtracted|added|doubled|tripled|halved|then)";

static CHAIN_SPLIT: LazyLock<Re> =
    LazyLock::new(|| re(&format!(r"\s*(?:,\s*(?:and\s+)?(?:then\s+)?|\s+and\s+(?:then\s+)?|\s+then\s+)(?=(?:the result is\s+)?{}\b)", VERB)));
static P_LESS: LazyLock<Re> = LazyLock::new(|| re(r"^(.+?)\s+(?:less|fewer|smaller)\s+than\s+(.+)$"));
static P_MORE: LazyLock<Re> = LazyLock::new(|| re(r"^(.+?)\s+(?:more|greater|larger|bigger)\s+than\s+(.+)$"));
static P_SUBTRACTED: LazyLock<Re> = LazyLock::new(|| re(r"^(.+?)\s+(?:is\s+|are\s+)?subtracted\s+from\s+(.+)$"));
static P_ADDED: LazyLock<Re> = LazyLock::new(|| re(r"^(.+?)\s+(?:is\s+|are\s+)?added\s+to\s+(.+)$"));
static P_PLUS: LazyLock<Re> = LazyLock::new(|| re(r"\s+(?:plus|increased by|added with)\s+"));
static P_MINUS: LazyLock<Re> = LazyLock::new(|| re(r"\s+(?:minus|decreased by|reduced by|take away)\s+"));
static P_TIMES: LazyLock<Re> = LazyLock::new(|| re(r"\s+(?:times|multiplied by)\s+"));
static P_DIV: LazyLock<Re> = LazyLock::new(|| re(r"\s+(?:divided by|over)\s+"));
static P_OF: LazyLock<Re> = LazyLock::new(|| re(&format!(r"^({})\s+of\s+(.+)$", NUM_PAT)));
static P_TWICE: LazyLock<Re> = LazyLock::new(|| re(r"^(?:twice|double)\s+(.+)$"));
static P_TRIPLE: LazyLock<Re> = LazyLock::new(|| re(r"^triple\s+(.+)$"));
static P_SUMDIFF: LazyLock<Re> = LazyLock::new(|| re(r"^the\s+(sum|difference|product|quotient)\s+of\s+(.+?)\s+and\s+(.+)$"));
static P_MULDIV: LazyLock<Re> = LazyLock::new(|| re(r"^(?:multiply|divide)\s+(.+?)\s+by\s+(.+)$"));
static P_SUBTRACT_FROM: LazyLock<Re> = LazyLock::new(|| re(r"^subtract\s+(.+?)\s+from\s+(.+)$"));
static P_ADD_TO: LazyLock<Re> = LazyLock::new(|| re(r"^add\s+(.+?)\s+to\s+(.+)$"));
static P_UNKNOWN: LazyLock<Re> = LazyLock::new(|| re(UNKNOWN));
static P_NUM: LazyLock<Re> = LazyLock::new(|| re(NUM_PAT));
static E_PREFIX1: LazyLock<Re> = LazyLock::new(|| re(r"^(?:the result of|the value of|the answer to|the number that is|the number which is)\s+"));
static E_PREFIX2: LazyLock<Re> = LazyLock::new(|| re(r"^(?:the same as|equal to|exactly)\s+"));

/// Parse a phrase into a Lin, or fail.
fn expr(s: &str) -> Pr<Lin> {
    let s = strip(s, " ,.;");
    let s = E_PREFIX1.sub(&s, "");
    let s = E_PREFIX2.sub(&s, "");
    if s.is_empty() {
        return Err(NotAPuzzle);
    }
    let parts = CHAIN_SPLIT.split(&s);
    if parts.len() > 1 {
        return chain(&parts);
    }
    phrase(&s)
}

fn phrase(s: &str) -> Pr<Lin> {
    let s = strip(s, " ,.;");
    let s = s.as_str();
    // "A less than B" -> B - A   ,   "A more than B" -> B + A
    if let Some(m) = P_LESS.matches(s) {
        return Ok(expr(m.g(2))?.sub(expr(m.g(1))?));
    }
    if let Some(m) = P_MORE.matches(s) {
        return Ok(expr(m.g(2))?.add(expr(m.g(1))?));
    }
    // "A subtracted from B" -> B - A,  "A added to B" -> B + A    (also passive: "4 is subtracted from a number")
    if let Some(m) = P_SUBTRACTED.matches(s) {
        return Ok(expr(m.g(2))?.sub(expr(m.g(1))?));
    }
    if let Some(m) = P_ADDED.matches(s) {
        return Ok(expr(m.g(2))?.add(expr(m.g(1))?));
    }
    // left-to-right additive operators: split at the LAST one
    let mut best: Option<(usize, usize, i32)> = None; // (start, end, sign)
    for (rx, sign) in [(&*P_PLUS, 1), (&*P_MINUS, -1)] {
        for m in rx.find_all(s) {
            if best.map_or(true, |b| m.start() > b.0) {
                best = Some((m.start(), m.end(), sign));
            }
        }
    }
    if let Some((start, end, sign)) = best {
        let left = expr(&s[..start])?;
        let right = expr(&s[end..])?;
        return Ok(if sign > 0 { left.add(right) } else { left.sub(right) });
    }
    // multiplicative operators: split at the LAST one
    let mut best: Option<(usize, usize, char)> = None;
    for (rx, op) in [(&*P_TIMES, '*'), (&*P_DIV, '/')] {
        for m in rx.find_all(s) {
            if best.map_or(true, |b| m.start() > b.0) {
                best = Some((m.start(), m.end(), op));
            }
        }
    }
    if let Some((start, end, op)) = best {
        let left = expr(&s[..start])?;
        let right = expr(&s[end..])?;
        return if op == '*' { left.mul(right) } else { left.div(right) };
    }
    // fractions of something: "1/2 of a number", "4/5 of it"
    if let Some(m) = P_OF.matches(s) {
        return constant(to_fraction(m.g(1))?).mul(expr(m.g(2))?);
    }
    // prefix forms
    if let Some(m) = P_TWICE.matches(s) {
        return constant(Q::int(2)).mul(expr(m.g(1))?);
    }
    if let Some(m) = P_TRIPLE.matches(s) {
        return constant(Q::int(3)).mul(expr(m.g(1))?);
    }
    if let Some(m) = P_SUMDIFF.matches(s) {
        let a = expr(m.g(2))?;
        let b = expr(m.g(3))?;
        return match m.g(1) {
            "sum" => Ok(a.add(b)),
            "difference" => Ok(a.sub(b)),
            "product" => a.mul(b),
            _ => a.div(b),
        };
    }
    if let Some(m) = P_MULDIV.matches(s) {
        let a = expr(m.g(1))?;
        let b = expr(m.g(2))?;
        return if s.starts_with("multiply") { a.mul(b) } else { a.div(b) };
    }
    if let Some(m) = P_SUBTRACT_FROM.matches(s) {
        return Ok(expr(m.g(2))?.sub(expr(m.g(1))?));
    }
    if let Some(m) = P_ADD_TO.matches(s) {
        return Ok(expr(m.g(2))?.add(expr(m.g(1))?));
    }
    // atoms
    if P_UNKNOWN.is_fullmatch(s) || s == "it" || s == "that" || s == "the result" {
        return Ok(X);
    }
    if P_NUM.is_fullmatch(s) {
        return Ok(constant(to_fraction(s)?));
    }
    Err(NotAPuzzle)
}

static S_RESULT_IS: LazyLock<Re> = LazyLock::new(|| re(r"^the result is\s+"));
static S_THEN: LazyLock<Re> = LazyLock::new(|| re(r"^then\s+"));
static S_ADD: LazyLock<Re> = LazyLock::new(|| re(&format!(r"^(?:add|plus|increased by|increase (?:it )?by|increase by|added)\s+({})(?:\s+to it)?$", NUM_PAT)));
static S_SUB: LazyLock<Re> = LazyLock::new(|| {
    re(&format!(r"^(?:subtract|take away|minus|decreased by|decrease (?:it )?by|reduced by|reduce (?:it )?by|subtracted)\s+({})(?:\s+from it)?$", NUM_PAT))
});
static S_MUL: LazyLock<Re> = LazyLock::new(|| re(&format!(r"^(?:multiply (?:it |the result )?by|multiplied by|times)\s+({})$", NUM_PAT)));
static S_DIV: LazyLock<Re> = LazyLock::new(|| re(&format!(r"^(?:divide (?:it |the result )?by|divided by)\s+({})$", NUM_PAT)));
static S_DOUBLE: LazyLock<Re> = LazyLock::new(|| re(r"(?:double it|doubled)"));
static S_TRIPLE: LazyLock<Re> = LazyLock::new(|| re(r"(?:triple it|tripled)"));
static S_HALVE: LazyLock<Re> = LazyLock::new(|| re(r"(?:halve it|halved)"));

/// One later step of a chain ("and add 5", "then increased by 9", "the result is multiplied by 5") on the running value.
fn apply_step(prev: Lin, step: &str) -> Pr<Lin> {
    let s = strip(step, " ,.;");
    let s = S_RESULT_IS.sub(&s, "");
    let s = S_THEN.sub(&s, "");
    let s = s.as_str();
    if let Some(m) = S_ADD.matches(s) {
        return Ok(prev.add(constant(to_fraction(m.g(1))?)));
    }
    if let Some(m) = S_SUB.matches(s) {
        return Ok(prev.sub(constant(to_fraction(m.g(1))?)));
    }
    if let Some(m) = S_MUL.matches(s) {
        return prev.mul(constant(to_fraction(m.g(1))?));
    }
    if let Some(m) = S_DIV.matches(s) {
        return prev.div(constant(to_fraction(m.g(1))?));
    }
    if S_DOUBLE.is_fullmatch(s) {
        return prev.mul(constant(Q::int(2)));
    }
    if S_TRIPLE.is_fullmatch(s) {
        return prev.mul(constant(Q::int(3)));
    }
    if S_HALVE.is_fullmatch(s) {
        return prev.mul(constant(Q::new(1, 2)));
    }
    Err(NotAPuzzle)
}

static C_IMPERATIVE: LazyLock<Re> = LazyLock::new(|| re(r"(?:double|triple|halve) it"));
static C_MULDIV_IT: LazyLock<Re> = LazyLock::new(|| re(&format!(r"^(multiply|divide)\s+it\s+by\s+({})$", NUM_PAT)));

fn chain(parts: &[String]) -> Pr<Lin> {
    let first = strip(&parts[0], " ,.;");
    // an imperative start on the unknown: "triple it", "multiply it by 6", "divide it by 7"
    let mut value = if C_IMPERATIVE.is_fullmatch(&first) || C_MULDIV_IT.matches(&first).is_some() {
        apply_step(X, &first)?
    } else {
        phrase(&first)?
    };
    for step in &parts[1..] {
        value = apply_step(value, step)?;
    }
    Ok(value)
}

// ---------------------------------------------------------------------------------------------------------
// Whole problems
// ---------------------------------------------------------------------------------------------------------
const RELATIONS: [&str; 11] = [
    r"\s+is the same as\s+",
    r"\s+is equal to\s+",
    r"\s+you(?:'ll| will)? get\s+",
    r"\s+gives\s+",
    r"\s+equals\s+",
    r"\s+is\s+",
    r"\s+are\s+",
    r"\s+makes\s+",
    r"\s+results in\s+",
    r"\s+comes to\s+",
    r"\s+the same as\s+",
];
static RELATION_RES: LazyLock<Vec<Re>> = LazyLock::new(|| RELATIONS.iter().map(|p| re(p)).collect());
static SKIP_SENTENCE: LazyLock<Re> = LazyLock::new(|| re(r"^(?:find|what|which|how|solve|give|list|determine|calculate|tell|i'?m thinking|i am thinking|think of)\b"));
static CS_IF: LazyLock<Re> = LazyLock::new(|| re(r"^(?:if|when|suppose|given that|after)\s+"));
static CS_YOU_I: LazyLock<Re> = LazyLock::new(|| re(r"^(?:you|i)\s+(?=(?:add|subtract|multiply|divide|double|triple|halve|take))"));
static CS_YOU: LazyLock<Re> = LazyLock::new(|| re(r"^you\s+"));

fn clean_sentence(sent: &str) -> String {
    let s = sent.trim().to_lowercase().replace('\u{2019}', "'");
    let s = CS_IF.sub(&s, "");
    let s = CS_YOU_I.sub(&s, "");
    let s = CS_YOU.sub(&s, "");
    words_to_digits(&s)
}

static SE_PRE: LazyLock<Vec<Re>> = LazyLock::new(|| {
    vec![
        re(r",\s*(?:then\s+)?(?:you\s+(?:will\s+)?get|the (?:answer|result) is|it (?:is|equals|gives))\s+"),
        re(r"\s+(?:you\s+(?:will\s+)?get|the (?:answer|result) is)\s+"),
    ]
});
static SE_SAME: LazyLock<Re> = LazyLock::new(|| re(r"\s+you(?:'ll| will)? get (?:the same as|exactly the same as)\s+"));
static SE_COMMA_IS: LazyLock<Re> = LazyLock::new(|| re(r"\s*,\s*(?:is|gives)\s+"));

/// Every place the sentence could be split into two sides: (left text, right text).
fn split_equation(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for pre in SE_PRE.iter() {
        for m in pre.find_all(s) {
            out.push((s[..m.start()].to_string(), s[m.end()..].to_string()));
        }
    }
    for m in SE_SAME.find_all(s) {
        out.push((s[..m.start()].to_string(), s[m.end()..].to_string()));
    }
    for rx in RELATION_RES.iter() {
        for m in rx.find_all(s) {
            out.push((s[..m.start()].to_string(), s[m.end()..].to_string()));
        }
    }
    if let Some(m) = SE_COMMA_IS.search(s) {
        out.push((s[..m.start()].to_string(), s[m.end()..].to_string()));
    }
    out
}

static SOLVE_NUMBER: LazyLock<Re> = LazyLock::new(|| re(r"\bnumber\b"));
static SOLVE_THINK: LazyLock<Re> = LazyLock::new(|| re(r"\bthink of\b|\bit\b"));
static SOLVE_BAD: LazyLock<Re> = LazyLock::new(|| re(r"\b(numbers|digits?|consecutive|integers|squared|square|cubed|reciprocal|percent|%|\$)\b"));
static SOLVE_LETTER_DIGIT: LazyLock<Re> = LazyLock::new(|| re(r"[a-z]\d|\d[a-z]"));
static SOLVE_SENTENCES: LazyLock<Re> = LazyLock::new(|| re(r"(?<=[.!?])\s+"));
static SOLVE_AND_END: LazyLock<Re> = LazyLock::new(|| re(r"[\s,]*\band\s*$"));

/// The puzzle's answer for a plain one-unknown number puzzle, otherwise None.
pub fn solve(problem: &str) -> Option<Puzzle> {
    let text = problem.trim();
    let low = text.to_lowercase();
    if !SOLVE_NUMBER.is_match(&low) && !SOLVE_THINK.is_match(&low) {
        return None;
    }
    if SOLVE_BAD.is_match(&low) || SOLVE_LETTER_DIGIT.is_match(&low.replace("1st", "")) {
        return None;
    }
    let sentences: Vec<String> = SOLVE_SENTENCES.split(text).into_iter().filter(|x| !x.trim().is_empty()).collect();
    let mut found: Vec<Puzzle> = Vec::new();
    for sent in &sentences {
        if sent.trim().ends_with('?') {
            continue;
        }
        let s = clean_sentence(sent.trim_end_matches(|c| ".!? ".contains(c)));
        if SKIP_SENTENCE.matches(&s).is_some() {
            continue;
        }
        for (left, right) in split_equation(&s) {
            let left = SOLVE_AND_END.sub(&left, "");
            let (lhs, rhs) = match (expr(&left), expr(&right)) {
                (Ok(l), Ok(r)) => (l, r),
                _ => continue,
            };
            let denom = lhs.a - rhs.a;
            if denom.is_zero() || (lhs.a.is_zero() && rhs.a.is_zero()) {
                continue;
            }
            let answer = (rhs.b - lhs.b) / denom;
            let shown = sent.trim().trim_end_matches(|c| ".!? ".contains(c)).to_string();
            found.push(Puzzle {
                answer,
                equation: format!("{} = {}", show(lhs), show(rhs)),
                reading: format!("{}  ->  {} = {}", shown, show(lhs), show(rhs)),
            });
            break;
        }
    }
    if found.len() != 1 {
        return None;
    }
    found.pop()
}

pub fn answer_text(p: &Puzzle) -> String {
    show_fraction(p.answer)
}
