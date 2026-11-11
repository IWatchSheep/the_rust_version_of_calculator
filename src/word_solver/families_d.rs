//! Families 52-58 of `word_solver.py` (transversal angles ... complementary / supplementary), in Python's FAMILIES order.

use super::common::*;
use super::families_b::rfind_max;
use super::rx::re;
use crate::lazy_re;
use std::collections::BTreeSet;

fn qi(n: i128) -> Q {
    Q::new(n, 1)
}

fn bare_sentences(t: &str) -> Vec<String> {
    lazy_re!(BARE, r"[.?!]");
    BARE.split(t).into_iter().collect()
}

lazy_re!(LETTERS_APOS, r"[a-z']+");

fn words_in(text: &str) -> BTreeSet<String> {
    LETTERS_APOS.find_all(text).iter().map(|m| m.text().to_string()).collect()
}

fn word_list(text: &str) -> Vec<String> {
    LETTERS_APOS.find_all(text).iter().map(|m| m.text().to_string()).collect()
}

// ---------------------------------------------------------------------------------------------------------
// 52. parallel lines and a transversal: corresponding / alternate interior / alternate exterior angles are equal
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TRANS_KIND, r"\b(corresponding|alternate interior|alternate exterior)\s+angles\b");
lazy_re!(
    TRANS_OTHER,
    r"\b(?:same[- ]side|consecutive|co-?interior|supplementary|complementary|adjacent|linear pair|straight line|add to|adds to|sum|total|together|triangle|difference|product|ratio|half|double|twice|bisect\w*|quadrilateral|cyclic|inscribed|circle|arc|chord|polygon|exterior angle of)\b"
);
lazy_re!(TWO_PARALLEL, r"\btwo parallel lines\b");
lazy_re!(PARALLEL_LINES, r"\bparallel lines\b");
lazy_re!(TRANSVERSAL, r"\btransversal\b");
lazy_re!(TR_EXPR, r"\(\s*(\d*)\s*x\s*([+-])\s*(\d+(?:\.\d+)?)\s*\)\s*(?:°|degrees?)?");
lazy_re!(ANGLES_WORD, r"\bangles?\b");
lazy_re!(BOTH_ANGLES, r"\bboth angles\b");
lazy_re!(X_WORD, r"\bx\b");

fn sign_word(b: Q) -> &'static str {
    if b >= Q::ZERO {
        "+"
    } else {
        "-"
    }
}

pub fn transversal_equal_angles(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if CHANGES_EQ_RE.is_match(&t) || TRANS_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&TWO_PARALLEL.sub(&t, " ")) {
        return None;
    }
    if !PARALLEL_LINES.is_match(&t) || !TRANSVERSAL.is_match(&t) {
        return None;
    }
    let kinds = TRANS_KIND.find_all(&t);
    if kinds.len() != 1 {
        return None;
    }
    let kind = kinds[0].g(1).to_string();
    let ex = TR_EXPR.find_all(&t);
    if ex.len() != 2 {
        return None;
    }
    let rest = format!("{} {} {}", &t[..ex[0].start()], &t[ex[0].end()..ex[1].start()], &t[ex[1].end()..]);
    if DIGIT_RE.is_match(&rest) {
        return None;
    }
    let coef = |m: &super::rx::M| {
        let a = match m.opt(1) {
            Some(s) if !s.is_empty() => fr(s),
            _ => Q::ONE,
        };
        let b = fr(m.g(3)) * if m.g(2) == "+" { Q::ONE } else { -Q::ONE };
        (a, b)
    };
    let ((a1, b1), (a2, b2)) = (coef(&ex[0]), coef(&ex[1]));
    if a1 == a2 {
        return None;
    }
    let x = (b2 - b1) / (a1 - a2);
    let angle = a1 * x + b1;
    if !(Q::ZERO < angle && angle <= Q::int(180)) {
        return None;
    }
    let q = rfind_max(&t, &["find", "what", "how"])?;
    let tail = &t[q..];
    let asks_angle = ANGLES_WORD.is_match(tail) && !BOTH_ANGLES.is_match(tail);
    let asks_x = X_WORD.is_match(tail);
    if !(asks_angle || asks_x) {
        return None;
    }
    let mut vals = vec![];
    if asks_x {
        vals.push(x);
    }
    if asks_angle {
        vals.push(angle);
    }
    let eq = format!("{}x {} {} = {}x {} {}", show(a1), sign_word(b1), show(b1.abs()), show(a2), sign_word(b2), show(b2.abs()));
    res(
        "transversal_equal_angles",
        vals,
        eq.clone(),
        format!("{} angles are equal: {}, so x = {} and each angle is {} degrees", kind, eq, show_exact(x), show_exact(angle)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 53. a class ratio after more students join:   b k / (g k + n) = p / q
// ---------------------------------------------------------------------------------------------------------
lazy_re!(JOIN_RATIO, r"\bratio of ([a-z]+) to ([a-z]+)(?: [a-z ]*?)? is (\d+)\s*:\s*(\d+)");
lazy_re!(JOIN_AFTER, r"\bafter (\d+) more ([a-z]+) (?:join|joins|joined|arrive|arrives|arrived|enrol|enroll|enrolls|enrolled)\b");
lazy_re!(JOIN_NEW, r"\b(?:the )?ratio (?:becomes|changes to|is then|is now|will be|then is)\s+(\d+)\s*:\s*(\d+)");
lazy_re!(JOIN_OTHER, r"\b(?:leave|leaves|left|drop out|dropped out|transfer\w*|absent|twice|half|double|total of|altogether|together|each)\b|%|\$");
lazy_re!(RATIO_OF, r"\bratio of\b");
lazy_re!(HOW_MANY, r"\bhow many\b");
lazy_re!(JOIN_ORIG, r"\b(?:originally|at first|to begin with|in the beginning)\b");
lazy_re!(JOIN_NOW, r"\b(?:now|currently|today|afterwards|after that|finally|at the end)\b");
lazy_re!(JOIN_HOW_MANY_OF, r"\bhow many (?:of the )?([a-z]+)");

pub fn class_ratio_after_join(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 5 || CHANGES_EQ_RE.is_match(&t) || JOIN_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let rm = JOIN_RATIO.search(&t)?;
    let am = JOIN_AFTER.search(&t)?;
    let nm = JOIN_NEW.search(&t)?;
    if RATIO_OF.find_all(&t).len() != 1 {
        return None;
    }
    let (xs, ys) = (rm.g(1), rm.g(2));
    let (b, g) = (fr(rm.g(3)), fr(rm.g(4)));
    let (n, joined) = (fr(am.g(1)), am.g(2));
    let (p, q) = (fr(nm.g(1)), fr(nm.g(2)));
    if xs == ys || [b, g, n, p, q].iter().any(|v| *v <= Q::ZERO) {
        return None;
    }
    let (k, eq);
    if joined == ys {
        let denom = b * q - g * p;
        if denom.is_zero() {
            return None;
        }
        k = n * p / denom;
        eq = format!("{}k / ({}k + {}) = {}/{}", show(b), show(g), show(n), show(p), show(q));
    } else if joined == xs {
        let denom = p * g - q * b;
        if denom.is_zero() {
            return None;
        }
        k = q * n / denom;
        eq = format!("({}k + {}) / {}k = {}/{}", show(b), show(n), show(g), show(p), show(q));
    } else {
        return None;
    }
    let (bb, gg) = (b * k, g * k);
    if k <= Q::ZERO || !bb.is_int() || !gg.is_int() || bb <= Q::ZERO || gg <= Q::ZERO {
        return None;
    }
    let asking: Vec<String> = bare_sentences(&t).into_iter().filter(|x| HOW_MANY.is_match(x)).collect();
    if asking.len() != 1 || HOW_MANY.find_all(&t).len() != 1 || !JOIN_ORIG.is_match(&asking[0]) || JOIN_NOW.is_match(&asking[0]) {
        return None;
    }
    let qm = JOIN_HOW_MANY_OF.search(&asking[0])?;
    if !["students", "pupils", "children", "people", "members", "kids", "learners"].contains(&qm.g(1)) {
        return None;
    }
    let total = bb + gg;
    res(
        "class_ratio_after_join",
        vec![total],
        eq.clone(),
        format!("{}, so k = {}: {} {} and {} {} at first, {} in all", eq, show_exact(k), show_exact(bb), xs, show_exact(gg), ys, show_exact(total)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 54. v = u + a t:   how long to go from u to v at acceleration a
// ---------------------------------------------------------------------------------------------------------
lazy_re!(KIN_CARET2, r"\^\s*2\b");
lazy_re!(KIN_FROM, r"\bfrom\s+(\d+(?:\.\d+)?)\s*([a-z]+)/([a-z]+)\s+to\s+(\d+(?:\.\d+)?)\s*([a-z]+)/([a-z]+)");
lazy_re!(KIN_ACC, r"\bacceleration(?: of)?\s+(\d+(?:\.\d+)?)\s*([a-z]+)/([a-z]+)(?:²|\^2)");
lazy_re!(KIN_OTHER, r"\b(?:deceleration|decelerat\w*|slows?|brakes?|braking|stops?|then)\b|%|\$");
lazy_re!(KIN_Q, r"\bhow (?:long|many (seconds|hours|minutes)|much time)\b");

pub fn kinematics_time(problem: &str) -> Option<SolveResult> {
    let problem = KIN_CARET2.sub(problem, "²");
    let problem = problem.as_str();
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&t) || KIN_OTHER.is_match(&t) {
        return None;
    }
    let fm = KIN_FROM.search(&t)?;
    let am = KIN_ACC.search(&t)?;
    let (u, v, a) = (fr(fm.g(1)), fr(fm.g(4)), fr(am.g(1)));
    if !(fm.g(2) == fm.g(5) && fm.g(5) == am.g(2) && fm.g(3) == fm.g(6) && fm.g(6) == am.g(3)) {
        return None;
    }
    let tname = match fm.g(3) {
        "s" => "second",
        "h" => "hour",
        "min" => "minute",
        "hr" => "hour",
        _ => return None,
    };
    let qm = KIN_Q.search(&t)?;
    if let Some(g) = qm.opt(1) {
        let want = match g {
            "seconds" => "second",
            "hours" => "hour",
            _ => "minute",
        };
        if want != tname {
            return None;
        }
    }
    if a <= Q::ZERO {
        return None;
    }
    let tt = (v - u) / a;
    if tt <= Q::ZERO {
        return None;
    }
    let eq = format!("{} + {}t = {}", show(u), show(a), show(v));
    res(
        "kinematics_time",
        vec![tt],
        eq.clone(),
        format!("{}, so t = ({} - {}) / {} = {} {}{}", eq, show(v), show(u), show(a), show_exact(tt), tname, if tt == Q::ONE { "" } else { "s" }),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 55. pipes that fill and a drain that empties:   1/a + 1/b - 1/c per hour
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PIPE_IN, r"\bin\s+(\d+(?:\.\d+)?)\s+(hours?|minutes?|days?)\b");
lazy_re!(PIPE_FILL_WORD, r"\b(?:pipe|fills?|inlet|tap|faucet|hose)\b");
lazy_re!(PIPE_DRAIN_WORD, r"\b(?:drain|drains|empties|empty|leak\w*|outlet)\b");
lazy_re!(ALL_THREE, r"\ball three\b");
lazy_re!(PIPE_BAD1, r"\b(?:then|after|alone|only|closed|shut|half|twice|%)\b|\$");
lazy_re!(
    PIPE_BAD2,
    r"\b(?:not|no|never|neither|nor|except|without|off|stopped|stops?|broken|blocked|clogged|jammed|out of order|sealed|capped|plugged|stoppered|corked|locked|bolted|disabled|inactive|covered|but|however|although|though|yet|unless)\b|n't\b"
);
lazy_re!(PIPE_HOW_LONG, r"\bhow long\b");
lazy_re!(PIPE_FILL_Q, r"\bhow long\b[^.?!]*\bfill\b");
lazy_re!(PIPE_EMPTY_Q, r"\bhow long\b[^.?!]*\bempty\b");
lazy_re!(PIPE_FILL_TAIL, r"(?s)\bfill\b(.*)$");

const PIPE_FILLER: &[&str] = &["if", "when", "with", "then", "together", "now", "at", "the", "same", "time", "once", "all", "both", "are", "is", "be"];
const PIPE_HANG_ON: &[&str] = &[
    "if", "when", "while", "as", "with", "and", "also", "too", "but", "since", "because", "once", "after", "before", "until", "plus", "besides", "even", "still", "in", "through", "from", "into", "by",
];
const PIPE_VESSEL: &[&str] = &["tank", "pool", "tub", "bathtub", "cistern", "basin", "reservoir", "barrel", "drum", "bucket", "vat", "trough", "pond", "container", "bath", "sink", "pot"];
const PIPE_ENDING: &[&str] = &[
    "up", "it", "them", "the", "a", "an", "this", "that", "empty", "whole", "entire", "completely", "fully", "entirely", "one", "all", "swimming", "water", "storage", "big", "large", "small",
];
const PIPE_ASK: &[&str] = &[
    "how", "long", "does", "do", "did", "will", "would", "should", "can", "could", "it", "they", "them", "take", "takes", "to", "for", "the", "a", "an", "this", "that", "all", "up", "empty", "whole",
    "entire", "tank", "pool", "tub", "bathtub", "cistern", "basin", "reservoir", "barrel", "drum", "bucket", "vat", "trough", "pond", "container", "bath", "sink", "pot",
];

fn outside(set: &BTreeSet<String>, allowed: &[&str]) -> bool {
    set.iter().any(|w| !allowed.contains(&w.as_str()))
}

pub fn pipes_and_drain(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    let occ = PIPE_IN.find_all(&t);
    if occ.len() < 3 || nums.len() != occ.len() || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&ALL_THREE.sub(&t, " ")) || PIPE_BAD1.is_match(&t) {
        return None;
    }
    let units: BTreeSet<&str> = occ.iter().map(|m| m.g(2).trim_end_matches('s')).collect();
    if units.len() != 1 {
        return None;
    }
    let (mut fills, mut drains): (Vec<Q>, Vec<Q>) = (vec![], vec![]);
    let mut prev = 0;
    for m in &occ {
        let seg = &t[prev..m.start()];
        prev = m.end();
        let (is_drain, is_fill) = (PIPE_DRAIN_WORD.is_match(seg), PIPE_FILL_WORD.is_match(seg));
        if is_drain == is_fill {
            return None;
        }
        if is_drain {
            drains.push(fr(m.g(1)));
        } else {
            fills.push(fr(m.g(1)));
        }
    }
    if fills.is_empty() || drains.is_empty() || fills.iter().chain(drains.iter()).any(|v| *v <= Q::ZERO) {
        return None;
    }
    let devices = r"(?:all three|all of them|everything|(?:both |all )?(?:the )?(?:pipes|taps) and (?:the )?(?:drain|leak|outlet))";
    let state = r"(?:open|opened|running|working|flowing)";
    let cue_src = format!(r"{devices}\s+(?:(?:are|is|being|stay|stays|remain|remains)\s+)?(?:all\s+)?{state}\b");
    let cue_phrase = re(&cue_src);
    let bare_all = re(&format!(r"(?:^|[.?!]\s*|\bwith\s+)all {state}\b(?=\s*(?:[,.;?!]|$|\b(?:how|what|then|and)\b))"));
    if !(cue_phrase.is_match(&t) || bare_all.is_match(&t)) {
        return None;
    }
    let lead = r"(?:\b(?:with|if|when|once)\s+)?";
    let lead_cue = re(&format!("{lead}{cue_src}"));
    let lead_all = re(&format!(r"(?:^|{lead})\s*all {state}\b"));
    for sent in bare_sentences(&t) {
        if sent.trim().is_empty() {
            continue;
        }
        let times = PIPE_IN.find_all(&sent);
        if let Some(last) = times.last() {
            if !sent[last.end()..].trim_matches(|c| c == ' ' || c == ',' || c == ';').is_empty() {
                return None;
            }
        } else {
            let opened = lead_cue.sub(&sent, " ");
            let opened = lead_all.sub(&opened, " ");
            let q = PIPE_HOW_LONG.search(&opened).map(|m| m.start());
            if opened == sent && q.is_none() {
                return None;
            }
            let before_q = match q {
                Some(i) => &opened[..i],
                None => &opened[..],
            };
            if outside(&words_in(before_q), PIPE_FILLER) {
                return None;
            }
            if let Some(i) = q {
                if words_in(&opened[i..]).iter().any(|w| PIPE_HANG_ON.contains(&w.as_str())) {
                    return None;
                }
                if let Some(fm) = PIPE_FILL_TAIL.search(&opened[i..]) {
                    if outside(&words_in(&opened[i..i + fm.start()]), PIPE_ASK) {
                        return None;
                    }
                    let tail = word_list(fm.g(1));
                    let named = tail.iter().position(|w| PIPE_VESSEL.contains(&w.as_str()));
                    let after: BTreeSet<String> = match named {
                        Some(k) => tail[k + 1..].iter().cloned().collect(),
                        None => tail.iter().cloned().collect(),
                    };
                    if outside(&after, PIPE_ENDING) {
                        return None;
                    }
                }
            }
        }
        if PIPE_DRAIN_WORD.is_match(&sent) && !PIPE_IN.is_match(&sent) {
            if !cue_phrase.is_match(&sent) || PIPE_DRAIN_WORD.is_match(&cue_phrase.sub(&sent, " ")) {
                return None;
            }
        }
    }
    if PIPE_BAD2.is_match(&t) {
        return None;
    }
    if !PIPE_FILL_Q.is_match(&t) || PIPE_EMPTY_Q.is_match(&t) {
        return None;
    }
    let mut rate = Q::ZERO;
    for f in &fills {
        rate = rate + Q::ONE / *f;
    }
    for d in &drains {
        rate = rate - Q::ONE / *d;
    }
    if rate <= Q::ZERO {
        return None;
    }
    let tt = Q::ONE / rate;
    let unit = occ[0].g(2).trim_end_matches('s');
    let mut eq = fills.iter().map(|f| format!("1/{}", show(*f))).collect::<Vec<_>>().join(" + ");
    for d in &drains {
        eq.push_str(&format!(" - 1/{}", show(*d)));
    }
    res(
        "pipes_and_drain",
        vec![tt],
        format!("{} = {}", eq, show_exact(rate)),
        format!("{} = {} of the pool per {}, so it takes 1 / {} = {} {}s", eq, show_exact(rate), unit, show_exact(rate), show_exact(tt), unit),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 56. a percent spent, with money or fuel left:   x - p% x = left
// ---------------------------------------------------------------------------------------------------------
const SPENT_AT_FIRST: &str = r"(?: at first| originally| to begin with)?";
lazy_re!(
    SPENT_MONEY,
    &format!(r"^[a-z]+ spent {NUM}% of (?:a|the|his|her|their|my|our) ([a-z]+)(?: on [a-z]+)? and (?:had|has|have) \${NUM} left\. how much was the \2{SPENT_AT_FIRST}\?$")
);
lazy_re!(
    SPENT_FUEL,
    &format!(
        r"^after (?:using|burning) {NUM}% of the ([a-z]+) in (?:a|the) ([a-z]+), {NUM} (?:liters?|litres?|gallons?) (?:remain|remains|remained|are left|were left)\. how much \2 was there{SPENT_AT_FIRST}\?$"
    )
);

pub fn percent_spent_left(problem: &str) -> Option<SolveResult> {
    let t = clean(problem).trim().to_string();
    if n_numbers(problem) != 2 || t.matches('%').count() != 1 {
        return None;
    }
    let (pct, left) = if let Some(m) = SPENT_MONEY.matches(&t) {
        (fr(m.g(1)), fr(m.g(3)))
    } else {
        let m = SPENT_FUEL.matches(&t)?;
        (fr(m.g(1)), fr(m.g(4)))
    };
    if !(Q::ZERO < pct && pct < Q::int(100)) || left <= Q::ZERO {
        return None;
    }
    let rest = Q::int(100) - pct;
    let x = left * Q::int(100) / rest;
    let eq = format!("x - {}% of x = {}", show(pct), show(left));
    res("percent_spent_left", vec![x], eq, format!("{}% was not spent, so x = {} / {}% = {}", show_exact(rest), show(left), show_exact(rest), show_exact(x)))
}

// ---------------------------------------------------------------------------------------------------------
// 57. a price times a count, minus a coupon:   p n - c = paid
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    COUPON,
    &format!(r"^([a-z]+(?: [a-z]+){{0,3}}) cost \${NUM} each\. with a \${NUM} coupon, [a-z]+ paid \${NUM}\. how many \1 were bought\?$")
);

pub fn price_minus_coupon(problem: &str) -> Option<SolveResult> {
    let t = clean(problem).trim().to_string();
    let m = COUPON.matches(&t)?;
    if n_numbers(problem) != 3 {
        return None;
    }
    let (price, coupon, paid) = (fr(m.g(2)), fr(m.g(3)), fr(m.g(4)));
    if price <= Q::ZERO || coupon <= Q::ZERO || paid <= Q::ZERO || coupon > paid {
        return None;
    }
    let n = (paid + coupon) / price;
    if !n.is_int() || n <= Q::ZERO {
        return None;
    }
    let eq = format!("{}n - {} = {}", show(price), show(coupon), show(paid));
    res("price_minus_coupon", vec![n], eq.clone(), format!("{}, so {}n = {} and n = {}", eq, show(price), show(paid + coupon), show_exact(n)))
}

// ---------------------------------------------------------------------------------------------------------
// 58. complementary and supplementary angles:   x + (k x +- c) = 90 or 180
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    COMPSUPP,
    &format!(
        r"^two angles are (complementary|supplementary)\. one angle is (?:{NUM} (more|less) than )?{NUM} times the other(?: angle)?\. (?:find (?:both|the two) angles\.?|what are (?:both|the two) angles\?)$"
    )
);

pub fn complementary_supplementary(problem: &str) -> Option<SolveResult> {
    let t = clean(problem).trim().to_string();
    let m = COMPSUPP.matches(&t)?;
    if n_numbers(problem) != if m.opt(2).is_some() { 2 } else { 1 } {
        return None;
    }
    let total = qi(if m.g(1) == "complementary" { 90 } else { 180 });
    let c = match m.opt(2) {
        Some(s) => fr(s) * if m.g(3) == "more" { Q::ONE } else { -Q::ONE },
        None => Q::ZERO,
    };
    let k = fr(m.g(4));
    if k <= Q::ZERO {
        return None;
    }
    let x = (total - c) / (k + Q::ONE);
    let other = k * x + c;
    if x <= Q::ZERO || other <= Q::ZERO {
        return None;
    }
    let eq = format!("x + ({}x {} {}) = {}", show(k), if c < Q::ZERO { "-" } else { "+" }, show(c.abs()), show(total));
    res(
        "complementary_supplementary",
        vec![x, other],
        eq.clone(),
        format!("{}, so {}x = {} and x = {}; the angle described is {} degrees", eq, show_exact(k + Q::ONE), show(total - c), show_exact(x), show_exact(other)),
    )
}

pub fn list() -> Vec<(&'static str, Family)> {
    vec![
        ("transversal_equal_angles", transversal_equal_angles),
        ("class_ratio_after_join", class_ratio_after_join),
        ("kinematics_time", kinematics_time),
        ("pipes_and_drain", pipes_and_drain),
        ("percent_spent_left", percent_spent_left),
        ("price_minus_coupon", price_minus_coupon),
        ("complementary_supplementary", complementary_supplementary),
    ]
}
