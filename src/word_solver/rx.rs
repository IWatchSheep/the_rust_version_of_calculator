//! Python-style regular expressions for the solver, on top of `fancy-regex` (which has the backreferences and look-around the Python patterns use).
//!
//! `search` = `re.search`, `matches` = `re.match` (anchored at the start), `fullmatch` = `re.fullmatch`, `find_all` = `re.finditer`, `sub` = `re.sub`.

use fancy_regex::{Captures, Regex};

pub struct Re {
    search: Regex,
    start: Regex,
    full: Regex,
}

/// Compile a Python pattern.  The same pattern is also compiled anchored at the start and at both ends for `matches` / `fullmatch`.
pub fn re(pattern: &str) -> Re {
    let mk = |p: &str| Regex::new(p).unwrap_or_else(|e| panic!("bad regex {p}: {e}"));
    Re {
        search: mk(pattern),
        start: mk(&format!(r"\A(?:{})", pattern)),
        full: mk(&format!(r"\A(?:{})\z", pattern)),
    }
}

/// One match with its groups (`m.group(i)`, `m.start()`, `m.end()`).
pub struct M<'t> {
    caps: Captures<'t>,
}

impl<'t> M<'t> {
    /// `m.group(i)` — None for a group that did not take part.
    pub fn opt(&self, i: usize) -> Option<&'t str> {
        self.caps.get(i).map(|m| m.as_str())
    }

    /// `m.group(i)` for a group that must exist (a missing one is a bug in the port, so it declines).
    pub fn g(&self, i: usize) -> &'t str {
        self.opt(i).unwrap_or("")
    }

    pub fn name(&self, n: &str) -> Option<&'t str> {
        self.caps.name(n).map(|m| m.as_str())
    }

    pub fn name_end(&self, n: &str) -> Option<usize> {
        self.caps.name(n).map(|m| m.end())
    }

    pub fn name_start(&self, n: &str) -> Option<usize> {
        self.caps.name(n).map(|m| m.start())
    }

    pub fn start(&self) -> usize {
        self.caps.get(0).unwrap().start()
    }

    pub fn end(&self) -> usize {
        self.caps.get(0).unwrap().end()
    }

    pub fn group_start(&self, i: usize) -> Option<usize> {
        self.caps.get(i).map(|m| m.start())
    }

    pub fn group_end(&self, i: usize) -> Option<usize> {
        self.caps.get(i).map(|m| m.end())
    }

    pub fn text(&self) -> &'t str {
        self.caps.get(0).unwrap().as_str()
    }
}

impl Re {
    pub fn search<'t>(&self, t: &'t str) -> Option<M<'t>> {
        self.search.captures(t).ok().flatten().map(|caps| M { caps })
    }

    pub fn is_match(&self, t: &str) -> bool {
        self.search.is_match(t).unwrap_or(false)
    }

    /// `re.match`: the pattern must match at the start of the text.
    pub fn matches<'t>(&self, t: &'t str) -> Option<M<'t>> {
        self.start.captures(t).ok().flatten().map(|caps| M { caps })
    }

    /// `re.fullmatch`
    pub fn fullmatch<'t>(&self, t: &'t str) -> Option<M<'t>> {
        self.full.captures(t).ok().flatten().map(|caps| M { caps })
    }

    pub fn is_fullmatch(&self, t: &str) -> bool {
        self.full.is_match(t).unwrap_or(false)
    }

    /// `re.finditer`
    pub fn find_all<'t>(&self, t: &'t str) -> Vec<M<'t>> {
        self.search.captures_iter(t).filter_map(|c| c.ok()).map(|caps| M { caps }).collect()
    }

    /// `re.sub(pattern, repl, text)` with a literal replacement (no group references).
    pub fn sub(&self, t: &str, repl: &str) -> String {
        let mut out = String::new();
        let mut last = 0;
        for m in self.find_all(t) {
            out.push_str(&t[last..m.start()]);
            out.push_str(repl);
            last = m.end();
        }
        out.push_str(&t[last..]);
        out
    }

    /// `re.sub` with a function of the match.
    pub fn sub_with<F: Fn(&M) -> String>(&self, t: &str, f: F) -> String {
        let mut out = String::new();
        let mut last = 0;
        for m in self.find_all(t) {
            out.push_str(&t[last..m.start()]);
            out.push_str(&f(&m));
            last = m.end();
        }
        out.push_str(&t[last..]);
        out
    }

    /// `re.split(pattern, text)` for a pattern without capturing groups.
    pub fn split(&self, t: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut last = 0;
        for m in self.find_all(t) {
            if m.end() == m.start() {
                continue; // Python (3.7+) splits on empty matches too; the solver's patterns never match empty
            }
            parts.push(t[last..m.start()].to_string());
            last = m.end();
        }
        parts.push(t[last..].to_string());
        parts
    }
}

/// Python's `str.strip(chars)`.
pub fn strip(s: &str, chars: &str) -> String {
    s.trim_matches(|c| chars.contains(c)).to_string()
}

/// The text from byte `from` for at most `n` characters (a Python slice `t[from:from+n]` that never cuts a character in two).
pub fn take(t: &str, from: usize, n: usize) -> &str {
    if from >= t.len() {
        return "";
    }
    let rest = &t[from..];
    match rest.char_indices().nth(n) {
        Some((i, _)) => &rest[..i],
        None => rest,
    }
}

/// The text up to byte `to` for at most `n` characters before it (Python `t[to-n:to]`).
pub fn take_before(t: &str, to: usize, n: usize) -> &str {
    let head = &t[..to.min(t.len())];
    let count = head.chars().count();
    if count <= n {
        return head;
    }
    let skip = count - n;
    let (i, _) = head.char_indices().nth(skip).unwrap();
    &head[i..]
}

/// `static NAME: LazyLock<Re>` compiled once from a Python pattern (`lazy_re!(pub NAME, ...)` for a public one).
#[macro_export]
macro_rules! lazy_re {
    (pub $name:ident, $pat:expr) => {
        #[allow(dead_code)]
        pub static $name: std::sync::LazyLock<$crate::word_solver::rx::Re> = std::sync::LazyLock::new(|| $crate::word_solver::rx::re($pat));
    };
    ($name:ident, $pat:expr) => {
        #[allow(dead_code)]
        static $name: std::sync::LazyLock<$crate::word_solver::rx::Re> = std::sync::LazyLock::new(|| $crate::word_solver::rx::re($pat));
    };
}
