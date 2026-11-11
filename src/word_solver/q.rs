//! Exact fractions, the Rust counterpart of Python's `fractions.Fraction`, in i128 so a value is `Copy` and the solver code can use plain `+ - * /`.
//!
//! A division by zero, or a number too large for i128, is a "decline": it stops the family (like Python's ZeroDivisionError / ValueError, which
//! `word_solver.solve` catches) by unwinding with a `Decline` payload that `solve` catches.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// The payload of the unwind that means "this family declines".
#[derive(Debug)]
pub struct Decline;

/// Stop the current family: the problem is declined.
pub fn decline() -> ! {
    std::panic::panic_any(Decline)
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

impl Q {
    pub fn new(n: i128, d: i128) -> Q {
        if d == 0 {
            decline();
        }
        let g = gcd(n, d).max(1);
        let (mut n, mut d) = (n / g, d / g);
        if d < 0 {
            n = -n;
            d = -d;
        }
        Q { n, d }
    }

    pub fn int(n: i64) -> Q {
        Q { n: n as i128, d: 1 }
    }

    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub const ONE: Q = Q { n: 1, d: 1 };

    pub fn numer(self) -> i128 {
        self.n
    }

    pub fn denom(self) -> i128 {
        self.d
    }

    pub fn is_int(self) -> bool {
        self.d == 1
    }

    pub fn is_zero(self) -> bool {
        self.n == 0
    }

    pub fn abs(self) -> Q {
        Q { n: self.n.abs(), d: self.d }
    }

    pub fn floor(self) -> i128 {
        self.n.div_euclid(self.d)
    }

    pub fn ceil(self) -> i128 {
        -((-self.n).div_euclid(self.d))
    }

    /// Python's `round()` (banker's rounding) of the fraction, as Python does for `round(Fraction)`.
    pub fn round(self) -> i128 {
        let f = self.floor();
        let rem = self - Q::new(f, 1);
        let half = Q::new(1, 2);
        match rem.cmp(&half) {
            Ordering::Less => f,
            Ordering::Greater => f + 1,
            Ordering::Equal => {
                if f % 2 == 0 {
                    f
                } else {
                    f + 1
                }
            }
        }
    }

    pub fn to_f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }

    pub fn pow(self, k: u32) -> Q {
        let mut r = Q::ONE;
        for _ in 0..k {
            r = r * self;
        }
        r
    }

    /// `Fraction("7.50")`, `Fraction("1/2")`, `Fraction("-3")`.  None for anything else.
    pub fn parse(text: &str) -> Option<Q> {
        let t = text.trim();
        if let Some((a, b)) = t.split_once('/') {
            return Some(Q::new(a.trim().parse::<i128>().ok()?, b.trim().parse::<i128>().ok()?));
        }
        let (neg, body) = match t.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, t.strip_prefix('+').unwrap_or(t)),
        };
        let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
        if whole.is_empty() && frac.is_empty() {
            return None;
        }
        let w: i128 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
        let (f, scale): (i128, i128) = if frac.is_empty() {
            (0, 1)
        } else {
            (frac.parse().ok()?, 10i128.checked_pow(frac.len() as u32)?)
        };
        let n = w.checked_mul(scale)?.checked_add(f)?;
        Some(Q::new(if neg { -n } else { n }, scale))
    }

    /// The exact rational square root when the fraction is the square of a rational, else None.
    pub fn exact_sqrt(self) -> Option<Q> {
        if self.n < 0 {
            return None;
        }
        let rn = isqrt(self.n)?;
        let rd = isqrt(self.d)?;
        Some(Q::new(rn, rd))
    }
}

/// The integer square root of n when n is a perfect square.
pub fn isqrt(n: i128) -> Option<i128> {
    if n < 0 {
        return None;
    }
    let mut r = (n as f64).sqrt() as i128;
    while r * r > n {
        r -= 1;
    }
    while (r + 1) * (r + 1) <= n {
        r += 1;
    }
    if r * r == n {
        Some(r)
    } else {
        None
    }
}

fn mul128(a: i128, b: i128) -> i128 {
    a.checked_mul(b).unwrap_or_else(|| decline())
}

fn add128(a: i128, b: i128) -> i128 {
    a.checked_add(b).unwrap_or_else(|| decline())
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(add128(mul128(self.n, o.d), mul128(o.n, self.d)), mul128(self.d, o.d))
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(add128(mul128(self.n, o.d), -mul128(o.n, self.d)), mul128(self.d, o.d))
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(mul128(self.n, o.n), mul128(self.d, o.d))
    }
}

impl Div for Q {
    type Output = Q;
    fn div(self, o: Q) -> Q {
        if o.n == 0 {
            decline();
        }
        Q::new(mul128(self.n, o.d), mul128(self.d, o.n))
    }
}

impl Neg for Q {
    type Output = Q;
    fn neg(self) -> Q {
        Q { n: -self.n, d: self.d }
    }
}

impl From<i64> for Q {
    fn from(n: i64) -> Q {
        Q::int(n)
    }
}

impl From<i32> for Q {
    fn from(n: i32) -> Q {
        Q::int(n as i64)
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        match (self.n.checked_mul(o.d), o.n.checked_mul(self.d)) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => self.to_f64().partial_cmp(&o.to_f64()).unwrap_or(Ordering::Equal),
        }
    }
}

impl fmt::Debug for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Q {
    /// `3` or `7/2`: the exact text (Python `show_exact`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.d == 1 {
            write!(f, "{}", self.n)
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}
