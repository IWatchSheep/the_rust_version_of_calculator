//! Typed math -> an operator tree for a row (the "Mathematical Templates" panel); the Rust twin of `operator_row_math/template_math.py`.
//!
//! The user types some math, for example
//!
//! ```text
//! (□ + 3) × 2            empty box, plus 3, all times 2
//! sqrt(16) + abs(-3)     operators written by name
//! Time dilation(1, □)    any operator in the library, with its inputs in the brackets
//! sin(pi / 2)            pi and e are constants
//! ```
//!
//! and gets the same kind of tree the Operators list builds, so it can be put into a row.  Nothing here evaluates text: errors are readable messages.

use crate::layout::{self, CardKind};
use crate::ops;
use crate::row::Row;
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

const SYMBOL_OPS: &[(&str, &str)] = &[
    ("+", "math/arithmetic/addition"),
    ("-", "math/arithmetic/subtraction"),
    ("−", "math/arithmetic/subtraction"),
    ("*", "math/arithmetic/multiplication"),
    ("×", "math/arithmetic/multiplication"),
    ("·", "math/arithmetic/multiplication"),
    ("/", "math/arithmetic/division"),
    ("÷", "math/arithmetic/division"),
    ("%", "math/arithmetic/modulo"),
    ("^", "math/algebra/power"),
];
const NEGATE: &str = "math/arithmetic/negate";

/// Short names people expect -> the operator's folder name.
const ALIASES: &[(&str, &str)] = &[
    ("sin", "sine"),
    ("cos", "cosine"),
    ("tan", "tangent"),
    ("csc", "cosecant"),
    ("sec", "secant"),
    ("cot", "cotangent"),
    ("asin", "inverse_sine"),
    ("acos", "inverse_cosine"),
    ("atan", "inverse_tangent"),
    ("atan2", "two_argument_arctangent"),
    ("sinh", "hyperbolic_sine"),
    ("cosh", "hyperbolic_cosine"),
    ("tanh", "hyperbolic_tangent"),
    ("asinh", "inverse_hyperbolic_sine"),
    ("acosh", "inverse_hyperbolic_cosine"),
    ("atanh", "inverse_hyperbolic_tangent"),
    ("sqrt", "square_root"),
    ("cbrt", "cube_root"),
    ("abs", "absolute_value"),
    ("ln", "natural_log"),
    ("log", "log_base_10"),
    ("log10", "log_base_10"),
    ("exp", "exponential"),
    ("floor", "round_down"),
    ("ceil", "round_up"),
    ("round", "round_nearest"),
    ("min", "minimum"),
    ("max", "maximum"),
    ("det", "determinant"),
    ("fact", "factorial"),
    ("e", "euler_s_number"),
    ("recip", "reciprocal"),
    ("deg", "degrees_to_radians"),
    ("rad", "radians_to_degrees"),
    ("gcd", "greatest_common_divisor"),
    ("lcm", "least_common_multiple"),
    ("mod", "modulo"),
];

/// An error with a message a person can read.  `incomplete` = the math is not wrong, just not finished yet (so a guide can say "keep typing").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TplError {
    pub incomplete: bool,
    pub message: String,
}

impl std::fmt::Display for TplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Python's `repr` of a string: single quotes (double quotes when the text holds a single quote).
fn repr(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        format!("'{}'", s.replace('\'', "\\'"))
    }
}

fn bad<T>(msg: impl Into<String>) -> Result<T, TplError> {
    Err(TplError { incomplete: false, message: msg.into() })
}

fn unfinished<T>(msg: impl Into<String>) -> Result<T, TplError> {
    Err(TplError { incomplete: true, message: msg.into() })
}

fn norm(text: &str) -> String {
    text.to_lowercase().replace('’', "'").chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect()
}

fn index() -> &'static HashMap<String, &'static str> {
    static INDEX: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut ops_sorted: Vec<&'static crate::operator::Operator> = ops::all_operators().iter().collect();
        ops_sorted.sort_by_key(|o| (!o.key.starts_with("math/"), o.key));
        let mut idx: HashMap<String, &'static str> = HashMap::new();
        for op in ops_sorted {
            let last = op.key.rsplit('/').next().unwrap_or("");
            let short = op.name.split(" (").next().unwrap_or(op.name);
            for name in [last, op.name, short] {
                idx.entry(norm(name)).or_insert(op.key);
            }
        }
        for (alias, ident) in ALIASES {
            if let Some(k) = idx.get(&norm(ident)).copied() {
                idx.insert(norm(alias), k);
            }
        }
        idx
    })
}

/// The operator key for a typed name, or None.
pub fn find_operator(name: &str) -> Option<&'static str> {
    index().get(&norm(name)).copied()
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(String),
    Box,
    Word(String),
    Sym(char),
}

fn tokens(text: &str) -> Result<Vec<Tok>, TplError> {
    let chars: Vec<char> = text.trim().chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).map_or(false, |d| d.is_ascii_digit())) {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                let mut j = i + 1;
                if j < chars.len() && (chars[j] == '+' || chars[j] == '-') {
                    j += 1;
                }
                if j < chars.len() && chars[j].is_ascii_digit() {
                    while j < chars.len() && chars[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            out.push(Tok::Num(chars[start..i].iter().collect()));
        } else if c == '□' || c == '?' {
            out.push(Tok::Box);
            i += 1;
        } else if c == '_' && !chars.get(i + 1).map_or(false, |d| d.is_alphanumeric() || *d == '_') {
            out.push(Tok::Box);
            i += 1;
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '\'' || chars[i] == '’') {
                i += 1;
            }
            out.push(Tok::Word(chars[start..i].iter().collect()));
        } else if "-+−*×·/÷^%(),".contains(c) {
            out.push(Tok::Sym(c));
            i += 1;
        } else {
            return bad(format!("I don't understand {} in the math.", repr(&c.to_string())));
        }
    }
    Ok(out)
}

/// A value while parsing: a number as typed, an empty box, or an operator with its boxes.
enum V {
    Num(String),
    Empty,
    Node(Row),
}

struct Parser {
    toks: Vec<Tok>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i)
    }

    fn accept(&mut self, syms: &str) -> Option<char> {
        if let Some(Tok::Sym(c)) = self.peek() {
            if syms.contains(*c) {
                let c = *c;
                self.i += 1;
                return Some(c);
            }
        }
        None
    }

    fn make(&self, key: &str, args: Vec<V>) -> Result<Row, TplError> {
        let Some(op) = ops::get(key) else { return bad(format!("There is no operator {key}.")) };
        let mut row = Row::with_operator(op);
        if row.inputs.len() != args.len() {
            return bad(format!("{} needs {} input(s) but was given {}.", op.name, row.inputs.len(), args.len()));
        }
        for (j, a) in args.into_iter().enumerate() {
            match a {
                V::Num(t) => row.inputs[j] = t,
                V::Empty => {}
                V::Node(n) => row.children[j] = Some(Box::new(n)),
            }
        }
        Ok(row)
    }

    fn parse(&mut self) -> Result<Row, TplError> {
        if self.toks.is_empty() {
            return unfinished("Type some math first.");
        }
        let value = self.expr()?;
        if let Some(t) = self.peek() {
            let shown = match t {
                Tok::Num(s) | Tok::Word(s) => s.clone(),
                Tok::Box => "□".to_string(),
                Tok::Sym(c) => c.to_string(),
            };
            if !matches!(t, Tok::Sym(_)) || shown == "(" {
                return bad(format!("Put an operator between {} and what comes before it (for example 2 × {}).", repr(&shown), shown));
            }
            return bad(format!("Unexpected {}.", repr(&shown)));
        }
        match value {
            V::Node(n) => Ok(n),
            _ => unfinished("A template needs at least one operator, for example (□ + 3) × 2."),
        }
    }

    fn sym_key(c: char) -> &'static str {
        SYMBOL_OPS.iter().find(|(s, _)| s.chars().next() == Some(c)).map(|(_, k)| *k).unwrap_or("")
    }

    fn expr(&mut self) -> Result<V, TplError> {
        let mut left = self.term()?;
        while let Some(op) = self.accept("+-−") {
            let right = self.term()?;
            left = V::Node(self.make(Self::sym_key(op), vec![left, right])?);
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<V, TplError> {
        let mut left = self.unary()?;
        while let Some(op) = self.accept("*×·/÷%") {
            let right = self.unary()?;
            left = V::Node(self.make(Self::sym_key(op), vec![left, right])?);
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<V, TplError> {
        if let Some(Tok::Sym(c)) = self.peek() {
            if *c == '-' || *c == '−' {
                self.i += 1;
                let after_is_power = matches!(self.toks.get(self.i + 1), Some(Tok::Sym('^')));
                if let Some(Tok::Num(n)) = self.peek() {
                    if !after_is_power {
                        let n = n.clone();
                        self.i += 1;
                        return Ok(V::Num(format!("-{n}"))); // -5 is the number minus five
                    }
                }
                let inner = self.unary()?;
                return Ok(V::Node(self.make(NEGATE, vec![inner])?));
            }
            if *c == '+' {
                self.i += 1;
                return self.unary();
            }
        }
        self.power()
    }

    fn power(&mut self) -> Result<V, TplError> {
        let base = self.atom()?;
        if self.accept("^").is_some() {
            let exp = self.unary()?; // right-associative: 2^3^2 = 2^(3^2)
            return Ok(V::Node(self.make(Self::sym_key('^'), vec![base, exp])?));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<V, TplError> {
        match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.i += 1;
                Ok(V::Num(n))
            }
            Some(Tok::Box) => {
                self.i += 1;
                Ok(V::Empty)
            }
            Some(Tok::Sym('(')) => {
                self.i += 1;
                let inner = self.expr()?;
                if self.accept(")").is_none() {
                    return unfinished("A bracket ( is never closed.");
                }
                Ok(inner)
            }
            Some(Tok::Word(_)) => {
                let mut words = Vec::new();
                while let Some(Tok::Word(w)) = self.peek().cloned() {
                    words.push(w);
                    self.i += 1;
                }
                let name = words.join(" ");
                if self.accept("(").is_some() {
                    return self.call(&name);
                }
                if words.len() > 1 {
                    return bad(format!("Write {} with its inputs in brackets, like {}( ... ).", repr(&name), name));
                }
                if let Some(key) = find_operator(&name) {
                    if ops::get(key).map_or(false, |o| o.arity() == 0) {
                        return Ok(V::Node(self.make(key, vec![])?)); // a constant such as pi
                    }
                }
                Ok(V::Empty) // any other bare letter is an empty box
            }
            None => unfinished("The math ends too soon: something is missing at the end."),
            Some(Tok::Sym(c)) => bad(format!("Unexpected {}.", repr(&c.to_string()))),
        }
    }

    fn call(&mut self, name: &str) -> Result<V, TplError> {
        let Some(key) = find_operator(name) else {
            return bad(format!("There is no operator called {}. Try its name from the Operators list, or a short name such as sqrt, sin, ln.", repr(name)));
        };
        let mut args = Vec::new();
        if self.accept(")").is_none() {
            loop {
                args.push(self.expr()?);
                if self.accept(")").is_some() {
                    break;
                }
                if self.accept(",").is_none() {
                    return bad(format!("Expected , or ) in {name}( ... )."));
                }
            }
        }
        Ok(V::Node(self.make(key, args)?))
    }
}

/// The operator tree (a row) for typed math.
pub fn parse_math(text: &str) -> Result<Row, TplError> {
    let toks = tokens(text)?;
    Parser { toks, i: 0 }.parse()
}

/// How the template reads, with brackets around nested parts, e.g. `(2 + □) × 4` (□ = an empty box).
pub fn describe(row: &Row) -> String {
    let Some(layout) = row.operator().and_then(|o| layout::get(o.key)) else { return String::new() };
    let mut parts: Vec<(CardKind, String)> = Vec::new();
    for card in layout.cards {
        match card.kind {
            CardKind::Box => {
                let j = card.slot as usize;
                match row.children.get(j).and_then(|c| c.as_deref()) {
                    Some(child) => {
                        let inner = describe(child);
                        let first = child.operator().and_then(|o| layout::get(o.key)).and_then(|l| l.cards.iter().find(|c| c.kind != CardKind::Out));
                        let opens = first.map_or(false, |c| c.kind == CardKind::Op && c.text.ends_with('('));
                        parts.push((CardKind::Op, if opens { inner } else { format!("({inner})") }));
                    }
                    None => {
                        let t = row.inputs.get(j).cloned().unwrap_or_default();
                        parts.push((CardKind::Op, if t.is_empty() { "□".to_string() } else { t }));
                    }
                }
            }
            kind => parts.push((kind, card.text.to_string())),
        }
    }
    let levels: Vec<i8> = layout.cards.iter().map(|c| c.level).collect();
    let has_out = layout.cards.iter().any(|c| c.kind == CardKind::Out);
    let eq = layout.cards.iter().position(|c| c.kind == CardKind::Op && c.text == "=");
    let (parts, levels) = match (has_out, eq) {
        (true, Some(e)) => (parts[e + 1..].to_vec(), levels[e + 1..].to_vec()),
        _ => (parts, levels),
    };
    let mut pieces: Vec<String> = Vec::new();
    for (n, (kind, text)) in parts.iter().enumerate() {
        if *kind == CardKind::Out {
            continue;
        }
        let level = levels.get(n).copied().unwrap_or(0);
        if level != 0 && !pieces.is_empty() {
            let last = pieces.len() - 1;
            pieces[last].push(if level < 0 { '^' } else { '_' });
            pieces[last].push_str(text);
        } else {
            pieces.push(text.clone());
        }
    }
    pieces.join(" ")
}

/// How many boxes are still empty (a variable the picture writes twice counts once).
pub fn empty_boxes(row: &Row) -> usize {
    let set: BTreeSet<Vec<usize>> = row.flatten().into_iter().filter(|c| c.kind == CardKind::Box && c.text.is_empty()).map(|c| c.path).collect();
    set.len()
}

// ---------------------------------------------------------------- saved templates (a small JSON file)

/// (saved templates, hide the how-to guide?) from the file.  A missing or damaged file gives (empty, false).
pub fn load_data(path: &std::path::Path) -> (Vec<String>, bool) {
    let Ok(text) = std::fs::read_to_string(path) else { return (vec![], false) };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return (vec![], false) };
    let hide = v.get("hide_guide").and_then(|h| h.as_bool()).unwrap_or(false);
    let list = if v.is_array() { v.as_array() } else { v.get("templates").and_then(|t| t.as_array()) };
    let templates = list
        .map(|a| a.iter().filter_map(|t| t.as_str()).filter(|t| !t.trim().is_empty()).map(|t| t.to_string()).collect())
        .unwrap_or_default();
    (templates, hide)
}

pub fn save_data(path: &std::path::Path, templates: &[String], hide_guide: bool) -> Result<(), String> {
    let v = serde_json::json!({ "templates": templates, "hide_guide": hide_guide });
    std::fs::write(path, serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
