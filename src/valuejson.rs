//! A tiny JSON reader for the parity files.  Plain JSON plus tags for the Python types JSON lacks:
//! `{"t": [...]}` tuple, `{"s": [...]}` set, `{"d": [[k, v], ...]}` dict, `{"c": [re, im]}` complex,
//! `{"f": "inf" | "-inf" | "nan"}` a special float.  Arrays are lists, strings are text.

use crate::value::Value;

struct J<'a> {
    s: &'a [u8],
    i: usize,
}

pub fn parse_json_value(text: &str) -> Result<Value, String> {
    let mut j = J { s: text.as_bytes(), i: 0 };
    let v = j.value()?;
    j.ws();
    if j.i != j.s.len() {
        return Err("trailing text after the value".into());
    }
    Ok(v)
}

impl J<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] as char).is_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(format!("expected '{}' at {}", c as char, self.i))
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.s.get(self.i).ok_or("unterminated string")?;
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|_| "bad utf-8".to_string()),
                b'\\' => {
                    let e = *self.s.get(self.i).ok_or("bad escape")?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let h = std::str::from_utf8(&self.s[self.i..self.i + 4]).map_err(|_| "bad \\u")?;
                            self.i += 4;
                            let ch = char::from_u32(u32::from_str_radix(h, 16).map_err(|_| "bad \\u")?).unwrap_or('?');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }
    fn array(&mut self) -> Result<Vec<Value>, String> {
        self.expect(b'[')?;
        let mut v = vec![];
        if self.eat(b']') {
            return Ok(v);
        }
        loop {
            v.push(self.value()?);
            if self.eat(b']') {
                return Ok(v);
            }
            self.expect(b',')?;
        }
    }
    fn value(&mut self) -> Result<Value, String> {
        self.ws();
        match self.s.get(self.i).copied() {
            Some(b'[') => Ok(Value::List(self.array()?)),
            Some(b'"') => Ok(Value::Text(self.string()?)),
            Some(b'{') => {
                self.i += 1;
                let tag = self.string()?;
                self.expect(b':')?;
                let v = match tag.as_str() {
                    "t" => Value::Tuple(self.array()?),
                    "s" => Value::Set(self.array()?),
                    "d" => {
                        let mut pairs = vec![];
                        for p in self.array()? {
                            match p {
                                Value::List(kv) if kv.len() == 2 => pairs.push((kv[0].clone(), kv[1].clone())),
                                _ => return Err("bad dict entry".into()),
                            }
                        }
                        Value::Dict(pairs)
                    }
                    "c" => match self.array()?.as_slice() {
                        [Value::Number(a), Value::Number(b)] => Value::Complex(*a, *b),
                        _ => return Err("bad complex".into()),
                    },
                    "f" => match self.string()?.as_str() {
                        "inf" => Value::Number(f64::INFINITY),
                        "-inf" => Value::Number(f64::NEG_INFINITY),
                        _ => Value::Number(f64::NAN),
                    },
                    other => return Err(format!("unknown tag {other}")),
                };
                self.expect(b'}')?;
                Ok(v)
            }
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Value::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Value::Bool(false))
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let st = self.i;
                while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[st..self.i]).unwrap();
                t.parse::<f64>().map(Value::Number).map_err(|_| format!("bad number {t}"))
            }
            _ => Err(format!("unexpected character at {}", self.i)),
        }
    }
}

/// Are two values the same answer?  Numbers within 1e-9 relative (plus 1e-12 absolute), sequences item by item
/// (a list, a tuple and a pair of numbers all count as sequences), sets and dicts without regard to order.
pub fn values_match(got: &Value, want: &Value) -> bool {
    use Value::*;
    fn seq(v: &Value) -> Option<Vec<Value>> {
        match v {
            List(a) | Tuple(a) => Some(a.clone()),
            Pair(a, b) => Some(vec![Number(*a), Number(*b)]),
            _ => None,
        }
    }
    fn close(a: f64, b: f64) -> bool {
        if a == b || (a.is_nan() && b.is_nan()) {
            return true;
        }
        (a - b).abs() <= 1e-9 * b.abs() + 1e-12
    }
    match (got, want) {
        (Number(a), Number(b)) => close(*a, *b),
        (Bool(a), Bool(b)) => a == b,
        (Text(a), Text(b)) | (Func(a), Func(b)) => a == b,
        (Complex(a, b), Complex(c, d)) => (a - c).hypot(b - d) <= 1e-9 * c.hypot(*d) + 1e-12,
        (Number(a), Complex(c, d)) | (Complex(c, d), Number(a)) => close(*a, *c) && d.abs() <= 1e-12,
        (Set(a), Set(b)) => {
            a.len() == b.len() && {
                let mut used = vec![false; b.len()];
                a.iter().all(|x| match (0..b.len()).find(|k| !used[*k] && values_match(x, &b[*k])) {
                    Some(k) => {
                        used[k] = true;
                        true
                    }
                    None => false,
                })
            }
        }
        (Dict(a), Dict(b)) => {
            a.len() == b.len() && a.iter().all(|(k, v)| b.iter().any(|(k2, v2)| values_match(k, k2) && values_match(v, v2)))
        }
        _ => match (seq(got), seq(want)) {
            (Some(a), Some(b)) => a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| values_match(x, y)),
            _ => false,
        },
    }
}
