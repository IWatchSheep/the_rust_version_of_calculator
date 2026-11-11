//! Families 37-51 of `word_solver.py` (two rates on one pile ... two candles), in Python's FAMILIES order.

use super::common::*;
use super::families_a::{dist_group, speed_group, SPEED};
use super::families_b::{rfind_max, sentence_list, stem_unit, FIXED_CONTINUOUS};
use super::q::isqrt;
use super::rx::{re, Re};
use crate::lazy_re;
use std::collections::BTreeSet;

fn qi(n: i128) -> Q {
    Q::new(n, 1)
}

/// The sentence-ish pieces split on `[.?!]` (no trailing whitespace swallowed), blanks dropped.
fn bare_sentences(t: &str) -> Vec<String> {
    lazy_re!(BARE, r"[.?!]");
    BARE.split(t).into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// The last sentence (from the end) that matches `pat`.
fn last_sentence_with(t: &str, pat: &Re) -> Option<String> {
    sentence_list(t).into_iter().rev().find(|s| pat.is_match(s))
}

fn pos_in(nums_n: &Num, p: usize) -> bool {
    nums_n.start <= p && p < nums_n.end
}

// ---------------------------------------------------------------------------------------------------------
// 37. two rates working on one pile together:   (r1 + r2) t = W
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PILE_R0, r"(\d+(?:\.\d+)?)\s*(?:[a-z]+\s+)?and\s+(\d+(?:\.\d+)?)\s+([a-z]+)\s+(?:per|a|each)\s+([a-z]+)");
lazy_re!(PILE_R1, r"(\d+(?:\.\d+)?)\s+([a-z]+)\s+(?:per|a|each)\s+([a-z]+)\s+and\s+(\d+(?:\.\d+)?)\s+(?:\2)\s+(?:per|a|each)\s+(?:\3)");
lazy_re!(
    PILE_WORKERS,
    r"\btwo (?:typists|workers|machines|printers|pumps|pipes|people|painters|bakers|copiers|computers|robots|trucks|scanners|clerks|writers|friends|students|assistants)\b"
);
lazy_re!(
    PILE_OTHER,
    r"\b(?:alone|then|first|after|before|later|earlier|breaks?|stops?|rests?|each other|faster|slower|more than|less than|per hour of|how fast|how many (?:words|pages|items|pieces|units) per)\b|%|\$"
);
lazy_re!(PILE_TOGETHER, r"\b(?:together|both|combined|at the same time|at once)\b");
lazy_re!(PILE_Q, r"\bhow (?:long|many (minute|min|hour|hr|second|sec|day|week)s?|much time)\b");

fn pile_time(w: &str) -> Option<&'static str> {
    let look = |k: &str| match k {
        "minute" | "min" => Some("minute"),
        "hour" | "hr" => Some("hour"),
        "second" | "sec" => Some("second"),
        "day" => Some("day"),
        "week" => Some("week"),
        _ => None,
    };
    look(w.trim_end_matches('s')).or_else(|| look(w))
}

pub fn two_rates_one_pile(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || PILE_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&PILE_WORKERS.sub(&t, " ")) {
        return None;
    }
    if !PILE_TOGETHER.is_match(&t) {
        return None;
    }
    let (r1, r2, unit, per, p2);
    if let Some(m) = PILE_R0.search(&t) {
        r1 = fr(m.g(1));
        r2 = fr(m.g(2));
        unit = m.g(3).to_string();
        per = m.g(4).to_string();
        p2 = (m.group_start(1)?, m.group_start(2)?);
    } else if let Some(m) = PILE_R1.search(&t) {
        r1 = fr(m.g(1));
        r2 = fr(m.g(4));
        unit = m.g(2).to_string();
        per = m.g(3).to_string();
        p2 = (m.group_start(1)?, m.group_start(4)?);
    } else {
        return None;
    }
    let tname = pile_time(&per)?;
    let rest: Vec<Num> = nums.iter().filter(|n| !(pos_in(n, p2.0) || pos_in(n, p2.1))).copied().collect();
    if rest.len() != 1 || rest[0].money {
        return None;
    }
    let w_n = rest[0];
    re(&format!(r"\s*(?:of\s+)?{}\b", unit)).matches(&t[w_n.end..])?;
    let qm = PILE_Q.search(&t)?;
    if let Some(u) = qm.opt(1) {
        if pile_time(u)? != tname {
            return None;
        }
    }
    if r1 <= Q::ZERO || r2 <= Q::ZERO || w_n.v <= Q::ZERO {
        return None;
    }
    let tt = w_n.v / (r1 + r2);
    let eq = format!("({} + {})t = {}", show(r1), show(r2), show(w_n.v));
    res("two_rates_one_pile", vec![tt], eq.clone(), format!("t = {}s: {}, so t = {} {}{}", tname, eq, show_exact(tt), tname, if tt == Q::ONE { "" } else { "s" }))
}

// ---------------------------------------------------------------------------------------------------------
// 38. the least number sold for a profit:   p n - F >= G, the least whole n
// ---------------------------------------------------------------------------------------------------------
lazy_re!(LEAST_FIXED_AFTER, r"\s*(?:to\s+(?:run|host|put on|hold|stage|organize|set up|produce|operate)|in (?:costs|expenses))\b");
lazy_re!(LEAST_GOAL_BEFORE, r"\bat least\s*$");
lazy_re!(LEAST_PRICE_BEFORE, r"(?:([a-z]+)\s+)?(?:are|is|cost|costs|sell for|sells for|go for|goes for|priced at|sold at|at)\s*$");
lazy_re!(LEAST_OTHER, r"\b(?:at most|no more than|maximum|fewer|less than|tax|discount\w*|free|loss|break even|each month|per month|first)\b|%");
lazy_re!(LEAST_PROFIT, r"\bprofit\b");
lazy_re!(LEAST_ATLEAST, r"\bat least\b");
lazy_re!(LEAST_PER_UNIT, r"\s*(?:each|apiece|per\s+[a-z]+)\b");
lazy_re!(HOW_MANY, r"\bhow many\b");
lazy_re!(LEAST_MUST, r"\b(?:must|need to|have to|has to|should)\b[^.?!]*\b(?:sold|sell|sale)\b|\b(?:sold|sell)\b[^.?!]*\b(?:must|need)\b");
lazy_re!(HOW_MANY_WORD, r"\bhow many ([a-z]+)");

pub fn least_for_profit(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || !nums.iter().all(|n| n.money) || CHANGES_EQ_RE.is_match(&t) || LEAST_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    if !LEAST_PROFIT.is_match(&t) || !LEAST_ATLEAST.is_match(&t) {
        return None;
    }
    let g: Vec<Num> = nums.iter().filter(|n| LEAST_GOAL_BEFORE.is_match(t[..n.start].trim_end())).copied().collect();
    let f: Vec<Num> = nums.iter().filter(|n| LEAST_FIXED_AFTER.matches(&t[n.end..]).is_some()).copied().collect();
    if g.len() != 1 || f.len() != 1 || g[0].start == f[0].start {
        return None;
    }
    let p: Vec<Num> = nums.iter().filter(|n| n.start != g[0].start && n.start != f[0].start).copied().collect();
    if p.len() != 1 {
        return None;
    }
    let price_n = p[0];
    let pm = LEAST_PRICE_BEFORE.search(t[..price_n.start].trim_end());
    let per_unit = LEAST_PER_UNIT.matches(&t[price_n.end..]);
    if pm.is_none() && per_unit.is_none() {
        return None;
    }
    if EACH_AFTER.matches(&t[f[0].end..]).is_some() || EACH_AFTER.matches(&t[g[0].end..]).is_some() {
        return None;
    }
    let q = last_sentence_with(&t, &HOW_MANY)?;
    if !LEAST_MUST.is_match(&q) {
        return None;
    }
    let qm = HOW_MANY_WORD.search(&q);
    let noun = pm.as_ref().and_then(|m| m.opt(1));
    if let (Some(noun), Some(qm)) = (noun, &qm) {
        if stem_unit(noun) != stem_unit(qm.g(1)) {
            return None;
        }
    }
    let (cost, price, goal) = (f[0].v, price_n.v, g[0].v);
    if price <= Q::ZERO || goal < Q::ZERO || cost < Q::ZERO {
        return None;
    }
    let need = (goal + cost) / price;
    let n = need.ceil();
    if n < 1 || !(price * qi(n) - cost >= goal && price * qi(n - 1) - cost < goal) {
        return None;
    }
    let eq = format!("{}n - {} >= {}", show(price), show(cost), show(goal));
    res(
        "least_for_profit",
        vec![qi(n)],
        eq.clone(),
        format!("{}, so n >= {} and the least is {} ({} sold make ${}, one fewer makes ${})", eq, show_exact(need), n, n, show_exact(price * qi(n) - cost), show_exact(price * qi(n - 1) - cost)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 39. two seat prices written "at $p", a known count and a total:   x + y = N,  p1 x + p2 y = T
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    SEAT_TOTAL_BEFORE,
    r"(?:\b(?:totaled|totalled|total|sales|revenue|receipts|brought in|collected|raised|earned|made|grossed|came to|comes to)(?:\s+(?:was|were|is|of|came to|comes to|totaled|totalled))?)\s*$"
);
lazy_re!(SEAT_TWO_WORDS, r"\btwo (?:kinds|types|sections|classes|levels|sizes|categories|prices)\b");
lazy_re!(SEAT_OTHER, r"\b(?:tax|discount\w*|fee|fees|percent|free|extra|also|third|another|rest|remaining|bonus|refund\w*)\b|%");
lazy_re!(SEAT_AT, r"\bat\s*$");
lazy_re!(SEAT_IN_TOTAL, r"\s*(?:in total|altogether)\b");
lazy_re!(SEAT_WORD, r"\s*[a-z]+");
lazy_re!(EACH_WORD, r"\beach\b");

pub fn two_priced_items(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 4 || CHANGES_EQ_RE.is_match(&t) || SEAT_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&SEAT_TWO_WORDS.sub(&t, " ")) {
        return None;
    }
    let money: Vec<Num> = nums.iter().filter(|n| n.money).copied().collect();
    let plain: Vec<Num> = nums.iter().filter(|n| !n.money).copied().collect();
    if money.len() != 3 || plain.len() != 1 {
        return None;
    }
    let prices: Vec<Num> = money.iter().filter(|n| SEAT_AT.is_match(t[..n.start].trim_end())).copied().collect();
    let others: Vec<Num> = money.iter().filter(|n| !prices.iter().any(|p| p.start == n.start)).copied().collect();
    if prices.len() != 2 || others.len() != 1 {
        return None;
    }
    let tn = others[0];
    if !(SEAT_TOTAL_BEFORE.is_match(t[..tn.start].trim_end()) || SEAT_IN_TOTAL.matches(&t[tn.end..]).is_some()) || EACH_AFTER.matches(&t[tn.end..]).is_some() {
        return None;
    }
    if !(prices[0].start < prices[1].start && prices[1].start < tn.start) {
        return None;
    }
    let n = plain[0].v;
    if n <= Q::ZERO || !n.is_int() || SEAT_WORD.matches(&t[plain[0].end..]).is_none() {
        return None;
    }
    let q = last_sentence_with(&t, &HOW_MANY)?;
    if !EACH_WORD.is_match(&q) {
        return None;
    }
    let (p1, p2, total) = (prices[0].v, prices[1].v, tn.v);
    if p1 == p2 || p1 <= Q::ZERO || p2 <= Q::ZERO {
        return None;
    }
    let y = (total - p1 * n) / (p2 - p1);
    let x = n - y;
    if x <= Q::ZERO || y <= Q::ZERO || !x.is_int() || !y.is_int() {
        return None;
    }
    let e1 = format!("x + y = {}", show(n));
    let e2 = format!("{}x + {}y = {}", show(p1), show(p2), show(total));
    res(
        "two_priced_items",
        vec![x, y],
        format!("{};  {}", e1, e2),
        format!("x = number at ${}, y = number at ${}: {} and {}, so x = {} and y = {}", show(p1), show(p2), e1, e2, show_exact(x), show_exact(y)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 40. a plan that includes some, then charges extra:   B + r (d - I) = T
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PLAN_INCLUDES, r"\bincludes?\s+(\d+(?:\.\d+)?)\s*([a-z]+)\b");
lazy_re!(PLAN_EXTRA0, r"\b(?:each|every|per)\s+(?:extra|additional)\s+([a-z]+)\s+(?:costs?|is|are|charges?)\s+\$(\d+(?:\.\d+)?)");
lazy_re!(PLAN_EXTRA1, r"\$(\d+(?:\.\d+)?)\s+(?:per|for each|for every|a)\s+(?:extra|additional)\s+([a-z]+)");
lazy_re!(PLAN_OTHER, r"\b(?:unlimited|tax|discount\w*|fee|fees|percent|free|rest|remaining|how much extra|how many extra|extra data)\b|%");
lazy_re!(PLAN_BILL_BEFORE, r"\b(?:bill|total|charge|charged|cost|owed|owes?|paid|pays?)(?:\s+(?:was|is|came to|comes to))?\s*$");
lazy_re!(PLAN_PERIOD, r"\s*(?:a|per|each|every)\s+(?:month|week|day|year)\b");
lazy_re!(HOW_MUCH_MANY, r"\bhow (?:much|many)\b");
lazy_re!(PLAN_USED, r"\b(?:used|use|uses|consumed)\b");

pub fn plan_with_included(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 4 || CHANGES_EQ_RE.is_match(&t) || PLAN_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let im = PLAN_INCLUDES.find_all(&t);
    if im.len() != 1 {
        return None;
    }
    let (inc, unit) = (fr(im[0].g(1)), im[0].g(2).to_string());
    let (eunit, rate, rate_pos);
    if let Some(em) = PLAN_EXTRA0.search(&t) {
        eunit = em.g(1).to_string();
        rate = fr(em.g(2));
        rate_pos = em.group_start(2)?;
    } else {
        let em = PLAN_EXTRA1.search(&t)?;
        rate = fr(em.g(1));
        eunit = em.g(2).to_string();
        rate_pos = em.group_start(1)?;
    }
    if stem_unit(&eunit) != stem_unit(&unit) {
        return None;
    }
    let inc_pos = im[0].group_start(1)?;
    let rest: Vec<Num> = nums.iter().filter(|n| !(pos_in(n, inc_pos) || pos_in(n, rate_pos))).copied().collect();
    if rest.len() != 2 || !rest.iter().all(|n| n.money) {
        return None;
    }
    let (base, bill) = (rest[0], rest[1]);
    if PLAN_PERIOD.matches(&t[base.end..]).is_none() {
        return None;
    }
    if !PLAN_BILL_BEFORE.is_match(t[..bill.start].trim_end()) || EACH_AFTER.matches(&t[bill.end..]).is_some() {
        return None;
    }
    let q = last_sentence_with(&t, &HOW_MUCH_MANY)?;
    if !PLAN_USED.is_match(&q) {
        return None;
    }
    if let Some(hm) = HOW_MANY_WORD.search(&q) {
        if stem_unit(hm.g(1)) != stem_unit(&unit) {
            return None;
        }
    }
    let (b, tt) = (base.v, bill.v);
    if rate <= Q::ZERO || inc < Q::ZERO || tt <= b {
        return None;
    }
    let d = inc + (tt - b) / rate;
    let eq = format!("{} + {}(d - {}) = {}", show(b), show(rate), show(inc), show(tt));
    res(
        "plan_with_included",
        vec![d],
        eq.clone(),
        format!("d = total {} used: {}, so d = {} + ({} - {}) / {} = {} {}", unit, eq, show(inc), show(tt), show(b), show(rate), show_exact(d), unit),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 41. the most that still fits under a limit:   w + b n <= L, the greatest whole n
// ---------------------------------------------------------------------------------------------------------
const FITS_UNIT: &str = r"(?:lbs?|pounds?|kg|kilograms?)";
lazy_re!(FITS_LIMIT_BEFORE, r"\b(?:at most|up to|maximum of|no more than|capacity of|limit of|holds|can hold)\s*$");
lazy_re!(
    FITS_OTHER,
    r"\b(?:at least|more than|fewer|less than|trips|each way|round trip|twice|empty|already over|tax|discount\w*)\b|%|\$"
);
lazy_re!(FITS_CUE, r"\bat most\b|\bup to\b|\bno more than\b|\bmaximum\b|\bcapacity\b|\blimit\b");
lazy_re!(FITS_Q, r"\b(?:go|fit|fits|be loaded|be carried|be added|be put|can be|can go|can the)\b");

pub fn most_that_fits(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || FITS_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    if !FITS_CUE.is_match(&t) {
        return None;
    }
    let unit_match = re(&format!(r"\s*({})\b", FITS_UNIT));
    let unit_item = re(&format!(r"\s*{}\s+([a-z]+)", FITS_UNIT));
    let mut units: BTreeSet<&'static str> = BTreeSet::new();
    for n in &nums {
        if n.money {
            return None;
        }
        let m = unit_match.matches(&t[n.end..])?;
        units.insert(super::families_a::unit_name(m.g(1)));
    }
    if units.len() != 1 {
        return None;
    }
    let lim: Vec<Num> = nums.iter().filter(|n| FITS_LIMIT_BEFORE.is_match(t[..n.start].trim_end())).copied().collect();
    if lim.len() != 1 {
        return None;
    }
    let l = lim[0];
    let others: Vec<Num> = nums.iter().filter(|n| n.start != l.start).copied().collect();
    let q = last_sentence_with(&t, &HOW_MANY)?;
    if !FITS_Q.is_match(&q) {
        return None;
    }
    let qm = HOW_MANY_WORD.search(&q)?;
    let noun = stem_unit(qm.g(1));
    let item: Vec<Num> = others
        .iter()
        .filter(|n| unit_item.matches(&t[n.end..]).map_or(false, |m| stem_unit(m.g(1)) == noun))
        .copied()
        .collect();
    if item.len() != 1 {
        return None;
    }
    let body = item[0];
    let base = others.iter().find(|n| n.start != body.start).copied()?;
    let base_re = re(&format!(
        r"\s*{}\s+(?:worker|person|man|woman|operator|driver|passenger|helper|employee|attendant|cart|trolley|pallet|loader|mover)\b",
        FITS_UNIT
    ));
    base_re.matches(&t[base.end..])?;
    let (limit, w, b) = (l.v, base.v, body.v);
    if b <= Q::ZERO || w < Q::ZERO || w >= limit {
        return None;
    }
    let n = ((limit - w) / b).floor();
    if n < 1 || !(w + b * qi(n) <= limit && limit < w + b * qi(n + 1)) {
        return None;
    }
    let eq = format!("{} + {}n <= {}", show(w), show(b), show(limit));
    res(
        "most_that_fits",
        vec![qi(n)],
        eq.clone(),
        format!("{}, so n <= {} and the most is {} (one more would weigh {}, over {})", eq, show_exact((limit - w) / b), show_exact(qi(n)), show_exact(w + b * qi(n + 1)), show(limit)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 42. a straight fare from two rides:   4m + b = y1,  11m + b = y2, then  m x + b = target
// ---------------------------------------------------------------------------------------------------------
lazy_re!(LINE2_RIDE, r"(?<![\w$.])(\d+(?:\.\d+)?)\s+([a-z]+)\s+cost\s+\$(\d+(?:\.\d+)?)");
lazy_re!(LINE2_TARGET, r"\bhow many ([a-z]+)\s+cost\s+\$(\d+(?:\.\d+)?)");
lazy_re!(LINE2_OTHER, r"\b(?:quadratic|exponential|squared|per|each|tax|discount\w*|%)\b|%");
lazy_re!(LINE2_ASKS_MB, r"\bfind m and b\b|\bfind the rate and\b|\bfind m, b\b");

pub fn line_from_two_points(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if CHANGES_EQ_RE.is_match(&t) || LINE2_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let rides = LINE2_RIDE.find_all(&t);
    let tm = LINE2_TARGET.find_all(&t);
    if rides.len() < 2 || tm.len() != 1 || nums.len() != 2 * rides.len() + 1 {
        return None;
    }
    let unit = stem_unit(rides[0].g(2));
    if rides.iter().any(|r| stem_unit(r.g(2)) != unit) || stem_unit(tm[0].g(1)) != unit {
        return None;
    }
    let pts: Vec<(Q, Q)> = rides.iter().map(|r| (fr(r.g(1)), fr(r.g(3)))).collect();
    let ((x1, y1), (x2, y2)) = (pts[0], pts[1]);
    if x1 == x2 {
        return None;
    }
    let m = (y2 - y1) / (x2 - x1);
    let b = y1 - m * x1;
    if pts[2..].iter().any(|&(x, y)| y != m * x + b) {
        return None;
    }
    if m <= Q::ZERO || b < Q::ZERO {
        return None;
    }
    let target = fr(tm[0].g(2));
    let x = (target - b) / m;
    if x <= Q::ZERO || (!x.is_int() && !FIXED_CONTINUOUS.contains(&unit.as_str())) {
        return None;
    }
    let asks_mb = LINE2_ASKS_MB.is_match(&t);
    let vals = if asks_mb { vec![m, b, x] } else { vec![x] };
    let eq = format!("{}m + b = {};  {}m + b = {};  {}x + {} = {}", show(x1), show(y1), show(x2), show(y2), show(m), show(b), show(target));
    res(
        "line_from_two_points",
        vals,
        eq,
        format!(
            "m = ({} - {}) / ({} - {}) = {} and b = {} - {} x {} = {}: C = {}x + {}; {}x + {} = {}, so x = {} {}{}",
            show(y2),
            show(y1),
            show(x2),
            show(x1),
            show_exact(m),
            show(y1),
            show_exact(m),
            show(x1),
            show_exact(b),
            show_exact(m),
            show_exact(b),
            show_exact(m),
            show_exact(b),
            show(target),
            show_exact(x),
            unit,
            if x == Q::ONE { "" } else { "s" }
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 43. an election margin:   (2p - 100)% of V = M
// ---------------------------------------------------------------------------------------------------------
lazy_re!(VOTE_OTHER, r"\b(?:three|third|runner-up|others?|independent|spoiled|abstain\w*|absent|turnout|each|per)\b|\$");
lazy_re!(VOTE_TWO, r"\btwo[- ](?:candidate|person|way|party|man|woman)s?\b");
lazy_re!(VOTE_PCT_OF, r"\s*%\s*of\s+the\s+(?:votes|vote|ballots|total)\b");
lazy_re!(VOTE_WINNER, r"\b(?:winner|won|wins|winning)\b");
lazy_re!(VOTE_LOSER, r"\b(?:loser|lost|lose|loses)\b");
lazy_re!(VOTE_VOTES, r"\s*(?:votes|ballots)\b");
lazy_re!(VOTE_WON_BY, r"\b(?:won|wins|win) by\s*$|\bmargin of\s*$");
lazy_re!(VOTE_Q, r"\b(?:votes|ballots)\b[^.?!]*\b(?:cast|in all|total|altogether|counted)\b");

pub fn election_margin(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || t.matches('%').count() != 1 || CHANGES_EQ_RE.is_match(&t) || VOTE_OTHER.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&VOTE_TWO.sub(&t, " ")) {
        return None;
    }
    let pct: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    let other: Vec<Num> = nums.iter().filter(|n| !pct.iter().any(|p| p.start == n.start)).copied().collect();
    if pct.len() != 1 || other.len() != 1 || other[0].money {
        return None;
    }
    if VOTE_PCT_OF.matches(&t[pct[0].end..]).is_none() {
        return None;
    }
    if !VOTE_WINNER.is_match(&t) || VOTE_LOSER.is_match(&t) {
        return None;
    }
    if VOTE_VOTES.matches(&t[other[0].end..]).is_none() || !VOTE_WON_BY.is_match(t[..other[0].start].trim_end()) {
        return None;
    }
    let q = t.rfind("how many")?;
    if !VOTE_Q.is_match(&t[q..]) {
        return None;
    }
    let (p, m) = (pct[0].v, other[0].v);
    if !(Q::int(50) < p && p < Q::int(100)) || m <= Q::ZERO {
        return None;
    }
    let gap = (Q::int(2) * p - Q::int(100)) / Q::int(100);
    let v = m / gap;
    if v <= Q::ZERO || !v.is_int() {
        return None;
    }
    let eq = format!("{}% of V - {}% of V = {}", show(p), show(Q::int(100) - p), show(m));
    res("election_margin", vec![v], eq.clone(), format!("{}, so {}V = {} and V = {}", eq, show(gap), show(m), show_exact(v)))
}

// ---------------------------------------------------------------------------------------------------------
// 44. the same time with and against the wind:   a / (p + w) = b / (p - w)
// ---------------------------------------------------------------------------------------------------------
const WIND_SPEED_UNIT: &str = r"(?:mph|km/h|kph|miles per hour|kilometers per hour|kilometres per hour)";
const WIND_DIST_UNIT: &str = r"(?:miles?|mi|km|kilometers?|kilometres?)";
lazy_re!(
    WIND_WITH,
    &format!(r"(\d+(?:\.\d+)?)\s*({WIND_DIST_UNIT})\s+with\s+an?\s+(\d+(?:\.\d+)?)\s*({WIND_SPEED_UNIT})\s+(?:tailwind|wind)")
);
lazy_re!(
    WIND_AGAINST,
    &format!(r"in the same time (?:it|the \w+) flies\s+(\d+(?:\.\d+)?)\s*({WIND_DIST_UNIT})\s+(?:into|against)\s+(?:the same|that|this|the)\s+(?:wind|headwind)")
);
lazy_re!(WIND_OTHER, r"\b(?:round trip|return\w*|back|stops?|rests?|twice|fuel|climb\w*|descend\w*)\b|%|\$");
lazy_re!(
    WIND_ASKS,
    r"\bspeed\b[^.?!]*\b(?:still air|calm air|no wind|without (?:the )?wind)\b|\b(?:still air|calm air)\b[^.?!]*\bspeed\b"
);

pub fn wind_same_time(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || WIND_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let wm = WIND_WITH.search(&t)?;
    let am = WIND_AGAINST.search(&t)?;
    let (a, w, b) = (fr(wm.g(1)), fr(wm.g(3)), fr(am.g(1)));
    let (d1, d2) = (dist_group(wm.g(2)), dist_group(am.g(2)));
    if !(d1 == d2 && d2 == speed_group(wm.g(4))) {
        return None;
    }
    if !WIND_ASKS.is_match(&t) {
        return None;
    }
    if a <= Q::ZERO || b <= Q::ZERO || w <= Q::ZERO || b >= a {
        return None;
    }
    let p = w * (a + b) / (a - b);
    if p <= w {
        return None;
    }
    let eq = format!("{}/(p + {}) = {}/(p - {})", show(a), show(w), show(b), show(w));
    res(
        "wind_same_time",
        vec![p],
        eq.clone(),
        format!("{}, so {}(p - {}) = {}(p + {}) and p = {} {}", eq, show(a), show(w), show(b), show(w), show_exact(p), if d1 == "mi" { "mph" } else { "km/h" }),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 45. an allowance plus extra chores over some weeks:   w a + c n = T
// ---------------------------------------------------------------------------------------------------------
lazy_re!(EXTRA_BASE, r"\$(\d+(?:\.\d+)?)\s+(?:allowance|pay|salary|wage|stipend|base pay)\s+(?:a|per|each|every)\s+(week|month|day|year)\b");
lazy_re!(EXTRA_RATE, r"\$(\d+(?:\.\d+)?)\s+(?:for|per)\s+(?:each|every)\s+(?:extra|additional)\s+([a-z]+)\b");
lazy_re!(EXTRA_PERIOD, r"\b(?:over|for|in|during)\s+(\d+(?:\.\d+)?)\s+(weeks?|months?|days?|years?)\b");
lazy_re!(EXTRA_TOTAL_BEFORE, r"\b(?:came to|comes to|totaled|totalled|total of|earned|received|got|made|paid|add(?:ed)? up to|adds up to)\s*$");
lazy_re!(EXTRA_OTHER, r"\b(?:tax|bonus|deduct\w*|fine|fines|lost|missed|half|twice|each week they|average)\b|%");
lazy_re!(HOW_WORD, r"\bhow\b");
lazy_re!(EXTRA_Q_BAD, r"\bhow much\b|\btotal\b|\bper\b");
lazy_re!(EXTRA_HOW_MANY, r"\bhow many (?:extra |additional )?([a-z]+)");

pub fn base_plus_extra_over_time(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 4 || CHANGES_EQ_RE.is_match(&t) || EXTRA_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let bm = EXTRA_BASE.search(&t)?;
    let rm = EXTRA_RATE.search(&t)?;
    let pm = EXTRA_PERIOD.search(&t)?;
    let (a, c, w) = (fr(bm.g(1)), fr(rm.g(1)), fr(pm.g(1)));
    if stem_unit(pm.g(2)) != stem_unit(bm.g(2)) {
        return None;
    }
    let used = [bm.group_start(1)?, rm.group_start(1)?, pm.group_start(1)?];
    let rest: Vec<Num> = nums.iter().filter(|n| !used.iter().any(|&u| pos_in(n, u))).copied().collect();
    if rest.len() != 1 || !rest[0].money {
        return None;
    }
    let tn = rest[0];
    if !EXTRA_TOTAL_BEFORE.is_match(t[..tn.start].trim_end()) || EACH_AFTER.matches(&t[tn.end..]).is_some() {
        return None;
    }
    let q = last_sentence_with(&t, &HOW_WORD)?;
    if EXTRA_Q_BAD.is_match(&q) {
        return None;
    }
    let qm = EXTRA_HOW_MANY.search(&q)?;
    if stem_unit(qm.g(1)) != stem_unit(rm.g(2)) {
        return None;
    }
    if c <= Q::ZERO || a < Q::ZERO || w <= Q::ZERO || tn.v <= a * w {
        return None;
    }
    let n = (tn.v - a * w) / c;
    if n <= Q::ZERO || !n.is_int() {
        return None;
    }
    let eq = format!("{} x {} + {}n = {}", show(w), show(a), show(c), show(tn.v));
    res(
        "base_plus_extra_over_time",
        vec![n],
        eq.clone(),
        format!("n = number of extra {}s: {}, so n = ({} - {}) / {} = {}", stem_unit(rm.g(2)), eq, show(tn.v), show(a * w), show(c), show_exact(n)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 46. distance, speed and time:   D = S t, the one the question asks for
// ---------------------------------------------------------------------------------------------------------
lazy_re!(DST_DIST, &format!(r"{NUM}\s*(miles?|mi|kilometers?|kilometres?|km)\b(?![/\w])"));
lazy_re!(DST_TIME, &format!(r"{NUM}\s*(hours?|hrs?|h)(?![\w/])"));
lazy_re!(
    DST_OTHER,
    r"\b(?:then|after that|back|return\w*|round trip|each way|stops?|rests?|breaks?|also|another|faster than|slower than|twice|acceleration|average speed|against|with the|current|wind|stream)\b|%|\$|²"
);
lazy_re!(DST_ASKS_TIME, r"\bhow long\b|\bhow (?:many|much) (?:hours?|time)\b|\bwhat time\b");
lazy_re!(DST_ASKS_DIST, r"\bhow far\b|\bwhat distance\b|\bhow many (?:miles|kilometers|kilometres|km)\b");
lazy_re!(DST_ASKS_SPEED, r"\bhow fast\b|\bwhat (?:speed|rate)\b|\bwhat is the speed\b|\bhow many (?:mph|km/h)\b");

pub fn distance_speed_time(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || CHANGES_EQ_RE.is_match(&t) || DST_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let sp = SPEED.find_all(&t);
    let spans: Vec<(usize, usize)> = sp.iter().map(|m| (m.start(), m.end())).collect();
    let outside = |s: usize| !spans.iter().any(|&(a, b)| a <= s && s < b);
    let dist: Vec<_> = DST_DIST.find_all(&t).into_iter().filter(|m| outside(m.start())).collect();
    let tim: Vec<_> = DST_TIME.find_all(&t).into_iter().filter(|m| outside(m.start())).collect();
    let counts = [("speed", sp.len()), ("distance", dist.len()), ("time", tim.len())];
    let present: Vec<&str> = counts.iter().filter(|(_, n)| *n == 1).map(|(k, _)| *k).collect();
    if present.len() != 2 || counts.iter().any(|(_, n)| *n > 1) || counts.iter().map(|(_, n)| n).sum::<usize>() != 2 {
        return None;
    }
    let q = match t.rfind("how ") {
        Some(i) => i,
        None => t.rfind("what ")?,
    };
    let tail = &t[q..];
    let asks_time = DST_ASKS_TIME.is_match(tail);
    let asks_dist = DST_ASKS_DIST.is_match(tail);
    let asks_speed = DST_ASKS_SPEED.is_match(tail);
    if [asks_time, asks_dist, asks_speed].iter().filter(|b| **b).count() != 1 {
        return None;
    }
    let missing: Vec<&str> = ["speed", "distance", "time"].iter().filter(|k| !present.contains(k)).copied().collect();
    if missing.len() != 1 {
        return None;
    }
    let asked = match missing[0] {
        "time" => asks_time,
        "distance" => asks_dist,
        _ => asks_speed,
    };
    if !asked {
        return None;
    }
    let mut s_v = Q::ZERO;
    let mut d_v = Q::ZERO;
    let mut t_v = Q::ZERO;
    let mut sgroup = "";
    let mut dgroup = "";
    if let Some(m) = sp.first() {
        s_v = fr(m.g(1));
        sgroup = speed_group(m.g(2));
    }
    if let Some(m) = dist.first() {
        d_v = fr(m.g(1));
        dgroup = dist_group(m.g(2));
    }
    if let Some(m) = tim.first() {
        t_v = fr(m.g(1));
    }
    let has_s = !sp.is_empty();
    let has_d = !dist.is_empty();
    let has_t = !tim.is_empty();
    if has_s && has_d && sgroup != dgroup {
        return None;
    }
    let unit_d = if has_s {
        if sgroup == "mi" {
            "miles"
        } else {
            "km"
        }
    } else if dgroup == "mi" {
        "miles"
    } else {
        "km"
    };
    let unit_s = if (has_s && sgroup == "mi") || (!has_s && dgroup == "mi") { "mph" } else { "km/h" };
    if (has_s && s_v <= Q::ZERO) || (has_d && d_v <= Q::ZERO) || (has_t && t_v <= Q::ZERO) {
        return None;
    }
    let (r, eq, text);
    match missing[0] {
        "time" => {
            r = d_v / s_v;
            eq = format!("{} = {}t", show(d_v), show(s_v));
            text = format!("{}, so t = {} hour{}", eq, show_exact(r), if r == Q::ONE { "" } else { "s" });
        }
        "distance" => {
            r = s_v * t_v;
            eq = format!("d = {} x {}", show(s_v), show(t_v));
            text = format!("{}, so d = {} {}", eq, show_exact(r), unit_d);
        }
        _ => {
            r = d_v / t_v;
            eq = format!("{} = s x {}", show(d_v), show(t_v));
            text = format!("{}, so s = {} {}", eq, show_exact(r), unit_s);
        }
    }
    res("distance_speed_time", vec![r], eq, text)
}

// ---------------------------------------------------------------------------------------------------------
// 47. pages a day to finish a book:   (N - R) / D
// ---------------------------------------------------------------------------------------------------------
lazy_re!(PAGES_TOTAL, r"\b(\d+)-pages?\b|\b(book|novel|report|packet|manual)\s+(?:has|of|with)\s+(\d+)\s+pages\b");
const PAGES_BOOKISH: &[&str] = &["book", "novel", "story", "text", "volume", "paperback", "textbook"];
const PAGES_FUNCTION: &[&str] = &["the", "this", "that", "it", "my", "her", "his", "their", "its", "reading", "read", "rest", "remaining", "of", "pages", "page"];
lazy_re!(PAGES_READ, r"\b(\d+)\s+pages?\s+(?:already\s+)?(?:read|finished|done|completed|written|typed)\b");
lazy_re!(PAGES_DAYS, r"\bfinish((?:\s+[a-z]+){0,5}?)\s+(?:in|within|over)\s+(\d+)\s+(days?|weeks?|nights?)\b");
lazy_re!(PAGES_OTHER, r"\b(?:more than|less than|twice|double|half|each day she|already read twice|skip\w*|except|weekend\w*|at most|at least)\b|%|\$");
lazy_re!(PAGES_NOUN_AFTER, r"\s+([a-z]+)");
lazy_re!(PAGES_FINISH, r"\bfinish(?:ing)?\b");
lazy_re!(LETTERS, r"[a-z]+");
lazy_re!(PAGES_HOW_MANY, r"\bhow many pages\b");
lazy_re!(PAGES_MUST, r"\b(?:must|need to|have to|should|has to|needs to)\b");

pub fn pages_per_day(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || CHANGES_EQ_RE.is_match(&t) || PAGES_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let tm = PAGES_TOTAL.search(&t)?;
    let rm = PAGES_READ.search(&t)?;
    let dm = PAGES_DAYS.search(&t)?;
    let total_txt = tm.opt(1).or_else(|| tm.opt(3))?;
    let n = fr(total_txt);
    let (r, d) = (fr(rm.g(1)), fr(dm.g(2)));
    let starts: BTreeSet<usize> = [tm.start(), rm.start(), dm.start()].into_iter().collect();
    if starts.len() != 3 {
        return None;
    }
    let nm = if tm.opt(1).is_some() { PAGES_NOUN_AFTER.matches(&t[tm.end()..]) } else { None };
    let noun: String = match tm.opt(2) {
        Some(x) => x.to_string(),
        None => nm.map(|m| m.g(1).to_string())?,
    };
    let mut allowed: BTreeSet<&str> = PAGES_FUNCTION.iter().copied().collect();
    if PAGES_BOOKISH.contains(&noun.as_str()) {
        allowed.extend(PAGES_BOOKISH.iter().copied());
    }
    allowed.insert(noun.as_str());
    if dm.g(1).split_whitespace().any(|w| !allowed.contains(w)) {
        return None;
    }
    for fm in PAGES_FINISH.find_all(&t) {
        let rest = &t[fm.end()..];
        let after = match rest.find(|c| c == '.' || c == '?' || c == '!') {
            Some(i) => &rest[..i],
            None => rest,
        };
        let mut words: Vec<String> = LETTERS.find_all(after).iter().map(|m| m.text().to_string()).collect();
        for stop in ["in", "within", "over"] {
            if let Some(i) = words.iter().position(|w| w == stop) {
                let mut tail: Vec<String> = words[i + 1..].to_vec();
                if let Some(first) = tail.first() {
                    if ["day", "days", "week", "weeks", "night", "nights"].contains(&first.as_str()) {
                        tail.remove(0);
                    }
                }
                words.truncate(i);
                words.extend(tail);
            }
        }
        if words.iter().any(|w| !allowed.contains(w.as_str())) {
            return None;
        }
    }
    let unit = dm.g(3).trim_end_matches('s').to_string();
    let q = last_sentence_with(&t, &PAGES_HOW_MANY)?;
    if !re(&format!(r"\b(?:a|per|each|every)\s+{}\b", unit)).is_match(&q) || !PAGES_MUST.is_match(&q) {
        return None;
    }
    if d <= Q::ZERO || r < Q::ZERO || r >= n {
        return None;
    }
    let p = (n - r) / d;
    let eq = format!("{} + {}p = {}", show(r), show(d), show(n));
    res("pages_per_day", vec![p], eq.clone(), format!("p = pages per {}: {}, so p = ({} - {}) / {} = {}", unit, eq, show(n), show(r), show(d), show_exact(p)))
}

// ---------------------------------------------------------------------------------------------------------
// 48. opposite (vertical) angles are equal:   a1 x + b1 = a2 x + b2
// ---------------------------------------------------------------------------------------------------------
lazy_re!(OPPANG_EXPR, r"\(\s*(\d*)\s*x\s*([+-])\s*(\d+(?:\.\d+)?)\s*\)\s*(?:°|degrees?)?");
lazy_re!(
    OPPANG_OTHER,
    r"\b(?:adjacent|supplementary|complementary|complements?|supplements?|linear pair|straight line|add to|adds to|sum|total|together|triangle|parallel lines|difference|product|ratio|half|double|twice|bisect\w*|quadrilateral|cyclic|inscribed|circle|arc|chord|polygon)\b"
);
lazy_re!(TWO_LINES, r"\btwo (?:straight )?lines\b");
lazy_re!(OPPANG_CUE, r"\b(?:opposite|vertical) angles\b");
lazy_re!(OPPANG_FORMS, r"\bforms?\s+(?:an?\s+)?(?:acute|obtuse|right|straight|reflex)\s+angle");
lazy_re!(ANGLE_KIND, r"\b(acute|obtuse|right|straight|reflex)\s+angles?\b");
lazy_re!(ANGLES_WORD, r"\bangles?\b");
lazy_re!(X_WORD, r"\bx\b");
lazy_re!(BOTH_ANGLES, r"\bboth angles\b");

/// `(a, b)` of each "(2x + 4)" expression, or None when the problem has digits elsewhere.
fn two_expressions(t: &str) -> Option<((Q, Q), (Q, Q))> {
    let ex = OPPANG_EXPR.find_all(t);
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
    Some((coef(&ex[0]), coef(&ex[1])))
}

fn sign_word(b: Q) -> &'static str {
    if b >= Q::ZERO {
        "+"
    } else {
        "-"
    }
}

pub fn opposite_angles_equal(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if CHANGES_EQ_RE.is_match(&t) || OPPANG_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&TWO_LINES.sub(&t, " ")) {
        return None;
    }
    if !OPPANG_CUE.is_match(&t) {
        return None;
    }
    let ((a1, b1), (a2, b2)) = two_expressions(&t)?;
    if a1 == a2 {
        return None;
    }
    let x = (b2 - b1) / (a1 - a2);
    let angle = a1 * x + b1;
    if !(Q::ZERO < angle && angle <= Q::int(180)) {
        return None;
    }
    if OPPANG_FORMS.is_match(&t) {
        return None;
    }
    for m in ANGLE_KIND.find_all(&t) {
        let ok = match m.g(1) {
            "acute" => Q::ZERO < angle && angle < Q::int(90),
            "obtuse" => Q::int(90) < angle && angle < Q::int(180),
            "right" => angle == Q::int(90),
            "straight" => angle == Q::int(180),
            _ => false,
        };
        if !ok {
            return None;
        }
    }
    let q = rfind_max(&t, &["find", "what", "how"])?;
    let tail = &t[q..];
    let asks_angle = ANGLES_WORD.is_match(tail);
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
        "opposite_angles_equal",
        vals,
        eq.clone(),
        format!("opposite angles are equal: {}, so x = {} and each angle is {} degrees", eq, show_exact(x), show_exact(angle)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 49. a rectangle from its perimeter and its area:   l + w = P / 2,  l w = A
// ---------------------------------------------------------------------------------------------------------
lazy_re!(RECT_CARET2, r"\^\s*2\b");
lazy_re!(RECT_PERIM, r"\bperimeter of\s+(\d+(?:\.\d+)?)\s*([a-z]+)\b");
lazy_re!(RECT_AREA, r"\barea of\s+(\d+(?:\.\d+)?)\s*(?:(square|sq\.?)\s+)?([a-z]+)(²|\^2|2)?");
const RECT_LONG: &str = r"(?:longer|larger|greater|bigger|wider|broader|more|longest|largest|greatest|widest)";
const RECT_SHORT: &str = r"(?:shorter|smaller|lesser|narrower|less|shortest|smallest|least|narrowest)";
const RECT_BOUNDARY: &str = r"(?=\s*(?:$|[,;.:?!]|\b(?:and|but|while|so|then|if|when)\b))";
lazy_re!(
    RECT_REL,
    &format!(
        r"\b(length|width|it|one)\s+(?:is|are|being)\s+(?:the\s+)?({RECT_LONG}|{RECT_SHORT})(?:\s+(?:side|one|dimension))?(?:\s+than\s+(?:the\s+|its\s+)?(length|width|other(?:\s+(?:side|one|dimension))?))?{RECT_BOUNDARY}"
    )
);
lazy_re!(
    RECT_SIDE,
    &format!(r"\b({RECT_LONG}|{RECT_SHORT})\s+(?:side|dimension|one)\s+(?:is|being)\s+(?:the\s+)?(length|width){RECT_BOUNDARY}")
);
lazy_re!(RECT_LONG_RE, RECT_LONG);
lazy_re!(RECT_ANY_COMPARE, &format!(r"\b(?:{RECT_LONG}|{RECT_SHORT})\b"));
lazy_re!(
    RECT_OTHER,
    r"\b(?:same as|equal to|equals?|as long as|as wide as|square(?! (?:feet|foot|ft|meters?|metres?|m|cm|centimeters?|centimetres?|inch(?:es)?|in|yards?|yd|miles?|mi|km|kilometers?)\b)|diagonal|border|frame|path|cut|hole|cost|triangle|circle|twice|times as)\b|%|\$"
);
lazy_re!(RECT_RECTANGLE, r"\brectangle\b");
lazy_re!(
    RECT_Q,
    r"\b(?:find|what|determine|give|name|calculate|compute|state|list|work out|how (?:long|wide|big|tall))\b"
);
lazy_re!(RECT_COND, r"\b(?:if|when|given that|since|because|assuming|provided)\b");
lazy_re!(RECT_SPLIT, r"(?:,\s*)?\b(?:so|then|therefore)\b|;");
lazy_re!(
    RECT_NAMED,
    &format!(r"\b(?:(?:length|width)\s+of|how\s+(?:long|wide|big|tall)\s+(?:is|are))\s+the\s+({RECT_LONG}|{RECT_SHORT})\s+(?:side|one|dimension)\b(?=\s*(?:,|$))")
);
lazy_re!(RECT_WHICH, r"\b(length|width)\s*,?\s*(?:which|that)\s+(?:is|are)\s+([^,]*?)(?=\s*(?:,|$))");
lazy_re!(
    RECT_APPOS,
    &format!(r"\b(length|width)\s*,?\s*(?:the\s+)?((?:{RECT_LONG}|{RECT_SHORT})\s+(?:side|one|dimension))(?=\s*(?:,|$))")
);
lazy_re!(LENGTH_WORD, r"\blength\b");
lazy_re!(WIDTH_WORD, r"\bwidth\b");
lazy_re!(RECT_Q_BAD, r"\b(?:sum|difference|product|ratio|half|double|average|perimeter|area|per|more|less|larger than|longer than)\b");
lazy_re!(SIDES_WORD, r"\b(?:sides|dimensions)\b");
lazy_re!(
    RECT_OUTCOME_CUT,
    r"\b(?:find|what|determine|give|name|calculate|compute|state|list|work out|how (?:long|wide|big|tall))\b"
);

fn rect_unit(w: &str) -> &str {
    match w {
        "feet" | "foot" => "ft",
        "meters" | "metres" | "meter" | "metre" => "m",
        "centimeters" | "centimetres" => "cm",
        "inches" | "inch" => "in",
        "yards" | "yard" => "yd",
        "miles" | "mile" => "mi",
        "kilometers" | "kilometres" => "km",
        other => other,
    }
}

/// `Ok(Some(true))` when the length is the longer side, `Ok(Some(false))` when it is the shorter, `Ok(None)` when the text does not say,
/// `Err(())` when it contradicts itself or is unclear (Python's "bad").
fn rect_length_is_longer(text: &str, asked: Option<&str>) -> Result<Option<bool>, ()> {
    let mut found: Vec<(String, bool)> = vec![];
    for m in RECT_REL.find_all(text) {
        let dim = m.g(1);
        let rel = m.g(2);
        let other = m.opt(3);
        let after = &text[m.group_end(2).unwrap_or(m.end())..m.end()];
        if after.trim().is_empty() {
            return Err(());
        }
        let dim: String = if dim == "it" || dim == "one" {
            match asked {
                Some(a) => a.to_string(),
                None => return Err(()),
            }
        } else {
            dim.to_string()
        };
        if let Some(o) = other {
            if o == dim {
                return Err(());
            }
        }
        found.push((dim, RECT_LONG_RE.matches(rel).is_some()));
    }
    for m in RECT_SIDE.find_all(text) {
        found.push((m.g(2).to_string(), RECT_LONG_RE.matches(m.g(1)).is_some()));
    }
    let mut spans: Vec<(usize, usize)> = RECT_REL.find_all(text).iter().map(|m| (m.start(), m.end())).collect();
    spans.extend(RECT_SIDE.find_all(text).iter().map(|m| (m.start(), m.end())));
    for w in RECT_ANY_COMPARE.find_all(text) {
        if !spans.iter().any(|&(a, b)| a <= w.start() && w.start() < b) {
            return Err(());
        }
    }
    if found.is_empty() {
        return Ok(None);
    }
    let verdicts: BTreeSet<bool> = found.iter().map(|(dim, longer)| if dim == "length" { *longer } else { !*longer }).collect();
    if verdicts.len() == 1 {
        Ok(verdicts.into_iter().next())
    } else {
        Err(())
    }
}

pub fn rectangle_from_perimeter_area(problem: &str) -> Option<SolveResult> {
    let problem = RECT_CARET2.sub(problem, "²");
    let problem = problem.as_str();
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || CHANGES_EQ_RE.is_match(&t) || RECT_OTHER.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    if !RECT_RECTANGLE.is_match(&t) {
        return None;
    }
    let pm = RECT_PERIM.search(&t)?;
    let am = RECT_AREA.search(&t)?;
    if rect_unit(pm.g(2)) != rect_unit(am.g(3)) {
        return None;
    }
    if am.opt(2).is_none() && am.opt(4).is_none() {
        return None;
    }
    let sents = sentence_list(&t);
    let q_idx = sents.iter().rposition(|x| RECT_Q.is_match(x))?;
    let q = sents[q_idx].clone();
    let (mut q_main, mut cond): (String, String) = match RECT_COND.search(&q) {
        Some(cm) => (q[..cm.start()].to_string(), q[cm.end()..].to_string()),
        None => (q.clone(), String::new()),
    };
    let q_parts: Vec<String> = if q_main.is_empty() { vec![q_main.clone()] } else { RECT_SPLIT.split(&q_main) };
    if q_parts.len() > 1 {
        cond = format!("{} . {}", cond, q_parts[..q_parts.len() - 1].join(" . "));
        q_main = q_parts[q_parts.len() - 1].clone();
    }
    let mut named_kind: Option<&str> = None;
    if let Some(om) = RECT_NAMED.search(&q_main) {
        named_kind = Some(if RECT_LONG_RE.matches(om.g(1)).is_some() { "long" } else { "short" });
        q_main = format!("{} {}", &q_main[..om.start()], &q_main[om.end()..]);
    }
    let mut which = true;
    let mut wm = RECT_WHICH.search(&q_main);
    if wm.is_none() {
        wm = RECT_APPOS.search(&q_main);
        which = false;
    }
    if let Some(wm) = wm {
        let (g1, g2) = (wm.g(1).to_string(), wm.g(2).to_string());
        let (e1, e) = (wm.group_end(1)?, wm.end());
        let before = q_main[..e1].to_string();
        cond = format!("{} . {}{}{}", cond, g1, if which { " is " } else { " is the " }, g2);
        let new_main = format!("{}{}", &q_main[..e1], &q_main[e..]);
        q_main = new_main;
        if LENGTH_WORD.is_match(&before) && WIDTH_WORD.is_match(&before) {
            return None;
        }
    }
    if RECT_Q_BAD.is_match(&q_main) {
        return None;
    }
    if RECT_ANY_COMPARE.is_match(&q_main) {
        return None;
    }
    let q = q_main;
    let asks_len = LENGTH_WORD.is_match(&q);
    let asks_wid = WIDTH_WORD.is_match(&q);
    let asks_both = SIDES_WORD.is_match(&q) || (asks_len && asks_wid);
    if named_kind.is_some() && (asks_len || asks_wid || asks_both) {
        return None;
    }
    if !(asks_len || asks_wid || asks_both || named_kind.is_some()) {
        return None;
    }
    let mut parts: Vec<String> = sents.iter().enumerate().filter(|(i, _)| *i != q_idx).map(|(_, x)| x.clone()).collect();
    parts.push(cond);
    let others = parts.join(" . ");
    let asked_side = if asks_len && !asks_wid && !asks_both {
        Some("length")
    } else if asks_wid && !asks_len && !asks_both {
        Some("width")
    } else {
        None
    };
    let length_longer = rect_length_is_longer(&others, asked_side).ok()?;
    let (p, a) = (fr(pm.g(1)), fr(am.g(1)));
    if p <= Q::ZERO || a <= Q::ZERO {
        return None;
    }
    let s = p / Q::int(2);
    let disc = s * s - Q::int(4) * a;
    if disc < Q::ZERO {
        return None;
    }
    let rn = isqrt(disc.numer())?;
    let rd = isqrt(disc.denom())?;
    let root = Q::new(rn, rd);
    let (l, w) = ((s + root) / Q::int(2), (s - root) / Q::int(2));
    if w <= Q::ZERO {
        return None;
    }
    let unit = pm.g(2);
    let (length, width) = if length_longer == Some(false) { (w, l) } else { (l, w) };
    let vals = if let Some(k) = named_kind {
        if k == "long" {
            vec![l]
        } else {
            vec![w]
        }
    } else if asks_both {
        vec![length, width]
    } else if asks_len {
        vec![length]
    } else {
        vec![width]
    };
    res(
        "rectangle_from_perimeter_area",
        vals,
        format!("l + w = {};  l w = {}", show(s), show(a)),
        format!(
            "l + w = {} and l w = {}, so the sides are the roots of z^2 - {}z + {} = 0: l = {} and w = {} {}",
            show(s),
            show(a),
            show(s),
            show(a),
            show_exact(l),
            show_exact(w),
            unit
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 50. a falling temperature, in whole hours:   T0 - r h >= L  (or > L)
// ---------------------------------------------------------------------------------------------------------
lazy_re!(TEMP_VAL, r"(-?\d+(?:\.\d+)?)\s*°\s*([fc])\b");
lazy_re!(TEMP_FALL, r"\b(?:falling|dropping|decreasing|cooling|falls|drops|decreases|cools|going down)\s*(?:by\s+)?$");
lazy_re!(TEMP_INCLUSIVE, r"\b(?:at or above|at least|no lower than|not below|or above)\s*$");
lazy_re!(TEMP_STRICT, r"\b(?:above|over|higher than|warmer than)\s*$");
lazy_re!(
    TEMP_OTHER,
    r"\b(?:rising|climbing|warming|increasing|rises|climbs|warms|increases|going up|below|at or below|under|colder than|no higher than|at most)\b"
);
lazy_re!(AT_OR_ABOVE, r"\bat or above\b");
lazy_re!(TEMP_START_BEFORE, r"\b(?:is|starts at|begins at|reads|now|currently)\s*$");
lazy_re!(TEMP_PER_HOUR, r"\s*(?:per|an|each|every|a)\s+hour\b");
lazy_re!(TEMP_WHOLE, r"\bhow many (?:whole|full) hours\b");

pub fn temperature_whole_hours(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let vals = TEMP_VAL.find_all(&t);
    if vals.len() != 3 || digit_numbers(problem).len() != 3 || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&t) || t.contains('$') || t.contains('%') {
        return None;
    }
    let scales: BTreeSet<&str> = vals.iter().map(|v| v.g(2)).collect();
    if scales.len() != 1 {
        return None;
    }
    if TEMP_OTHER.is_match(&AT_OR_ABOVE.sub(&t, " ")) {
        return None;
    }
    let before = |v: &super::rx::M| t[..v.start()].trim_end().to_string();
    let rate: Vec<usize> = (0..3).filter(|&i| TEMP_FALL.is_match(&before(&vals[i]))).collect();
    let limit: Vec<usize> = (0..3).filter(|&i| TEMP_INCLUSIVE.is_match(&before(&vals[i])) || TEMP_STRICT.is_match(&before(&vals[i]))).collect();
    if rate.len() != 1 || limit.len() != 1 || rate[0] == limit[0] {
        return None;
    }
    let start: Vec<usize> = (0..3).filter(|&i| i != rate[0] && i != limit[0]).collect();
    if start.len() != 1 || !TEMP_START_BEFORE.is_match(&before(&vals[start[0]])) {
        return None;
    }
    if TEMP_PER_HOUR.matches(&t[vals[rate[0]].end()..]).is_none() {
        return None;
    }
    let inclusive = TEMP_INCLUSIVE.is_match(&before(&vals[limit[0]]));
    if !TEMP_WHOLE.is_match(&t) {
        return None;
    }
    let (t0, r, l) = (fr(vals[start[0]].g(1)), fr(vals[rate[0]].g(1)), fr(vals[limit[0]].g(1)));
    if r <= Q::ZERO {
        return None;
    }
    let q = (t0 - l) / r;
    let h = if inclusive { q.floor() } else { q.ceil() - 1 };
    if h < 1 {
        return None;
    }
    let unit = format!("°{}", vals[start[0]].g(2).to_uppercase());
    let eq = format!("{} - {}h {} {}", show(t0), show(r), if inclusive { ">=" } else { ">" }, show(l));
    res(
        "temperature_whole_hours",
        vec![qi(h)],
        eq.clone(),
        format!(
            "{}, so h {} {} and the most whole hours is {} (after {} hours it is {}{}, after {} it is {}{})",
            eq,
            if inclusive { "<=" } else { "<" },
            show_exact(q),
            h,
            h,
            show_exact(t0 - r * qi(h)),
            unit,
            h + 1,
            show_exact(t0 - r * qi(h + 1)),
            unit
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 51. two candles burning down until they are the same height:   a - ra t = b - rb t
// ---------------------------------------------------------------------------------------------------------
const CANDLE_UNIT: &str = r"(?:cm|mm|m|inch(?:es)?|in)";
lazy_re!(
    CANDLE_STMT,
    &format!(r"\bis\s+(\d+(?:\.\d+)?)\s*({CANDLE_UNIT})\s+tall\s+and\s+burns\s+(\d+(?:\.\d+)?)\s*({CANDLE_UNIT})\s+(?:per|an|each|every|a)\s+(hour|minute|day)\b")
);
lazy_re!(CANDLES_WORD, r"\bcandles?\b");
lazy_re!(TWO_CANDLES, r"\btwo candles\b");
lazy_re!(CANDLE_WHEN, r"\b(?:when|how long|after how many (?:hours|minutes|days))\b");
lazy_re!(CANDLE_SAME, r"\bsame height\b|\bequal height\b|\bsame size\b|\bequally tall\b");
lazy_re!(
    CANDLE_ASKS,
    r"\b(?:when|how|what|why|who|where|which|will|would|find|determine|calculate|compute|give|state|name|tell|list|solve|show)\b|\bsame height\b|\bequal height\b"
);
lazy_re!(CANDLE_AS, r"\b(?:same|equal) height\s+(?:as|to|with)\b(.*)$");
lazy_re!(CANDLE_EACH_OTHER, r"\s*(?:each other|one another|the other(?:\s+(?:candle|one))?)\s*$");
lazy_re!(CANDLE_HOW_TALL, r"\bhow (?:tall|high|old)\b|\bwhat (?:height|size)\b|\bhow much\b");

pub fn two_candles_equal(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let found = CANDLE_STMT.find_all(&t);
    if found.len() != 2 || digit_numbers(problem).len() != 4 || !CANDLES_WORD.is_match(&t) || CHANGES_EQ_RE.is_match(&t) || SPELLED_ANY_RE.is_match(&TWO_CANDLES.sub(&t, " ")) {
        return None;
    }
    let (m1, m2) = (&found[0], &found[1]);
    if !(m1.g(2) == m1.g(4) && m1.g(4) == m2.g(2) && m2.g(2) == m2.g(4)) || m1.g(5) != m2.g(5) {
        return None;
    }
    let rest = format!("{} {} {}", &t[..m1.start()], &t[m1.end()..m2.start()], &t[m2.end()..]);
    if DIGIT_RE.is_match(&rest) {
        return None;
    }
    let sentences = bare_sentences(&t);
    let remainders: Vec<String> = sentences.iter().map(|x| CANDLE_STMT.sub(x, " ")).collect();
    let asks: Vec<usize> = (0..remainders.len()).filter(|&i| CANDLE_WHEN.is_match(&remainders[i]) && CANDLE_SAME.is_match(&remainders[i])).collect();
    if asks.len() != 1 {
        return None;
    }
    let question = &remainders[asks[0]];
    if remainders.iter().enumerate().any(|(i, x)| i != asks[0] && CANDLE_ASKS.is_match(x)) {
        return None;
    }
    for cm in CANDLE_AS.find_all(question) {
        if CANDLE_EACH_OTHER.matches(cm.g(1)).is_none() {
            return None;
        }
    }
    if CANDLE_HOW_TALL.is_match(question) {
        return None;
    }
    let (a, ra, b, rb) = (fr(m1.g(1)), fr(m1.g(3)), fr(m2.g(1)), fr(m2.g(3)));
    if ra <= Q::ZERO || rb <= Q::ZERO || ra == rb {
        return None;
    }
    let tt = (a - b) / (ra - rb);
    if tt <= Q::ZERO {
        return None;
    }
    let height = a - ra * tt;
    if height <= Q::ZERO {
        return None;
    }
    let unit = m1.g(5);
    let eq = format!("{} - {}t = {} - {}t", show(a), show(ra), show(b), show(rb));
    res(
        "two_candles_equal",
        vec![tt],
        eq.clone(),
        format!("t = {}s: {}, so t = {} {}s (both candles are still {} {} tall)", unit, eq, show_exact(tt), unit, show_exact(height), m1.g(2)),
    )
}

pub fn list() -> Vec<(&'static str, Family)> {
    vec![
        ("two_rates_one_pile", two_rates_one_pile),
        ("least_for_profit", least_for_profit),
        ("two_priced_items", two_priced_items),
        ("plan_with_included", plan_with_included),
        ("most_that_fits", most_that_fits),
        ("line_from_two_points", line_from_two_points),
        ("election_margin", election_margin),
        ("wind_same_time", wind_same_time),
        ("base_plus_extra_over_time", base_plus_extra_over_time),
        ("distance_speed_time", distance_speed_time),
        ("pages_per_day", pages_per_day),
        ("opposite_angles_equal", opposite_angles_equal),
        ("rectangle_from_perimeter_area", rectangle_from_perimeter_area),
        ("temperature_whole_hours", temperature_whole_hours),
        ("two_candles_equal", two_candles_equal),
    ]
}
