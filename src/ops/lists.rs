//! Operators on lists / vectors of numbers (statistics, vector algebra, series, interpolation, transforms).
//! One function per operator, named after its folder; `a` holds the box values, then the optional constants.

use super::vals::*;
use crate::value::Value;
use std::f64::consts::PI;

// ---------------------------------------------------------------- vectors
pub fn tensor_product(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    Ok(vlist(x.iter().flat_map(|p| y.iter().map(move |q| p * q)).collect()))
}

pub fn wedge_product(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    let mut out = vec![];
    for i in 0..x.len() {
        for j in i + 1..x.len() {
            out.push(x[i] * at(&y, j as i64)? - x[j] * at(&y, i as i64)?);
        }
    }
    Ok(vlist(out))
}

pub fn cross_product(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    let g = |v: &Vec<f64>, i: i64| at(v, i);
    Ok(vlist(vec![
        g(&x, 1)? * g(&y, 2)? - g(&x, 2)? * g(&y, 1)?,
        g(&x, 2)? * g(&y, 0)? - g(&x, 0)? * g(&y, 2)?,
        g(&x, 0)? * g(&y, 1)? - g(&x, 1)? * g(&y, 0)?,
    ]))
}

fn dot(x: &[f64], y: &[f64]) -> f64 {
    x.iter().zip(y).map(|(p, q)| p * q).sum()
}

pub fn dot_product(a: &[Value]) -> R {
    n(dot(&nums(&a[0])?, &nums(&a[1])?))
}

pub fn inner_product(a: &[Value]) -> R {
    n(dot(&nums(&a[0])?, &nums(&a[1])?))
}

fn hypot(v: &[f64]) -> f64 {
    let m = v.iter().fold(0.0_f64, |acc, x| acc.max(x.abs()));
    if m == 0.0 || !m.is_finite() {
        return m;
    }
    m * v.iter().map(|x| (x / m) * (x / m)).sum::<f64>().sqrt()
}

pub fn norm(a: &[Value]) -> R {
    // v, p
    let (v, p) = (nums(&a[0])?, num(&a[1])?);
    let mut s = 0.0;
    for x in &v {
        s += pypow(x.abs(), p)?;
    }
    n(pypow(s, div(1.0, p)?)?)
}

pub fn unit_vector(a: &[Value]) -> R {
    let v = nums(&a[0])?;
    let h = hypot(&v);
    let mut out = vec![];
    for x in &v {
        out.push(div(*x, h)?);
    }
    Ok(vlist(out))
}

pub fn vector_magnitude(a: &[Value]) -> R {
    n(hypot(&nums(&a[0])?))
}

pub fn vector_projection(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    let mut out = vec![];
    for q in &y {
        out.push(div(dot(&x, &y), dot(&y, &y))? * q);
    }
    Ok(vlist(out))
}

pub fn metric_space_distance(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    need(x.len() == y.len(), "both points must have the same number of dimensions")?;
    let d: Vec<f64> = x.iter().zip(&y).map(|(p, q)| p - q).collect();
    n(hypot(&d))
}

// ---------------------------------------------------------------- sums, products, series
pub fn product(a: &[Value]) -> R {
    n(nums(&a[0])?.iter().product())
}

pub fn summation(a: &[Value]) -> R {
    n(nums(&a[0])?.iter().sum())
}

pub fn maclaurin_series(a: &[Value]) -> R {
    let (d, x) = (nums(&a[0])?, num(&a[1])?);
    let mut s = 0.0;
    for (k, dk) in d.iter().enumerate() {
        s += dk / factorial(k)? * pypow(x, k as f64)?;
    }
    n(s)
}

pub fn taylor_series(a: &[Value]) -> R {
    // derivs, a, x
    let (d, c, x) = (nums(&a[0])?, num(&a[1])?, num(&a[2])?);
    let mut s = 0.0;
    for (k, dk) in d.iter().enumerate() {
        s += dk / factorial(k)? * pypow(x - c, k as f64)?;
    }
    n(s)
}

pub fn generating_function(a: &[Value]) -> R {
    let (t, x) = (nums(&a[0])?, num(&a[1])?);
    let mut s = 0.0;
    for (k, tk) in t.iter().enumerate() {
        s += tk * pypow(x, k as f64)?;
    }
    n(s)
}

pub fn burnside_s_lemma(a: &[Value]) -> R {
    let f = nums(&a[0])?;
    n(div(f.iter().sum(), f.len() as f64)?)
}

pub fn recurrence_relation(a: &[Value]) -> R {
    // coeffs, start, n
    let (coeffs, start, idx) = (nums(&a[0])?, nums(&a[1])?, int_of(num(&a[2])?)?);
    need(start.len() >= coeffs.len(), "give at least as many starting terms as coefficients")?;
    let mut seq = start;
    while (seq.len() as i64) <= idx {
        let last = seq.len();
        let mut s = 0.0;
        for (i, c) in coeffs.iter().enumerate() {
            s += c * seq[last - 1 - i];
        }
        seq.push(s);
    }
    n(at(&seq, idx)?)
}

pub fn alternating_series_test(a: &[Value]) -> R {
    let t: Vec<f64> = nums(&a[0])?.iter().map(|x| x.abs()).collect();
    let dec = t.windows(2).all(|w| w[1] <= w[0]);
    Ok(Value::Tuple(vec![Value::Bool(dec), Value::Number(at(&t, -1)?)]))
}

pub fn comparison_test(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    b(x.len() == y.len() && x.iter().zip(&y).all(|(p, q)| 0.0 <= *p && p <= q))
}

pub fn convergence(a: &[Value]) -> R {
    // terms, L, eps
    let (t, l, eps) = (nums(&a[0])?, num(&a[1])?, num(&a[2])?);
    let mut last_bad: i64 = -1;
    for (k, x) in t.iter().enumerate() {
        if (x - l).abs() >= eps {
            last_bad = k as i64;
        }
    }
    need(last_bad != t.len() as i64 - 1, "the last term is not within eps of L")?;
    n((last_bad + 1) as f64)
}

pub fn infimum(a: &[Value]) -> R {
    let v = nums(&a[0])?;
    need(!v.is_empty(), "min() of an empty list")?;
    n(v.iter().cloned().fold(f64::INFINITY, f64::min))
}

pub fn supremum(a: &[Value]) -> R {
    let v = nums(&a[0])?;
    need(!v.is_empty(), "max() of an empty list")?;
    n(v.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
}

pub fn limit_inferior(a: &[Value]) -> R {
    let v = nums(&a[0])?;
    let tail = &v[v.len() / 2..];
    need(!tail.is_empty(), "min() of an empty list")?;
    n(tail.iter().cloned().fold(f64::INFINITY, f64::min))
}

pub fn limit_superior(a: &[Value]) -> R {
    let v = nums(&a[0])?;
    let tail = &v[v.len() / 2..];
    need(!tail.is_empty(), "max() of an empty list")?;
    n(tail.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
}

pub fn series_convergence_test(a: &[Value]) -> R {
    let t = nums(&a[0])?;
    let r = div(at(&t, -1)?, at(&t, -2)?)?.abs();
    let verdict = if r < 1.0 {
        "converges (ratio < 1)"
    } else if r > 1.0 {
        "diverges (ratio > 1)"
    } else {
        "inconclusive (ratio = 1)"
    };
    Ok(Value::Tuple(vec![Value::Number(r), Value::Text(verdict.into())]))
}

pub fn cauchy_sequence(a: &[Value]) -> R {
    // terms, eps
    let (t, eps) = (nums(&a[0])?, num(&a[1])?);
    for start in 0..t.len() {
        let tail = &t[start..];
        let hi = tail.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lo = tail.iter().cloned().fold(f64::INFINITY, f64::min);
        if hi - lo < eps {
            return n(start as f64);
        }
    }
    Err("the terms never settle within eps".into())
}

// ---------------------------------------------------------------- interpolation, transforms
pub fn lagrange_interpolation(a: &[Value]) -> R {
    // xs, ys, x
    let (xs, ys, x) = (nums(&a[0])?, nums(&a[1])?, num(&a[2])?);
    let mut total = 0.0;
    for i in 0..xs.len() {
        let mut p = 1.0;
        for j in 0..xs.len() {
            if j != i {
                p *= div(x - xs[j], xs[i] - xs[j])?;
            }
        }
        total += at(&ys, i as i64)? * p;
    }
    n(total)
}

pub fn newton_interpolation(a: &[Value]) -> R {
    let (xs, ys, x) = (nums(&a[0])?, nums(&a[1])?, num(&a[2])?);
    let nn = xs.len();
    let mut coef = ys;
    for j in 1..nn {
        for i in (j..nn).rev() {
            let (ci, cp) = (at(&coef, i as i64)?, at(&coef, i as i64 - 1)?);
            coef[i] = div(ci - cp, xs[i] - xs[i - j])?;
        }
    }
    let mut result = at(&coef, -1)?;
    for i in (0..nn.saturating_sub(1)).rev() {
        result = result * (x - xs[i]) + at(&coef, i as i64)?;
    }
    n(result)
}

pub fn spline(a: &[Value]) -> R {
    let (xs, ys, x) = (nums(&a[0])?, nums(&a[1])?, num(&a[2])?);
    let nn = xs.len();
    let mut sorted = xs.clone();
    sorted.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    need(nn >= 3 && sorted == xs, "give at least 3 increasing x values")?;
    let h: Vec<f64> = (0..nn - 1).map(|i| xs[i + 1] - xs[i]).collect();
    let mut m = vec![vec![0.0; nn + 1]; nn];
    m[0][0] = 1.0;
    m[nn - 1][nn - 1] = 1.0;
    for i in 1..nn - 1 {
        m[i][i - 1] = h[i - 1];
        m[i][i] = 2.0 * (h[i - 1] + h[i]);
        m[i][i + 1] = h[i];
        m[i][nn] = 3.0 * (div(at(&ys, i as i64 + 1)? - ys[i], h[i])? - div(ys[i] - ys[i - 1], h[i - 1])?);
    }
    let (r, _) = rref(&m)?;
    let c: Vec<f64> = (0..nn).map(|i| r[i][nn]).collect();
    need(xs[0] <= x && x <= xs[nn - 1], "x is outside the data")?;
    let i = (0..nn - 1).filter(|k| xs[*k] <= x).max().ok_or("x is outside the data")?;
    let dx = x - xs[i];
    let bb = div(at(&ys, i as i64 + 1)? - ys[i], h[i])? - h[i] * (2.0 * c[i] + c[i + 1]) / 3.0;
    let d = (c[i + 1] - c[i]) / (3.0 * h[i]);
    n(ys[i] + bb * dx + c[i] * dx.powi(2) + d * dx.powi(3))
}

pub fn convolution(a: &[Value]) -> R {
    let (f, g) = (nums(&a[0])?, nums(&a[1])?);
    if f.is_empty() && g.is_empty() {
        return Ok(vlist(vec![]));
    }
    let len = (f.len() + g.len()).saturating_sub(1);
    let mut out = vec![];
    for k in 0..len {
        let mut s = 0.0;
        for (j, fj) in f.iter().enumerate() {
            let d = k as i64 - j as i64;
            if 0 <= d && (d as usize) < g.len() {
                s += fj * g[d as usize];
            }
        }
        out.push(s);
    }
    Ok(vlist(out))
}

fn cplx(re: f64, im: f64) -> Value {
    Value::Complex(re, im)
}

pub fn discrete_fourier_transform(a: &[Value]) -> R {
    let x = nums(&a[0])?;
    let nn = x.len();
    let mut out = vec![];
    for k in 0..nn {
        let (mut re, mut im) = (0.0, 0.0);
        for (j, xj) in x.iter().enumerate() {
            let ang = -2.0 * PI * k as f64 * j as f64 / nn as f64;
            re += xj * ang.cos();
            im += xj * ang.sin();
        }
        out.push(cplx(re, im));
    }
    Ok(Value::List(out))
}

pub fn inverse_discrete_fourier_transform(a: &[Value]) -> R {
    // the input list may hold plain numbers (a complex spectrum cannot be typed in a box)
    let x = nums(&a[0])?;
    let nn = x.len();
    let mut out = vec![];
    for j in 0..nn {
        let (mut re, mut im) = (0.0, 0.0);
        for (k, xk) in x.iter().enumerate() {
            let ang = 2.0 * PI * k as f64 * j as f64 / nn as f64;
            re += xk * ang.cos();
            im += xk * ang.sin();
        }
        out.push(cplx(re / nn as f64, im / nn as f64));
    }
    Ok(Value::List(out))
}

pub fn z_transform(a: &[Value]) -> R {
    let (x, z) = (nums(&a[0])?, num(&a[1])?);
    let mut s = 0.0;
    for (k, xk) in x.iter().enumerate() {
        s += xk * pypow(z, -(k as f64))?;
    }
    n(s)
}

// ---------------------------------------------------------------- statistics
pub fn chi_square_statistic(a: &[Value]) -> R {
    let (o, e) = (nums(&a[0])?, nums(&a[1])?);
    let mut s = 0.0;
    for (p, q) in o.iter().zip(&e) {
        s += div((p - q).powi(2), *q)?;
    }
    n(s)
}

pub fn correlation(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    need(x.len() == y.len(), "correlation requires that both inputs have same number of data points")?;
    need(x.len() >= 2, "correlation requires at least two data points")?;
    let (xb, yb) = (st_mean(&x)?, st_mean(&y)?);
    let sxy: f64 = x.iter().zip(&y).map(|(p, q)| (p - xb) * (q - yb)).sum();
    let (sxx, syy) = (st_ss(&x)?, st_ss(&y)?);
    need(sxx != 0.0 && syy != 0.0, "at least one of the inputs is constant")?;
    n(sxy / (sxx * syy).sqrt())
}

pub fn covariance(a: &[Value]) -> R {
    let (x, y) = (nums(&a[0])?, nums(&a[1])?);
    need(x.len() == y.len(), "covariance requires that both inputs have same number of data points")?;
    need(x.len() >= 2, "covariance requires at least two data points")?;
    let (xb, yb) = (st_mean(&x)?, st_mean(&y)?);
    n(x.iter().zip(&y).map(|(p, q)| (p - xb) * (q - yb)).sum::<f64>() / (x.len() - 1) as f64)
}

pub fn expected_value(a: &[Value]) -> R {
    let (v, p) = (nums(&a[0])?, nums(&a[1])?);
    n(v.iter().zip(&p).map(|(x, q)| x * q).sum())
}

pub fn list_maximum(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "max() of an empty list")?;
    n(d.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
}

pub fn list_minimum(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "min() of an empty list")?;
    n(d.iter().cloned().fold(f64::INFINITY, f64::min))
}

pub fn third_quartile(a: &[Value]) -> R {
    let q = st_quantiles(&nums(&a[0])?, 4)?;
    n(q[2])
}

pub fn first_quartile(a: &[Value]) -> R {
    let q = st_quantiles(&nums(&a[0])?, 4)?;
    n(q[0])
}

pub fn expected_square(a: &[Value]) -> R {
    let (v, p) = (nums(&a[0])?, nums(&a[1])?);
    n(v.iter().zip(&p).map(|(x, q)| x * x * q).sum())
}

pub fn variance_of_random_variable(a: &[Value]) -> R {
    let (v, p) = (nums(&a[0])?, nums(&a[1])?);
    let ex2: f64 = v.iter().zip(&p).map(|(x, q)| x * x * q).sum();
    let ex: f64 = v.iter().zip(&p).map(|(x, q)| x * q).sum();
    n(ex2 - ex.powi(2))
}

pub fn interquartile_range(a: &[Value]) -> R {
    let q = st_quantiles(&nums(&a[0])?, 4)?;
    n(q[2] - q[0])
}

pub fn percentile(a: &[Value]) -> R {
    // data, k
    let (d, k) = (nums(&a[0])?, whole(num(&a[1])?)?);
    let q = st_quantiles(&d, 100)?;
    n(at(&q, k - 1)?)
}

pub fn quartiles(a: &[Value]) -> R {
    Ok(vlist(st_quantiles(&nums(&a[0])?, 4)?))
}

pub fn linear_regression(a: &[Value]) -> R {
    let (s, i) = st_regression(&nums(&a[0])?, &nums(&a[1])?)?;
    Ok(vtuple(vec![s, i]))
}

pub fn least_squares(a: &[Value]) -> R {
    linear_regression(a)
}

pub fn mean(a: &[Value]) -> R {
    n(st_mean(&nums(&a[0])?)?)
}

pub fn median(a: &[Value]) -> R {
    let mut d = nums(&a[0])?;
    need(!d.is_empty(), "no median for empty data")?;
    d.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    let m = d.len();
    n(if m % 2 == 1 { d[m / 2] } else { (d[m / 2 - 1] + d[m / 2]) / 2.0 })
}

pub fn mode(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "no mode for empty data")?;
    let mut counts: Vec<(f64, usize)> = vec![];
    for x in &d {
        match counts.iter_mut().find(|(v, _)| v == x) {
            Some(slot) => slot.1 += 1,
            None => counts.push((*x, 1)),
        }
    }
    let mut best = counts[0];
    for c in &counts[1..] {
        if c.1 > best.1 {
            best = *c;
        }
    }
    n(best.0)
}

pub fn range(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "max() of an empty list")?;
    n(d.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - d.iter().cloned().fold(f64::INFINITY, f64::min))
}

pub fn sample_variance(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(d.len() >= 2, "variance requires at least two data points")?;
    n(st_ss(&d)? / (d.len() - 1) as f64)
}

pub fn sample_standard_deviation(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(d.len() >= 2, "stdev requires at least two data points")?;
    n((st_ss(&d)? / (d.len() - 1) as f64).sqrt())
}

pub fn variance(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "pvariance requires at least one data point")?;
    n(st_ss(&d)? / d.len() as f64)
}

pub fn standard_deviation(a: &[Value]) -> R {
    let d = nums(&a[0])?;
    need(!d.is_empty(), "pstdev requires at least one data point")?;
    n((st_ss(&d)? / d.len() as f64).sqrt())
}

// ---------------------------------------------------------------- circuits
pub fn resistors_in_parallel(a: &[Value]) -> R {
    let r = nums(&a[0])?;
    let mut s = 0.0;
    for x in &r {
        s += div(1.0, *x)?;
    }
    n(div(1.0, s)?)
}

pub fn resistors_in_series(a: &[Value]) -> R {
    n(nums(&a[0])?.iter().sum())
}

// ---------- quaternions (w, x, y, z; real part first): the same operations in the same order as the Python files ----------

pub(super) fn quaternion(v: &Value) -> Result<[f64; 4], String> {
    let q = nums(v)?;
    need(q.len() == 4, "a quaternion has 4 numbers: w, x, y, z")?;
    Ok([q[0], q[1], q[2], q[3]])
}

pub(super) fn quat_multiply(p: &[f64; 4], q: &[f64; 4]) -> [f64; 4] {
    [
        p[0] * q[0] - p[1] * q[1] - p[2] * q[2] - p[3] * q[3],
        p[0] * q[1] + p[1] * q[0] + p[2] * q[3] - p[3] * q[2],
        p[0] * q[2] - p[1] * q[3] + p[2] * q[0] + p[3] * q[1],
        p[0] * q[3] + p[1] * q[2] - p[2] * q[1] + p[3] * q[0],
    ]
}

pub(super) fn quat_size(q: &[f64; 4]) -> f64 {
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt()
}

pub(super) fn vector3(v: &Value) -> Result<[f64; 3], String> {
    let x = nums(v)?;
    need(x.len() == 3, "a vector has 3 numbers: x, y, z")?;
    Ok([x[0], x[1], x[2]])
}

pub fn quaternion_multiply(a: &[Value]) -> R {
    let (p, q) = (quaternion(&a[0])?, quaternion(&a[1])?);
    Ok(vlist(quat_multiply(&p, &q).to_vec()))
}

pub fn quaternion_conjugate(a: &[Value]) -> R {
    let [w, x, y, z] = quaternion(&a[0])?;
    Ok(vlist(vec![w, -x, -y, -z]))
}

pub fn quaternion_inverse(a: &[Value]) -> R {
    let [w, x, y, z] = quaternion(&a[0])?;
    let s = w * w + x * x + y * y + z * z;
    need(s != 0.0, "the quaternion is zero, it has no inverse")?;
    Ok(vlist(vec![w / s, -x / s, -y / s, -z / s]))
}

pub fn quaternion_normalize(a: &[Value]) -> R {
    let q = quaternion(&a[0])?;
    let s = quat_size(&q);
    need(s != 0.0, "the quaternion is zero, it cannot be normalised")?;
    Ok(vlist(q.iter().map(|c| c / s).collect()))
}

pub fn quaternion_from_axis_angle(a: &[Value]) -> R {
    let axis = vector3(&a[0])?;
    let angle = num(&a[1])?;
    let size = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    need(size != 0.0, "the axis has length zero")?;
    let s = (angle / 2.0).sin();
    Ok(vlist(vec![(angle / 2.0).cos(), axis[0] / size * s, axis[1] / size * s, axis[2] / size * s]))
}

pub fn rotate_vector_by_quaternion(a: &[Value]) -> R {
    let q = quaternion(&a[0])?;
    let v = vector3(&a[1])?;
    let s = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    need(s != 0.0, "the quaternion is zero, it cannot rotate a vector")?;
    let r = quat_multiply(&quat_multiply(&q, &[0.0, v[0], v[1], v[2]]), &[q[0], -q[1], -q[2], -q[3]]);
    Ok(vlist(vec![r[1] / s, r[2] / s, r[3] / s]))
}

pub fn quaternion_to_yaw_pitch_roll(a: &[Value]) -> R {
    let q = quaternion(&a[0])?;
    let s = quat_size(&q);
    need(s != 0.0, "the quaternion is zero, it has no yaw, pitch and roll")?;
    let (w, x, y, z) = (q[0] / s, q[1] / s, q[2] / s, q[3] / s);
    let sine = 2.0 * (w * y - z * x);
    need(sine.abs() <= 1.0 + 1e-9, "not a rotation: the pitch sine is outside -1 to 1")?;
    need(sine.abs() < 1.0 - 1e-9, "gimbal lock: pitch is straight up or straight down, so roll and yaw are not separate")?;
    Ok(vlist(vec![
        (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z)),
        sine.asin(),
        (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y)),
    ]))
}

pub fn quaternion_from_yaw_pitch_roll(a: &[Value]) -> R {
    let (yaw, pitch, roll) = (num(&a[0])?, num(&a[1])?, num(&a[2])?);
    let q_yaw = [(yaw / 2.0).cos(), 0.0, 0.0, (yaw / 2.0).sin()];
    let q_pitch = [(pitch / 2.0).cos(), 0.0, (pitch / 2.0).sin(), 0.0];
    let q_roll = [(roll / 2.0).cos(), (roll / 2.0).sin(), 0.0, 0.0];
    Ok(vlist(quat_multiply(&quat_multiply(&q_yaw, &q_pitch), &q_roll).to_vec()))
}

pub fn quaternion_slerp(a: &[Value]) -> R {
    let (q0, q1) = (quaternion(&a[0])?, quaternion(&a[1])?);
    let t = num(&a[2])?;
    let (s0, s1) = (quat_size(&q0), quat_size(&q1));
    need(s0 != 0.0 && s1 != 0.0, "the quaternion is zero, it cannot be blended")?;
    need((0.0..=1.0).contains(&t), "the blend is outside 0 to 1")?;
    let ua: Vec<f64> = q0.iter().map(|c| c / s0).collect();
    let mut ub: Vec<f64> = q1.iter().map(|c| c / s1).collect();
    let mut d = ua[0] * ub[0] + ua[1] * ub[1] + ua[2] * ub[2] + ua[3] * ub[3];
    if d < 0.0 {
        ub = ub.iter().map(|c| -c).collect();
        d = -d;
    }
    d = d.min(1.0);
    if d > 0.9995 {
        let r: Vec<f64> = (0..4).map(|k| ua[k] + t * (ub[k] - ua[k])).collect();
        let s = quat_size(&[r[0], r[1], r[2], r[3]]);
        return Ok(vlist(r.iter().map(|c| c / s).collect()));
    }
    let omega = d.acos();
    let k0 = ((1.0 - t) * omega).sin() / omega.sin();
    let k1 = (t * omega).sin() / omega.sin();
    Ok(vlist((0..4).map(|k| k0 * ua[k] + k1 * ub[k]).collect()))
}

pub fn angle_between_quaternions(a: &[Value]) -> R {
    let (q0, q1) = (quaternion(&a[0])?, quaternion(&a[1])?);
    let (s0, s1) = (quat_size(&q0), quat_size(&q1));
    need(s0 != 0.0 && s1 != 0.0, "the quaternion is zero, it has no orientation")?;
    let ua: Vec<f64> = q0.iter().map(|c| c / s0).collect();
    let ub: Vec<f64> = q1.iter().map(|c| c / s1).collect();
    let d = (ua[0] * ub[0] + ua[1] * ub[1] + ua[2] * ub[2] + ua[3] * ub[3]).abs().min(1.0);
    n(2.0 * d.acos())
}

pub fn projectile_position(a: &[Value]) -> R {
    // v, angle, t, g
    let (v, angle, t, g) = (num(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?);
    need(v >= 0.0, "speed cannot be negative")?;
    need(angle.abs() <= std::f64::consts::FRAC_PI_2, "the angle is past straight up")?;
    need(g > 0.0, "gravity must pull downward")?;
    need(t >= 0.0, "time cannot be negative")?;
    Ok(vlist(vec![v * angle.cos() * t, v * angle.sin() * t - 0.5 * g * t * t]))
}

pub fn centre_of_mass(a: &[Value]) -> R {
    let (masses, positions) = (nums(&a[0])?, mat(&a[1])?);
    need(masses.len() == positions.len(), "each mass needs a position")?;
    need(positions.iter().all(|p| p.len() == 3), "a position has 3 numbers: x, y, z")?;
    need(masses.iter().all(|m| *m >= 0.0), "mass cannot be negative")?;
    need(!masses.is_empty(), "there are no points")?;
    let total: f64 = masses.iter().fold(0.0, |s, m| s + m);
    need(total != 0.0, "the total mass is zero")?;
    Ok(vlist((0..3).map(|k| masses.iter().zip(&positions).fold(0.0, |s, (m, p)| s + m * p[k]) / total).collect()))
}

pub fn dead_reckoning(a: &[Value]) -> R {
    // position, speed, heading, t
    let position = nums(&a[0])?;
    need(position.len() == 2, "a position has 2 numbers: east, north")?;
    let (speed, heading, t) = (num(&a[1])?, num(&a[2])?, num(&a[3])?);
    need(t >= 0.0, "time cannot be negative")?;
    Ok(vlist(vec![position[0] + speed * heading.cos() * t, position[1] + speed * heading.sin() * t]))
}

pub fn local_east_north_up(a: &[Value]) -> R {
    // lat, delta_lat, delta_lon, delta_height
    let (lat, dlat, dlon, dh) = (num(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?);
    need(lat.abs() <= std::f64::consts::FRAC_PI_2, "latitude is past the pole")?;
    Ok(vlist(vec![6371000.0 * lat.cos() * dlon, 6371000.0 * dlat, dh]))
}

fn point3(v: &Value) -> Result<[f64; 3], String> {
    let x = nums(v)?;
    need(x.len() == 3, "a point has 3 numbers: x, y, z")?;
    Ok([x[0], x[1], x[2]])
}

pub fn distance_3d(a: &[Value]) -> R {
    let (p, q) = (point3(&a[0])?, point3(&a[1])?);
    let (dx, dy, dz) = (q[0] - p[0], q[1] - p[1], q[2] - p[2]);
    n((dx * dx + dy * dy + dz * dz).sqrt())
}

/// (distance, closing speed) of a fixed obstacle `position` from the robot that moves with `velocity`.
fn closing(a: &[Value]) -> Result<(f64, f64), String> {
    let (p, v) = (nums(&a[0])?, nums(&a[1])?);
    need(p.len() == 3 && v.len() == 3, "a vector has 3 numbers: x, y, z")?;
    let distance = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    need(distance != 0.0, "the distance is zero")?;
    Ok((distance, -(p[0] * v[0] + p[1] * v[1] + p[2] * v[2]) / distance))
}

pub fn time_to_collision(a: &[Value]) -> R {
    let (distance, closing) = closing(a)?;
    need(closing > 0.0, "not approaching")?;
    n(distance / closing)
}

pub fn closing_speed(a: &[Value]) -> R {
    n(closing(a)?.1)
}

pub fn point_inside_sphere(a: &[Value]) -> R {
    // centre, radius, point
    let (c, p) = (nums(&a[0])?, nums(&a[2])?);
    need(c.len() == 3 && p.len() == 3, "a point has 3 numbers: x, y, z")?;
    let radius = num(&a[1])?;
    need(radius >= 0.0, "radius cannot be negative")?;
    let (dx, dy, dz) = (p[0] - c[0], p[1] - c[1], p[2] - c[2]);
    n(if dx * dx + dy * dy + dz * dz <= radius * radius { 1.0 } else { 0.0 })
}

pub fn point_inside_cylinder(a: &[Value]) -> R {
    // centre, radius, height, point
    let (c, p) = (nums(&a[0])?, nums(&a[3])?);
    need(c.len() == 2, "a centre has 2 numbers: x, y")?;
    need(p.len() == 3, "a point has 3 numbers: x, y, z")?;
    let (radius, height) = (num(&a[1])?, num(&a[2])?);
    need(radius >= 0.0, "radius cannot be negative")?;
    need(height >= 0.0, "a length cannot be negative")?;
    let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
    n(if dx * dx + dy * dy <= radius * radius && 0.0 <= p[2] && p[2] <= height { 1.0 } else { 0.0 })
}

pub fn distance_to_segment(a: &[Value]) -> R {
    // a, b, p
    let (p0, b, p) = (nums(&a[0])?, nums(&a[1])?, nums(&a[2])?);
    need(p0.len() == 3 && b.len() == 3 && p.len() == 3, "a point has 3 numbers: x, y, z")?;
    let ab = [b[0] - p0[0], b[1] - p0[1], b[2] - p0[2]];
    let length2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    need(length2 != 0.0, "the segment has length zero")?;
    let t = (((p[0] - p0[0]) * ab[0] + (p[1] - p0[1]) * ab[1] + (p[2] - p0[2]) * ab[2]) / length2).max(0.0).min(1.0);
    let (dx, dy, dz) = (p[0] - (p0[0] + t * ab[0]), p[1] - (p0[1] + t * ab[1]), p[2] - (p0[2] + t * ab[2]));
    n((dx * dx + dy * dy + dz * dz).sqrt())
}

pub fn point_inside_camera_cone(a: &[Value]) -> R {
    // camera, direction, half_angle, point
    let (camera, direction, point) = (nums(&a[0])?, nums(&a[1])?, nums(&a[3])?);
    need(camera.len() == 3 && direction.len() == 3 && point.len() == 3, "a point has 3 numbers: x, y, z")?;
    let half_angle = num(&a[2])?;
    let d_size = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
    need(d_size != 0.0, "the look direction has length zero")?;
    let v = [point[0] - camera[0], point[1] - camera[1], point[2] - camera[2]];
    let v_size = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    need(v_size != 0.0, "the point is the camera")?;
    need((0.0..=PI).contains(&half_angle), "the field of view must be from 0 to pi")?;
    let along = v[0] * direction[0] + v[1] * direction[1] + v[2] * direction[2];
    n(if along > 0.0 && along / (v_size * d_size) >= half_angle.cos() - 1e-12 { 1.0 } else { 0.0 })
}

pub fn potential_field_step(a: &[Value]) -> R {
    // position, goal, obstacle, influence, k_att, k_rep
    let (position, goal, obstacle) = (nums(&a[0])?, nums(&a[1])?, nums(&a[2])?);
    need(position.len() == 3 && goal.len() == 3 && obstacle.len() == 3, "a point has 3 numbers: x, y, z")?;
    let (influence, k_att, k_rep) = (num(&a[3])?, num(&a[4])?, num(&a[5])?);
    let away = [position[0] - obstacle[0], position[1] - obstacle[1], position[2] - obstacle[2]];
    let d = (away[0] * away[0] + away[1] * away[1] + away[2] * away[2]).sqrt();
    need(d != 0.0, "the distance is zero")?;
    need(influence > 0.0, "the influence distance must be positive")?;
    need(k_att >= 0.0 && k_rep >= 0.0, "a gain cannot be negative")?;
    let push = if d < influence { k_rep * (1.0 / d - 1.0 / influence) * (1.0 / (d * d)) } else { 0.0 };
    Ok(vlist((0..3).map(|k| k_att * (goal[k] - position[k]) + push * away[k] / d).collect()))
}
