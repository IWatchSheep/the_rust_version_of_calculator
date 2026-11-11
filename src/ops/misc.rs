//! The remaining operators: constants, a few number-in / structure-out operators, and the polynomial text
//! operators (`expand`, `simplify`, `factor`), which work on exact fractions like Python's `Fraction`.

use super::vals::*;
use crate::value::Value;

// ---------------------------------------------------------------- constants
pub fn imaginary_unit(a: &[Value]) -> R {
    let _ = a;
    Ok(Value::Complex(0.0, 1.0))
}

pub fn infinity(a: &[Value]) -> R {
    let _ = a;
    n(f64::INFINITY)
}

pub fn euler_mascheroni_constant(a: &[Value]) -> R {
    let _ = a;
    n(0.5772156649015329)
}

pub fn empty_set(a: &[Value]) -> R {
    let _ = a;
    Ok(Value::Set(vec![]))
}

// ---------------------------------------------------------------- number in, structure out
fn py_mod(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 && (r < 0.0) != (b < 0.0) {
        r + b
    } else {
        r
    }
}

pub fn prime_factorization(a: &[Value]) -> R {
    let mut x = num(&a[0])?;
    let mut factors = vec![];
    let mut d = 2.0_f64;
    while d * d <= x {
        while py_mod(x, d) == 0.0 {
            factors.push(d);
            x = (x / d).floor();
        }
        d += 1.0;
    }
    if x > 1.0 {
        factors.push(x);
    }
    Ok(vlist(factors))
}

pub fn identity_matrix(a: &[Value]) -> R {
    let k = whole(num(&a[0])?)?.max(0) as usize;
    need(k <= 2000, "that matrix is too large")?;
    Ok(vmat((0..k).map(|i| (0..k).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect()))
}

pub fn characteristic_equation(a: &[Value]) -> R {
    let (p, q, c) = (num(&a[0])?, num(&a[1])?, num(&a[2])?);
    let disc = q * q - 4.0 * p * c;
    let (dre, dim) = if disc >= 0.0 { (disc.sqrt(), 0.0) } else { (0.0, (-disc).sqrt()) };
    let two_a = 2.0 * p;
    need(two_a != 0.0, "division by zero")?;
    Ok(Value::Tuple(vec![Value::Complex((-q + dre) / two_a, dim / two_a), Value::Complex((-q - dre) / two_a, -dim / two_a)]))
}

// ---------------------------------------------------------------- exact fractions and polynomials
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fr {
    n: i128,
    d: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn ovf<T>(o: Option<T>) -> Result<T, String> {
    o.ok_or_else(|| "the numbers got too large".to_string())
}

impl Fr {
    fn new(n: i128, d: i128) -> Result<Fr, String> {
        need(d != 0, "division by zero")?;
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Ok(Fr { n: s * n / g, d: s * d / g })
    }
    fn int(n: i128) -> Fr {
        Fr { n, d: 1 }
    }
    fn zero() -> Fr {
        Fr::int(0)
    }
    fn add(self, o: Fr) -> Result<Fr, String> {
        Fr::new(ovf(ovf(self.n.checked_mul(o.d))?.checked_add(ovf(o.n.checked_mul(self.d))?))?, ovf(self.d.checked_mul(o.d))?)
    }
    #[allow(dead_code)]
    fn sub(self, o: Fr) -> Result<Fr, String> {
        self.add(Fr { n: -o.n, d: o.d })
    }
    fn mul(self, o: Fr) -> Result<Fr, String> {
        Fr::new(ovf(self.n.checked_mul(o.n))?, ovf(self.d.checked_mul(o.d))?)
    }
    fn div(self, o: Fr) -> Result<Fr, String> {
        need(o.n != 0, "division by zero")?;
        Fr::new(ovf(self.n.checked_mul(o.d))?, ovf(self.d.checked_mul(o.n))?)
    }
    fn neg(self) -> Fr {
        Fr { n: -self.n, d: self.d }
    }
    fn is_zero(&self) -> bool {
        self.n == 0
    }
    fn text(&self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }
}

/// A polynomial in x: (power, coefficient) pairs; a plain Vec keeps the code short (the polynomials are tiny).
type Poly = Vec<(i64, Fr)>;

fn coeff(p: &Poly, k: i64) -> Option<Fr> {
    p.iter().find(|(e, _)| *e == k).map(|(_, c)| *c)
}

fn padd(a: &Poly, b: &Poly, sign: i128) -> Result<Poly, String> {
    let mut r = a.clone();
    for (k, c) in b {
        let c = if sign < 0 { c.neg() } else { *c };
        match r.iter_mut().find(|(e, _)| e == k) {
            Some(slot) => slot.1 = slot.1.add(c)?,
            None => r.push((*k, c)),
        }
    }
    Ok(r)
}

fn pmul(a: &Poly, b: &Poly) -> Result<Poly, String> {
    let mut r: Poly = vec![];
    for (i, ca) in a {
        for (j, cb) in b {
            let prod = ca.mul(*cb)?;
            match r.iter_mut().find(|(e, _)| *e == i + j) {
                Some(slot) => slot.1 = slot.1.add(prod)?,
                None => r.push((i + j, prod)),
            }
        }
    }
    Ok(r)
}

// A tiny parser for  + - * / **  with numbers, x and parentheses (what Python's ast gave `poly_parse`).
struct PP {
    t: Vec<char>,
    i: usize,
}

const ONLY_POLY: &str = "Only polynomials in x with + - * / ** can be handled here.";

impl PP {
    fn ws(&mut self) {
        while self.i < self.t.len() && self.t[self.i] == ' ' {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.ws();
        self.t.get(self.i).copied()
    }
    fn expr(&mut self) -> Result<Poly, String> {
        let mut l = self.term()?;
        loop {
            match self.peek() {
                Some('+') => {
                    self.i += 1;
                    let r = self.term()?;
                    l = padd(&l, &r, 1)?;
                }
                Some('-') => {
                    self.i += 1;
                    let r = self.term()?;
                    l = padd(&l, &r, -1)?;
                }
                _ => return Ok(l),
            }
        }
    }
    fn term(&mut self) -> Result<Poly, String> {
        let mut l = self.factor()?;
        loop {
            match self.peek() {
                Some('*') if self.t.get(self.i + 1) != Some(&'*') => {
                    self.i += 1;
                    let r = self.factor()?;
                    l = pmul(&l, &r)?;
                }
                Some('/') if self.t.get(self.i + 1) != Some(&'/') => {
                    self.i += 1;
                    let r = self.factor()?;
                    if r.len() == 1 && r[0].0 == 0 {
                        let d = r[0].1;
                        l = l.iter().map(|(k, c)| c.div(d).map(|v| (*k, v))).collect::<Result<_, _>>()?;
                    } else {
                        return Err(ONLY_POLY.into());
                    }
                }
                _ => return Ok(l),
            }
        }
    }
    fn factor(&mut self) -> Result<Poly, String> {
        match self.peek() {
            Some('-') => {
                self.i += 1;
                let v = self.factor()?;
                return Ok(v.into_iter().map(|(k, c)| (k, c.neg())).collect());
            }
            Some('+') => {
                self.i += 1;
                return self.factor();
            }
            _ => {}
        }
        let base = self.atom()?;
        if self.peek() == Some('*') && self.t.get(self.i + 1) == Some(&'*') {
            self.i += 2;
            let e = self.factor()?;
            if e.len() == 1 && e[0].0 == 0 && e[0].1.d == 1 && e[0].1.n >= 0 {
                let mut r: Poly = vec![(0, Fr::int(1))];
                for _ in 0..e[0].1.n {
                    r = pmul(&r, &base)?;
                }
                return Ok(r);
            }
            return Err(ONLY_POLY.into());
        }
        Ok(base)
    }
    fn atom(&mut self) -> Result<Poly, String> {
        match self.peek() {
            Some('(') => {
                self.i += 1;
                let v = self.expr()?;
                if self.peek() != Some(')') {
                    return Err("I can't read that.".into());
                }
                self.i += 1;
                Ok(v)
            }
            Some('x') => {
                self.i += 1;
                if self.t.get(self.i).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
                    return Err(ONLY_POLY.into());
                }
                Ok(vec![(1, Fr::int(1))])
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let st = self.i;
                while self.i < self.t.len() && (self.t[self.i].is_ascii_digit() || self.t[self.i] == '.') {
                    self.i += 1;
                }
                let text: String = self.t[st..self.i].iter().collect();
                let (whole, frac) = text.split_once('.').unwrap_or((&text, ""));
                let digits = format!("{whole}{frac}");
                let num: i128 = digits.parse().map_err(|_| "I can't read that number.".to_string())?;
                let den = 10_i128.checked_pow(frac.len() as u32).ok_or("the number is too long")?;
                Ok(vec![(0, Fr::new(num, den)?)])
            }
            _ => Err("I can't read that.".into()),
        }
    }
}

/// `_helpers.poly_parse`: text to {power: coefficient}, zero coefficients dropped.
fn poly_parse(text: &str) -> Result<Poly, String> {
    let text = text.replace('^', "**").replace('−', "-").replace('×', "*").replace('·', "*");
    let mut out: Vec<char> = vec![];
    for ch in text.chars() {
        if let Some(last) = out.last() {
            if (ch == '(' || ch == 'x') && (last.is_ascii_digit() || *last == ')' || *last == 'x') {
                out.push('*');
            }
        }
        out.push(ch);
    }
    let trimmed: String = out.iter().collect::<String>().trim().to_string();
    let mut p = PP { t: trimmed.chars().collect(), i: 0 };
    let v = p.expr()?;
    p.ws();
    need(p.i == p.t.len(), "I can't read that.")?;
    Ok(v.into_iter().filter(|(_, c)| !c.is_zero()).collect())
}

fn sup(k: i64) -> String {
    k.to_string()
        .chars()
        .map(|c| ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'][c.to_digit(10).unwrap() as usize])
        .collect()
}

fn poly_str(p: &Poly) -> String {
    if p.is_empty() {
        return "0".into();
    }
    let mut terms = p.clone();
    terms.sort_by(|a, b| b.0.cmp(&a.0));
    let parts: Vec<(char, String)> = terms
        .iter()
        .map(|(k, c)| {
            let mag = Fr { n: c.n.abs(), d: c.d };
            let num = mag.text();
            let one = mag.n == 1 && mag.d == 1;
            let body = format!(
                "{}{}",
                if !one || *k == 0 { num } else { String::new() },
                if *k == 0 { String::new() } else { format!("x{}", if *k > 1 { sup(*k) } else { String::new() }) }
            );
            (if c.n < 0 { '-' } else { '+' }, body)
        })
        .collect();
    let mut s = format!("{}{}", if parts[0].0 == '-' { "-" } else { "" }, parts[0].1);
    for (sign, body) in &parts[1..] {
        s.push_str(&format!(" {sign} {body}"));
    }
    s
}

fn eval_at(p: &Poly, r: Fr) -> Result<Fr, String> {
    let mut total = Fr::zero();
    for (k, c) in p {
        let mut pw = Fr::int(1);
        for _ in 0..*k {
            pw = pw.mul(r)?;
        }
        total = total.add(c.mul(pw)?)?;
    }
    Ok(total)
}

fn divide_linear(p: &Poly, r: Fr) -> Result<Poly, String> {
    let deg = p.iter().map(|(k, _)| *k).max().ok_or("empty polynomial")?;
    let mut q: Vec<Fr> = vec![];
    let mut carry = Fr::zero();
    for k in (0..=deg).rev() {
        carry = coeff(p, k).unwrap_or(Fr::zero()).add(carry.mul(r)?)?;
        q.push(carry);
    }
    let rem = q.pop().unwrap();
    need(rem.is_zero(), "the division left a remainder")?;
    Ok(q.into_iter().enumerate().filter(|(_, c)| !c.is_zero()).map(|(i, c)| (deg - 1 - i as i64, c)).collect())
}

fn divisors(nn: i128) -> Vec<i128> {
    if nn == 0 {
        return vec![1];
    }
    (1..=nn.abs()).filter(|d| nn % d == 0).take(400).collect()
}

fn poly_factor(p: &Poly) -> Result<String, String> {
    if p.is_empty() {
        return Ok("0".into());
    }
    let mut p = p.clone();
    let mut factors: Vec<String> = vec![];
    while !p.is_empty() && p.iter().map(|(k, _)| *k).max().unwrap() >= 1 {
        let lowest = p.iter().map(|(k, _)| *k).min().unwrap();
        if lowest > 0 {
            factors.push("x".into());
            p = p.into_iter().map(|(k, c)| (k - 1, c)).collect();
            continue;
        }
        let a0 = coeff(&p, 0).unwrap();
        let top = p.iter().map(|(k, _)| *k).max().unwrap();
        let an = coeff(&p, top).unwrap();
        let mut found: Option<Fr> = None;
        if a0.d == 1 && an.d == 1 {
            'search: for pn in divisors(a0.n.abs()) {
                for qd in divisors(an.n.abs()) {
                    for sgn in [1_i128, -1] {
                        let r = Fr::new(sgn * pn, qd)?;
                        if eval_at(&p, r)?.is_zero() {
                            found = Some(r);
                            break 'search;
                        }
                    }
                }
            }
        }
        let Some(r) = found else { break };
        p = divide_linear(&p, r)?;
        factors.push(if r.is_zero() {
            "x".to_string()
        } else {
            format!("(x {} {})", if r.n > 0 { "-" } else { "+" }, Fr { n: r.n.abs(), d: r.d }.text())
        });
    }
    let rest = if p.is_empty() { "1".to_string() } else { poly_str(&p) };
    if !p.is_empty() && !(p.len() == 1 && coeff(&p, 0) == Some(Fr::int(1))) {
        let top = p.iter().map(|(k, _)| *k).max().unwrap();
        factors.insert(0, if p.len() > 1 || top > 0 { format!("({rest})") } else { rest.clone() });
    }
    Ok(if factors.is_empty() { rest } else { factors.join(" ") })
}

fn text_arg(v: &Value) -> Result<String, String> {
    match v {
        Value::Text(s) => Ok(s.clone()),
        other => Err(format!("expected text, got {}", other.kind_name())),
    }
}

pub fn expand(a: &[Value]) -> R {
    Ok(Value::Text(poly_str(&poly_parse(&text_arg(&a[0])?)?)))
}

pub fn simplify(a: &[Value]) -> R {
    expand(a)
}

pub fn factor(a: &[Value]) -> R {
    Ok(Value::Text(poly_factor(&poly_parse(&text_arg(&a[0])?)?)?))
}
