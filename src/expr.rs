//! Functions typed as text, e.g. `x**2 + 3*x`, `sin(x)*y`, `[2*x, 2*y]`, `x > 2`.
//!
//! This is the Rust twin of `make_function` in `operator_row_math/advanced_input.py`: the same small Python
//! expression syntax, the same whitelist of names, the same way the arguments are handed to x, y, z, t, u, w,
//! and Python's error behaviour (division by zero, math domain errors and overflow are errors, a negative number
//! to a fractional power becomes a complex number).  Nothing is evaluated by any other means.

use crate::value::Value;

const VARS: [&str; 6] = ["x", "y", "z", "t", "u", "w"];
const FUNCS: [&str; 19] = [
    "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "sqrt", "exp", "log", "ln", "log10", "abs",
    "floor", "ceil", "min", "max",
];

#[derive(Clone, Debug)]
enum Node {
    Num(f64),
    Bool(bool),
    Var(usize),
    Const(f64),
    Neg(Box<Node>),
    Pos(Box<Node>),
    Not(Box<Node>),
    Bin(char, Box<Node>, Box<Node>), // + - * / % (floor div is 'f', power is 'p')
    Cmp(Box<Node>, Vec<(String, Node)>),
    And(Vec<Node>),
    Or(Vec<Node>),
    If(Box<Node>, Box<Node>, Box<Node>),
    Call(String, Vec<Node>),
    List(Vec<Node>),
    Tuple(Vec<Node>),
    Index(Box<Node>, Box<Node>),
}

#[derive(Clone, Debug)]
pub struct Callable {
    root: Node,
    /// the variables the text uses, in x, y, z, t, u, w order
    names: Vec<usize>,
    pub text: String,
}

// ------------------------------------------------------------------ tokens
#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Op(String),
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = vec![];
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && i + 1 < cs.len() && cs[i + 1].is_ascii_digit()) {
            let st = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.') {
                i += 1;
            }
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') {
                let mut j = i + 1;
                if j < cs.len() && (cs[j] == '+' || cs[j] == '-') {
                    j += 1;
                }
                if j < cs.len() && cs[j].is_ascii_digit() {
                    while j < cs.len() && cs[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            let t: String = cs[st..i].iter().collect();
            out.push(Tok::Num(t.parse::<f64>().map_err(|_| "I can't read that function.".to_string())?));
        } else if c.is_alphabetic() || c == '_' {
            let st = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_') {
                i += 1;
            }
            out.push(Tok::Name(cs[st..i].iter().collect()));
        } else {
            let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
            if ["**", "//", "==", "!=", "<=", ">="].contains(&two.as_str()) {
                out.push(Tok::Op(two));
                i += 2;
            } else if "+-*/%()[],<>".contains(c) {
                out.push(Tok::Op(c.to_string()));
                i += 1;
            } else {
                return Err("I can't read that function. Example:  x**2 + 3*x".into());
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------ parser
struct P {
    t: Vec<Tok>,
    i: usize,
    used: Vec<usize>,
}

const BAD: &str = "I can't read that function. Example:  x**2 + 3*x";

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn op(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Op(o)) if o == s)
    }
    fn kw(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Name(o)) if o == s)
    }
    fn eat_op(&mut self, s: &str) -> bool {
        if self.op(s) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.eat_op(s) {
            Ok(())
        } else {
            Err(BAD.into())
        }
    }

    fn expr(&mut self) -> Result<Node, String> {
        let body = self.or()?;
        if self.kw("if") {
            self.i += 1;
            let cond = self.or()?;
            if !self.kw("else") {
                return Err(BAD.into());
            }
            self.i += 1;
            let other = self.expr()?;
            return Ok(Node::If(Box::new(cond), Box::new(body), Box::new(other)));
        }
        Ok(body)
    }
    fn or(&mut self) -> Result<Node, String> {
        let mut v = vec![self.and()?];
        while self.kw("or") {
            self.i += 1;
            v.push(self.and()?);
        }
        Ok(if v.len() == 1 { v.pop().unwrap() } else { Node::Or(v) })
    }
    fn and(&mut self) -> Result<Node, String> {
        let mut v = vec![self.not()?];
        while self.kw("and") {
            self.i += 1;
            v.push(self.not()?);
        }
        Ok(if v.len() == 1 { v.pop().unwrap() } else { Node::And(v) })
    }
    fn not(&mut self) -> Result<Node, String> {
        if self.kw("not") {
            self.i += 1;
            return Ok(Node::Not(Box::new(self.not()?)));
        }
        self.cmp()
    }
    fn cmp(&mut self) -> Result<Node, String> {
        let first = self.arith()?;
        let mut rest = vec![];
        loop {
            let o = match self.peek() {
                Some(Tok::Op(o)) if ["<", ">", "<=", ">=", "==", "!="].contains(&o.as_str()) => o.clone(),
                _ => break,
            };
            self.i += 1;
            rest.push((o, self.arith()?));
        }
        Ok(if rest.is_empty() { first } else { Node::Cmp(Box::new(first), rest) })
    }
    fn arith(&mut self) -> Result<Node, String> {
        let mut l = self.term()?;
        loop {
            let c = if self.op("+") {
                '+'
            } else if self.op("-") {
                '-'
            } else {
                break;
            };
            self.i += 1;
            l = Node::Bin(c, Box::new(l), Box::new(self.term()?));
        }
        Ok(l)
    }
    fn term(&mut self) -> Result<Node, String> {
        let mut l = self.factor()?;
        loop {
            let c = if self.op("*") {
                '*'
            } else if self.op("/") {
                '/'
            } else if self.op("//") {
                'f'
            } else if self.op("%") {
                '%'
            } else {
                break;
            };
            self.i += 1;
            l = Node::Bin(c, Box::new(l), Box::new(self.factor()?));
        }
        Ok(l)
    }
    fn factor(&mut self) -> Result<Node, String> {
        if self.eat_op("-") {
            return Ok(Node::Neg(Box::new(self.factor()?)));
        }
        if self.eat_op("+") {
            return Ok(Node::Pos(Box::new(self.factor()?)));
        }
        let base = self.postfix()?;
        if self.eat_op("**") {
            let e = self.factor()?;
            return Ok(Node::Bin('p', Box::new(base), Box::new(e)));
        }
        Ok(base)
    }
    fn postfix(&mut self) -> Result<Node, String> {
        let mut n = self.atom()?;
        loop {
            if self.op("(") {
                let name = match &n {
                    Node::Call(nm, a) if a.is_empty() && nm.starts_with('\u{1}') => nm[1..].to_string(),
                    _ => return Err(format!("Allowed functions: {}", sorted_funcs())),
                };
                self.i += 1;
                let mut args = vec![];
                if !self.op(")") {
                    loop {
                        args.push(self.expr()?);
                        if self.op("=") {
                            return Err(BAD.into());
                        }
                        if !self.eat_op(",") {
                            break;
                        }
                    }
                }
                self.expect(")")?;
                n = Node::Call(name, args);
            } else if self.op("[") {
                self.i += 1;
                let idx = self.expr()?;
                self.expect("]")?;
                n = Node::Index(Box::new(n), Box::new(idx));
            } else {
                break;
            }
        }
        Ok(n)
    }
    fn atom(&mut self) -> Result<Node, String> {
        match self.peek().cloned() {
            Some(Tok::Num(v)) => {
                self.i += 1;
                Ok(Node::Num(v))
            }
            Some(Tok::Name(nm)) => {
                self.i += 1;
                if nm == "True" || nm == "False" {
                    return Ok(Node::Bool(nm == "True"));
                }
                if FUNCS.contains(&nm.as_str()) {
                    // only legal when called; marked so postfix() can tell a function name from a variable
                    return Ok(Node::Call(format!("\u{1}{nm}"), vec![]));
                }
                if nm == "pi" {
                    return Ok(Node::Const(std::f64::consts::PI));
                }
                if nm == "e" {
                    return Ok(Node::Const(std::f64::consts::E));
                }
                if let Some(ix) = VARS.iter().position(|v| *v == nm) {
                    if !self.used.contains(&ix) {
                        self.used.push(ix);
                    }
                    return Ok(Node::Var(ix));
                }
                Err(format!("Unknown name '{nm}'. Use x, y, z, t (variables), pi or e."))
            }
            Some(Tok::Op(o)) if o == "(" => {
                self.i += 1;
                if self.eat_op(")") {
                    return Ok(Node::Tuple(vec![]));
                }
                let first = self.expr()?;
                if self.eat_op(",") {
                    let mut items = vec![first];
                    while !self.op(")") {
                        items.push(self.expr()?);
                        if !self.eat_op(",") {
                            break;
                        }
                    }
                    self.expect(")")?;
                    return Ok(Node::Tuple(items));
                }
                self.expect(")")?;
                Ok(first)
            }
            Some(Tok::Op(o)) if o == "[" => {
                self.i += 1;
                let mut items = vec![];
                while !self.op("]") {
                    items.push(self.expr()?);
                    if !self.eat_op(",") {
                        break;
                    }
                }
                self.expect("]")?;
                Ok(Node::List(items))
            }
            _ => Err(BAD.into()),
        }
    }
}

fn sorted_funcs() -> String {
    let mut f = FUNCS.to_vec();
    f.sort();
    f.join(", ")
}

/// A bare function name that is never called (`sin + 1`) is not an expression.
fn check_calls(n: &Node) -> Result<(), String> {
    match n {
        Node::Call(nm, a) => {
            if nm.starts_with('\u{1}') && a.is_empty() {
                return Err(BAD.into());
            }
            a.iter().try_for_each(check_calls)
        }
        Node::Neg(a) | Node::Pos(a) | Node::Not(a) => check_calls(a),
        Node::Bin(_, a, b) | Node::Index(a, b) => check_calls(a).and(check_calls(b)),
        Node::Cmp(a, r) => check_calls(a).and(r.iter().try_for_each(|(_, x)| check_calls(x))),
        Node::And(v) | Node::Or(v) | Node::List(v) | Node::Tuple(v) => v.iter().try_for_each(check_calls),
        Node::If(a, b, c) => check_calls(a).and(check_calls(b)).and(check_calls(c)),
        _ => Ok(()),
    }
}

impl Callable {
    pub fn parse(src: &str) -> Result<Callable, String> {
        let text = src.trim().replace('^', "**").replace('×', "*").replace('÷', "/").replace('−', "-");
        if text.to_lowercase().starts_with("lambda") {
            return Err("Just type the expression, for example  x**2 + 3*x".into());
        }
        let mut p = P { t: tokenize(&text)?, i: 0, used: vec![] };
        let root = p.expr()?;
        if p.i != p.t.len() {
            return Err(BAD.into());
        }
        check_calls(&root)?;
        let mut names = p.used.clone();
        names.sort();
        Ok(Callable { root, names, text })
    }

    /// Python's `fn(*args)`.
    pub fn call(&self, args: &[Value]) -> Result<Value, String> {
        let mut args: Vec<Value> = args.to_vec();
        let single_seq = args.len() == 1 && matches!(args[0], Value::List(_) | Value::Tuple(_) | Value::Pair(..));
        if single_seq && self.names.len() > 1 {
            args = seq_items(&args[0]).unwrap();
        }
        let cmode = args.iter().any(|a| matches!(a, Value::Complex(..)));
        let mut env: Vec<Option<Value>> = vec![None; VARS.len()];
        if args.len() == 1 && self.names.len() == 1 {
            env[self.names[0]] = Some(args[0].clone());
        } else {
            for (var_idx, value) in self.names.iter().zip(args.iter()) {
                env[*var_idx] = Some(value.clone());
            }
        }
        let missing: Vec<&str> = self.names.iter().filter(|n| env[**n].is_none()).map(|n| VARS[*n]).collect();
        if !missing.is_empty() {
            return Err(format!(
                "The function needs {} but only {} value(s) were given.",
                missing.join(", "),
                args.len()
            ));
        }
        eval(&self.root, &env, cmode)
    }

    /// Call with real numbers and expect a real number back.
    pub fn f(&self, args: &[f64]) -> Result<f64, String> {
        let v: Vec<Value> = args.iter().map(|a| Value::Number(*a)).collect();
        real(&self.call(&v)?)
    }
}

/// A real number out of a result (booleans count, as in Python); complex results are not supported by the real-valued operators.
pub fn real(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::Bool(b) => Ok(*b as i32 as f64),
        Value::Complex(re, im) if *im == 0.0 => Ok(*re),
        Value::Complex(..) => Err("the result is a complex number".into()),
        other => Err(format!("expected a number, got {}", other.kind_name())),
    }
}

fn seq_items(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(a) | Value::Tuple(a) => Some(a.clone()),
        Value::Pair(a, b) => Some(vec![Value::Number(*a), Value::Number(*b)]),
        _ => None,
    }
}

// ------------------------------------------------------------------ evaluation
type C = (f64, f64);

fn cparts(v: &Value) -> Result<C, String> {
    match v {
        Value::Number(n) => Ok((*n, 0.0)),
        Value::Bool(b) => Ok((*b as i32 as f64, 0.0)),
        Value::Complex(a, b) => Ok((*a, *b)),
        other => Err(format!("unsupported operand: {}", other.kind_name())),
    }
}

fn is_cplx(v: &Value) -> bool {
    matches!(v, Value::Complex(..))
}

fn cvalue(c: C) -> Value {
    Value::Complex(c.0, c.1)
}

fn cmul(a: C, b: C) -> C {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cdiv(a: C, b: C) -> Result<C, String> {
    let d = b.0 * b.0 + b.1 * b.1;
    if d == 0.0 {
        return Err("division by zero".into());
    }
    Ok(((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d))
}

fn cexp(a: C) -> C {
    let e = a.0.exp();
    (e * a.1.cos(), e * a.1.sin())
}

fn clog(a: C) -> Result<C, String> {
    if a.0 == 0.0 && a.1 == 0.0 {
        return Err("math domain error".into());
    }
    Ok((a.0.hypot(a.1).ln(), a.1.atan2(a.0)))
}

fn csqrt(a: C) -> C {
    let m = a.0.hypot(a.1);
    let re = ((m + a.0) / 2.0).sqrt();
    let im = ((m - a.0) / 2.0).sqrt();
    (re, if a.1 < 0.0 { -im } else { im })
}

fn cpow(a: C, b: C) -> Result<C, String> {
    if a == (0.0, 0.0) {
        if b == (0.0, 0.0) {
            return Ok((1.0, 0.0));
        }
        if b.0 > 0.0 && b.1 == 0.0 {
            return Ok((0.0, 0.0));
        }
        return Err("zero to a negative or complex power".into());
    }
    // small whole exponents by repeated multiplication (what Python does), otherwise exp(b ln a)
    if b.1 == 0.0 && b.0.fract() == 0.0 && b.0.abs() <= 100.0 {
        let n = b.0.abs() as u32;
        let mut r = (1.0, 0.0);
        for _ in 0..n {
            r = cmul(r, a);
        }
        return if b.0 < 0.0 { cdiv((1.0, 0.0), r) } else { Ok(r) };
    }
    let l = clog(a)?;
    Ok(cexp(cmul(b, l)))
}

fn real_pow(a: f64, b: f64) -> Result<Value, String> {
    if a == 0.0 && b < 0.0 {
        return Err("0 cannot be raised to a negative power".into());
    }
    if a < 0.0 && b.fract() != 0.0 {
        let c = cpow((a, 0.0), (b, 0.0))?;
        return Ok(cvalue(c));
    }
    let r = a.powf(b);
    if !r.is_finite() && a.is_finite() && b.is_finite() {
        return Err("Numerical result out of range".into());
    }
    Ok(Value::Number(r))
}

fn py_mod(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("modulo by zero".into());
    }
    let r = a % b;
    Ok(if r != 0.0 && (r < 0.0) != (b < 0.0) { r + b } else { r })
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Number(n) => *n != 0.0,
        Value::Bool(b) => *b,
        Value::Complex(a, b) => *a != 0.0 || *b != 0.0,
        Value::List(x) | Value::Tuple(x) | Value::Set(x) => !x.is_empty(),
        Value::Text(s) => !s.is_empty(),
        _ => true,
    }
}

fn arith(op: char, l: &Value, r: &Value) -> Result<Value, String> {
    // Python strings: "ab" + "c" joins, "ab" * 3 repeats (a negative count gives the empty string)
    match (op, l, r) {
        ('+', Value::Text(a), Value::Text(b)) => return Ok(Value::Text(format!("{a}{b}"))),
        ('*', Value::Text(t), Value::Number(k)) | ('*', Value::Number(k), Value::Text(t)) => {
            if k.fract() != 0.0 {
                return Err("can't multiply a sequence by a non-int".into());
            }
            return Ok(Value::Text(t.repeat(if *k < 0.0 { 0 } else { *k as usize })));
        }
        _ => {}
    }
    if let ('+', Some(a), Some(b)) = (op, seq_items(l), seq_items(r)) {
        let mut a = a;
        a.extend(b);
        return Ok(Value::List(a));
    }
    if is_cplx(l) || is_cplx(r) {
        let (a, b) = (cparts(l)?, cparts(r)?);
        return Ok(cvalue(match op {
            '+' => (a.0 + b.0, a.1 + b.1),
            '-' => (a.0 - b.0, a.1 - b.1),
            '*' => cmul(a, b),
            '/' => cdiv(a, b)?,
            'p' => cpow(a, b)?,
            _ => return Err("unsupported operation on complex numbers".into()),
        }));
    }
    let (a, b) = (cparts(l)?.0, cparts(r)?.0);
    Ok(match op {
        '+' => Value::Number(a + b),
        '-' => Value::Number(a - b),
        '*' => Value::Number(a * b),
        '/' => {
            if b == 0.0 {
                return Err("division by zero".into());
            }
            Value::Number(a / b)
        }
        'f' => {
            if b == 0.0 {
                return Err("division by zero".into());
            }
            Value::Number((a / b).floor())
        }
        '%' => Value::Number(py_mod(a, b)?),
        'p' => real_pow(a, b)?,
        _ => unreachable!(),
    })
}

fn compare(op: &str, l: &Value, r: &Value) -> Result<bool, String> {
    if op == "==" || op == "!=" {
        let eq = match (cparts(l), cparts(r)) {
            (Ok(a), Ok(b)) => a == b,
            _ => l.same_item(r),
        };
        return Ok(if op == "==" { eq } else { !eq });
    }
    if let (Value::Text(x), Value::Text(y)) = (l, r) {
        return Ok(match op {
            "<" => x < y,
            ">" => x > y,
            "<=" => x <= y,
            _ => x >= y,
        });
    }
    if is_cplx(l) || is_cplx(r) {
        return Err("complex numbers cannot be ordered".into());
    }
    let (a, b) = (cparts(l)?.0, cparts(r)?.0);
    Ok(match op {
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        _ => a >= b,
    })
}

fn eval(n: &Node, env: &[Option<Value>], cm: bool) -> Result<Value, String> {
    match n {
        Node::Num(v) => Ok(Value::Number(*v)),
        Node::Const(v) => Ok(Value::Number(*v)),
        Node::Bool(b) => Ok(Value::Bool(*b)),
        Node::Var(i) => env[*i].clone().ok_or_else(|| format!("{} has no value", VARS[*i])),
        Node::Neg(a) => match eval(a, env, cm)? {
            Value::Complex(x, y) => Ok(Value::Complex(-x, -y)),
            v => Ok(Value::Number(-cparts(&v)?.0)),
        },
        Node::Pos(a) => eval(a, env, cm),
        Node::Not(a) => Ok(Value::Bool(!truthy(&eval(a, env, cm)?))),
        Node::Bin(op, a, b) => arith(*op, &eval(a, env, cm)?, &eval(b, env, cm)?),
        Node::Cmp(first, rest) => {
            let mut left = eval(first, env, cm)?;
            for (op, rn) in rest {
                let right = eval(rn, env, cm)?;
                if !compare(op, &left, &right)? {
                    return Ok(Value::Bool(false));
                }
                left = right;
            }
            Ok(Value::Bool(true))
        }
        Node::And(v) => {
            let mut last = Value::Bool(true);
            for x in v {
                last = eval(x, env, cm)?;
                if !truthy(&last) {
                    return Ok(last);
                }
            }
            Ok(last)
        }
        Node::Or(v) => {
            let mut last = Value::Bool(false);
            for x in v {
                last = eval(x, env, cm)?;
                if truthy(&last) {
                    return Ok(last);
                }
            }
            Ok(last)
        }
        Node::If(c, a, b) => {
            if truthy(&eval(c, env, cm)?) {
                eval(a, env, cm)
            } else {
                eval(b, env, cm)
            }
        }
        Node::List(v) => Ok(Value::List(v.iter().map(|x| eval(x, env, cm)).collect::<Result<_, _>>()?)),
        Node::Tuple(v) => Ok(Value::Tuple(v.iter().map(|x| eval(x, env, cm)).collect::<Result<_, _>>()?)),
        Node::Index(a, i) => {
            let items = seq_items(&eval(a, env, cm)?).ok_or("only lists can be indexed")?;
            let idx = cparts(&eval(i, env, cm)?)?.0;
            if idx.fract() != 0.0 {
                return Err("list indices must be whole numbers".into());
            }
            let k = if idx < 0.0 { items.len() as f64 + idx } else { idx };
            if k < 0.0 || k as usize >= items.len() {
                return Err("list index out of range".into());
            }
            Ok(items[k as usize].clone())
        }
        Node::Call(name, args) => {
            let vals: Vec<Value> = args.iter().map(|x| eval(x, env, cm)).collect::<Result<_, _>>()?;
            call_function(name, &vals, cm)
        }
    }
}

fn call_function(name: &str, a: &[Value], cm: bool) -> Result<Value, String> {
    match name {
        "min" | "max" => {
            let items: Vec<Value> = if a.len() == 1 { seq_items(&a[0]).ok_or("min/max need several values")? } else { a.to_vec() };
            if items.is_empty() {
                return Err("min/max of nothing".into());
            }
            let mut best = items[0].clone();
            for v in &items[1..] {
                let better = compare(if name == "min" { "<" } else { ">" }, v, &best)?;
                if better {
                    best = v.clone();
                }
            }
            return Ok(best);
        }
        "abs" => {
            if a.len() != 1 {
                return Err("abs takes one value".into());
            }
            return Ok(match &a[0] {
                Value::Complex(x, y) => Value::Number(x.hypot(*y)),
                v => Value::Number(cparts(v)?.0.abs()),
            });
        }
        "floor" | "ceil" => {
            if a.len() != 1 {
                return Err("takes one value".into());
            }
            let x = cparts(&a[0])?.0;
            return Ok(Value::Number(if name == "floor" { x.floor() } else { x.ceil() }));
        }
        _ => {}
    }
    if name == "log" && a.len() == 2 {
        let (x, b) = (cparts(&a[0])?, cparts(&a[1])?);
        if cm || is_cplx(&a[0]) || is_cplx(&a[1]) {
            return Ok(cvalue(cdiv(clog(x)?, clog(b)?)?));
        }
        if x.0 <= 0.0 || b.0 <= 0.0 {
            return Err("math domain error".into());
        }
        let d = b.0.ln();
        if d == 0.0 {
            return Err("division by zero".into());
        }
        return Ok(Value::Number(x.0.ln() / d));
    }
    if a.len() != 1 {
        return Err(format!("{name} takes one value"));
    }
    let z = cparts(&a[0])?;
    if cm || is_cplx(&a[0]) {
        let r: C = match name {
            "sqrt" => csqrt(z),
            "exp" => cexp(z),
            "log" | "ln" => clog(z)?,
            "log10" => {
                let l = clog(z)?;
                (l.0 / std::f64::consts::LN_10, l.1 / std::f64::consts::LN_10)
            }
            "sin" => (z.0.sin() * z.1.cosh(), z.0.cos() * z.1.sinh()),
            "cos" => (z.0.cos() * z.1.cosh(), -z.0.sin() * z.1.sinh()),
            "sinh" => (z.0.sinh() * z.1.cos(), z.0.cosh() * z.1.sin()),
            "cosh" => (z.0.cosh() * z.1.cos(), z.0.sinh() * z.1.sin()),
            "tan" => cdiv((z.0.sin() * z.1.cosh(), z.0.cos() * z.1.sinh()), (z.0.cos() * z.1.cosh(), -z.0.sin() * z.1.sinh()))?,
            "tanh" => cdiv((z.0.sinh() * z.1.cos(), z.0.cosh() * z.1.sin()), (z.0.cosh() * z.1.cos(), z.0.sinh() * z.1.sin()))?,
            _ => return Err(format!("{name} of a complex number is not supported here")),
        };
        return Ok(cvalue(r));
    }
    let x = z.0;
    let domain = |ok: bool| if ok { Ok(()) } else { Err("math domain error".to_string()) };
    let r = match name {
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "asin" => {
            domain(x.abs() <= 1.0)?;
            x.asin()
        }
        "acos" => {
            domain(x.abs() <= 1.0)?;
            x.acos()
        }
        "atan" => x.atan(),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "sqrt" => {
            domain(x >= 0.0)?;
            x.sqrt()
        }
        "exp" => x.exp(),
        "log" | "ln" => {
            domain(x > 0.0)?;
            x.ln()
        }
        "log10" => {
            domain(x > 0.0)?;
            x.log10()
        }
        _ => return Err(format!("unknown function {name}")),
    };
    if !r.is_finite() && x.is_finite() {
        return Err("Numerical result out of range".into());
    }
    Ok(Value::Number(r))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(src: &str, x: f64) -> f64 {
        Callable::parse(src).unwrap().f(&[x]).unwrap()
    }

    #[test]
    fn basics() {
        assert_eq!(f("x**2 + 3*x", 2.0), 10.0);
        assert_eq!(f("2^3", 0.0), 8.0);
        assert!((f("sin(x)*pi", 1.0) - 1.0_f64.sin() * std::f64::consts::PI).abs() < 1e-15);
        assert_eq!(f("-x**2", 3.0), -9.0);
        assert_eq!(f("x if x > 0 else -x", -4.0), 4.0);
        assert!(Callable::parse("x +").is_err());
        assert!(Callable::parse("foo(x)").is_err());
        assert!(Callable::parse("1/x").unwrap().f(&[0.0]).is_err());
    }

    #[test]
    fn several_variables_and_lists() {
        let g = Callable::parse("x*y + z").unwrap();
        assert_eq!(g.f(&[2.0, 3.0, 4.0]).unwrap(), 10.0);
        let noncontig = Callable::parse("x + z").unwrap();
        assert_eq!(noncontig.f(&[2.0, 4.0]).unwrap(), 6.0);
        assert_eq!(Callable::parse("x + y").unwrap().call(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]).unwrap(), Value::Number(3.0));
        let v = Callable::parse("[2*x, 2*y]").unwrap().f(&[1.0, 2.0]);
        assert!(v.is_err()); // a list is not a real number
        let pt = Callable::parse("x[0]**2 + x[1]").unwrap();
        let arg = Value::List(vec![Value::Number(3.0), Value::Number(1.0)]);
        assert_eq!(real(&pt.call(&[arg]).unwrap()).unwrap(), 10.0);
    }
}
