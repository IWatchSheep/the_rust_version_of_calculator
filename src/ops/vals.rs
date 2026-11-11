//! Helpers shared by the operators that take typed values (lists, matrices, sets, graphs, functions).
//! They mirror what the Python operators rely on: `operators_one_operator_per_file/_helpers.py`, list indexing,
//! `statistics`, and Python's habit of raising on 1/0, empty input and out-of-range indexes.

use crate::expr::Callable;
use crate::value::Value;

pub type R = Result<Value, String>;
pub type Mat = Vec<Vec<f64>>;

pub fn n(x: f64) -> R {
    Ok(Value::Number(x))
}

pub fn b(x: bool) -> R {
    Ok(Value::Bool(x))
}

pub fn need(cond: bool, msg: &str) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(msg.into())
    }
}

/// x / y, an error when y is 0 (Python's ZeroDivisionError).
pub fn div(x: f64, y: f64) -> Result<f64, String> {
    if y == 0.0 {
        Err("division by zero".into())
    } else {
        Ok(x / y)
    }
}

/// Python's `a ** b` for real numbers: an error for 0 ** negative, and for a negative base with a fractional
/// exponent (Python gives a complex number there; these operators are real-valued).
pub fn pypow(a: f64, e: f64) -> Result<f64, String> {
    if a == 0.0 && e < 0.0 {
        return Err("0 cannot be raised to a negative power".into());
    }
    if a < 0.0 && e.fract() != 0.0 {
        return Err("a negative number to a fractional power is complex".into());
    }
    let r = a.powf(e);
    if !r.is_finite() && a.is_finite() && e.is_finite() {
        return Err("Numerical result out of range".into());
    }
    Ok(r)
}

/// A number out of a value (booleans count as 0 / 1, like Python).
pub fn num(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(x) => Ok(*x),
        Value::Bool(t) => Ok(*t as i32 as f64),
        other => Err(format!("expected a number, got {}", other.kind_name())),
    }
}

pub fn items(v: &Value) -> Result<&[Value], String> {
    match v {
        Value::List(a) | Value::Tuple(a) | Value::Set(a) => Ok(a),
        other => Err(format!("expected a list, got {}", other.kind_name())),
    }
}

/// A list of numbers.
pub fn nums(v: &Value) -> Result<Vec<f64>, String> {
    items(v)?.iter().map(num).collect()
}

/// A matrix (list of rows of numbers).
pub fn mat(v: &Value) -> Result<Mat, String> {
    items(v)?.iter().map(nums).collect()
}

pub fn vlist(xs: Vec<f64>) -> Value {
    Value::List(xs.into_iter().map(Value::Number).collect())
}

pub fn vmat(m: Mat) -> Value {
    Value::List(m.into_iter().map(vlist).collect())
}

pub fn vtuple(xs: Vec<f64>) -> Value {
    Value::Tuple(xs.into_iter().map(Value::Number).collect())
}

pub fn func(v: &Value) -> Result<Callable, String> {
    match v {
        Value::Func(s) => Callable::parse(s),
        other => Err(format!("expected a function, got {}", other.kind_name())),
    }
}

/// Python list indexing: negative counts from the end, out of range is an error.
pub fn at<T: Clone>(xs: &[T], i: i64) -> Result<T, String> {
    let k = if i < 0 { i + xs.len() as i64 } else { i };
    if k < 0 || k as usize >= xs.len() {
        return Err("list index out of range".into());
    }
    Ok(xs[k as usize].clone())
}

/// `int(x)` for a number that should be a whole index: truncates toward zero like Python's int().
pub fn int_of(x: f64) -> Result<i64, String> {
    if !x.is_finite() {
        return Err("cannot convert to an integer".into());
    }
    Ok(x.trunc() as i64)
}

/// An index that must already be a whole number (a float index is a TypeError in Python).
pub fn whole(x: f64) -> Result<i64, String> {
    if x.fract() != 0.0 || !x.is_finite() {
        return Err("a whole number is needed here".into());
    }
    Ok(x as i64)
}

pub fn factorial(k: usize) -> Result<f64, String> {
    if k > 170 {
        return Err("result is too large".into());
    }
    Ok((2..=k).fold(1.0, |acc, i| acc * i as f64))
}

// ---------------------------------------------------------------- matrices (the `_helpers.py` ones)
pub fn transpose_m(a: &Mat) -> Mat {
    if a.is_empty() {
        return vec![];
    }
    let cols = a.iter().map(|r| r.len()).min().unwrap_or(0);
    (0..cols).map(|j| a.iter().map(|r| r[j]).collect()).collect()
}

pub fn matmul(a: &Mat, bm: &Mat) -> Mat {
    let cols = transpose_m(bm);
    a.iter().map(|row| cols.iter().map(|col| row.iter().zip(col).map(|(x, y)| x * y).sum()).collect()).collect()
}

/// Reduced row echelon form and the pivot columns (`_helpers.rref`).
pub fn rref(a: &Mat) -> Result<(Mat, Vec<usize>), String> {
    rref_tol(a, 1e-10)
}

pub fn rref_tol(a: &Mat, tol: f64) -> Result<(Mat, Vec<usize>), String> {
    let mut m = a.clone();
    need(!m.is_empty(), "list index out of range")?;
    let (rows, cols) = (m.len(), m[0].len());
    let mut pivots = vec![];
    let mut r = 0;
    for c in 0..cols {
        if r >= rows {
            break;
        }
        // max(range(r, rows), key=abs(M[i][c])): the first of equal maxima
        let mut p = r;
        for i in r..rows {
            if m[i][c].abs() > m[p][c].abs() {
                p = i;
            }
        }
        if m[p][c].abs() < tol {
            continue;
        }
        m.swap(r, p);
        let d = m[r][c];
        m[r] = m[r].iter().map(|v| v / d).collect();
        for i in 0..rows {
            if i != r && m[i][c].abs() > 0.0 {
                let f = m[i][c];
                let pr = m[r].clone();
                m[i] = m[i].iter().zip(&pr).map(|(x, y)| x - f * y).collect();
            }
        }
        pivots.push(c);
        r += 1;
    }
    Ok((m, pivots))
}

pub fn nullspace(a: &Mat) -> Result<Mat, String> {
    let (r, piv) = rref(a)?;
    let cols = r[0].len();
    let mut basis = vec![];
    for f in (0..cols).filter(|c| !piv.contains(c)) {
        let mut v = vec![0.0; cols];
        v[f] = 1.0;
        for (row, p) in piv.iter().enumerate() {
            v[*p] = -r[row][f];
        }
        basis.push(v);
    }
    Ok(basis)
}

/// Eigenvalues of a symmetric matrix by Jacobi rotations, largest first (`_helpers.jacobi_eigen`).
pub fn jacobi_eigen(s: &Mat, sweeps: usize) -> Vec<f64> {
    let mut a = s.clone();
    let nn = a.len();
    for _ in 0..sweeps {
        let mut off = 0.0;
        for i in 0..nn {
            for j in 0..nn {
                if i != j {
                    off += a[i][j].powi(2);
                }
            }
        }
        if off < 1e-22 {
            break;
        }
        for p in 0..nn.saturating_sub(1) {
            for q in p + 1..nn {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let th = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = (if th >= 0.0 { 1.0 } else { -1.0 }) / (th.abs() + (th * th + 1.0).sqrt());
                let c = 1.0 / (t * t + 1.0).sqrt();
                let sn = t * c;
                for k in 0..nn {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - sn * akq;
                    a[k][q] = sn * akp + c * akq;
                }
                for k in 0..nn {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - sn * aqk;
                    a[q][k] = sn * apk + c * aqk;
                }
            }
        }
    }
    let mut ev: Vec<f64> = (0..nn).map(|i| a[i][i]).collect();
    ev.sort_by(|x, y| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    ev
}

// ---------------------------------------------------------------- statistics (what `statistics.*` does)
pub fn st_mean(d: &[f64]) -> Result<f64, String> {
    need(!d.is_empty(), "mean requires at least one data point")?;
    Ok(d.iter().sum::<f64>() / d.len() as f64)
}

/// Sum of squared deviations from the mean.
pub fn st_ss(d: &[f64]) -> Result<f64, String> {
    let m = st_mean(d)?;
    Ok(d.iter().map(|x| (x - m) * (x - m)).sum())
}

/// `statistics.quantiles(data, n=n)` with the default (exclusive) method.
pub fn st_quantiles(data: &[f64], nn: usize) -> Result<Vec<f64>, String> {
    let mut d = data.to_vec();
    d.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ld = d.len() as i64;
    // Python 3.13+: one data point gives that value repeated; none is an error (older versions needed two)
    if ld == 1 {
        return Ok(vec![d[0]; nn - 1]);
    }
    need(ld >= 2, "must have at least one data point")?;
    let m = ld + 1;
    let n_i = nn as i64;
    let mut out = vec![];
    for i in 1..n_i {
        let mut j = i * m / n_i;
        if j < 1 {
            j = 1;
        } else if j > ld - 1 {
            j = ld - 1;
        }
        let delta = i * m - j * n_i;
        out.push((d[(j - 1) as usize] * (n_i - delta) as f64 + d[j as usize] * delta as f64) / n_i as f64);
    }
    Ok(out)
}

/// Slope and intercept of the least-squares line (`statistics.linear_regression`).
pub fn st_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64), String> {
    need(x.len() == y.len(), "linear regression requires that both inputs have same number of data points")?;
    need(x.len() >= 2, "linear regression requires at least two data points")?;
    let (xb, yb) = (st_mean(x)?, st_mean(y)?);
    let sxy: f64 = x.iter().zip(y).map(|(a, c)| (a - xb) * (c - yb)).sum();
    let sxx: f64 = x.iter().map(|a| (a - xb) * (a - xb)).sum();
    need(sxx != 0.0, "x is constant")?;
    let slope = sxy / sxx;
    Ok((slope, yb - slope * xb))
}
