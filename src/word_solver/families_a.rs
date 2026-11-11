//! Families 1-10 of `word_solver.py`: sum and difference, consecutive integers, work together, percent change, proportion, mixtures, interest, purchases, price blend.
//! (Families 11 and on are in `families_b.rs` and `families_c.rs`, in Python's FAMILIES order.)

use super::common::*;
use super::rx::{re, Re};
use crate::lazy_re;
use std::collections::BTreeSet;
use std::sync::LazyLock;

const MORE: &str = r"(?:more|bigger|larger|greater|longer|higher|taller|heavier|older)";
const UNITS_LEN: &str = r"(?:cm|m|mm|km|ft|feet|foot|in|inch|inches|yd|yards?|meters?|metres?)";

lazy_re!(SD1, &format!(r"\bthe sum of two numbers is {NUM} and their difference is {NUM}"));
lazy_re!(SD2, &format!(r"\btwo numbers (?:that |which )?(?:add|sum|total)(?: up)?(?: to| is| are)? {NUM}\b.*?\bone (?:of them )?is {NUM} {MORE} than the other"));
lazy_re!(
    SD3,
    &format!(
        r"\ba {NUM} ?{UNITS_LEN}\b (?:board|rope|wire|stick|ribbon|pipe|string|rod|plank|pole|cable|bar) is (?:cut|divided|broken|split) into two (?:pieces|parts)\b.*?\bone (?:piece|part) is {NUM} {UNITS_LEN} {MORE} than the other"
    )
);
lazy_re!(AGE_SUM, &format!(r"\b([a-z']+) is {NUM} years? older than ([a-z']+)\. the sum of their ages is {NUM}"));

// ---------------------------------------------------------------------------------------------------------
// 1. two numbers with a sum and a difference
// ---------------------------------------------------------------------------------------------------------
pub fn sum_difference(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if n_numbers(problem) != 2 {
        return None;
    }
    for rx in [&*SD1, &*SD2, &*SD3] {
        if let Some(m) = rx.search(&t) {
            let (s, d) = (fr(m.g(1)), fr(m.g(2)));
            let two = Q::int(2);
            let (big, small) = ((s + d) / two, (s - d) / two);
            if d <= Q::ZERO || small <= Q::ZERO {
                return None;
            }
            return res(
                "sum_difference",
                vec![big, small],
                format!("x + y = {};  x - y = {}", show(s), show(d)),
                format!("two numbers with sum {} and difference {}: larger = {}, smaller = {}", show(s), show(d), show(big), show(small)),
            );
        }
    }
    None
}

pub fn age_sum(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let m = AGE_SUM.search(&t)?;
    if n_numbers(problem) != 2 {
        return None;
    }
    let (older, d, younger, s) = (m.g(1), fr(m.g(2)), m.g(3), fr(m.g(4)));
    let y = (s - d) / Q::int(2);
    if y <= Q::ZERO || !y.is_int() {
        return None;
    }
    res(
        "sum_difference",
        vec![y, y + d],
        format!("x + (x + {}) = {}", show(d), show(s)),
        format!("{} is {} older than {}, ages add to {}: {} is {}, {} is {}", older, show(d), younger, show(s), younger, show(y), older, show(y + d)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 2. consecutive integers
// ---------------------------------------------------------------------------------------------------------
static CONSEC: LazyLock<Vec<Re>> = LazyLock::new(|| {
    vec![
        re(&format!(r"\b(?:the )?(?:sum|total) of ({KWORD}) consecutive (even |odd )?(?:whole )?(?:integers|numbers) (?:is|equals|=) {NUM}")),
        re(&format!(r"\b(?:add up|adding|add) ({KWORD}) (even |odd )?(?:consecutive )?(?:integers|numbers) in a row.*?\b(?:get|equals?|is|total) {NUM}")),
        re(&format!(r"\b({KWORD}) (even |odd )?(?:consecutive )?(?:integers|numbers) in a row (?:sum|add|total)(?: up)?(?: to| is)? {NUM}")),
        re(&format!(r"\b({KWORD}) consecutive (even |odd )?(?:integers|numbers) (?:sum|add|total)(?: up)?(?: to)? {NUM}")),
    ]
});

pub fn consecutive(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    for rx in CONSEC.iter() {
        let Some(m) = rx.search(&t) else { continue };
        let k_text = m.g(1);
        let k = fr(k_text).floor();
        let kind = m.opt(2).unwrap_or("").trim().to_string();
        let s = fr(m.g(3));
        let expected = 1 + if k_text.chars().all(|c| c.is_ascii_digit()) { 1 } else { 0 };
        if n_numbers(problem) != expected || k < 2 || k > 10 {
            return None;
        }
        let step: i128 = if kind.is_empty() { 1 } else { 2 };
        let offset = step * k * (k - 1) / 2;
        let x = (s - Q::new(offset, 1)) / Q::new(k, 1);
        if !x.is_int() {
            return None;
        }
        let x = x.numer();
        if (kind == "even" && x.rem_euclid(2) != 0) || (kind == "odd" && x.rem_euclid(2) == 0) {
            return None;
        }
        let vals: Vec<Q> = (0..k).map(|i| Q::new(x + step * i, 1)).collect();
        let mut terms = vec!["x".to_string()];
        terms.extend((1..k).map(|i| format!("(x + {})", step * i)));
        let kind_text = if kind.is_empty() { String::new() } else { format!("{} ", kind) };
        return res(
            "consecutive",
            vals,
            format!("{} = {}", terms.join(" + "), show(s)),
            format!("{} consecutive {}integers add to {}: {}x + {} = {}, so x = {}", k, kind_text, show(s), k, offset, show(s), x),
        );
    }
    None
}

// ---------------------------------------------------------------------------------------------------------
// 3. two workers, separate times -> together
// ---------------------------------------------------------------------------------------------------------
pub const TIME_UNIT: &str = r"(hours?|hrs?|minutes?|mins?|days?|weeks?)";
pub const SPELLED: &str = r"\b(?:two|three|four|five|six|seven|eight|nine|ten|twice|double|triple|half|another|second)\b";
pub const GROUPED: &str = r"\b(?:both|pairs?|couple)\b";
const BOTH_GROUPS: &str = r"\bboth\s+(?:pairs?|teams?|crews?|groups?|sets?|copies)\b";
const SPELLED_REPEAT: &str = r"\b(?:two|three|four|five|six|seven|eight|nine|ten)\s+(?:such\s+|identical\s+|similar\s+|more\s+)?(?:pairs?|teams?|crews?|groups?|sets?|copies)\b|\b(?:twice|double|triple|half)\b";
const TOGETHER: &str = r"\b(together|both|team(?:s|ed)? up|combined|at the same time|jointly)\b";
const ONE_WORKER: &str = r"\b(alone|the other|each other|one of them)\b";

lazy_re!(SPELLED_RE, SPELLED);
lazy_re!(GROUPED_RE, GROUPED);
lazy_re!(SPELLED_REPEAT_RE, SPELLED_REPEAT);
lazy_re!(BOTH_GROUPS_RE, BOTH_GROUPS);
lazy_re!(SUCH_RE, r"\b(?:such|identical|like them|just like)\b");
lazy_re!(TOGETHER_RE, TOGETHER);
lazy_re!(ONE_WORKER_RE, ONE_WORKER);
lazy_re!(WT_DRAIN, r"\b(?:drain|leak|empt|plug|overflow)\w*|\bout of\b");
lazy_re!(WT_UNIT, &format!(r"\s*{TIME_UNIT}\b"));
lazy_re!(WT_IN_NUM, r"\bin\s+\d+(?:\.\d+)?$");
lazy_re!(WT_HOW, r"\bhow (?:long|many (?:hours|minutes|days|weeks)|much time)\b");

fn time_unit(word: &str) -> String {
    let word = word.strip_suffix('s').unwrap_or(word);
    match word {
        "hr" | "hour" => "hour".into(),
        "min" | "minute" => "minute".into(),
        "day" => "day".into(),
        "week" => "week".into(),
        other => other.to_string(),
    }
}

pub fn work_together(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 {
        return None;
    }
    if SPELLED_REPEAT_RE.is_match(&t) || BOTH_GROUPS_RE.is_match(&t) || SUCH_RE.is_match(&t) {
        return None;
    }
    let qs = questions(&t);
    if qs.len() != 1 || !TOGETHER_RE.is_match(&qs[0]) || ONE_WORKER_RE.is_match(&qs[0]) {
        return None;
    }
    // a together-word BEFORE the question means the setup already gives the combined time
    if TOGETHER_RE.is_match(&t[..t.find(&qs[0])?]) {
        return None;
    }
    if WT_DRAIN.is_match(&t) || t.contains('%') || problem.contains('$') {
        return None;
    }
    let mut units: Vec<String> = Vec::new();
    for (i, n) in nums.iter().enumerate() {
        if let Some(m) = WT_UNIT.matches(&t[n.end..]) {
            units.push(time_unit(m.g(1)));
        } else if i == 1 && !units.is_empty() && WT_IN_NUM.is_match(&t[..n.end]) {
            units.push(units[0].clone()); // "in 6 hours, Sue in 3": the second time keeps the first unit
        } else {
            return None;
        }
    }
    if units[0] != units[1] {
        return None;
    }
    if !WT_HOW.is_match(&qs[0]) {
        return None;
    }
    let (a, b) = (nums[0].v, nums[1].v);
    if a <= Q::ZERO || b <= Q::ZERO {
        return None;
    }
    let tt = a * b / (a + b);
    res(
        "work_together",
        vec![tt],
        format!("t/{} + t/{} = 1", show(a), show(b)),
        format!("one takes {} and the other {} {}s; together t/{} + t/{} = 1, so t = {} {}s", show(a), show(b), units[0], show(a), show(b), show_exact(tt), units[0]),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 4. a price after a percent change
// ---------------------------------------------------------------------------------------------------------
const UP: &str = r"\b(raise|raised|markup|marked up|marks up|mark up|increase|increased|went up|goes up|rose|risen|tax|tip|gratuity|adds|added|surcharge|service charge|inflation)\b";
const DOWN: &str = r"\b(discount|discounted|off|marked down|mark down|reduced|reduction|decrease|decreased|dropped|fell|cheaper|sale)\b";
const ASK_BEFORE: &str = r"\b(before|original|old|wholesale|without|initial|earlier|itself|what did (?:the )?(?:store|shop|company|owner|dealer|\w+) (?:pay|spend)|what was the cost)\b";

lazy_re!(UP_RE, UP);
lazy_re!(DOWN_RE, DOWN);
lazy_re!(ASK_BEFORE_RE, ASK_BEFORE);
lazy_re!(
    PC_REJECT,
    r"\b(interest|commission|per month|each month|a year|annual|compound|profit|loss|of which|out of|sav(?:e|es|ed|ing|ings)|gain(?:ed)?|lost|difference|fee)\b"
);
lazy_re!(PC_DOLLAR, r"(?:\bby|\bof) \$\d|\$\d[0-9.]* (?:off|more|less|extra|extra)");

pub fn percent_change(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || t.matches('%').count() != 1 {
        return None;
    }
    if SPELLED_RE.is_match(&t) || GROUPED_RE.is_match(&t) {
        return None;
    }
    let pct: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    let other: Vec<Num> = nums.iter().filter(|n| !pct.contains(n)).copied().collect();
    if pct.len() != 1 || other.len() != 1 {
        return None;
    }
    if PC_REJECT.is_match(&t) {
        return None;
    }
    if PC_DOLLAR.is_match(&problem.to_lowercase().replace(',', "")) {
        return None;
    }
    let (up, down) = (UP_RE.is_match(&t), DOWN_RE.is_match(&t));
    if up == down {
        return None;
    }
    let qs = questions(&t);
    if qs.len() != 1 || !ASK_BEFORE_RE.is_match(&qs[0]) {
        return None;
    }
    let (r, p) = (pct[0].v, other[0].v);
    let factor = (Q::int(100) + if up { r } else { -r }) / Q::int(100);
    if factor <= Q::ZERO {
        return None;
    }
    let x = p / factor; // exact; the nearest cent is only for the reading
    let f = show(factor);
    let money = other[0].money || problem.contains('$');
    res(
        "percent_change",
        vec![x],
        format!("{}x = {}", f, show(p)),
        format!("{}% {} the old value to give {}: {}x = {}, so x = {}", show(r), if up { "added to" } else { "taken off" }, show(p), f, show(p), show_answer(x, money)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 5. direct proportion with three numbers
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    REJECT_PROP,
    r"\b(inverse|inversely|workers?|men|pipes?|taps?|pumps?|faster|slower|speeds?|km/h|kmh|mph|kph|ratio|average|mean|angle|degrees?|scale|map|similar|shadow|tall|height)\b|%|:"
);
lazy_re!(UA_WORD, r"\s*(?:of\s+)?([a-z]+)");
lazy_re!(UA_PLURAL, r"(?<=[a-z])s$");
lazy_re!(PR_QUESTION, r"\bhow (?:much|many|long|far)\b");
lazy_re!(PR_SPELLED, r"\b(?:two|three|four|five|six|seven|eight|nine|ten|twice|double|triple|half|another|second|both)\b");
lazy_re!(PR_TOGETHER, r"\d\s+and\s+\d|\btogether\b|\bper (?:minute|hour|second|day|week)\b");
lazy_re!(PR_BAD, r"(?:/|=|\bfirst\b|after that|overtime|split|among|times as|more than|less than|plus|minus|\bis \d)");
lazy_re!(
    PR_CUE,
    r"\b(cost|costs|price|pay|pays|paid|earn|earns|for|per|rate|varies directly|needs|need|uses|use|travels|travel|drives|weigh|weighs|buy|gets|makes|cover|covers|stamp|stamps|print|prints)\b"
);

pub fn unit_after(t: &str, end: usize) -> String {
    let Some(m) = UA_WORD.matches(&t[end.min(t.len())..]) else { return String::new() };
    let u = m.g(1);
    let u = match u {
        "hrs" | "hr" => "hour",
        "kilograms" | "kilogram" | "kgs" => "kg",
        "pounds" | "lbs" => "lb",
        other => other,
    };
    if u.chars().count() > 3 {
        UA_PLURAL.sub(u, "")
    } else {
        u.to_string()
    }
}

pub fn proportion(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 3 || REJECT_PROP.is_match(&t) {
        return None;
    }
    let q = PR_QUESTION.search(&t)?;
    // roles by position: the known pair comes BEFORE the question, the quantity asked about comes AFTER it
    if !(nums[0].start < q.start() && nums[1].start < q.start() && nums[2].start > q.start()) {
        return None;
    }
    if PR_SPELLED.is_match(&t) || PR_TOGETHER.is_match(&t) || PR_BAD.is_match(&t) || !PR_CUE.is_match(&t) {
        return None;
    }
    let units: Vec<String> = nums.iter().map(|n| if n.money { "$".to_string() } else { unit_after(&t, n.end) }).collect();
    let (u1, u2, u3) = (&units[0], &units[1], &units[2]);
    if u1.is_empty() || u2.is_empty() || u3.is_empty() || u1 == u2 {
        return None;
    }
    let (a, b, c) = (nums[0].v, nums[1].v, nums[2].v);
    let (x, rule);
    if u3 == u1 {
        x = b * c / a;
        rule = format!("{} per {}", show(b / a), u1);
    } else if u3 == u2 {
        x = a * c / b;
        rule = format!("{} per {}", show(a / b), u2);
    } else {
        return None;
    }
    if x <= Q::ZERO || a.is_zero() || b.is_zero() {
        return None;
    }
    let eq = if u3 == u2 { format!("{}/{} = x/{}", show(a), show(b), show(c)) } else { format!("{}/{} = x/{}", show(b), show(a), show(c)) };
    res("proportion", vec![x], eq, format!("direct proportion: {}, so x = {}", rule, show(x)))
}

// ---------------------------------------------------------------------------------------------------------
// 6. two solutions with different percents mixed to a target percent
// ---------------------------------------------------------------------------------------------------------
const AMOUNT_UNIT: &str = r"(?:liters?|litres?|l|ml|milliliters?|millilitres?|kg|kilograms?|g|grams?|oz|ounces?|lbs?|pounds?|gallons?|quarts?|cups?)";
lazy_re!(
    MIX_OTHER,
    r"\b(?:water|pure|evaporat\w*|boil\w*|drain\w*|replac\w*|remov\w*|dilut\w*|add|adds|added|adding|price|priced|cost|costs|per pound|per lb|worth|selling)\b|/lb|\$"
);
lazy_re!(MIX_EXTRA, r"\b(?:times as|twice|double|triple|half|as much|ratio|more than|less than|each of the)\b");
lazy_re!(MIX_MAKE, r"\b(?:make|makes|making|get|gets|need|needs|needed|obtain|produce|produces|want|wants|total|of the mixture)\b");
lazy_re!(MIX_THE_END, r"\bthe\s*$");
lazy_re!(MIX_BIG, r"\b(?:three|four|five|six|seven|eight|nine|ten)\b");
lazy_re!(MIX_VERB, r"\b(?:mix\w*|combin\w*|blend\w*|alloy\w*|solutions?|make|use)\b");
lazy_re!(MIX_HOW, r"\bhow (?:much|many)\b");
lazy_re!(MIX_UNIT, &format!(r"\s*({AMOUNT_UNIT})\b"));

pub fn two_solution_mix(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let mut nums = digit_numbers(problem);
    // "How much of the 10% do you use?" repeats a source percent to say which amount is asked for: it is not a new number
    if let Some(q) = t.rfind("how ") {
        let snapshot = nums.clone();
        for n in snapshot.iter() {
            if n.start > q
                && take(&t, n.end, 1) == "%"
                && MIX_THE_END.is_match(&t[..n.start])
                && snapshot.iter().any(|m| m.start != n.start && m.v == n.v && m.start < q)
            {
                if let Some(pos) = nums.iter().position(|x| x == n) {
                    nums.remove(pos);
                }
            }
        }
    }
    if nums.len() != 4 || problem.contains('$') {
        return None;
    }
    if MIX_OTHER.is_match(&t) || MIX_EXTRA.is_match(&t) || MIX_BIG.is_match(&t) {
        return None;
    }
    if !MIX_VERB.is_match(&t) || !MIX_HOW.is_match(&t) {
        return None;
    }
    let pct: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    let amt: Vec<Num> = nums.iter().filter(|n| !pct.contains(n)).copied().collect();
    if pct.len() != 3 || amt.len() != 1 {
        return None;
    }
    let (total, a_start, a_end) = (amt[0].v, amt[0].start, amt[0].end);
    // the amount must be followed by a unit, and introduced by "make / get / need ..." (the total wanted)
    let um = MIX_UNIT.matches(&t[a_end..])?;
    if !MIX_MAKE.is_match(&last_words(&t[..a_start], 3)) {
        return None;
    }
    // the target is the percent that comes right after the amount (within six words)
    let after: Vec<Num> = pct.iter().filter(|n| n.start >= a_end).copied().collect();
    let target_n = *after.iter().min_by_key(|n| n.start)?;
    if t[a_end..target_n.start].split_whitespace().count() > 6 {
        return None;
    }
    let sources: Vec<Num> = pct.iter().filter(|n| n.start != target_n.start).copied().collect();
    let (p1, p2, tg) = (sources[0].v, sources[1].v, target_n.v);
    if p1 == p2 || !(p1.min(p2) < tg && tg < p1.max(p2)) || total <= Q::ZERO {
        return None;
    }
    let x1 = total * (tg - p2) / (p1 - p2);
    let x2 = total - x1;
    if x1 <= Q::ZERO || x2 <= Q::ZERO {
        return None;
    }
    let unit = um.g(1);
    let hundred = Q::int(100);
    let eq = format!("{}x + {}({} - x) = {}({})", show(p1 / hundred), show(p2 / hundred), show(total), show(tg / hundred), show(total));
    res(
        "two_solution_mix",
        vec![x1, x2],
        eq.clone(),
        format!(
            "x = amount of the {}% solution, {} - x of the {}%: {}, so x = {} {} of the {}% and {} {} of the {}%",
            show(p1),
            show(total),
            show(p2),
            eq,
            show_exact(x1),
            unit,
            show(p1),
            show_exact(x2),
            unit,
            show(p2)
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 7. money split between two interest rates
// ---------------------------------------------------------------------------------------------------------
lazy_re!(INT_WORD, r"\b(?:interest|earn|earns|earned|earning|earnings|yield|yields|yielded|return|returns)\b");
lazy_re!(
    INT_OTHER,
    r"\b(?:compound\w*|monthly|quarterly|per month|each month|a month|semi|daily|months?|two years|how long|how many years|same interest|equal|equally|overall|such|of the year|for part of|of the time|how much more|extra|additional|rate of|what rate|loan|borrow\w*|owe\w*|times as|twice|double|triple|half|as much|more than|less than|three|four|five|six|seven|eight|nine|ten)\b"
);
lazy_re!(INVEST_WORD, r"\b(?:invest\w*|account|accounts|deposit\w*|savings|bonds?|put|saved|fund|funds)\b");
lazy_re!(HOW_MUCH, r"\bhow much\b");
lazy_re!(INT_SPLIT, r"\b(?:rest|remainder|remaining|part|some|split|divided|each|one at|two (?:accounts|funds|bonds|investments|loans))\b");

pub fn two_rate_interest(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let mut nums = digit_numbers(problem);
    // "How much was invested at 6%?" repeats one of the two rates to say which amount is asked for: not a new number
    if let Some(q) = t.rfind("how ") {
        let snapshot = nums.clone();
        for n in snapshot.iter() {
            if n.start > q && take(&t, n.end, 1) == "%" && snapshot.iter().any(|m| m.start != n.start && m.v == n.v && m.start < q) {
                if let Some(pos) = nums.iter().position(|x| x == n) {
                    nums.remove(pos);
                }
            }
        }
    }
    if nums.len() != 4 {
        return None;
    }
    let pct: Vec<Num> = nums.iter().filter(|n| is_pct(&t, n)).copied().collect();
    let money: Vec<Num> = nums.iter().filter(|n| !pct.contains(n)).copied().collect();
    if pct.len() != 2 || money.len() != 2 || !money.iter().all(|n| n.money) {
        return None;
    }
    if INT_OTHER.is_match(&t) || !INT_WORD.is_match(&t) || !INVEST_WORD.is_match(&t) {
        return None;
    }
    if !HOW_MUCH.is_match(&t) || !INT_SPLIT.is_match(&t) {
        return None;
    }
    // the interest is the amount nearest a word like "interest" / "earned"; the other amount is the money invested
    let ws: Vec<&str> = t.split_whitespace().collect();
    let marks: Vec<usize> = ws.iter().enumerate().filter(|(_, w)| INT_WORD.is_fullmatch(w.trim_matches(|c| ".,;:?!".contains(c)))).map(|(i, _)| i).collect();
    if marks.is_empty() {
        return None;
    }
    let word_index = |n: &Num| t[..n.start].split_whitespace().count() as i64;
    let dist: Vec<i64> = money.iter().map(|n| marks.iter().map(|m| (word_index(n) - *m as i64).abs()).min().unwrap()).collect();
    if (dist[0] - dist[1]).abs() < 2 {
        return None;
    }
    let (interest, total) = if dist[0] < dist[1] { (money[0], money[1]) } else { (money[1], money[0]) };
    let (i, tt) = (interest.v, total.v);
    let (r1, r2) = (pct[0].v, pct[1].v);
    if r1 == r2 || tt <= Q::ZERO || i <= Q::ZERO {
        return None;
    }
    let hundred = Q::int(100);
    let x1 = (i - r2 * tt / hundred) / ((r1 - r2) / hundred); // invested at the first rate
    let x2 = tt - x1;
    if x1 <= Q::ZERO || x2 <= Q::ZERO {
        return None;
    }
    let eq = format!("{}x + {}({} - x) = {}", show(r1 / hundred), show(r2 / hundred), show(tt), show(i));
    res(
        "two_rate_interest",
        vec![x1, x2],
        eq.clone(),
        format!("x = amount at {}%, {} - x at {}%: {}, so ${} at {}% and ${} at {}%", show(r1), show(tt), show(r2), eq, show_exact(x1), show(r1), show_exact(x2), show(r2)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 8. two purchases of the same two items
// ---------------------------------------------------------------------------------------------------------
pub fn count_word(w: &str) -> Option<i64> {
    Some(match w {
        "a" | "an" | "one" => 1,
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

const CNT: &str = r"(?:\d+|a|an|one|two|three|four|five|six|seven|eight|nine|ten)";
const ITEM: &str = r"[a-z]+(?:\s[a-z]+)?";
lazy_re!(
    PURCHASE,
    &format!(r"(?<![\w.])({CNT})\s+({ITEM})\s+and\s+({CNT})\s+({ITEM})\s+(?:cost|costs|come to|comes to|total|totals)\s+(\d+(?:\.\d+)?)")
);
lazy_re!(SUCH_WORD, r"\bsuch\b");
lazy_re!(EACH_AFTER_RE, r"\s*(?:(?:for|a)\s+)?(?:each|apiece|per|piece)\b");
lazy_re!(pub ANY_DIGIT, r"\d");
lazy_re!(TP_PRICE, r"\b(?:price|cost)\b");
lazy_re!(TP_EACH, r"\beach\b|\bone of\b|\bper\b");

fn count(w: &str) -> i64 {
    if w.chars().all(|c| c.is_ascii_digit()) {
        w.parse().unwrap_or_else(|_| decline())
    } else {
        count_word(w).unwrap_or_else(|| decline())
    }
}

fn singular_word(w: &str) -> String {
    let n = w.chars().count();
    if n > 4 && w.ends_with("ies") {
        return format!("{}y", &w[..w.len() - 3]);
    }
    if n > 4 && ["sses", "xes", "zes", "ches", "shes"].iter().any(|s| w.ends_with(s)) {
        return w[..w.len() - 2].to_string();
    }
    if n > 3 && w.ends_with('s') && !w.ends_with("ss") {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}

/// Only used to tell that "passes" and "pass" (or "pears" and "pear") name the same item; the reading keeps the words as written.
fn singular(phrase: &str) -> String {
    phrase.split_whitespace().map(singular_word).collect::<Vec<_>>().join(" ")
}

pub fn two_purchases(problem: &str) -> Option<SolveResult> {
    let t = clean(problem).replace('$', "");
    let found = PURCHASE.find_all(&t);
    if found.len() != 2 || SUCH_WORD.is_match(&t) {
        return None;
    }
    // a price followed by each / apiece / per ("$5 each", "$5 for each", "$5 a piece") is the price of ONE item, not a total
    if found.iter().any(|m| EACH_AFTER_RE.matches(&t[m.end()..]).is_some()) {
        return None;
    }
    struct Row {
        c1: i64,
        i1: String,
        c2: i64,
        i2: String,
        price: Q,
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut digit_counts = 0usize;
    for m in &found {
        let (c1, i1, c2, i2, price) = (m.g(1), m.g(2), m.g(3), m.g(4), m.g(5));
        digit_counts += c1.chars().all(|c| c.is_ascii_digit()) as usize + c2.chars().all(|c| c.is_ascii_digit()) as usize;
        rows.push(Row { c1: count(c1), i1: singular(i1), c2: count(c2), i2: singular(i2), price: fr(price) });
    }
    // every number written in digits must be a count or a price of these two statements
    if digit_numbers(problem).len() != digit_counts + 2 {
        return None;
    }
    let (r1, r2) = (&rows[0], &rows[1]);
    let (a, n1, b, n2, t1) = (Q::int(r1.c1), r1.i1.clone(), Q::int(r1.c2), r1.i2.clone(), r1.price);
    let (mut c, mut m1, mut d, mut m2, t2) = (Q::int(r2.c1), r2.i1.clone(), Q::int(r2.c2), r2.i2.clone(), r2.price);
    if n1 == n2 {
        return None;
    }
    if m1 == n2 && m2 == n1 {
        // the second purchase names the items the other way round
        std::mem::swap(&mut c, &mut d);
        std::mem::swap(&mut m1, &mut m2);
    }
    if m1 != n1 || m2 != n2 {
        return None;
    }
    // what is left after the two statements may only ask for the prices ("Find the price of one of each.")
    let mut rest = t.clone();
    for m in found.iter().rev() {
        rest = format!("{} {}", &rest[..m.start()], &rest[m.end()..]);
    }
    if ANY_DIGIT.is_match(&rest) || SPELLED_RE.is_match(&rest) || GROUPED_RE.is_match(&rest) {
        return None;
    }
    if !TP_PRICE.is_match(&rest) || !TP_EACH.is_match(&rest) {
        return None;
    }
    let det = a * d - b * c;
    if det.is_zero() || a.min(b).min(c).min(d) <= Q::ZERO {
        return None;
    }
    let x = (t1 * d - b * t2) / det;
    let y = (a * t2 - c * t1) / det;
    if x <= Q::ZERO || y <= Q::ZERO {
        return None;
    }
    let eq1 = format!("{}x + {}y = {}", a, b, show(t1));
    let eq2 = format!("{}x + {}y = {}", c, d, show(t2));
    let (i1, i2) = (found[0].g(2), found[0].g(4));
    res(
        "two_purchases",
        vec![x, y],
        format!("{};  {}", eq1, eq2),
        format!("x = price of one '{}', y = price of one '{}': {} and {}, so x = {} and y = {}", i1, i2, eq1, eq2, show_exact(x), show_exact(y)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 9. price-per-pound blend
// ---------------------------------------------------------------------------------------------------------
pub fn unit_name(w: &str) -> &'static str {
    match w {
        "lb" | "lbs" | "pound" | "pounds" => "lb",
        "kg" | "kilogram" | "kilograms" => "kg",
        _ => "oz",
    }
}

const UNIT_RE: &str = r"(kilograms|kilogram|pounds|ounces|pound|ounce|lbs|lb|kg|oz)\b";
lazy_re!(PER_UNIT, &format!(r"\s*(?:/\s*|(?:per|a)\s+){UNIT_RE}"));
lazy_re!(UNIT_AFTER_AMOUNT, &format!(r"\s*{UNIT_RE}"));
lazy_re!(
    BLEND_OTHER,
    r"\b(?:add|adds|added|adding|already|water|evaporat\w*|remov\w*|replac\w*|profit|loss|discount\w*|tax|times as|twice|double|triple|half|as much|ratio|more than|less than|each of the|extra|another|three|four|five|six|seven|eight|nine|ten)\b|%"
);
lazy_re!(BLEND_MAKE, r"\b(?:make|makes|making|get|gets|need|needs|needed|obtain|produce|produces|want|wants)\b");
lazy_re!(
    BLEND_TARGET_CUE,
    r"\b(?:worth|selling for|sells for|sell for|sold for|selling at|sells at|sell at|sold at|priced at|valued at)\s*$"
);
lazy_re!(BL_ASK, r"\b(?:how (?:much|many)|find|determine|what (?:amount|weight))\b");
lazy_re!(BL_MIX, r"\b(?:mix\w*|blend\w*|combin\w*)\b");
lazy_re!(BL_TWO_UNITS, r"\btwo\s+(?:lb|lbs|pounds?|kg|kilograms?|oz|ounces?)\b");
lazy_re!(BL_OF_BLEND, r"\s*of (?:a |the |an )?(?:blend|mixture|mix)\b");

pub fn price_blend(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 4 || BLEND_OTHER.is_match(&t) {
        return None;
    }
    let prices: Vec<Num> = nums.iter().filter(|n| n.money).copied().collect();
    let amts: Vec<Num> = nums.iter().filter(|n| !n.money).copied().collect();
    if prices.len() != 3 || amts.len() != 1 {
        return None;
    }
    if !BL_ASK.is_match(&t) || !BL_MIX.is_match(&t) || BL_TWO_UNITS.is_match(&t) {
        return None;
    }
    // every price is "per unit" ("$6 a pound", "$6/lb", "$6 per kg"), all with the same unit
    let mut units: BTreeSet<&'static str> = BTreeSet::new();
    for n in &prices {
        let m = PER_UNIT.matches(&t[n.end..])?;
        units.insert(unit_name(m.g(1)));
    }
    let (total, a_start, a_end) = (amts[0].v, amts[0].start, amts[0].end);
    let um = UNIT_AFTER_AMOUNT.matches(&t[a_end..])?;
    let only: BTreeSet<&'static str> = [unit_name(um.g(1))].into_iter().collect();
    if units != only {
        return None;
    }
    // the amount is the weight of the blend wanted: "to make 40 lb ..." or "40 lb of a blend ..."
    let unit_end = a_end + um.end();
    if !(BLEND_MAKE.is_match(&last_words(&t[..a_start], 3)) || BL_OF_BLEND.matches(&t[unit_end..]).is_some()) {
        return None;
    }
    // the blend's price is the first price after the amount, introduced by "worth / selling for / priced at ..."
    let after: Vec<Num> = prices.iter().filter(|n| n.start >= unit_end).copied().collect();
    let target_n = *after.iter().min_by_key(|n| n.start)?;
    let head = t[..target_n.start].trim_end().trim_end_matches('$').trim_end();
    if !BLEND_TARGET_CUE.is_match(head) || t[unit_end..target_n.start].split_whitespace().count() > 8 {
        return None;
    }
    let sources: Vec<Num> = prices.iter().filter(|n| n.start != target_n.start).copied().collect();
    let (p1, p2, tg) = (sources[0].v, sources[1].v, target_n.v);
    if p1 == p2 || !(p1.min(p2) < tg && tg < p1.max(p2)) || total <= Q::ZERO {
        return None;
    }
    let x1 = total * (tg - p2) / (p1 - p2);
    let x2 = total - x1;
    if x1 <= Q::ZERO || x2 <= Q::ZERO {
        return None;
    }
    let unit = unit_name(um.g(1));
    let eq = format!("{}x + {}({} - x) = {}({})", show(p1), show(p2), show(total), show(tg), show(total));
    res(
        "price_blend",
        vec![x1, x2],
        eq.clone(),
        format!("x = {} at ${}, {} - x at ${}: {}, so {} {} at ${} and {} {} at ${}", unit, show(p1), show(total), show(p2), eq, show_exact(x1), unit, show(p1), show_exact(x2), unit, show(p2)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 10. two savers, same total:   a + ra*w = b + rb*w
// ---------------------------------------------------------------------------------------------------------
lazy_re!(
    SAVE_STMT,
    r"(?<![\w$])(?P<who>i|[a-z]+(?:\s[a-z]+)?)\s+(?:has|have|had|starts?|started)(?:\s+with)?\s+\$(?P<a>\d+(?:\.\d+)?)(?:\s+(?:saved|in savings|in the bank|in (?:her|his|their|my) account))?\s*(?:,|and|but)?\s*(?:and\s+)?(?:saves?|adds?|puts? away|deposits?)\s+\$(?P<r>\d+(?:\.\d+)?)\s+(?:a|each|per|every)\s+(?P<u>week|month|day)\b"
);
lazy_re!(SAV_NOW, r"\b(?:now|currently|today|at the moment|so far)\b");
lazy_re!(SAV_WHEN, r"\b(?:when|how (?:many|long)|after how)\b");
lazy_re!(SAV_SAME, r"\b(?:same|equal|even|as much)\b");
lazy_re!(SAV_HOW_MUCH, r"\bhow much\b|\bwhat amount\b|\bhow many dollars\b");

pub fn savings_equal(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let found = SAVE_STMT.find_all(&t);
    if found.len() != 2 || digit_numbers(problem).len() != 4 || CHANGES_EQ_RE.is_match(&t) {
        return None;
    }
    // the starting amounts are totals: "$200 each" / "$200 apiece" would change the equation
    if found.iter().any(|m| EACH_AFTER.matches(&t[m.name_end("a").unwrap()..]).is_some()) {
        return None;
    }
    let (m1, m2) = (&found[0], &found[1]);
    if m1.name("who") == m2.name("who") || m1.name("u") != m2.name("u") {
        return None;
    }
    let rest = format!("{} {} {}", &t[..m1.start()], &t[m1.end()..m2.start()], &t[m2.end()..]);
    if DIGIT_RE.is_match(&rest) || SPELLED_ANY_RE.is_match(&rest) || SAV_NOW.is_match(&rest) {
        return None;
    }
    // the question asks WHEN they are equal, not an amount of money
    if !SAV_WHEN.is_match(&rest) || !SAV_SAME.is_match(&rest) || SAV_HOW_MUCH.is_match(&rest) {
        return None;
    }
    let (a, b) = (fr(m1.name("a")?), fr(m2.name("a")?));
    let (ra, rb) = (fr(m1.name("r")?), fr(m2.name("r")?));
    if ra == rb {
        return None;
    }
    let w = (b - a) / (ra - rb);
    if w <= Q::ZERO || !w.is_int() {
        return None;
    }
    let unit = m1.name("u")?;
    let each = a + ra * w;
    let eq = format!("{} + {}w = {} + {}w", show(a), show(ra), show(b), show(rb));
    res("savings_equal", vec![w], eq.clone(), format!("w = number of {}s: {}, so w = {} {}s (each will have ${})", unit, eq, show_exact(w), unit, show_exact(each)))
}

// ---------------------------------------------------------------------------------------------------------
// 11. the score needed on the next test for a target average
// ---------------------------------------------------------------------------------------------------------
pub fn ordinal(w: &str) -> i128 {
    match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "sixth" => 6,
        "seventh" => 7,
        "eighth" => 8,
        "ninth" => 9,
        _ => 10,
    }
}

const TEST_NOUN: &str = r"(?:tests?|exams?|quizzes|quiz|games?|papers?|assignments?|rounds?|scores|marks|grades)";
lazy_re!(
    AVG_OTHER,
    r"\b(?:weighted|weights?|worth|counts?|counted|drop\w*|lowest|highest|best|worst|discard\w*|ignor\w*|curve\w*|bonus|extra|credit|replac\w*|median|mode|total|sum|at least|at most|minimum|maximum|no less|no more|passing|pass|out of|each)\b|%|\$"
);
lazy_re!(AVG_WORD, r"\baverag\w*|\bmean\b|\bavg\b");
lazy_re!(
    AVG_TARGET,
    r"\b(?:to average|to be averaging|an average of|the average of|average of|average score of|mean of|to bring (?:my|the|our|her|his) average (?:up )?to|to raise (?:my|the|our|her|his) average to|average at)\s+(\d+(?:\.\d+)?)\b(?!\s*,|\s+and\s+\d)"
);
lazy_re!(AVG_COUNT_NOUN, &format!(r"\s+{TEST_NOUN}\b"));
lazy_re!(AVG_COUNT_LEAD, r"\b(?:on|in|after|over|across|from)\s+(?:the\s+|my\s+|her\s+|his\s+|these\s+|those\s+|all\s+)?(?:first\s+)?$");
lazy_re!(AVG_WORD_COUNT, &format!(r"\b(two|three|four|five|six|seven|eight|nine|ten)\s+{TEST_NOUN}\b"));
lazy_re!(AVG_ORDS, r"\b(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth)\b");
lazy_re!(AVG_WHAT, r"\b(?:what|which|how (?:many )?(?:points|marks|high|well))\b");
lazy_re!(AVG_NEED, r"\b(?:need|needed|must|should|have to|has to|require|required|get|score|make|earn|take)\b");
lazy_re!(AVG_NEXT, r"\b(?:next|last|final|another|one more)\b");

pub fn average_needed(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if AVG_OTHER.is_match(&t) || AVG_WORD.find_all(&t).len() != 1 {
        return None;
    }
    let tm = AVG_TARGET.find_all(&t);
    if tm.len() != 1 {
        return None;
    }
    let (tstart, tend) = (tm[0].group_start(1)?, tm[0].group_end(1)?);
    let target_n: Vec<Num> = nums.iter().filter(|n| (n.start <= tstart && tstart < n.end) || n.start == tstart).copied().collect();
    if target_n.len() != 1 || target_n[0].end != tend {
        return None;
    }
    let target = target_n[0];
    // a stated count of tests ("on 5 tests", "after four tests") is not a score; it must agree with the scores listed
    let mut counts: Vec<Num> = Vec::new();
    for n in &nums {
        if n.start == target.start {
            continue;
        }
        let lead = format!("{} ", t[..n.start].trim_end());
        if AVG_COUNT_NOUN.matches(&t[n.end..]).is_some() && AVG_COUNT_LEAD.is_match(&lead) {
            counts.push(*n);
        }
    }
    if counts.len() > 1 {
        return None;
    }
    let scores: Vec<Num> = nums.iter().filter(|n| n.start != target.start && !counts.contains(n)).copied().collect();
    let k = scores.len() as i128;
    if k < 2 {
        return None;
    }
    if !counts.is_empty() && counts[0].v != Q::new(k, 1) {
        return None;
    }
    let mut work = t.clone();
    let word_counts = AVG_WORD_COUNT.find_all(&work);
    if word_counts.len() > 1 || (!word_counts.is_empty() && !counts.is_empty()) {
        return None;
    }
    if let Some(wc) = word_counts.first() {
        if count_word(wc.g(1)).unwrap() as i128 != k {
            return None;
        }
        work = format!("{} {}", &work[..wc.start()], &work[wc.end()..]);
    }
    if SPELLED_ANY_RE.is_match(&work) || CHANGES_EQ_RE.is_match(&work) {
        return None;
    }
    // ordinals: only "the fourth test" naming the unknown score, and it must be number k + 1
    let ords: Vec<String> = AVG_ORDS.find_all(&work).iter().map(|m| m.g(1).to_string()).collect();
    if !ords.is_empty() {
        let again = re(&format!(r"\b{}\s+{TEST_NOUN}\b", ords[0]));
        if ords.len() != 1 || !again.is_match(&work) || ordinal(&ords[0]) != k + 1 {
            return None;
        }
    }
    // the question asks for the score on the next / last / final / (k+1)th test
    if !AVG_WHAT.is_match(&work) || !AVG_NEED.is_match(&work) {
        return None;
    }
    if !(!ords.is_empty() || AVG_NEXT.is_match(&work)) {
        return None;
    }
    let total = scores.iter().fold(Q::ZERO, |acc, n| acc + n.v);
    let x = target.v * Q::new(k + 1, 1) - total;
    if x < Q::ZERO {
        return None;
    }
    if nums.iter().all(|n| n.v <= Q::int(100)) && x > Q::int(100) {
        return None;
    }
    let listed = scores.iter().map(|n| show(n.v)).collect::<Vec<_>>().join(" + ");
    let eq = format!("({} + x) / {} = {}", listed, k + 1, show(target.v));
    res(
        "average_needed",
        vec![x],
        eq.clone(),
        format!("x = the next score: {}, so {} + x = {} and x = {}", eq, show(total), show(target.v * Q::new(k + 1, 1)), show_exact(x)),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 12. heads and feet of a two-legged and a four-legged animal
// ---------------------------------------------------------------------------------------------------------
const TWO_LEGGED: &[&str] = &[
    "hen", "chicken", "duck", "goose", "geese", "turkey", "rooster", "bird", "ostrich", "pigeon", "swan", "peacock", "parrot", "penguin", "flamingo", "chick", "duckling", "gosling",
    "emu",
];
const FOUR_LEGGED: &[&str] = &[
    "cow", "sheep", "pig", "goat", "horse", "dog", "cat", "rabbit", "donkey", "lamb", "calf", "calves", "bull", "ox", "oxen", "deer", "mule", "pony", "llama", "camel", "elephant", "tiger",
    "lion", "puppy", "kitten", "piglet", "foal", "hog",
];
const OTHER_CREATURE: &[&str] = &[
    "spider", "beetle", "insect", "bug", "ant", "fly", "flie", "snake", "fish", "octopus", "centipede", "millipede", "cricket", "ladybug", "bee", "butterfly", "person", "people", "man",
    "men", "woman", "women", "human", "child", "children", "kid", "boy", "girl", "farmer", "bicycle", "tricycle", "car", "truck", "wheel", "motorcycle", "table", "stool", "chair", "tripod",
    "alien", "worm", "monkey", "ape", "frog", "turtle", "lizard", "crab", "scorpion", "mite", "tick", "ostriches",
];

fn is_creature(w: &str) -> bool {
    TWO_LEGGED.contains(&w) || FOUR_LEGGED.contains(&w) || OTHER_CREATURE.contains(&w)
}

fn animal_stem(w: &str) -> String {
    if is_creature(w) || ["sheep", "geese", "oxen", "calves"].contains(&w) {
        return w.to_string();
    }
    if w.ends_with("ies") {
        return format!("{}y", &w[..w.len() - 3]);
    }
    if w.ends_with("es") && is_creature(&w[..w.len() - 2]) {
        return w[..w.len() - 2].to_string();
    }
    if w.ends_with('s') {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}

lazy_re!(HF_LEGGED, r"\blegged\b|\b(?:wings?|tails?|horns?|arms?|wheels?)\b");
lazy_re!(HF_HEADS, r"\s+(?:heads|animals|creatures)\b");
lazy_re!(HF_LEGS, r"\s+(?:legs|feet)\b");
lazy_re!(pub HF_ALPHA, r"[a-z]+");
lazy_re!(HF_TAIL_BAD, r"\b(?:legs|feet|heads|total|altogether|more|fewer|less)\b");

pub fn heads_and_feet(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 2 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) {
        return None;
    }
    if SPELLED_ANY_RE.is_match(&t) || HF_LEGGED.is_match(&t) {
        return None;
    }
    let heads: Vec<Num> = nums.iter().filter(|n| HF_HEADS.matches(&t[n.end..]).is_some()).copied().collect();
    let legs: Vec<Num> = nums.iter().filter(|n| HF_LEGS.matches(&t[n.end..]).is_some()).copied().collect();
    if heads.len() != 1 || legs.len() != 1 || heads[0].start == legs[0].start {
        return None;
    }
    // one two-legged animal and one four-legged animal, nothing else with legs
    let mut stems: Vec<String> = Vec::new();
    let mut shown: Vec<(String, String)> = Vec::new();
    for w in HF_ALPHA.find_all(&t) {
        let w = w.text();
        let s = animal_stem(w);
        if is_creature(&s) && !stems.contains(&s) {
            stems.push(s.clone());
            shown.push((s, w.to_string()));
        }
    }
    if stems.len() != 2 {
        return None;
    }
    let two: Vec<&String> = stems.iter().filter(|s| TWO_LEGGED.contains(&s.as_str())).collect();
    let four: Vec<&String> = stems.iter().filter(|s| FOUR_LEGGED.contains(&s.as_str())).collect();
    if two.len() != 1 || four.len() != 1 {
        return None;
    }
    // the question asks how many of the animals, not for legs or heads
    let q = t.rfind("how many")?;
    let tail = &t[q..];
    if HF_TAIL_BAD.is_match(tail) {
        return None;
    }
    let tail_words: Vec<String> = HF_ALPHA.find_all(tail).iter().map(|m| m.text().to_string()).collect();
    let named = tail_words.iter().any(|w| {
        let s = animal_stem(w);
        s == *two[0] || s == *four[0]
    });
    if !tail_words.iter().any(|w| w == "each") && !named {
        return None;
    }
    let (h, l) = (heads[0].v, legs[0].v);
    let s_count = (l - Q::int(2) * h) / Q::int(2);
    let h_count = h - s_count;
    if s_count <= Q::ZERO || h_count <= Q::ZERO || !s_count.is_int() || !h_count.is_int() {
        return None;
    }
    let find = |s: &String| shown.iter().find(|(k, _)| k == s).map(|(_, w)| w.clone()).unwrap();
    let (n2, n4) = (find(two[0]), find(four[0]));
    let eq = format!("h + s = {};  2h + 4s = {}", show(h), show(l));
    res(
        "heads_and_feet",
        vec![h_count, s_count],
        eq,
        format!(
            "h = number of {} (2 feet), s = number of {} (4 feet): h + s = {} and 2h + 4s = {}, so h = {} and s = {}",
            n2,
            n4,
            show(h),
            show(l),
            show_exact(h_count),
            show_exact(s_count)
        ),
    )
}

// ---------------------------------------------------------------------------------------------------------
// 13-15. distance and geometry: opposite directions, catch-up, interior angles of a polygon
// ---------------------------------------------------------------------------------------------------------
lazy_re!(pub SPEED, &format!(r"{NUM}\s*({SPEED_UNIT})(?![\w/])"));
lazy_re!(pub DIST, &format!(r"{NUM}\s*(miles?|mi|kilometers?|kilometres?|km)\s+apart\b"));

pub fn speed_group(unit: &str) -> &'static str {
    if unit.starts_with("mph") || unit.starts_with("mi") || unit.starts_with("miles") {
        "mi"
    } else {
        "km"
    }
}

pub fn dist_group(unit: &str) -> &'static str {
    if unit.starts_with("mi") {
        "mi"
    } else {
        "km"
    }
}

lazy_re!(
    pub TWO_VEHICLES,
    r"\btwo (?:cars|trains|cyclists|runners|planes|boats|buses|trucks|bikes|bicycles|friends|people|hikers|walkers|vehicles|ships|joggers|riders|drivers|motorcycles)\b"
);
lazy_re!(pub ASKS_TIME, r"\b(?:how (?:many hours|long|much time)|when|after how many hours)\b");
lazy_re!(
    pub ASKS_OTHER,
    r"\b(?:how fast|what speed|what (?:is|are) (?:the|their) speeds?|how far|how many (?:miles|kilometers|kilometres|km))\b"
);
lazy_re!(OPPOSITE, r"\bopposite directions?\b");
lazy_re!(
    OPP_BAD,
    r"\b(?:same direction|toward|towards|meet|meets|later|earlier|head start|ahead|behind|follow\w*|catch\w*|chase\w*|round trip|return\w*|back|then|current|stream|wind|stop\w*|rest\w*|break)\b"
);

pub fn opposite_directions(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || digit_numbers(problem).len() != 3 {
        return None;
    }
    if !OPPOSITE.is_match(&t) || OPP_BAD.is_match(&t) {
        return None;
    }
    let work = TWO_VEHICLES.sub(&t, " "); // "Two cars" is the count of the two speeds the equation uses
    if SPELLED_ANY_RE.is_match(&work) {
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
    res("opposite_directions", vec![tt], eq.clone(), format!("t = hours: {}, so t = {} hours", eq, show_exact(tt)))
}

pub fn head_word(w: &str) -> Option<i64> {
    count_word(w)
}

lazy_re!(
    pub HEAD_START,
    r"(\d+(?:\.\d+)?|a|an|one|two|three|four|five|six|seven|eight|nine|ten)\s+(?:hours?|hrs?)\s+later\b[,]?"
);
lazy_re!(FOLLOW_VERB, r"\b(?:follows?|following|chases?|chasing)\b");
lazy_re!(pub CATCH_CUE, r"\b(?:catch|catches|catching|overtake|overtakes)\b");
lazy_re!(
    CU_BAD,
    r"\b(?:opposite|toward|towards|meet|meets|apart|return\w*|back|stops?|stopped|rests?|breaks?|detour|lunch|current|stream|wind|also|another)\b"
);
lazy_re!(CU_TAIL, r"(?:how (?:long|many hours)|when)\b");
lazy_re!(CU_SENTENCES, r"[.?!]\s*");
lazy_re!(
    CU_EARLIER,
    r"\b(?:that|who|which)\s+(?:left|started|set off|departed|had)\b|\bearlier\b|\bahead\b|\bbefore\b|\balready\b|\bhad (?:left|started)\b"
);
lazy_re!(
    CU_SPEED_NOUN,
    &format!(r"(?:{SPEED_UNIT})\s+(?:train|car|bus|bike|bicycle|truck|plane|runner|cyclist|boat|ship|jogger|walker|hiker|vehicle|motorcycle|rider|driver)\b")
);
lazy_re!(CU_AFTER, r"\bafter\s+(.*?)\s+(?:leaves|starts|departs|sets off|begins)\b");
lazy_re!(CU_SINCE, r"\b(?:since|from the start|in all|in total|altogether)\b");

const SUBJECT_VERBS: &[&str] = &[
    "follows", "follow", "leaves", "leave", "starts", "start", "departs", "chases", "drives", "drive", "travels", "travel", "rides", "ride", "runs", "run", "sets", "goes", "heads", "walks",
    "walk", "jogs", "is",
];
pub const ARTICLES: &[&str] = &["a", "an", "the", "his", "her", "their", "its", "my", "our"];

pub fn subject_tokens(sentence: &str) -> BTreeSet<String> {
    let mut out: Vec<String> = Vec::new();
    for m in HF_ALPHA.find_all(sentence) {
        if SUBJECT_VERBS.contains(&m.text()) {
            break;
        }
        out.push(m.text().to_string());
    }
    out.into_iter().filter(|w| !ARTICLES.contains(&w.as_str())).collect()
}

pub fn catch_up(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    if t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) || CU_BAD.is_match(&t) {
        return None;
    }
    let hs = HEAD_START.find_all(&t);
    if hs.len() != 1 || !FOLLOW_VERB.is_match(&t) || !CATCH_CUE.is_match(&t) {
        return None;
    }
    let hw = hs[0].g(1);
    let head = match head_word(hw) {
        Some(n) => Q::int(n),
        None => fr(hw),
    };
    let work = format!("{} {}", &t[..hs[0].start()], &t[hs[0].end()..]); // "Two hours later" is the head start the equation uses
    if SPELLED_ANY_RE.is_match(&work) {
        return None;
    }
    let speeds = SPEED.find_all(&t);
    if speeds.len() != 2 || speed_group(speeds[0].g(2)) != speed_group(speeds[1].g(2)) {
        return None;
    }
    // every number in digits is a speed or the head start
    if digit_numbers(problem).len() != 2 + if head_word(hw).is_none() { 1 } else { 0 } {
        return None;
    }
    let (s1, s2) = (fr(speeds[0].g(1)), fr(speeds[1].g(1)));
    if s1 <= Q::ZERO || head <= Q::ZERO || s2 <= s1 {
        return None;
    }
    // the question: "how long ... catches" asks the FOLLOWER's time; "after <the leader> leaves" or a distance declines
    let q = match (t.rfind("how "), t.rfind("when ")) {
        (Some(a), Some(b)) => a.max(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };
    let tail = &t[q..];
    if !CATCH_CUE.is_match(tail) || CU_TAIL.matches(tail).is_none() || ASKS_OTHER.is_match(tail) {
        return None;
    }
    let sents: Vec<String> = CU_SENTENCES.split(&t).into_iter().filter(|s| !s.trim().is_empty()).collect();
    let lead_s = sents.iter().find(|s| s.contains(speeds[0].text()))?;
    let foll_s = sents.iter().find(|s| s.contains(speeds[1].text()))?;
    if lead_s == foll_s {
        return None;
    }
    // the head start belongs to the sentence of the SECOND speed, and that sentence's subject is the follower: the vehicle that leaves later must be the faster one
    if !HEAD_START.is_match(foll_s) || HEAD_START.is_match(lead_s) || FOLLOW_VERB.is_match(lead_s) {
        return None;
    }
    if CU_EARLIER.is_match(&t) || CU_SPEED_NOUN.is_match(&t) {
        return None;
    }
    let foll_s = HEAD_START.sub(foll_s, " ");
    let (f, ld) = (subject_tokens(&foll_s), subject_tokens(lead_s));
    let pronouns: BTreeSet<String> = ["it", "he", "she", "they", "him", "her", "them", "i", "we", "you"].iter().map(|s| s.to_string()).collect();
    if f.is_empty() || f.is_subset(&pronouns) {
        return None;
    }
    for am in CU_AFTER.find_all(tail) {
        let c: BTreeSet<String> = HF_ALPHA.find_all(am.g(1)).iter().map(|m| m.text().to_string()).filter(|w| !ARTICLES.contains(&w.as_str())).collect();
        let only_f: BTreeSet<String> = f.difference(&ld).cloned().collect();
        let only_l: BTreeSet<String> = ld.difference(&f).cloned().collect();
        if c.intersection(&only_f).next().is_none() || c.intersection(&only_l).next().is_some() {
            return None;
        }
    }
    if CU_SINCE.is_match(tail) {
        return None;
    }
    let tt = s1 * head / (s2 - s1);
    let eq = format!("{}(t + {}) = {}t", show(s1), show(head), show(s2));
    res("catch_up", vec![tt], eq.clone(), format!("t = hours after the follower leaves: {}, so t = {} hours", eq, show_exact(tt)))
}

lazy_re!(POLY_SUM, r"\b(?:interior|inside|inner) angles\b");
lazy_re!(POLYGON, r"\bpolygon\b");
lazy_re!(
    POLY_BAD,
    r"\b(?:regular|each|every|exterior|outside|outer|equal|measures?|diagonals?|one angle|same measure|triangle|quadrilateral|pentagon|hexagon|octagon|heptagon|decagon|square|rectangle)\b"
);
lazy_re!(POLY_DEGREES, r"\s*(?:°|degrees?|deg)\b|\s*°");
lazy_re!(POLY_SUM_WORD, r"\b(?:sum|add up|adds up|add to|total|totals|sum to|add)\b");
lazy_re!(POLY_HOW, r"\bhow many (?:sides|vertices|corners|edges)\b");

pub fn polygon_angle_sum(problem: &str) -> Option<SolveResult> {
    let t = clean(problem);
    let nums = digit_numbers(problem);
    if nums.len() != 1 || t.contains('$') || t.contains('%') || CHANGES_EQ_RE.is_match(&t) {
        return None;
    }
    if !POLY_SUM.is_match(&t) || !POLYGON.is_match(&t) || POLY_BAD.is_match(&t) || SPELLED_ANY_RE.is_match(&t) {
        return None;
    }
    let s = nums[0];
    if POLY_DEGREES.matches(&t[s.end..]).is_none() || !POLY_SUM_WORD.is_match(&t) || !POLY_HOW.is_match(&t) {
        return None;
    }
    let n = s.v / Q::int(180) + Q::int(2);
    if !n.is_int() || n < Q::int(3) {
        return None;
    }
    let eq = format!("(n - 2) * 180 = {}", show(s.v));
    res("polygon_angle_sum", vec![n], eq.clone(), format!("n = number of sides: {}, so n = {} sides", eq, show_exact(n)))
}

pub fn list() -> Vec<(&'static str, Family)> {
    vec![
        ("sum_difference", sum_difference),
        ("age_sum", age_sum),
        ("consecutive", consecutive),
        ("work_together", work_together),
        ("percent_change", percent_change),
        ("proportion", proportion),
        ("two_solution_mix", two_solution_mix),
        ("two_rate_interest", two_rate_interest),
        ("two_purchases", two_purchases),
        ("price_blend", price_blend),
        ("savings_equal", savings_equal),
        ("average_needed", average_needed),
        ("heads_and_feet", heads_and_feet),
        ("opposite_directions", opposite_directions),
        ("catch_up", catch_up),
        ("polygon_angle_sum", polygon_angle_sum),
    ]
}
