//! Math operators ported from `operators_one_operator_per_file/math/*` that are not in `mod.rs`.
//! Same conventions as `physics.rs`: one function per operator named after its folder, box values first,
//! then the optional constants with their defaults filled in; a NaN / infinite result becomes an error in
//! `Operator::run`.
//!
//! Layer 1 (`Value::Bool` / `Value::Complex`) is ported here. Still waiting on List / Matrix:
//! `prime_factorization`, `identity_matrix`, and `characteristic_equation` (two complex roots).
//!
//! Integers: Python uses unbounded integers; here counting results are exact up to 2^53 (an error is returned
//! beyond what u128 can hold, and the result is shown as a decimal number).

use crate::value::Value;
use std::f64::consts::PI;

type R = Result<Value, String>;

fn n(x: f64) -> R {
    Ok(Value::Number(x))
}

fn b(x: bool) -> R {
    Ok(Value::Bool(x))
}

fn c(re: f64, im: f64) -> R {
    Ok(Value::Complex(re, im))
}

/// Python `bool(x)` on a number: nonzero is true.
fn truthy(x: f64) -> bool {
    x != 0.0
}

/// a / b, but an error (like Python's ZeroDivisionError) when b is 0.
fn d(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("division by zero".into())
    } else {
        Ok(a / b)
    }
}

/// n! as a number: exact up to 18!, rounded beyond that, an error past 170! (it no longer fits a number).
fn factorial_f64(k: u64) -> Result<f64, String> {
    need(k <= 170, "result is too large")?;
    Ok((2..=k).fold(1.0, |acc, i| acc * i as f64))
}

fn need(cond: bool, msg: &str) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(msg.into())
    }
}

/// A whole number >= 0 (Python's math.factorial / comb / range reject anything else).
fn nat(x: f64, what: &str) -> Result<u64, String> {
    if x.is_finite() && x >= 0.0 && x.fract() == 0.0 && x <= 1e15 {
        Ok(x as u64)
    } else {
        Err(format!("{what} must be a whole number ≥ 0"))
    }
}

fn too_big<T>(o: Option<T>) -> Result<T, String> {
    o.ok_or_else(|| "result is too large".to_string())
}

/// C(n, k) exactly; 0 when k > n.
fn comb(nn: u64, k: u64) -> Result<u128, String> {
    if k > nn {
        return Ok(0);
    }
    let k = k.min(nn - k);
    let mut r: u128 = 1;
    for i in 0..k as u128 {
        r = too_big(r.checked_mul(nn as u128 - i))? / (i + 1);
    }
    Ok(r)
}

/// Python's `%`: the result takes the sign of the divisor.
fn py_mod(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r != 0.0 && (r < 0.0) != (b < 0.0) {
        r + b
    } else {
        r
    }
}

fn inf_text() -> R {
    Ok(Value::Text("inf".into()))
}

// ---------------------------------------------------------------- error function (for the p-value)
fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    if x < 2.5 {
        // Taylor series of erf, accurate to ~1e-16 absolute here
        let mut term = x;
        let mut sum = x;
        let mut k = 0.0;
        loop {
            k += 1.0;
            term *= -x * x / k;
            let add = term / (2.0 * k + 1.0);
            sum += add;
            if add.abs() < 1e-18 {
                break;
            }
        }
        1.0 - 2.0 / PI.sqrt() * sum
    } else {
        // continued fraction, evaluated from the tail
        let mut f = x;
        for i in (1..=80).rev() {
            f = x + (i as f64 / 2.0) / f;
        }
        (-x * x).exp() / (PI.sqrt() * f)
    }
}

// ---------------------------------------------------------------- abstract algebra
pub fn alternating_group_order(a: &[f64]) -> R {
    let k = nat(a[0], "n")?;
    n((factorial_f64(k)? / 2.0).floor()) // Python: n! // 2
}
pub fn quotient_group_order(a: &[f64]) -> R {
    n((a[0] / a[1]).floor())
}
pub fn symmetric_group_order(a: &[f64]) -> R {
    let k = nat(a[0], "n")?;
    n(factorial_f64(k)?)
}

// ---------------------------------------------------------------- algebra, calculus
pub fn binomial_theorem(a: &[f64]) -> R {
    let k = nat(a[2], "n")?;
    let mut sum = 0.0;
    for j in 0..=k {
        sum += comb(k, j)? as f64 * a[0].powi((k - j) as i32) * a[1].powi(j as i32);
    }
    n(sum)
}
pub fn chain_rule(a: &[f64]) -> R {
    n(a[0] * a[1])
}
pub fn product_rule(a: &[f64]) -> R {
    // f, fprime, g, gprime
    n(a[1] * a[2] + a[0] * a[3])
}
pub fn quotient_rule(a: &[f64]) -> R {
    n((a[1] * a[2] - a[0] * a[3]) / a[2].powi(2))
}
pub fn quaternions(a: &[f64]) -> R {
    n((a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3]).sqrt())
}
pub fn rational_numbers(a: &[f64]) -> R {
    need(a[1] != 0.0, "q can't be 0")?;
    n(a[0] / a[1])
}
pub fn mobius_transformation(a: &[f64]) -> R {
    // a, b, c, d, z
    n((a[0] * a[4] + a[1]) / (a[2] * a[4] + a[3]))
}
pub fn proportional_to(a: &[f64]) -> R {
    n(a[1] / a[0])
}
pub fn gaussian_curvature(a: &[f64]) -> R {
    n(a[0] * a[1])
}

// ---------------------------------------------------------------- number theory, counting
pub fn euler_s_totient(a: &[f64]) -> R {
    let mut m = nat(a[0], "n")?;
    let mut result = m;
    let mut p = 2;
    while p * p <= m {
        if m % p == 0 {
            while m % p == 0 {
                m /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }
    n(result as f64)
}
pub fn inclusion_exclusion(a: &[f64]) -> R {
    n(a[0] + a[1] - a[2])
}
pub fn modular_arithmetic(a: &[f64]) -> R {
    n(py_mod(a[0], a[1]))
}
pub fn pigeonhole_principle(a: &[f64]) -> R {
    n((a[0] / a[1]).ceil())
}
pub fn bell_number(a: &[f64]) -> R {
    // Bell triangle (exact integers)
    let k = nat(a[0], "n")? as usize;
    let mut row: Vec<u128> = vec![1];
    for _ in 0..k {
        let mut next = vec![*row.last().unwrap()];
        for v in &row {
            let last = *next.last().unwrap();
            next.push(too_big(last.checked_add(*v))?);
        }
        row = next;
    }
    n(row[0] as f64)
}
pub fn catalan_number(a: &[f64]) -> R {
    let k = nat(a[0], "n")?;
    n((comb(2 * k, k)? / (k as u128 + 1)) as f64)
}
pub fn partition_number(a: &[f64]) -> R {
    let k = nat(a[0], "n")? as usize;
    let mut p = vec![0u128; k + 1];
    p[0] = 1;
    for part in 1..=k {
        for i in part..=k {
            p[i] = too_big(p[i].checked_add(p[i - part]))?;
        }
    }
    n(p[k] as f64)
}
pub fn stirling_number_second_kind(a: &[f64]) -> R {
    // S(n, k) by the recurrence S(n,k) = k S(n-1,k) + S(n-1,k-1)
    let (nn, k) = (nat(a[0], "n")? as usize, nat(a[1], "k")? as usize);
    let mut s = vec![vec![0u128; k + 2]; nn + 1];
    s[0][0] = 1;
    for i in 1..=nn {
        // only the entries that S(n, k) depends on: S(i, j) with j >= k - (n - i); the others are never used (and can be far too big for nothing: S(100, 100) is 1)
        for j in (k + i).saturating_sub(nn).max(1)..=k.min(i) {
            s[i][j] = too_big((j as u128).checked_mul(s[i - 1][j]).and_then(|v| v.checked_add(s[i - 1][j - 1])))?;
        }
    }
    n(s[nn][k] as f64)
}
pub fn euler_characteristic(a: &[f64]) -> R {
    n(a[0] - a[1] + a[2])
}

// ---------------------------------------------------------------- analysis
pub fn radius_of_convergence(a: &[f64]) -> R {
    if a[0] == 0.0 {
        return inf_text();
    }
    n(1.0 / a[0])
}
pub fn ratio_test(a: &[f64]) -> R {
    n((a[1] / a[0]).abs())
}
pub fn root_test(a: &[f64]) -> R {
    n(a[0].abs().powf(d(1.0, a[1])?))
}
pub fn dirac_delta(a: &[f64]) -> R {
    if a[0] == 0.0 {
        return inf_text();
    }
    n(0.0)
}
pub fn heaviside_step(a: &[f64]) -> R {
    n(if a[0] > 0.0 {
        1.0
    } else if a[0] < 0.0 {
        0.0
    } else {
        0.5
    })
}
pub fn wronskian(a: &[f64]) -> R {
    // f, fprime, g, gprime
    n(a[0] * a[3] - a[2] * a[1])
}

// ---------------------------------------------------------------- statistics and probability
pub fn bayes_theorem(a: &[f64]) -> R {
    n(a[0] * a[1] / a[2])
}
pub fn standard_deviation_from_variance(a: &[f64]) -> R {
    need(a[0] >= 0.0, "math domain error")?;
    n(a[0].sqrt())
}
pub fn sample_standard_deviation_from_variance(a: &[f64]) -> R {
    need(a[0] >= 0.0, "math domain error")?;
    n(a[0].sqrt())
}
pub fn range_from_ends(a: &[f64]) -> R {
    n(a[0] - a[1])
}
pub fn interquartile_range_from_quartiles(a: &[f64]) -> R {
    n(a[0] - a[1])
}
pub fn variance_from_expectations(a: &[f64]) -> R {
    n(a[0] - a[1].powi(2))
}
pub fn binomial_distribution(a: &[f64]) -> R {
    let (nn, k) = (nat(a[0], "n")?, nat(a[1], "k")?);
    // k above n: Python's (1 - p) ** (n - k) has a NEGATIVE exponent, which is a division by zero when p is 1
    let (q, e) = (1.0 - a[2], nn as i64 - k as i64);
    if q == 0.0 && e < 0 {
        return Err("division by zero".into());
    }
    n(comb(nn, k)? as f64 * a[2].powi(k as i32) * q.powi(e as i32))
}
pub fn conditional_probability(a: &[f64]) -> R {
    n(a[0] / a[1])
}
pub fn confidence_interval(a: &[f64]) -> R {
    // mean, sd, n, z
    let half = a[3] * a[1] / a[2].sqrt();
    Ok(Value::Pair(a[0] - half, a[0] + half))
}
pub fn exponential_distribution(a: &[f64]) -> R {
    n(if a[0] >= 0.0 { a[1] * (-a[1] * a[0]).exp() } else { 0.0 })
}
pub fn normal_distribution(a: &[f64]) -> R {
    // x, mu, sigma
    n((-((a[0] - a[1]).powi(2)) / (2.0 * a[2].powi(2))).exp() / (a[2] * (2.0 * PI).sqrt()))
}
pub fn p_value(a: &[f64]) -> R {
    // 2 (1 - Phi(|z|)) = erfc(|z| / sqrt 2)
    n(erfc(a[0].abs() / 2.0_f64.sqrt()))
}
pub fn poisson_distribution(a: &[f64]) -> R {
    let k = nat(a[0], "k")?;
    n(a[1].powi(k as i32) * (-a[1]).exp() / factorial_f64(k)?)
}
pub fn probability(a: &[f64]) -> R {
    n(a[0] / a[1])
}
pub fn t_test_statistic(a: &[f64]) -> R {
    n((a[0] - a[1]) / d(a[2], a[3].sqrt())?)
}
pub fn uniform_distribution(a: &[f64]) -> R {
    // x, a, b
    n(if a[1] <= a[0] && a[0] <= a[2] { 1.0 / (a[2] - a[1]) } else { 0.0 })
}
pub fn z_score(a: &[f64]) -> R {
    n((a[0] - a[1]) / a[2])
}

// ---------------------------------------------------------------- trigonometry
pub fn angle_sum(a: &[f64]) -> R {
    n(a[0].sin() * a[1].cos() + a[0].cos() * a[1].sin())
}
pub fn half_angle(a: &[f64]) -> R {
    n(((1.0 - a[0].cos()) / 2.0).sqrt())
}
pub fn inverse_hyperbolic_cosine(a: &[f64]) -> R {
    need(a[0] >= 1.0, "acosh needs x ≥ 1")?;
    n(a[0].acosh())
}
pub fn inverse_hyperbolic_sine(a: &[f64]) -> R {
    n(a[0].asinh())
}
pub fn inverse_hyperbolic_tangent(a: &[f64]) -> R {
    need(a[0].abs() < 1.0, "atanh needs |x| < 1")?;
    n(a[0].atanh())
}

// ---------------------------------------------------------------- comparisons / relations (bool)
pub fn equals(a: &[f64]) -> R {
    b(a[0] == a[1])
}
pub fn not_equal(a: &[f64]) -> R {
    b(a[0] != a[1])
}
pub fn greater_than(a: &[f64]) -> R {
    b(a[0] > a[1])
}
pub fn greater_than_or_equal(a: &[f64]) -> R {
    b(a[0] >= a[1])
}
pub fn less_than(a: &[f64]) -> R {
    b(a[0] < a[1])
}
pub fn less_than_or_equal(a: &[f64]) -> R {
    b(a[0] <= a[1])
}
pub fn approximately_equal(a: &[f64]) -> R {
    // a, b, rel_tol (default filled by Operator::extra)
    b(math_isclose(a[0], a[1], a[2]))
}

/// Match Python `math.isclose(a, b, rel_tol=...)` (default abs_tol=0.0).
fn math_isclose(a: f64, b: f64, rel_tol: f64) -> bool {
    if a == b {
        return true;
    }
    (a - b).abs() <= rel_tol * a.abs().max(b.abs())
}

// ---------------------------------------------------------------- logic (bool; Python uses truthiness of numbers)
pub fn and(a: &[f64]) -> R {
    b(truthy(a[0]) && truthy(a[1]))
}
pub fn or(a: &[f64]) -> R {
    b(truthy(a[0]) || truthy(a[1]))
}
pub fn not(a: &[f64]) -> R {
    b(!truthy(a[0]))
}
pub fn xor(a: &[f64]) -> R {
    b(truthy(a[0]) != truthy(a[1]))
}
pub fn implies(a: &[f64]) -> R {
    // Python: `(not a) or bool(b)` — `not` on a float is True only for 0.0
    b(!truthy(a[0]) || truthy(a[1]))
}
pub fn if_and_only_if(a: &[f64]) -> R {
    b(truthy(a[0]) == truthy(a[1]))
}

// ---------------------------------------------------------------- predicates
pub fn prime_check(a: &[f64]) -> R {
    let n = a[0];
    if n < 2.0 || n.fract() != 0.0 {
        return b(false);
    }
    let n = n as i64;
    let mut i = 2i64;
    while i * i <= n {
        if n % i == 0 {
            return b(false);
        }
        i += 1;
    }
    b(true)
}
pub fn field(a: &[f64]) -> R {
    // Z/nZ is a field iff n is prime
    prime_check(a)
}
pub fn integral_domain(a: &[f64]) -> R {
    prime_check(a)
}
pub fn integers(a: &[f64]) -> R {
    b(a[0].is_finite() && a[0].fract() == 0.0)
}
pub fn natural_numbers(a: &[f64]) -> R {
    b(a[0].is_finite() && a[0].fract() == 0.0 && a[0] >= 0.0)
}
pub fn real_numbers(a: &[f64]) -> R {
    let _ = a;
    b(true)
}

// ---------------------------------------------------------------- complex (single complex result, or |z| / Re / Im / Arg as numbers)
pub fn complex_number(a: &[f64]) -> R {
    c(a[0], a[1])
}
pub fn complex_numbers(a: &[f64]) -> R {
    c(a[0], a[1])
}
pub fn complex_conjugate(a: &[f64]) -> R {
    // one box: real z → conjugate is itself (im = 0)
    c(a[0], 0.0)
}
pub fn modulus_of_complex_number(a: &[f64]) -> R {
    n(a[0].abs())
}
pub fn argument(a: &[f64]) -> R {
    // phase of a real number: atan2(0, re)
    n(0.0f64.atan2(a[0]))
}
pub fn real_part(a: &[f64]) -> R {
    n(a[0])
}
pub fn imaginary_part(a: &[f64]) -> R {
    let _ = a;
    n(0.0)
}
pub fn euler_s_formula(a: &[f64]) -> R {
    let theta = a[0];
    c(theta.cos(), theta.sin())
}
pub fn de_moivre_s_theorem(a: &[f64]) -> R {
    let (r, theta, n) = (a[0], a[1], a[2]);
    let rn = r.powf(n);
    let ang = n * theta;
    c(rn * ang.cos(), rn * ang.sin())
}
