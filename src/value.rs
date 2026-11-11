//! Values that flow through boxes and `compute`.

use std::fmt;

/// A computed or typed value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    /// Two real roots / two results (e.g. quadratic formula).
    Pair(f64, f64),
    Bool(bool),
    /// Complex number `re + im·i` (Python `complex`).
    Complex(f64, f64),
    Text(String),
    /// A Python list (also used for vectors and, as a list of lists, for matrices).
    List(Vec<Value>),
    /// A Python tuple (a point, an edge, a pair of results ...).
    Tuple(Vec<Value>),
    /// A Python set.  Kept in a fixed order (see `Value::set_of`) so equal sets compare and print the same.
    Set(Vec<Value>),
    /// A Python dict as (key, value) pairs in insertion order (graphs, assignments).
    Dict(Vec<(Value, Value)>),
    /// A function typed as text such as `x**2 + 3*x`; parsed on use by `crate::expr`.
    Func(String),
}

impl Value {
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(n) if n.is_finite() => Ok(*n),
            Value::Number(_) => Err("not a finite number".into()),
            Value::Pair(_, _) => Err("expected one number, got a pair".into()),
            Value::Bool(_) => Err("expected a number, got a bool".into()),
            Value::Complex(_, _) => Err("expected a real number, got a complex".into()),
            Value::Text(s) => s
                .parse::<f64>()
                .map_err(|_| format!("cannot parse '{s}' as a number")),
            other => Err(format!("expected a number, got {}", other.kind_name())),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "a number",
            Value::Pair(..) | Value::Tuple(_) => "a tuple",
            Value::Bool(_) => "a bool",
            Value::Complex(..) => "a complex number",
            Value::Text(_) => "text",
            Value::List(_) => "a list",
            Value::Set(_) => "a set",
            Value::Dict(_) => "a dict",
            Value::Func(_) => "a function",
        }
    }

    /// A set value: duplicates removed (numbers equal within rounding are the same item), items in a fixed order.
    pub fn set_of(items: Vec<Value>) -> Value {
        let mut out: Vec<Value> = Vec::new();
        for v in items {
            if !out.iter().any(|o| o.same_item(&v)) {
                out.push(v);
            }
        }
        out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        Value::Set(out)
    }

    /// Equality the way Python's `==` treats set members (1 == 1.0, strings by text, sequences item by item).
    pub fn same_item(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Bool(a), Number(b)) | (Number(b), Bool(a)) => (*a as i32 as f64) == *b,
            (Text(a), Text(b)) => a == b,
            (Complex(a, b), Complex(c, d)) => a == c && b == d,
            (Number(a), Complex(c, d)) | (Complex(c, d), Number(a)) => a == c && *d == 0.0,
            (List(a), List(b)) | (Tuple(a), Tuple(b)) => a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.same_item(y)),
            (Pair(a, b), Pair(c, d)) => a == c && b == d,
            (Pair(a, b), Tuple(t)) | (Tuple(t), Pair(a, b)) => t.len() == 2 && t[0].same_item(&Number(*a)) && t[1].same_item(&Number(*b)),
            (Set(a), Set(b)) => a.len() == b.len() && a.iter().all(|x| b.iter().any(|y| x.same_item(y))),
            _ => false,
        }
    }

    /// Text used only to put set members in a stable order: numbers by value first, then text.
    pub fn sort_key(&self) -> (u8, String) {
        match self {
            Value::Number(n) => (0, format!("{:020.6}", n + 1e12)),
            Value::Bool(b) => (0, format!("{:020.6}", (*b as i32 as f64) + 1e12)),
            Value::Text(t) => (1, t.clone()),
            other => (2, other.display()),
        }
    }

    pub fn is_finite_result(&self) -> bool {
        match self {
            Value::Number(n) => n.is_finite(),
            Value::Pair(a, b) => a.is_finite() && b.is_finite(),
            Value::Bool(_) => true,
            Value::Complex(re, im) => re.is_finite() && im.is_finite(),
            Value::Text(_) | Value::Func(_) => true,
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => v.iter().all(|x| x.is_finite_result()),
            Value::Dict(d) => d.iter().all(|(k, v)| k.is_finite_result() && v.is_finite_result()),
        }
    }

    /// How the calculator window shows an answer: the same as the Python program (a whole number as a whole number, otherwise 10 significant digits, so
    /// `7.415555556e-11` and not `0.000000000074`).  `display()` is the exact parity form used by the tests.
    pub fn show(&self) -> String {
        match self {
            Value::Number(n) => format_py(*n),
            Value::Pair(a, b) => format!("{}, {}", format_py(*a), format_py(*b)),
            Value::Tuple(v) if v.iter().all(|x| matches!(x, Value::Number(_))) => format!("({})", v.iter().map(|x| x.show()).collect::<Vec<_>>().join(", ")),
            Value::List(v) if v.iter().all(|x| matches!(x, Value::Number(_))) => format!("[{}]", v.iter().map(|x| x.show()).collect::<Vec<_>>().join(", ")),
            other => other.display(),
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Number(n) => format_num(*n),
            Value::Pair(a, b) => format!("{}, {}", format_num(*a), format_num(*b)),
            Value::Bool(b) => if *b { "true".into() } else { "false".into() },
            Value::Complex(re, im) => format_complex(*re, *im),
            Value::Text(s) | Value::Func(s) => s.clone(),
            Value::List(v) => format!("[{}]", v.iter().map(|x| x.display_item()).collect::<Vec<_>>().join(", ")),
            Value::Tuple(v) => format!("({})", v.iter().map(|x| x.display_item()).collect::<Vec<_>>().join(", ")),
            Value::Set(v) if v.is_empty() => "set()".into(),
            Value::Set(v) => format!("{{{}}}", v.iter().map(|x| x.display_item()).collect::<Vec<_>>().join(", ")),
            Value::Dict(d) => format!(
                "{{{}}}",
                d.iter().map(|(k, v)| format!("{}: {}", k.display_item(), v.display_item())).collect::<Vec<_>>().join(", ")
            ),
        }
    }

    /// Like `display`, but text is quoted so it can be told from a number inside a list.
    fn display_item(&self) -> String {
        match self {
            Value::Text(s) => format!("'{s}'"),
            Value::Pair(a, b) => format!("({}, {})", format_num(*a), format_num(*b)),
            other => other.display(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Python's `str(int)` for whole numbers below 1e15, otherwise `"%.10g"`.
pub fn format_py(n: f64) -> String {
    if !n.is_finite() {
        return n.to_string();
    }
    if n == 0.0 {
        return "0".into();
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let sci = format!("{:.9e}", n);
    let (mant, exp) = sci.split_once('e').unwrap();
    let e: i32 = exp.parse().unwrap();
    let trim = |t: String| -> String {
        if t.contains('.') {
            t.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            t
        }
    };
    if e < -4 || e >= 10 {
        let sign = if e < 0 { '-' } else { '+' };
        format!("{}e{}{:02}", trim(mant.to_string()), sign, e.abs())
    } else {
        trim(format!("{:.*}", (9 - e).max(0) as usize, n))
    }
}

fn format_num(n: f64) -> String {
    if !n.is_finite() {
        return n.to_string();
    }
    if n == 0.0 {
        return "0".into();
    }
    let s = format!("{:.12}", n);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s.is_empty() || s == "-" {
        "0".into()
    } else {
        s
    }
}

/// Match the catalog / parity convention: `re+imj` (Python-like, always with `j`).
pub fn format_complex(re: f64, im: f64) -> String {
    let re_s = format_num(re);
    let im_abs = format_num(im.abs());
    if im == 0.0 {
        format!("{re_s}+0j")
    } else if im < 0.0 {
        format!("{re_s}-{im_abs}j")
    } else {
        format!("{re_s}+{im_abs}j")
    }
}

/// Parse a parity / display token: `true`/`false`, `re±imj`, or a float.
pub fn parse_token(s: &str) -> Result<Value, String> {
    let t = s.trim();
    if t.eq_ignore_ascii_case("true") {
        return Ok(Value::Bool(true));
    }
    if t.eq_ignore_ascii_case("false") {
        return Ok(Value::Bool(false));
    }
    if let Some(v) = parse_complex_token(t) {
        return Ok(v);
    }
    t.parse::<f64>()
        .map(Value::Number)
        .map_err(|_| format!("cannot parse value token {t:?}"))
}

fn parse_complex_token(t: &str) -> Option<Value> {
    if !t.ends_with('j') && !t.ends_with('J') {
        return None;
    }
    let body = &t[..t.len() - 1];
    // Pure imaginary: "3j", "+3j", "-3j"
    if !body.contains('+') && !(body.matches('-').count() > 1 || (body.starts_with('-') && body[1..].contains('-'))) {
        // no real part separator: entire body is imag, unless empty
        if let Ok(im) = body.parse::<f64>() {
            return Some(Value::Complex(0.0, im));
        }
        if body.is_empty() {
            return Some(Value::Complex(0.0, 1.0));
        }
    }
    // Find the split between re and im: last + or - that isn't the leading sign of re.
    let bytes = body.as_bytes();
    let mut split = None;
    for i in (1..bytes.len()).rev() {
        // a sign right after an exponent marker (1e-05) belongs to the exponent, not to the imaginary part
        if (bytes[i] == b'+' || bytes[i] == b'-') && !matches!(bytes[i - 1], b'e' | b'E') {
            split = Some(i);
            break;
        }
    }
    let (re_s, im_s) = match split {
        Some(i) => (&body[..i], &body[i..]),
        None => return None,
    };
    let re = re_s.parse::<f64>().ok()?;
    let im = im_s.parse::<f64>().ok()?;
    Some(Value::Complex(re, im))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_round_trip_tokens() {
        assert_eq!(parse_token("true").unwrap(), Value::Bool(true));
        assert_eq!(parse_token("false").unwrap(), Value::Bool(false));
        assert_eq!(parse_token("3+4j").unwrap(), Value::Complex(3.0, 4.0));
        assert_eq!(parse_token("1-2j").unwrap(), Value::Complex(1.0, -2.0));
        assert_eq!(parse_token("-0.5+0.25j").unwrap(), Value::Complex(-0.5, 0.25));
        // signs inside exponents must not split re/im (Claude fix)
        assert_eq!(
            parse_token("2.8e-05-1.2e-05j").unwrap(),
            Value::Complex(2.8e-5, -1.2e-5)
        );
        assert_eq!(format_complex(3.0, 4.0), "3+4j");
        assert_eq!(format_complex(1.0, -2.0), "1-2j");
    }
}

#[cfg(test)]
mod show_tests {
    use super::*;

    #[test]
    fn answers_are_shown_like_python() {
        assert_eq!(Value::Number(7.415555555555556e-11).show(), "7.415555556e-11");
        assert_eq!(Value::Number(24.0).show(), "24");
        assert_eq!(Value::Number(75.39822368615503).show(), "75.39822369");
        assert_eq!(Value::Number(0.5).show(), "0.5");
        assert_eq!(Value::Number(-1234567.891).show(), "-1234567.891");
        assert_eq!(Value::Number(1.2345678901234e20).show(), "1.23456789e+20");
        assert_eq!(Value::Number(1e-5).show(), "1e-05");
        assert_eq!(Value::Pair(2.0, 1.0).show(), "2, 1");
    }
}
