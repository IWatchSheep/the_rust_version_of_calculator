//! Families 16-36 of `word_solver.py` (toward each other ... shared fixed cost), in Python's FAMILIES order.

use super::common::*;
use super::families_a::{dist_group, speed_group, subject_tokens, unit_name, ARTICLES, ASKS_OTHER, ASKS_TIME, CATCH_CUE, DIST, SPEED, TWO_VEHICLES};
use super::rx::{re, Re};
use crate::lazy_re;
use std::collections::BTreeSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------------------
// 16. walking / driving toward each other:   (s1 + s2) t = D
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TOWARD, r"\b(?:toward|towards|meet|meets)\b");
lazy_re!(
    TOWARD_BAD,
    r"\b(?:opposite|away|same direction|later|earlier|before|head start|ahead|behind|follow\w*|catch\w*|chase\w*|round trip|return\w*|back|then|current|stream|wind|stop\w*|rest\w*|break)\b"
);

pub fn toward_each_other(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || digit_numbers(problem).len() != 3 {
        return None;
    }
    if !TOWARD.is_match(&t) || TOWARD_BAD.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&TWO_VEHICLES.sub(&t, " ")) {
        return None;
    }
    let speeds = SPEED.find_all(&t);
    let dists = DIST.find_all(&t);
    if speeds.len() != 2 || dists.len() != 1 {
        return None;
    }
    let (g1, g2) = (speed_group(speeds[0].g(2)), speed_group(speeds[1].g(2)));
    if !(g1 == g2 && g2 == dist_group(dists[0].g(2))) {
        return None;
    }
    if !ASKS_TIME.is_match(&t) || ASKS_OTHER.is_match(&t) {
        return None;
    }
    let (s1, s2, d) = (fr(speeds[0].g(1)), fr(speeds[1].g(1)), fr(dists[0].g(1)));
    if s1 <= Q::ZERO || s2 <= Q::ZERO || d <= Q::ZERO {
        return None;
    }
    let tt = d / (s1 + s2);
    let eq = format!("({} + {})t = {}", show(s1), show(s2), show(d));
    res("toward_each_other", vec![tt], eq.clone(), format!("t = hours until they meet: {}, so t = {} hours", eq, show_exact(tt)))
}

// ---------------------------------------------------------------------------------------------------------
// 17. product of two consecutive (even / odd) integers:   x (x + step) = P
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PRODUCT_WORD, r"\b(?:product|multiply|multiplies|multiplied|times each other)\b");
lazy_re!(
    NEXT_WORD,
    r"\b(?:consecutive|one after another|one after the other|in a row|back to back|next to each other|following each other)\b"
);
lazy_re!(TWO_NUMBERS, r"\btwo\b(?= (?:(?:consecutive|even|odd|whole|positive|natural) )*(?:numbers|integers))");
lazy_re!(PAIR_OF, r"\bpair of\b");
lazy_re!(
    CP_BAD,
    r"\b(?:sum|add|adds|plus|difference|square|squares|squared|negative|larger|smaller|greater|less|average|mean|reciprocal|quotient|divide\w*)\b"
);
lazy_re!(CP_ASK, r"\b(?:what are|find|which|what numbers|what integers|name|list)\b");
lazy_re!(EVEN_RE, r"\beven\b");
lazy_re!(ODD_RE, r"\bodd\b");
lazy_re!(POSITIVE_RE, r"\bpositive\b");

pub fn consecutive_product(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 1 || nums[0].money || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) {
        return None;
    }
    let p = nums[0].v;
    if !p.is_int() || p <= Q::ZERO || t[nums[0].start..nums[0].end].contains('.') {
        return None;
    }
    if !TWO_NUMBERS.is_match(&t) && !PAIR_OF.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&TWO_NUMBERS.sub(&t, " ")) {
        return None;
    }
    if !PRODUCT_WORD.is_match(&t) || !NEXT_WORD.is_match(&t) || CP_BAD.is_match(&t) || !CP_ASK.is_match(&t) {
        return None;
    }
    let kind = if EVEN_RE.is_match(&t) {
        "even"
    } else if ODD_RE.is_match(&t) {
        "odd"
    } else {
        ""
    };
    let ws: Vec<&str> = t.split_whitespace().collect();
    if ws.contains(&"even") && ws.contains(&"odd") {
        return None;
    }
    let step: i128 = if kind.is_empty() { 1 } else { 2 };
    let root = super::q::isqrt(step * step + 4 * p.numer())?;
    let x2 = root - step;
    if x2.rem_euclid(2) != 0 {
        return None;
    }
    let x = Q::new(x2, 2);
    let xi = x.numer();
    if x < Q::ONE || (kind == "even" && xi.rem_euclid(2) != 0) || (kind == "odd" && xi.rem_euclid(2) != 1) {
        return None;
    }
    let st = Q::new(step, 1);
    let eq = format!("x(x + {}) = {}", step, show(p));
    let note = if POSITIVE_RE.is_match(&t) {
        String::new()
    } else {
        format!(" (the pair {} and {} also multiplies to {})", show_exact(-(x + st)), show_exact(-x), show(p))
    };
    res(
        "consecutive_product",
        vec![x, x + st],
        eq.clone(),
        format!("x = the smaller number: {}, so x = {} and the numbers are {} and {}{}", eq, show_exact(x), show_exact(x), show_exact(x + st), note),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 18. the most you can buy:   c n + f <= B, the greatest whole n
// ---------------------------------------------------------------------------------------------------------
lazy_re!(FEE_WORD, r"\b(?:fee|admission|entry|membership|delivery|shipping|deposit|cover charge|fixed|setup|registration)\b");
lazy_re!(
    BUDGET_BEFORE,
    r"\b(?:have|has|had|budget of|spend|spending|no more than|at most|up to|only|limit of|brought|bring|saved|allowance of|with)\s*(?:only\s+)?$"
);
lazy_re!(BUDGET_AFTER, r"^\s*(?:to spend|available|to use|budget|in total|total)\b");
lazy_re!(UNIT_AFTER, r"^\s*(?:(?:for|a)\s+)?(?:each|apiece|per\s+\w+)\b");
lazy_re!(UNIT_BEFORE, r"\b(?:cost|costs|costing|priced at|sell for|sells for|sold for|are|is|at)\s*$");
lazy_re!(
    BUY_OTHER,
    r"\b(?:discount\w*|coupon\w*|free|sale|bogo|off|tax|tip|bulk|every|additional|after the first|for the first|profit|least|fewest|minimum|must|how much|left|change|remain\w*|leftover|spare|altogether|total cost|loan|borrow\w*|interest|per month|a month|a week|per week)\b|%"
);
lazy_re!(BUY_MOST, r"\b(?:greatest|largest|most|maximum|max|biggest|highest)\b");
lazy_re!(BUY_CAN, r"\bhow many\b.*\b(?:can|could)\b.*\b(?:buy|afford|purchase|get)\b");
lazy_re!(BUY_BOUGHT, r"\bhow many\b.*\bcan be (?:bought|purchased)\b");
lazy_re!(BUY_WORD, r"\b(?:buy|bought|afford|purchase\w*|get|number|many)\b");
lazy_re!(BUY_OR, r"\bor\b|\beither\b");

pub fn rfind_max(t: &str, needles: &[&str]) -> Option<usize> {
    needles.iter().filter_map(|n| t.rfind(n)).max()
}

pub fn most_you_can_buy(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if !(nums.len() == 2 || nums.len() == 3) || !nums.iter().all(|n| n.money) || CHANGES_EQ_RE.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&t) || BUY_OTHER.is_match(&t.replace("no more than", " ").replace("at most", " ").replace("the most", " ")) {
        return None;
    }
    // the question: the greatest number you can buy (not money left over)
    let q = rfind_max(&t, &["what ", "how ", "which "])?;
    let tail = &t[q..];
    let asks_most = BUY_MOST.is_match(tail) || BUY_CAN.is_match(tail) || BUY_BOUGHT.is_match(tail);
    if !asks_most || !BUY_WORD.is_match(tail) {
        return None;
    }
    // "tickets at $12 or snacks at $4": two money amounts joined by "or" are a CHOICE, not a fee that must be paid
    if nums.windows(2).any(|w| BUY_OR.is_match(&t[w[0].end..w[1].start])) {
        return None;
    }
    let (mut fee, mut unit, mut budget): (Vec<Num>, Vec<Num>, Vec<Num>) = (vec![], vec![], vec![]);
    for n in &nums {
        let before = t[..n.start].trim_end();
        let after = &t[n.end..];
        let window = format!("{} {}", last_words(before, 3), after.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
        if FEE_WORD.is_match(&window) && UNIT_AFTER.matches(after).is_none() {
            fee.push(*n);
        } else if UNIT_AFTER.matches(after).is_some() {
            unit.push(*n);
        } else if BUDGET_BEFORE.is_match(before) || BUDGET_AFTER.matches(after).is_some() {
            budget.push(*n);
        } else if UNIT_BEFORE.is_match(before) {
            unit.push(*n);
        } else {
            return None;
        }
    }
    if unit.len() != 1 || budget.len() != 1 || fee.len() > 1 || nums.len() != 2 + fee.len() {
        return None;
    }
    let (b, c) = (budget[0].v, unit[0].v);
    let f = if fee.is_empty() { Q::ZERO } else { fee[0].v };
    if c <= Q::ZERO || b <= Q::ZERO || f < Q::ZERO {
        return None;
    }
    let n = ((b - f) / c).floor();
    let nq = Q::new(n, 1);
    if n < 1 || !(nq * c + f <= b && b < (nq + Q::ONE) * c + f) {
        return None;
    }
    let eq = if !f.is_zero() { format!("{}n + {} <= {}", show(c), show(f), show(b)) } else { format!("{}n <= {}", show(c), show(b)) };
    res(
        "most_you_can_buy",
        vec![nq],
        eq.clone(),
        format!(
            "n = the number bought: {}, so n <= {} and the most is {} (one more would cost ${}, over ${})",
            eq,
            show_exact((b - f) / c),
            show_exact(nq),
            show_exact((nq + Q::ONE) * c + f),
            show_exact(b)
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 19. still water and current
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    LEG,
    &format!(r"{NUM}\s*(km|kilometers?|kilometres?|miles?|mi|meters?|metres?)\b(?:\s+(?:downstream|upstream))?\s+in\s+{NUM}\s*(?:hours?\b|hrs?\b|h(?![a-z]))")
);
lazy_re!(DOWNSTREAM, r"\b(?:downstream|with the (?:current|stream|flow)|down the river|down river)\b");
lazy_re!(UPSTREAM, r"\b(?:upstream|against the (?:current|stream|flow)|against it|up the river|up river)\b");
lazy_re!(SW_BAD, r"\b(?:wind|air|plane|return\w*|back|round trip|stops?|rests?|drift\w*|minutes?|seconds?|lake|pond)\b");
lazy_re!(SENT_SPLIT, r"[.?!]\s*");
lazy_re!(SW_Q_BAD, r"\b(?:against|with the|downstream|upstream|how long|how far|how many)\b");
lazy_re!(SW_WANT_B, r"still water|speed of the (?:boat|canoe|rower|swimmer|kayak)|\b(?:boat|rowing|paddling|swimming)\s+speed\b");
lazy_re!(SW_CURRENT, r"\b(?:current|stream|flow)\b");
lazy_re!(SW_SPEED, r"\bspeed|\brate|how fast");

pub fn sentence_list(t: &str) -> Vec<String> {
    SENT_SPLIT.split(t).into_iter().filter(|s| !s.trim().is_empty()).collect()
}

pub fn still_water_current(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || digit_numbers(problem).len() != 4 {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&t) || SW_BAD.is_match(&t) {
        return None;
    }
    let legs = LEG.find_all(&t);
    if legs.len() != 2 || take(legs[0].g(2), 0, 2) != take(legs[1].g(2), 0, 2) {
        return None;
    }
    let segs = [&t[..legs[0].end()], &t[legs[0].end()..legs[1].end()]];
    let mut kinds: Vec<&str> = Vec::new();
    for seg in segs {
        let (d, u) = (DOWNSTREAM.is_match(seg), UPSTREAM.is_match(seg));
        if d == u {
            return None;
        }
        kinds.push(if d { "down" } else { "up" });
    }
    if kinds[0] == kinds[1] {
        return None;
    }
    let (mut vd, mut vu) = (Q::ZERO, Q::ZERO);
    for (leg, kind) in legs.iter().zip(&kinds) {
        let (d, tm) = (fr(leg.g(1)), fr(leg.g(3)));
        if d <= Q::ZERO || tm <= Q::ZERO {
            return None;
        }
        if *kind == "down" {
            vd = d / tm;
        } else {
            vu = d / tm;
        }
    }
    if vu <= Q::ZERO || vd <= vu {
        return None; // equal speeds (no current) or the "upstream" trip faster: decline
    }
    let two = Q::int(2);
    let (b, c) = ((vd + vu) / two, (vd - vu) / two);
    // what is asked: the speed in still water, the speed of the current, or both
    let sents = sentence_list(&t);
    let q = sents.last()?;
    if SW_Q_BAD.is_match(q) {
        return None;
    }
    let want_b = SW_WANT_B.is_match(q);
    let want_c = SW_CURRENT.is_match(q) && SW_SPEED.is_match(q);
    if !(want_b || want_c) {
        return None;
    }
    let unit = legs[0].g(2);
    let (mut vals, mut parts): (Vec<Q>, Vec<String>) = (vec![], vec![]);
    if want_b {
        vals.push(b);
        parts.push(format!("still water b = {}", show_exact(b)));
    }
    if want_c {
        vals.push(c);
        parts.push(format!("current c = {}", show_exact(c)));
    }
    let eq = format!("b + c = {};  b - c = {}", show_exact(vd), show_exact(vu));
    res(
        "still_water_current",
        vals,
        eq.clone(),
        format!("downstream speed = {}, upstream speed = {} ({} per hour): {}, so {}", show_exact(vd), show_exact(vu), unit, eq, parts.join(" and ")),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 20. a ratio split between two named substances, one amount known
// ---------------------------------------------------------------------------------------------------------
lazy_re!(RATIO, r"(?<![\d.])(\d+)\s*:\s*(\d+)(?!\d|\.\d)");
const NAME: &str = r"([a-z]+)";
static RATIO_NAMES: LazyLock<Vec<Re>> = LazyLock::new(|| {
    vec![
        re(&format!(r"\b{NAME} (?:is|are) (?:mixed|combined|blended|added|stirred|poured|diluted) (?:into|with|to|in) (?:the )?{NAME} in (?:the )?ratio")),
        re(&format!(r"\b{NAME} and {NAME}(?: [a-z]+)? (?:are )?(?:mixed |used |combined |blended )?in (?:the )?ratio")),
        re(&format!(r"\bratio of {NAME} to {NAME}\b")),
    ]
});
lazy_re!(
    RATIO_OTHER,
    r"\b(?:total|altogether|in all|whole|together|combined amount|times as|more|less|extra|remove\w*|dilut\w*|each|per|cost|price)\b|%|\$"
);
lazy_re!(RATIO_AMOUNT, &format!(r"\s*([a-z]+)\s+of\s+(?:the\s+)?{NAME}\b"));
lazy_re!(HOW_MUCH_MANY, r"\bhow (?:much|many)\b");

pub fn ratio_split(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&t) || RATIO_OTHER.is_match(&t) {
        return None;
    }
    let rm = RATIO.find_all(&t);
    if rm.len() != 1 {
        return None;
    }
    let (ra, rb) = (fr(rm[0].g(1)), fr(rm[0].g(2)));
    if ra <= Q::ZERO || rb <= Q::ZERO {
        return None;
    }
    let mut pair: Option<(String, String)> = None;
    for pat in RATIO_NAMES.iter() {
        if let Some(m) = pat.search(&t) {
            let found = (m.g(1).to_string(), m.g(2).to_string());
            if pair.as_ref().map_or(false, |p| *p != found) {
                return None;
            }
            pair = Some(found);
        }
    }
    let (x, y) = pair?;
    if x == y {
        return None;
    }
    // the known amount is attached to one of the two substances: "30 litres of water", "6 L of blue"
    let amount_n: Vec<Num> = nums.iter().filter(|n| !(n.end > rm[0].start() && n.start < rm[0].end())).copied().collect(); // the numbers that are not part of the ratio
    if amount_n.len() != 1 || amount_n[0].money {
        return None;
    }
    let am = RATIO_AMOUNT.matches(&t[amount_n[0].end..])?;
    if am.g(2) != x && am.g(2) != y {
        return None;
    }
    let known = am.g(2).to_string();
    let missing = if known == x { y.clone() } else { x.clone() };
    // the question must name the missing substance
    let sents = sentence_list(&t);
    let qs: Vec<&String> = sents.iter().filter(|s| HOW_MUCH_MANY.is_match(s)).collect();
    if qs.len() != 1 {
        return None;
    }
    let repeat = re(&format!(r"[\d.]+\s*[a-z]*\s+of\s+(?:the\s+)?{}\b", known));
    let asked = repeat.sub(qs[0], " "); // the known amount may be repeated in the question
    let has = |w: &str| re(&format!(r"\b{}\b", w)).is_match(&asked);
    if !has(&missing) || has(&known) {
        return None;
    }
    let (k_part, m_part) = if known == x { (ra, rb) } else { (rb, ra) };
    let amount = amount_n[0].v;
    let result = amount * m_part / k_part;
    let unit = am.g(1);
    res(
        "ratio_split",
        vec![result],
        format!("{} : {} = {} : {}", x, y, show(ra), show(rb)),
        format!("{} : {} = {} : {}, so {} = {} x {}/{} = {} {}", x, y, show(ra), show(rb), missing, show(amount), show(m_part), show(k_part), show_exact(result), unit),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 21. three numbers from their total and two pair sums
// ---------------------------------------------------------------------------------------------------------
const PAIR_WORDS: &str = r"(?:the first two|the last two|the first and (?:the )?second|the second and (?:the )?third|the first and (?:the )?(?:third|last)|the second and (?:the )?first)";
lazy_re!(
    TOTAL3,
    &format!(
        r"\b(?:the )?sum of (?:the )?three (?:numbers|integers) (?:is|equals|=) {NUM}|\bthree (?:numbers|integers) (?:add|sum|total|add up|sum up)(?: up)?(?: to| of)? {NUM}|\bthree (?:numbers|integers) have a sum of {NUM}"
    )
);
lazy_re!(PAIR3, &format!(r"({PAIR_WORDS})(?: numbers)?\s+(?:add|adds|sum|total|totals|make|makes|equal|equals|have a sum of|is|are)(?: up)?(?: to| of)?\s+{NUM}"));
lazy_re!(
    TN_BAD,
    r"\b(?:product|times|twice|double|triple|half|difference|more than|less than|greater|smaller|larger|average|mean|consecutive|ratio|square\w*|each other)\b"
);
lazy_re!(TN_STRIP, r"\bthree (?:numbers|integers)\b|\bthe (?:first|last) two\b");
lazy_re!(TN_ASK, r"\b(?:what|which|how|find|determine|name|list)\b");
lazy_re!(
    TN_ASK_OK,
    r"\b(?:what are|find|which|determine|name|list)\b.*\b(?:numbers|integers|them)\b|\bwhat are (?:they|the numbers)\b"
);

fn pair_index(p: &str) -> Option<(usize, usize)> {
    Some(match p {
        "the first two" | "the first and second" | "the first and the second" | "the second and first" | "the second and the first" => (0, 1),
        "the last two" | "the second and third" | "the second and the third" => (1, 2),
        "the first and third" | "the first and the third" | "the first and last" | "the first and the last" => (0, 2),
        _ => return None,
    })
}

pub fn three_numbers(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || TN_BAD.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&TN_STRIP.sub(&t, " ")) {
        return None;
    }
    let tot = TOTAL3.find_all(&t);
    let pairs = PAIR3.find_all(&t);
    if tot.len() != 1 || pairs.len() != 2 {
        return None;
    }
    let s = fr((1..=3).find_map(|g| tot[0].opt(g))?);
    let idx: Vec<(usize, usize)> = pairs.iter().map(|p| pair_index(p.g(1)).unwrap_or_else(|| decline())).collect();
    let set = |p: (usize, usize)| BTreeSet::from([p.0, p.1]);
    if set(idx[0]) == set(idx[1]) {
        return None;
    }
    let vals: Vec<Q> = pairs.iter().map(|p| fr(p.g(2))).collect();
    // the number left out of each pair is S minus the pair's sum
    let mut known: [Option<Q>; 3] = [None; 3];
    for ((i, j), v) in idx.iter().zip(&vals) {
        let k = (0..3).find(|x| x != i && x != j).unwrap();
        known[k] = Some(s - *v);
    }
    if known.iter().flatten().count() != 2 {
        return None;
    }
    let rest = (0..3).find(|x| known[*x].is_none()).unwrap();
    let sum_known = known.iter().flatten().fold(Q::ZERO, |a, b| a + *b);
    known[rest] = Some(s - sum_known);
    let out: Vec<Q> = known.iter().map(|k| k.unwrap()).collect();
    // a question, if there is one, must ask for the numbers
    for sent in sentence_list(&t) {
        if TN_ASK.is_match(&sent) && !TN_ASK_OK.is_match(&sent) {
            return None;
        }
    }
    let names = ["x", "y", "z"];
    let mut facts = vec![format!("x + y + z = {}", show(s))];
    for ((i, j), v) in idx.iter().zip(&vals) {
        facts.push(format!("{} + {} = {}", names[*i], names[*j], show(*v)));
    }
    res(
        "three_numbers",
        out.clone(),
        facts.join(";  "),
        format!("{}, so x = {}, y = {}, z = {}", facts.join(", "), show_exact(out[0]), show_exact(out[1]), show_exact(out[2])),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 23. a clock head start: "leaves at noon ... leaves at 2 pm ... when does the car catch the cyclist?"
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TIME_EXPR, r"\b(?:(noon)|(\d{1,2})\s*(?:pm|p\.m\.))(?![\w:])");
lazy_re!(
    CATCHER,
    r"\b(?:when|how long|how many hours)\b(?:\s+(?:does|do|did|will|would|can|could|is|until|after))*\s+(.*?)\s+(?:catch|catches|overtake|overtakes)\b"
);
lazy_re!(
    CLOCK_BAD,
    r"\b(?:later|earlier|head start|am|midnight|minutes?|half past|quarter|opposite|toward|towards|meet|meets|apart|return\w*|back|stops?|rests?|breaks?|detour|lunch|current|stream|wind|also|another)\b|\d:\d\d|a\.m\.|o'clock"
);
lazy_re!(CLOCK_WHEN, r"\b(?:when|how long|how many hours)\b");
lazy_re!(ALPHA, r"[a-z]+");
const QUESTION_STOP: &[&str] = &["it", "he", "she", "they", "him", "her", "them", "i", "we", "you", "up", "one"];

/// Hours after noon for "noon" (0) or "H pm" (H, with 12 pm = noon); None for anything else.
fn clock_hours(m: &super::rx::M) -> Option<Q> {
    if m.opt(1).is_some() {
        return Some(Q::ZERO);
    }
    let h: i64 = m.g(2).parse().ok()?;
    if h == 12 {
        return Some(Q::ZERO);
    }
    if (1..=11).contains(&h) {
        Some(Q::int(h))
    } else {
        None
    }
}

pub fn catch_up_clock(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    if CLOCK_BAD.is_match(&t) {
        return None;
    }
    let sents = sentence_list(&t);
    let qi: Vec<usize> = (0..sents.len()).filter(|&i| CATCH_CUE.is_match(&sents[i]) && CLOCK_WHEN.is_match(&sents[i])).collect();
    if qi.len() != 1 || ASKS_OTHER.is_match(&sents[qi[0]]) {
        return None;
    }
    let qs = &sents[qi[0]];
    let facts: Vec<&String> = sents.iter().enumerate().filter(|(i, s)| *i != qi[0] && SPEED.is_match(s)).map(|(_, s)| s).collect();
    if facts.len() != 2 {
        return None;
    }
    struct Info {
        speed: Q,
        hrs: Q,
        group: &'static str,
        toks: BTreeSet<String>,
    }
    let mut infos: Vec<Info> = Vec::new();
    let mut clock_digits = 0usize;
    for s in &facts {
        let (sp, tm) = (SPEED.find_all(s), TIME_EXPR.find_all(s));
        if sp.len() != 1 || tm.len() != 1 {
            return None;
        }
        let hrs = clock_hours(&tm[0])?;
        clock_digits += if tm[0].opt(1).is_some() { 0 } else { 1 };
        infos.push(Info { speed: fr(sp[0].g(1)), hrs, group: speed_group(sp[0].g(2)), toks: subject_tokens(s) });
    }
    if infos[0].group != infos[1].group || digit_numbers(problem).len() != 2 + clock_digits {
        return None;
    }
    if infos[0].hrs == infos[1].hrs {
        return None;
    }
    if infos[0].hrs > infos[1].hrs {
        infos.swap(0, 1); // the vehicle named second left first
    }
    let (i1, i2) = (&infos[0], &infos[1]);
    let (s1, h1, tok1, s2, h2, tok2) = (i1.speed, i1.hrs, &i1.toks, i2.speed, i2.hrs, &i2.toks);
    let head = h2 - h1;
    if s1 <= Q::ZERO || s2 <= s1 {
        return None; // the one that leaves later must be the faster one
    }
    let cm = CATCHER.search(qs)?;
    let c: BTreeSet<String> = ALPHA
        .find_all(cm.g(1))
        .iter()
        .map(|m| m.text().to_string())
        .filter(|w| !ARTICLES.contains(&w.as_str()) && !QUESTION_STOP.contains(&w.as_str()))
        .collect();
    let only2: BTreeSet<String> = tok2.difference(tok1).cloned().collect();
    let only1: BTreeSet<String> = tok1.difference(tok2).cloned().collect();
    if c.is_empty() || c.intersection(&only2).next().is_none() || c.intersection(&only1).next().is_some() {
        return None;
    }
    let tt = s1 * head / (s2 - s1);
    let eq = format!("{}(t + {}) = {}t", show(s1), show(head), show(s2));
    let after_noon = format!("{}{}", show_exact(h2 + tt), if h2 + tt == Q::ONE { " hour" } else { " hours" });
    res(
        "catch_up_clock",
        vec![tt],
        eq.clone(),
        format!(
            "the later vehicle leaves {} hours after the earlier one; t = hours after it leaves: {}, so t = {} hour{} ({} after noon)",
            show_exact(head),
            eq,
            show_exact(tt),
            if tt == Q::ONE { "" } else { "s" },
            after_noon
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 24. doubling, tripling, halving every period, exact whole numbers of periods
// ---------------------------------------------------------------------------------------------------------
const GROWTH_UNIT: &str = r"(?:hours?|hrs?|days?|weeks?|months?|years?|minutes?|mins?|seconds?|secs?)";
static PERIOD_PATTERNS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    vec![
        re(&format!(r"\bevery\s+(\d+(?:\.\d+)?|two|three|four|five|six|seven|eight|nine|ten)\s+({GROWTH_UNIT})\b")),
        re(&format!(r"\bevery\s+()({GROWTH_UNIT})\b")),
        re(&format!(r"\bhalf-life of\s+(\d+(?:\.\d+)?)\s+({GROWTH_UNIT})\b")),
    ]
});
lazy_re!(
    GROW_END_CUE,
    r"(?:reach(?:es)?|to|becomes?|decays to|falls to|grows to|when will(?: there be)?|will there be|there be|there are|are there)\s*$"
);
lazy_re!(GROW_BAD, r"\b(?:per|twice|thrice|quadrupl\w*|quarter\w*|tenfold|fold|then|other|percent)\b");
lazy_re!(GROW_DOUBLE, r"\b(?:doubles|double|doubling|doubled)\b");
lazy_re!(GROW_TRIPLE, r"\b(?:triples|triple|tripling|tripled)\b");
lazy_re!(GROW_HALVE, r"\b(?:halves|halving|halved|half-life)\b");
lazy_re!(GROW_SPELLED, r"\b(?:two|three|four|five|six|seven|eight|nine|ten|half)\b");
lazy_re!(GROW_QUESTION, &format!(r"\b(?:when|how long|how many {GROWTH_UNIT}|after how many)\b"));
lazy_re!(GROW_UNIT_RE, GROWTH_UNIT);

fn time_name(word: &str) -> &'static str {
    let w = word.trim_end_matches('s');
    let key = if matches!(w, "hour" | "hr" | "day" | "week" | "month" | "year" | "minute" | "min" | "second" | "sec") { w } else { word };
    match key {
        "hour" | "hr" => "hour",
        "day" => "day",
        "week" => "week",
        "month" => "month",
        "year" => "year",
        "minute" | "min" => "minute",
        "second" | "sec" => "second",
        _ => decline(),
    }
}

pub fn periodic_growth(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || GROW_BAD.is_match(&t) {
        return None;
    }
    let mut kinds: Vec<&str> = Vec::new();
    if GROW_DOUBLE.is_match(&t) {
        kinds.push("double");
    }
    if GROW_TRIPLE.is_match(&t) {
        kinds.push("triple");
    }
    if GROW_HALVE.is_match(&t) {
        kinds.push("halve");
    }
    if kinds.len() != 1 {
        return None;
    }
    let found: Vec<_> = PERIOD_PATTERNS.iter().flat_map(|p| p.find_all(&t)).collect();
    if found.len() != 1 {
        return None;
    }
    let pm = &found[0];
    let g1 = pm.opt(1).unwrap_or("");
    let period = if !g1.is_empty() { fr(g1) } else { Q::ONE };
    let unit = time_name(pm.g(2));
    let work = format!("{} {}", &t[..pm.start()], &t[pm.end()..]).replace("half-life", " ");
    if GROW_SPELLED.is_match(&work) {
        return None;
    }
    let nums = digit_numbers(problem);
    let period_digit = if !g1.is_empty() && g1.chars().next().unwrap().is_ascii_digit() { pm.group_start(1) } else { None };
    let others: Vec<Num> = nums.iter().filter(|n| period_digit.map_or(true, |pd| !(n.start <= pd && pd < n.end))).copied().collect();
    if others.len() != 2 || nums.len() != others.len() + if period_digit.is_some() { 1 } else { 0 } || others.iter().any(|n| n.money) {
        return None;
    }
    let ends: Vec<Num> = others.iter().filter(|n| GROW_END_CUE.is_match(t[..n.start].trim_end())).copied().collect();
    if ends.len() != 1 {
        return None;
    }
    let end = ends[0];
    let start = if others[1].start == end.start { others[0] } else { others[1] };
    let (a, b) = (start.v, end.v);
    if a <= Q::ZERO || b <= Q::ZERO || period <= Q::ZERO {
        return None;
    }
    // the question asks for the time, in the unit of the period
    let qs: Vec<String> = SENT_SPLIT.split(&t).into_iter().filter(|s| GROW_QUESTION.is_match(s)).collect();
    if qs.len() != 1 {
        return None;
    }
    for w in GROW_UNIT_RE.find_all(&qs[0]) {
        if time_name(w.text()) != unit {
            return None;
        }
    }
    let f: i128 = if kinds[0] == "triple" { 3 } else { 2 };
    let ratio = if kinds[0] == "halve" { a / b } else { b / a };
    if ratio <= Q::ONE || !ratio.is_int() {
        return None;
    }
    let (mut k, mut r) = (0i128, ratio.numer());
    while r % f == 0 {
        r /= f;
        k += 1;
    }
    if r != 1 || k < 1 {
        return None;
    }
    let total = period * Q::new(k, 1);
    let (eq, step);
    if kinds[0] == "halve" {
        eq = format!("{} / 2^k = {}", show(a), show(b));
        step = format!("{} / {} = {} = 2^{}", show(a), show(b), show_exact(ratio), k);
    } else {
        eq = format!("{} x {}^k = {}", show(a), f, show(b));
        step = format!("{} / {} = {} = {}^{}", show(b), show(a), show_exact(ratio), f, k);
    }
    res("periodic_growth", vec![total], eq, format!("{}, so {} periods of {} {}s: t = {} {}s", step, k, show_exact(period), unit, show_exact(total), unit))
}

// ---------------------------------------------------------------------------------------------------------
// 25. a fixed cost plus a rate:   F + r n = T
// ---------------------------------------------------------------------------------------------------------
lazy_re!(FIXED_RATE_AFTER, r"\s*(?:per|a|an|each|every|/)\s*([a-z]+)");
lazy_re!(
    FIXED_TOTAL_BEFORE,
    r"(?:\btotal(?: cost| bill)?(?: was| is| came to| comes to)?|\b(?:bill|fare|ride|trip|cost|costs|price|invoice|job)(?: was| is| came to| comes to| costs?| totaled| totalled)?|\b(?:paid|spent|owes?|owed|charged|billed|came to|comes to))\s*(?:a total of\s+)?$"
);
lazy_re!(
    FIXED_OTHER,
    r"\b(?:discount\w*|tax|tip|percent|coupon\w*|free|waived|first|additional|extra|minimum|maximum|at least|at most|between|rest|remaining|balance)\b|%"
);
lazy_re!(FIXED_AFTER_F, r"\b(?:plus|fee|and)\b|\+");
lazy_re!(FIXED_TOTAL_AFTER, r"\s*(?:in total|altogether)\b");
lazy_re!(FIXED_HOW, r"\bhow\b");
lazy_re!(FIXED_HOW_MUCH, r"\bhow much\b|\btotal\b");
lazy_re!(FIXED_HOW_MANY, r"\bhow many ([a-z]+)");
lazy_re!(FIXED_HOW_LONG, r"\bhow long\b");
pub const FIXED_CONTINUOUS: &[&str] = &["hour", "minute", "mile", "km", "kilometer", "kilometre", "meter", "metre", "foot", "feet", "pound", "lb", "kg", "gallon", "liter", "litre"];
const FIXED_TIME_UNITS: &[&str] = &["hour", "minute", "day", "week", "month", "year"];

pub fn stem_unit(w: &str) -> String {
    let w = w.to_lowercase();
    if w.ends_with("ies") {
        return format!("{}y", &w[..w.len() - 3]);
    }
    if w.ends_with('s') && !w.ends_with("ss") && w != "lbs" {
        return w[..w.len() - 1].to_string();
    }
    if w == "lbs" {
        "lb".to_string()
    } else {
        w
    }
}

pub fn fixed_plus_rate(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || !nums.iter().all(|n| n.money) || CHANGES_EQ_RE.is_match(&t) || FIXED_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let rates: Vec<(Num, String)> = nums.iter().filter_map(|n| FIXED_RATE_AFTER.matches(&t[n.end..]).map(|m| (*n, m.g(1).to_string()))).collect();
    if rates.len() != 1 {
        return None;
    }
    let (rate_n, rate_word) = (rates[0].0, rates[0].1.clone());
    let others: Vec<Num> = nums.iter().filter(|n| n.start != rate_n.start).copied().collect();
    let (f_n, t_n) = (others[0], others[1]);
    if !(f_n.start < rate_n.start && rate_n.start < t_n.start) {
        return None; // the order is: fixed cost, then the rate, then the total
    }
    // the fixed cost is followed by "plus" or a fee word; the total is introduced by a word like "total was / the ride cost / has paid"
    let after_f = t[f_n.end..].split_whitespace().take(3).collect::<Vec<_>>().join(" ");
    if !FIXED_AFTER_F.is_match(&after_f) {
        return None;
    }
    if !(FIXED_TOTAL_BEFORE.is_match(t[..t_n.start].trim_end()) || FIXED_TOTAL_AFTER.matches(&t[t_n.end..]).is_some()) {
        return None;
    }
    if EACH_AFTER.matches(&t[f_n.end..]).is_some() || EACH_AFTER.matches(&t[t_n.end..]).is_some() {
        return None;
    }
    let unit = stem_unit(&rate_word);
    // the question asks how many of the unit (or how long, for a time unit), not the total
    let sents = sentence_list(&t);
    let q = sents.iter().rev().find(|s| FIXED_HOW.is_match(s))?;
    if FIXED_HOW_MUCH.is_match(q) {
        return None;
    }
    let ok = FIXED_HOW_MANY.search(q).map_or(false, |qm| stem_unit(qm.g(1)) == unit) || (FIXED_HOW_LONG.is_match(q) && FIXED_TIME_UNITS.contains(&unit.as_str()));
    if !ok {
        return None;
    }
    let (f, r, total) = (f_n.v, rate_n.v, t_n.v);
    if r <= Q::ZERO || f < Q::ZERO || total <= f {
        return None;
    }
    let n = (total - f) / r;
    if n <= Q::ZERO || (!n.is_int() && !FIXED_CONTINUOUS.contains(&unit.as_str())) {
        return None;
    }
    let eq = format!("{} + {}n = {}", show(f), show(r), show(total));
    res(
        "fixed_plus_rate",
        vec![n],
        eq.clone(),
        format!("n = number of {}s: {}, so n = {} {}{}", unit, eq, show_exact(n), unit, if n == Q::ONE { "" } else { "s" }),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 26. two amounts changing at constant rates until equal:   a + ca t = b + cb t
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    GROW_STMT,
    r"(?<![\w$])(?P<who>[a-z]+(?:\s[a-z]+)?)\s+(?:has|had|have|starts? with|started with|begins? with)\s+(?P<a>\$?\d+(?:\.\d+)?)(?:\s+[a-z]+)?(?:,|\s+and|\s+but)?\s+(?:and\s+)?(?P<v>grows?|increases?|rises?|gains?|shrinks?|decreases?|falls?|drops?|loses?|declines?)(?:\s+by)?\s+(?P<c>\$?\d+(?:\.\d+)?)(?:\s+[a-z]+)?\s+(?:a|per|each|every)\s+(?P<u>year|month|week|day|hour|decade)\b"
);
const GROW_UNIT: &str = r"(?:cm|mm|m|meters?|metres?|inch(?:es)?|in|ft|feet|foot|kg|lbs?|pounds?)";
lazy_re!(
    GROW_STMT2,
    &format!(
        r"(?<![\w$])(?P<who>[a-z]+(?:\s[a-z]+)?)\s+(?:is|are|measures)\s+(?P<a>\d+(?:\.\d+)?)\s*(?P<au>{GROW_UNIT})(?:\s+(?:tall|long|high|heavy|deep|wide))?(?:,|\s+and|\s+but)?\s+(?:and\s+)?(?P<v>grows?|gains?|shrinks?|loses?|drops?|falls?|rises?|increases?|decreases?)(?:\s+by)?\s+(?P<c>\d+(?:\.\d+)?)\s*(?P<cu>{GROW_UNIT})\s+(?:a|per|each|every)\s+(?P<u>year|month|week|day|hour|decade)\b"
    )
);
lazy_re!(GROW_FALL, r"shrink|decreas|fall|drop|lose|declin");
lazy_re!(TC_WHEN, r"\b(?:when|how long|how many (?:years|months|weeks|days|hours|decades)|after how many)\b");
lazy_re!(TC_EQUAL, r"\b(?:equal|same)\b");
lazy_re!(
    TC_OTHER_Q,
    r"\bhow much\b|\bhow many (?:people|residents|dollars|items)\b|\bwhat (?:will|is|would)\b|\bhow (?:tall|heavy|high|big|deep|wide)\b|\bwhat (?:height|weight|size|length)\b"
);

pub fn two_changing_equal(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let mut found = GROW_STMT.find_all(&t);
    let mut measured = false;
    if found.is_empty() {
        found = GROW_STMT2.find_all(&t); // "is 13 cm tall and grows 3 cm a week": a continuous quantity
        measured = true;
    }
    let nums = digit_numbers(problem);
    if found.len() != 2 || nums.len() != 4 || CHANGES_EQ_RE.is_match(&t) || t.contains('%') {
        return None;
    }
    let (m1, m2) = (&found[0], &found[1]);
    if m1.name("who") == m2.name("who") || m1.name("u") != m2.name("u") {
        return None;
    }
    if measured && !(m1.name("au") == m1.name("cu") && m1.name("cu") == m2.name("au") && m2.name("au") == m2.name("cu")) {
        return None; // one unit of measure throughout
    }
    let money: BTreeSet<bool> = nums.iter().map(|n| n.money).collect();
    if money.len() != 1 {
        return None; // all money or none
    }
    let rest = format!("{} {} {}", &t[..m1.start()], &t[m1.end()..m2.start()], &t[m2.end()..]);
    if DIGIT_RE.is_match(&rest) || SPELLED_ANY_RE.is_match(&rest) {
        return None;
    }
    let unit = m1.name("u")?;
    // the question asks for the TIME they are equal, not an amount
    if !TC_WHEN.is_match(&rest) || !TC_EQUAL.is_match(&rest) || TC_OTHER_Q.is_match(&rest) {
        return None;
    }
    let val = |s: &str| fr(s.trim_start_matches('$'));
    let (a, b) = (val(m1.name("a")?), val(m2.name("a")?));
    let ca = if GROW_FALL.matches(m1.name("v")?).is_some() { -val(m1.name("c")?) } else { val(m1.name("c")?) };
    let cb = if GROW_FALL.matches(m2.name("v")?).is_some() { -val(m2.name("c")?) } else { val(m2.name("c")?) };
    if ca == cb {
        return None;
    }
    let tt = (b - a) / (ca - cb);
    if tt <= Q::ZERO || (!tt.is_int() && !measured) {
        return None;
    }
    let sa = if ca >= Q::ZERO { format!("+ {}", show(ca)) } else { format!("- {}", show(-ca)) };
    let sb = if cb >= Q::ZERO { format!("+ {}", show(cb)) } else { format!("- {}", show(-cb)) };
    let eq = format!("{} {} t = {} {} t", show(a), sa, show(b), sb);
    res("two_changing_equal", vec![tt], eq.clone(), format!("t = number of {}s: {}, so t = {} {}s", unit, eq, show_exact(tt), unit))
}

// ---------------------------------------------------------------------------------------------------------
// 27. adding to an amount you already have, to reach a blend price
// ---------------------------------------------------------------------------------------------------------
const ADDMIX_UNIT: &str = r"(?:lb|lbs|pounds?|kg|kilograms?|oz|ounces?)";
lazy_re!(
    ADDMIX,
    &format!(
        r"how (?:many|much)\s+(?:(?:{ADDMIX_UNIT})\s+of\s+)?(?:[a-z]+(?:\s[a-z]+)?\s+)?(?:worth|at|priced at|costing|that costs?|selling for)\s+\$(\d+(?:\.\d+)?)\s*(?:/\s*|per\s+|a\s+)({ADDMIX_UNIT})\b\s+must be added to\s+(\d+(?:\.\d+)?)\s*({ADDMIX_UNIT})\s+of\s+(?:[a-z]+(?:\s[a-z]+)?\s+)?(?:worth|at|priced at|costing|that costs?|selling for)\s+\$(\d+(?:\.\d+)?)\s*(?:/\s*|per\s+|a\s+)({ADDMIX_UNIT})\b\s+to\s+(?:make|get|obtain|produce)\s+(?:an?\s+)?(?:mix|blend|mixture)?\s*(?:that is\s+)?(?:worth|priced at|selling for|at)\s+\$(\d+(?:\.\d+)?)\s*(?:/\s*|per\s+|a\s+)({ADDMIX_UNIT})\b"
    )
);
lazy_re!(
    ADDMIX_OTHER,
    r"\b(?:water|pure|evaporat\w*|remov\w*|replac\w*|profit|loss|discount\w*|tax|times as|twice|double|triple|half|as much|ratio|more than|less than|three|four|five|six|seven|eight|nine|ten|another)\b|%"
);

pub fn add_to_blend(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 4 || CHANGES_EQ_RE.is_match(&t) || ADDMIX_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let m = ADDMIX.search(&t)?;
    let units: BTreeSet<&str> = [m.g(2), m.g(4), m.g(6), m.g(8)].iter().map(|u| unit_name(u)).collect();
    if units.len() != 1 {
        return None;
    }
    let (p2, a, p1, tg) = (fr(m.g(1)), fr(m.g(3)), fr(m.g(5)), fr(m.g(7)));
    if a <= Q::ZERO || p1 == p2 || !(p1.min(p2) < tg && tg < p1.max(p2)) {
        return None; // the blend price must lie strictly between the two prices
    }
    let x = a * (tg - p1) / (p2 - tg);
    if x <= Q::ZERO {
        return None;
    }
    let unit = unit_name(m.g(4));
    let eq = format!("{}({}) + {}x = {}({} + x)", show(p1), show(a), show(p2), show(tg), show(a));
    res("add_to_blend", vec![x], eq.clone(), format!("x = {} added: {}, so x = {} {}", unit, eq, show_exact(x), unit))
}

// ---------------------------------------------------------------------------------------------------------
// 28. two kinds of score:   x + y = N,  a x + b y = P
// ---------------------------------------------------------------------------------------------------------
lazy_re!(SCORE_KIND, r"\b(?:(\d+)|(two|three|four|five|six|seven))-(?:pointers?|points?)\b");
lazy_re!(SCORE_TOTAL, r"(?<![\w-])(\d+)\s+points?\b");
lazy_re!(SCORE_COUNT, r"\bfrom\s+(\d+)\s+[a-z]+(?:\s+[a-z]+)?");
lazy_re!(SCORE_OTHER, r"\b(?:free throws?|fouls?|safety|safeties|penalt\w*|conversions?|bonus\w*|overtime|other|rest|some)\b");
lazy_re!(SCORE_ALL, r"\b(?:all|only|just|nothing but)\b");
lazy_re!(SCORE_EACH, r"\beach\b");

pub fn two_scores(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || SCORE_OTHER.is_match(&t.replace("(with extra point)", " ")) {
        return None;
    }
    if !SCORE_ALL.is_match(&t) {
        return None;
    }
    let kinds = SCORE_KIND.find_all(&t);
    let values: Vec<Q> = kinds.iter().map(|m| if let Some(d) = m.opt(1) { fr(d) } else { fr(m.g(2)) }).collect();
    if values.len() != 2 || values[0] == values[1] || values[0].min(values[1]) < Q::ONE {
        return None;
    }
    let (tot, cnt) = (SCORE_TOTAL.find_all(&t), SCORE_COUNT.find_all(&t));
    if tot.len() != 1 || cnt.len() != 1 {
        return None;
    }
    let work = SCORE_KIND.sub(&t.replace("(with extra point)", " "), " ");
    if SPELLED_ANY_RE.is_match(&work) {
        return None;
    }
    let digit_kinds = kinds.iter().filter(|m| m.opt(1).is_some()).count();
    if nums.len() != 2 + digit_kinds {
        return None;
    }
    let (p, n) = (fr(tot[0].g(1)), fr(cnt[0].g(1)));
    if p == n || tot[0].group_start(1) == cnt[0].group_start(1) {
        return None;
    }
    // the question asks how many of each (or names a kind of score)
    let q = t.rfind("how many")?;
    if !(SCORE_EACH.is_match(&t[q..]) || SCORE_KIND.is_match(&t[q..])) {
        return None;
    }
    let (a, b) = (values[0], values[1]);
    let y = (p - a * n) / (b - a);
    let x = n - y;
    if x <= Q::ZERO || y <= Q::ZERO || !x.is_int() || !y.is_int() {
        return None;
    }
    let (e1, e2) = (format!("x + y = {}", show(n)), format!("{}x + {}y = {}", show(a), show(b), show(p)));
    res(
        "two_scores",
        vec![x, y],
        format!("{};  {}", e1, e2),
        format!("x = number of {}-point scores, y = number of {}-point scores: {} and {}, so x = {} and y = {}", show(a), show(b), e1, e2, show_exact(x), show_exact(y)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 29. two discounts, one after the other:   (1 - a)(1 - b) p = F
// ---------------------------------------------------------------------------------------------------------
lazy_re!(DISC2_FINAL, r"\b(?:final (?:price|cost)|now (?:costs?|sells? for|is)|pays?|paid|ends? up|after both)\b");
lazy_re!(DISC2_ORIGINAL_Q, r"\b(?:what|how much)\b[^.?!]*\b(?:original|regular|list|starting|before)\b");
lazy_re!(DISC2_OTHER, r"\b(?:tax|tip|both|combined|together|add up|adds up|total discount|save|saved|savings)\b");
lazy_re!(D2_OFF, r"\s*%\s*(?:off|discount)\b");
lazy_re!(D2_MARKED, r"\b(?:marked down|reduced|discounted)(?: by)?\s*$");
lazy_re!(D2_EXTRA, r"\b(?:extra|additional|further|another)\s*$");
lazy_re!(D2_THEN, r"\b(?:then|after that|next|on top)\b");
lazy_re!(D2_SALE_PRICE, r"\b(?:sale|discounted|reduced|new|marked-down|marked down|lower)\s+price\b");
lazy_re!(D2_ORIG_WORD, r"\b(?:original|regular|list|first)\b");

pub fn two_discounts(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || t.matches('%').count() != 2 || CHANGES_EQ_RE.is_match(&t) || DISC2_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let money: Vec<Num> = nums.iter().filter(|n| n.money).copied().collect();
    let pcts: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    if money.len() != 1 || pcts.len() != 2 {
        return None;
    }
    let (p1, p2) = (pcts[0], pcts[1]);
    // the first cut: "40% off" / "marked down 40%"; the second: an extra / additional / further cut taken off the SALE (reduced) price, not the original
    if !(D2_OFF.matches(&t[p1.end..]).is_some() || D2_MARKED.is_match(t[..p1.start].trim_end())) {
        return None;
    }
    if !D2_EXTRA.is_match(&last_words(&t[..p2.start], 2)) || !D2_THEN.is_match(&t) {
        return None;
    }
    let clause = t[p2.end..].split(|c| ".?!".contains(c)).next().unwrap_or("");
    if !D2_SALE_PRICE.is_match(clause) || D2_ORIG_WORD.is_match(clause) {
        return None;
    }
    if !DISC2_FINAL.is_match(&t) || !DISC2_ORIGINAL_Q.is_match(&t) {
        return None;
    }
    let hundred = Q::int(100);
    let (a, b, f) = (p1.v / hundred, p2.v / hundred, money[0].v);
    if !(Q::ZERO < a && a < Q::ONE && Q::ZERO < b && b < Q::ONE) || f <= Q::ZERO {
        return None;
    }
    let price = f / ((Q::ONE - a) * (Q::ONE - b));
    let eq = format!("{} x {} x p = {}", show(Q::ONE - a), show(Q::ONE - b), show(f));
    res("two_discounts", vec![price], eq.clone(), format!("p = original price; the second cut is taken off the sale price: {}, so p = {}", eq, show_answer(price, true)))
}

// ---------------------------------------------------------------------------------------------------------
// 30. angles of a triangle in a ratio of three whole numbers
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TRI_RATIO, r"(?<![\d.:])(\d+)\s*:\s*(\d+)\s*:\s*(\d+)(?!\d|\.\d|:)");
lazy_re!(
    TRI_OTHER,
    r"\b(?:right|isosceles|equilateral|exterior|outside|largest|smallest|biggest|difference|sum of two|quadrilateral|polygon|pentagon|hexagon|one angle|another|degrees more|more than|less than|twice|double)\b"
);
lazy_re!(TRI_ANGLES, r"\bangles of a triangle\b|\btriangle'?s? angles\b|\bthree angles of a triangle\b");
lazy_re!(RATIO_WORD, r"\bratio\b");
lazy_re!(
    TRI_ASK,
    r"\b(?:find|what (?:are|is)|determine|give|name)\b[^.?!]*\b(?:each angle|the angles|all (?:three )?angles|every angle|each of the angles)\b"
);

pub fn triangle_ratio(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || TRI_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    if !TRI_ANGLES.is_match(&t) || !RATIO_WORD.is_match(&t) {
        return None;
    }
    let rm = TRI_RATIO.find_all(&t);
    if rm.len() != 1 {
        return None;
    }
    let parts: Vec<Q> = (1..=3).map(|g| fr(rm[0].g(g))).collect();
    if parts.iter().copied().fold(parts[0], |a, b| a.min(b)) < Q::ONE {
        return None;
    }
    // the question asks for every angle
    if !TRI_ASK.is_match(&t) {
        return None;
    }
    let total = parts.iter().fold(Q::ZERO, |a, b| a + *b);
    let angles: Vec<Q> = parts.iter().map(|p| Q::int(180) * *p / total).collect();
    res(
        "triangle_ratio",
        angles.clone(),
        format!("{}k + {}k + {}k = 180", show(parts[0]), show(parts[1]), show(parts[2])),
        format!(
            "{} parts in all, so one part is 180/{} = {}: {}",
            show(total),
            show(total),
            show_exact(Q::int(180) / total),
            angles.iter().map(|a| format!("{} degrees", show_exact(*a))).collect::<Vec<_>>().join(", ")
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 31. two prices and a known count:   x + y = N,  p1 x + p2 y = T
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TP_RATE_AFTER, r"\s*(?:/\s*(?:h|hr|hrs|hour|lb|kg|ea|each)\b|(?:an?|per)\s+[a-z]+\b|each\b|apiece\b)");
lazy_re!(
    TP_TOTAL_BEFORE,
    r"(?:\b(?:for|earning|earned|earns|totaling|totalling|totaled|costing|cost|costs|paying|paid|made|worth|in all)\s*(?:a total of\s*)?|\btotal of\s*)$"
);
lazy_re!(TP_OTHER, r"\b(?:tax|discount|tip|bonus|overtime|percent|extra|also|third|another|rest|remaining)\b|%");
lazy_re!(TP_RATE_UNIT, r"\s*(?:/\s*|per\s+|an?\s+)([a-z]+)\b");
lazy_re!(
    TP_TWO_WORDS,
    r"\btwo (?:jobs|kinds|types|rates|prices|wages|sizes|brands|teas|coffees|grades|flavors|varieties|blends|candies|nuts|oils|fruits)\b"
);
lazy_re!(TP_CENTS, r"\s*(?:¢|cents?\b)");
lazy_re!(TP_WORD, r"\s*([a-z]+)");
lazy_re!(HOW_MANY, r"\bhow many\b");
lazy_re!(TP_EACH_RE, r"\beach\b");

fn tp_measure(w: &str) -> Option<&'static str> {
    Some(match w {
        "kg" | "kilogram" | "kilograms" => "kg",
        "lb" | "lbs" | "pound" | "pounds" => "lb",
        "oz" | "ounce" | "ounces" => "oz",
        "g" | "gram" | "grams" => "g",
        "h" | "hr" | "hrs" | "hour" | "hours" => "hour",
        "l" | "liter" | "liters" | "litre" | "litres" => "l",
        "gallon" | "gallons" | "gal" => "gal",
        "mile" | "miles" => "mile",
        "km" => "km",
        "min" | "minute" | "minutes" => "minute",
        "day" | "days" => "day",
        _ => return None,
    })
}

pub fn two_prices_known_count(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if CHANGES_EQ_RE.is_match(&t) || TP_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&TP_TWO_WORDS.sub(&t, " ")) {
        return None;
    }
    // the question may repeat a price ("How many hours were worked at the $16 job?"): not a new number
    let q = t.rfind("how ");
    let main: Vec<Num> = nums.iter().filter(|n| q.map_or(true, |q| n.start < q)).copied().collect();
    let repeated: Vec<Num> = nums.iter().filter(|n| q.map_or(false, |q| n.start >= q)).copied().collect();
    if repeated.iter().any(|n| !main.iter().any(|m| m.v == n.v)) {
        return None;
    }
    let cents: Vec<Num> = main.iter().filter(|n| TP_CENTS.matches(&t[n.end..]).is_some()).copied().collect();
    let rated: Vec<Num> = main.iter().filter(|n| n.money && TP_RATE_AFTER.matches(&t[n.end..]).is_some() && !cents.contains(n)).copied().collect();
    let plain: Vec<Num> = main.iter().filter(|n| !n.money && !cents.contains(n)).copied().collect();
    let money: Vec<Num> = main.iter().filter(|n| n.money && !rated.contains(n) && !cents.contains(n)).copied().collect();
    if plain.len() != 1 || money.len() != 1 {
        return None;
    }
    let (prices, in_cents) = if cents.len() == 2 && rated.is_empty() {
        (cents.clone(), true)
    } else if rated.len() == 2 && cents.is_empty() {
        (rated.clone(), false)
    } else {
        return None;
    };
    // the known amount's unit must match the unit of the price: 31 lb at $5/kg is not 31 kg; 61 hours at $16/h is
    let rate_units: Vec<Option<&'static str>> = prices.iter().map(|n| TP_RATE_UNIT.matches(&t[n.end..]).and_then(|um| tp_measure(um.g(1)))).collect();
    if rate_units.iter().any(|u| u.is_some()) {
        let set: BTreeSet<Option<&str>> = rate_units.iter().copied().collect();
        let cw = TP_WORD.matches(&t[plain[0].end..]);
        if set.len() != 1 || cw.is_none() || tp_measure(cw.unwrap().g(1)) != rate_units[0] {
            return None;
        }
    }
    let tn = money[0];
    if !(TP_TOTAL_BEFORE.is_match(t[..tn.start].trim_end()) || FIXED_TOTAL_AFTER.matches(&t[tn.end..]).is_some()) || EACH_AFTER.matches(&t[tn.end..]).is_some() {
        return None;
    }
    let n = plain[0].v;
    if n <= Q::ZERO || !n.is_int() {
        return None;
    }
    // the question asks how many of each (or at a named price)
    let q = q?;
    if !(HOW_MANY.is_match(&t[q..]) && (TP_EACH_RE.is_match(&t[q..]) || !repeated.is_empty())) {
        return None;
    }
    let scale = if in_cents { Q::new(1, 100) } else { Q::ONE };
    let (p1, p2, total) = (prices[0].v * scale, prices[1].v * scale, tn.v);
    if p1 == p2 || p1 <= Q::ZERO || p2 <= Q::ZERO {
        return None;
    }
    let y = (total - p1 * n) / (p2 - p1);
    let x = n - y;
    if x <= Q::ZERO || y <= Q::ZERO || !x.is_int() || !y.is_int() {
        return None;
    }
    let unit = if in_cents { "cents" } else { "dollars" };
    let (e1, e2) = (format!("x + y = {}", show(n)), format!("{}x + {}y = {}", show(p1), show(p2), show(total)));
    res(
        "two_prices_known_count",
        vec![x, y],
        format!("{};  {}", e1, e2),
        format!(
            "x = number at {} {}, y = number at {} {}{}: {} and {}, so x = {} and y = {}",
            show(prices[0].v),
            unit,
            show(prices[1].v),
            unit,
            if in_cents { " (a price in cents is divided by 100 to meet the dollar total)" } else { "" },
            e1,
            e2,
            show_exact(x),
            show_exact(y)
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 32. draining or filling to a mark:   V - r t = L  (drains)   or   V + r t = L  (fills)
// ---------------------------------------------------------------------------------------------------------
const DRAIN_UNIT: &str = r"(?:liters?|litres?|l|gallons?|gal|quarts?|kg|lbs?|pounds?|cups?)";
const DRAIN_TIME: &str = r"(?:seconds?|secs?|minutes?|mins?|hours?|hrs?|days?|weeks?)";
lazy_re!(
    DRAIN_START,
    &format!(r"\b(?:holds|contains|has|had|starts? with|started with|is filled with|is holding|is full with)\s+(\d+(?:\.\d+)?)\s*({DRAIN_UNIT})\b")
);
lazy_re!(
    DRAIN_RATE,
    &format!(r"\b(drains?|empties|leaks?|loses|lose|pumps? out|fills?|gains?|pumps? in|adds?)\s+(?:at|by|about)?\s*(\d+(?:\.\d+)?)\s*({DRAIN_UNIT})\s*(?:per|a|each|every|/)\s*({DRAIN_TIME})\b")
);
lazy_re!(
    DRAIN_OTHER,
    r"\b(?:while|but|and then|then|until|both|valve|second|another|also|stops?|resum\w*|after (?:that|which)|half|full)\b|%|\$"
);
lazy_re!(DRAIN_DOWN, r"drain|empt|leak|lose|pumps? out");
lazy_re!(DRAIN_TARGET_UNIT, &format!(r"\s*({DRAIN_UNIT})\b"));
lazy_re!(DRAIN_Q, &format!(r"\b(?:after how many|how many|how long)\s*({DRAIN_TIME})?"));
lazy_re!(DRAIN_AFTER, r"\b(?:remain\w*|left|hold|holds|contain\w*|have|reach\w*|be in)\b");
lazy_re!(DRAIN_HOW_MANY_UNIT, &format!(r"\bhow many (?:{DRAIN_UNIT})\b"));

fn drain_unit(w: &str) -> String {
    let w = w.to_lowercase();
    let norm = |k: &str| -> Option<&'static str> {
        Some(match k {
            "liter" | "litre" | "l" => "l",
            "gallon" | "gal" => "gal",
            "quart" => "quart",
            "kg" => "kg",
            "lb" | "pound" => "lb",
            "cup" => "cup",
            _ => return None,
        })
    };
    norm(&w).or_else(|| norm(w.trim_end_matches('s'))).map(|s| s.to_string()).unwrap_or(w)
}

fn drain_time(w: &str) -> String {
    let w = w.to_lowercase();
    let norm = |k: &str| -> Option<&'static str> {
        Some(match k {
            "second" | "sec" => "second",
            "minute" | "min" => "minute",
            "hour" | "hr" => "hour",
            "day" => "day",
            "week" => "week",
            _ => return None,
        })
    };
    norm(&w).or_else(|| norm(w.trim_end_matches('s'))).map(|s| s.to_string()).unwrap_or(w)
}

pub fn drain_or_fill(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || DRAIN_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let (sm, rm) = (DRAIN_START.find_all(&t), DRAIN_RATE.find_all(&t));
    if sm.len() != 1 || rm.len() != 1 {
        return None;
    }
    let verb = rm[0].g(1);
    let down = DRAIN_DOWN.matches(verb).is_some();
    let (v, r) = (fr(sm[0].g(1)), fr(rm[0].g(2)));
    let (u1, u2) = (drain_unit(sm[0].g(2)), drain_unit(rm[0].g(3)));
    if u1 != u2 {
        return None;
    }
    let tunit = drain_time(rm[0].g(4));
    // the third number is the mark to reach: "will 500 L remain", "will it hold 2,000 L"
    let used = [sm[0].group_start(1)?, rm[0].group_start(2)?];
    let rest: Vec<Num> = nums.iter().filter(|n| !used.iter().any(|u| n.start <= *u && *u < n.end)).copied().collect();
    if rest.len() != 1 || rest[0].money {
        return None;
    }
    let target = rest[0];
    let um = DRAIN_TARGET_UNIT.matches(&t[target.end..])?;
    if drain_unit(um.g(1)) != u1 {
        return None;
    }
    // the question asks for the TIME, in the unit of the rate, and mentions the mark: remain / left / hold / contain / reach
    let qm = DRAIN_Q.search(&t)?;
    if let Some(g1) = qm.opt(1) {
        if !g1.is_empty() && drain_time(g1) != tunit {
            return None;
        }
    }
    let has_time = qm.opt(1).map_or(false, |g| !g.is_empty());
    if !has_time && !qm.text().contains("how long") {
        return None;
    }
    let after = &t[qm.start()..];
    if !DRAIN_AFTER.is_match(after) || DRAIN_HOW_MANY_UNIT.is_match(&t) {
        return None;
    }
    let l = target.v;
    let tt = if down { (v - l) / r } else { (l - v) / r };
    if tt <= Q::ZERO || r <= Q::ZERO || v < Q::ZERO || l < Q::ZERO {
        return None;
    }
    let sign = if down { "-" } else { "+" };
    let eq = format!("{} {} {}t = {}", show(v), sign, show(r), show(l));
    res(
        "drain_or_fill",
        vec![tt],
        eq.clone(),
        format!("t = number of {}s: {}, so t = {} {}{}", tunit, eq, show_exact(tt), tunit, if tt == Q::ONE { "" } else { "s" }),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 33. a percent of a count:   count x percent / 100
// ---------------------------------------------------------------------------------------------------------
const PCTC_NOUN: &str = r"(?:questions?|problems?|items?|shots?|attempts?|students?|people|marbles|cards|tickets|games|answers|points)";
lazy_re!(
    PCTC_OTHER,
    r"\b(?:more|less|increase\w*|decrease\w*|off|discount\w*|tax|tip|of which|rest|remaining|incorrect\w*|wrong|missed|failed|fewer|than|each|per|doubl\w*)\b|\$"
);
lazy_re!(PCTC_NOUN_RE, &format!(r"\s*{PCTC_NOUN}\b"));
lazy_re!(PCTC_SCORED, r"\b(?:scored|score of|got|earned|made|scoring)\s+(?:an?\s+|only\s+|about\s+|a score of\s+)?$");
lazy_re!(PCTC_CORRECT, r"\b(?:correct\w*|right|passed|made|scored|earned|got)\b");
lazy_re!(PCTC_BAD_Q, r"\bwhat percent\b|\bhow much\b");

pub fn percent_of_count(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || t.matches('%').count() != 1 || CHANGES_EQ_RE.is_match(&t) || PCTC_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let pct: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    let cnt: Vec<Num> = nums.iter().filter(|n| !pct.contains(n)).copied().collect();
    if pct.len() != 1 || cnt.len() != 1 || cnt[0].money || !cnt[0].v.is_int() {
        return None;
    }
    PCTC_NOUN_RE.matches(&t[cnt[0].end..])?;
    // the percent must be a SCORE on the count ("scored 65%", "got 65%"); a bare "65% of the students passed" is a percent of something else
    if !PCTC_SCORED.is_match(&format!("{} ", t[..pct[0].start].trim_end())) {
        return None;
    }
    let q = t.rfind("how many")?;
    if !PCTC_CORRECT.is_match(&t[q..]) || PCTC_BAD_Q.is_match(&t) {
        return None;
    }
    let (n, p) = (cnt[0].v, pct[0].v);
    if n <= Q::ZERO || !(Q::ZERO <= p && p <= Q::int(100)) {
        return None;
    }
    let result = n * p / Q::int(100);
    if !result.is_int() {
        return None;
    }
    let eq = format!("{} x {}/100", show(n), show(p));
    res("percent_of_count", vec![result], eq.clone(), format!("{}, so {} answered correctly", eq, show_exact(result)))
}

// ---------------------------------------------------------------------------------------------------------
// 34. percent change from one amount to another:   (new - old) / old x 100
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PCHG_FROM_TO, r"\bfrom\s+\$?(\d+(?:\.\d+)?)(?:\s+[a-z]+)?\s+to\s+\$?(\d+(?:\.\d+)?)\b");
lazy_re!(PCHG_ASK, r"\bby what %[^.?!]*\bchange|%\s*change\b|\bchange in %|\bwhat %[^.?!]*\bchange|\bpercentage change\b");
lazy_re!(
    PCHG_OTHER,
    r"\b(?:increase\w*|decrease\w*|new|how many|how much|compound\w*|average|each year|per year|annual\w*|monthly|total|difference)\b"
);

pub fn percent_change_between(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    let money: BTreeSet<bool> = nums.iter().map(|n| n.money).collect();
    if nums.len() != 2 || CHANGES_EQ_RE.is_match(&t) || t.matches('%').count() != 1 || SPELLED_ANY_RE.is_match(&t) || money.len() != 1 {
        return None;
    }
    if nums.iter().any(|n| is_pct(&t, n)) {
        return None; // "from 20% to 25%" is points, not an amount
    }
    let m = PCHG_FROM_TO.search(&t)?;
    let sents = sentence_list(&t);
    let q = sents.iter().rev().find(|s| s.contains('%'))?;
    if !PCHG_ASK.is_match(q) || PCHG_OTHER.is_match(q) {
        return None;
    }
    let (old, new) = (fr(m.g(1)), fr(m.g(2)));
    if old <= Q::ZERO || new < Q::ZERO || old == new {
        return None;
    }
    let pct = (new - old) / old * Q::int(100);
    let eq = format!("({} - {}) / {} x 100", show(new), show(old), show(old));
    res("percent_change_between", vec![pct], eq.clone(), format!("{} = {} percent (a fall is negative)", eq, show_exact(pct)))
}

// ---------------------------------------------------------------------------------------------------------
// 35. ages with a later multiple:   j = k e,   j + y = m (e + y)
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    AGEM,
    r"(?P<a>[a-z]+(?: [a-z]+)?) is (?P<k>\d+) times as old as (?P<b>[a-z]+(?: [a-z]+)?)\.\s*in (?P<y>\d+) years?,\s*\k<a> will be (?P<m>\d+) times as old as \k<b>\b"
);
lazy_re!(AGE_BAD, r"\b(?:ago|sum|difference|together|older than|younger than)\b");
lazy_re!(AGE_ASK, r"\bhow old (?:is each|are they|are (?:both|the two))\b|\bfind (?:each|their|both) (?:age|of their ages)");

pub fn ages_later_multiple(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&t) || AGE_BAD.is_match(&t) {
        return None;
    }
    let m = AGEM.search(&t)?;
    if m.name("a") == m.name("b") {
        return None;
    }
    let rest = format!("{} {}", &t[..m.start()], &t[m.end()..]);
    if DIGIT_RE.is_match(&rest) || !AGE_ASK.is_match(&rest) {
        return None;
    }
    let (k, y, mm) = (fr(m.name("k")?), fr(m.name("y")?), fr(m.name("m")?));
    if k == mm || k <= Q::ZERO || mm <= Q::ZERO || y <= Q::ZERO {
        return None;
    }
    let e = y * (mm - Q::ONE) / (k - mm);
    let j = k * e;
    if e <= Q::ZERO || j <= Q::ZERO || !e.is_int() || !j.is_int() {
        return None;
    }
    let eq = format!("j = {}e;  j + {} = {}(e + {})", show(k), show(y), show(mm), show(y));
    res(
        "ages_later_multiple",
        vec![j, e],
        eq.clone(),
        format!("j = {}'s age, e = {}'s age: {}, so e = {} and j = {}", m.name("a")?, m.name("b")?, eq, show_exact(e), show_exact(j)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 36. a shared fixed cost plus a fee each:   B / n + f = E
// ---------------------------------------------------------------------------------------------------------
lazy_re!(SHARED_RATE, r"\s*(?:[a-z]+\s+){0,2}per\s+([a-z]+)");
lazy_re!(SHARED_OTHER, r"\b(?:tax|discount\w*|tip|free|first|extra|additional|chaperone\w*|teachers?|unequally|only)\b|%");
lazy_re!(SHARED_FOR, r"\s*(?:for|plus|\+|to)\b");
lazy_re!(SHARED_PAYS, r"\b(?:pays?|paid|costs?|owes?)(?: a total of)?$");
lazy_re!(SHARED_EACH, r"\b(?:each|every)\b");
lazy_re!(SHARED_BAD_Q, r"\bhow much\b|\bper\b|\bcost\b");

pub fn shared_fixed_cost(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || !nums.iter().all(|n| n.money) || CHANGES_EQ_RE.is_match(&t) || SHARED_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let rated: Vec<(Num, String)> = nums.iter().filter_map(|n| SHARED_RATE.matches(&t[n.end..]).map(|m| (*n, m.g(1).to_string()))).collect();
    if rated.len() != 1 {
        return None;
    }
    let (rate_n, rate_word) = (rated[0].0, rated[0].1.clone());
    let others: Vec<Num> = nums.iter().filter(|n| n.start != rate_n.start).copied().collect();
    let (b, e) = (others[0], others[1]);
    if !(b.start < e.start) {
        return None;
    }
    // the shared cost is followed by "for / plus"; the amount each person pays is introduced by "pays / paid / costs" in a sentence that says each / every
    if SHARED_FOR.matches(&t[b.end..]).is_none() || EACH_AFTER.matches(&t[b.end..]).is_some() {
        return None;
    }
    let head = &t[..e.start];
    let sent_start = [head.rfind('.'), head.rfind('?'), head.rfind('!')].iter().flatten().max().map_or(0, |i| i + 1);
    let before = t[sent_start..e.start].trim_end();
    if !SHARED_PAYS.is_match(before) || !SHARED_EACH.is_match(before) {
        return None;
    }
    let unit = stem_unit(&rate_word);
    let sents = sentence_list(&t);
    let q = sents.iter().rev().find(|s| FIXED_HOW.is_match(s))?;
    if SHARED_BAD_Q.is_match(q) {
        return None;
    }
    let qm = FIXED_HOW_MANY.search(q)?;
    if stem_unit(qm.g(1)) != unit {
        return None;
    }
    let (bus, fee, each) = (b.v, rate_n.v, e.v);
    if bus <= Q::ZERO || fee < Q::ZERO || each <= fee {
        return None;
    }
    let n = bus / (each - fee);
    if n <= Q::ZERO || !n.is_int() {
        return None;
    }
    let eq = format!("{}/n + {} = {}", show(bus), show(fee), show(each));
    res("shared_fixed_cost", vec![n], eq.clone(), format!("n = number of {}s: {}, so n = {} / {} = {}", unit, eq, show(bus), show_exact(each - fee), show_exact(n)))
}

pub fn list() -> Vec<(&'static str, Family)> {
    vec![
        ("toward_each_other", toward_each_other),
        ("consecutive_product", consecutive_product),
        ("most_you_can_buy", most_you_can_buy),
        ("still_water_current", still_water_current),
        ("ratio_split", ratio_split),
        ("three_numbers", three_numbers),
        ("catch_up_clock", catch_up_clock),
        ("periodic_growth", periodic_growth),
        ("fixed_plus_rate", fixed_plus_rate),
        ("two_changing_equal", two_changing_equal),
        ("add_to_blend", add_to_blend),
        ("two_scores", two_scores),
        ("two_discounts", two_discounts),
        ("triangle_ratio", triangle_ratio),
        ("two_prices_known_count", two_prices_known_count),
        ("drain_or_fill", drain_or_fill),
        ("percent_of_count", percent_of_count),
        ("percent_change_between", percent_change_between),
        ("ages_later_multiple", ages_later_multiple),
        ("shared_fixed_cost", shared_fixed_cost),
    ]
}
