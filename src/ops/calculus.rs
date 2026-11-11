//! Operators that take a function typed as text (calculus, numerical methods, differential equations,
//! complex contour integrals, group-theory tests that use an operation).  The numerical methods repeat the
//! Python steps in the same order, so the answers agree to rounding.

use super::vals::*;
use crate::expr::{real, Callable};
use crate::value::Value;
use std::f64::consts::PI;

// ---------------------------------------------------------------- calling functions
fn nv(x: f64) -> Value {
    Value::Number(x)
}

fn callv(f: &Callable, args: &[Value]) -> Result<Value, String> {
    f.call(args)
}

fn call(f: &Callable, args: &[f64]) -> Result<f64, String> {
    f.f(args)
}

/// A list / tuple of real numbers out of a function's answer.
fn seqf(v: &Value) -> Result<Vec<f64>, String> {
    match v {
        Value::List(a) | Value::Tuple(a) => a.iter().map(real).collect(),
        Value::Pair(p, q) => Ok(vec![*p, *q]),
        other => Err(format!("expected a list, got {}", other.kind_name())),
    }
}

fn point_args(p: &[f64]) -> Vec<f64> {
    p.to_vec()
}

fn shifted(point: &[f64], i: f64, d: f64) -> Vec<f64> {
    point.iter().enumerate().map(|(j, p)| if j as f64 == i { p + d } else { *p }).collect()
}

/// `_helpers.partial`: central difference of f (given as a closure of the point) in variable i.
fn partial<F: Fn(&[f64]) -> Result<f64, String>>(f: &F, point: &[f64], i: f64, h: f64) -> Result<f64, String> {
    Ok((f(&shifted(point, i, h))? - f(&shifted(point, i, -h))?) / (2.0 * h))
}

fn partial2<F: Fn(&[f64]) -> Result<f64, String>>(f: &F, point: &[f64], i: f64, j: f64, h: f64) -> Result<f64, String> {
    let up = partial(f, &shifted(point, j, h), i, 1e-4)?;
    let dn = partial(f, &shifted(point, j, -h), i, 1e-4)?;
    Ok((up - dn) / (2.0 * h))
}

fn simpson<F: Fn(f64) -> Result<f64, String>>(f: F, a: f64, b: f64, n: i64) -> Result<f64, String> {
    let n = n + n % 2;
    let h = (b - a) / n as f64;
    let mut s = f(a)? + f(b)?;
    for i in 1..n {
        s += (if i % 2 == 1 { 4.0 } else { 2.0 }) * f(a + i as f64 * h)?;
    }
    Ok(s * h / 3.0)
}

/// `_helpers.rk4` for a scalar y.
fn rk4_scalar(f: &Callable, x0: f64, y0: f64, x_end: f64) -> Result<f64, String> {
    let steps = 1000;
    let h = (x_end - x0) / steps as f64;
    let (mut x, mut y) = (x0, y0);
    for _ in 0..steps {
        let k1 = call(f, &[x, y])?;
        let k2 = call(f, &[x + h / 2.0, y + h / 2.0 * k1])?;
        let k3 = call(f, &[x + h / 2.0, y + h / 2.0 * k2])?;
        let k4 = call(f, &[x + h, y + h * k3])?;
        y += h / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        x += h;
    }
    Ok(y)
}

type C = (f64, f64);

fn cm(a: C, b: C) -> C {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cplx_of(v: &Value) -> Result<C, String> {
    match v {
        Value::Number(x) => Ok((*x, 0.0)),
        Value::Bool(t) => Ok((*t as i32 as f64, 0.0)),
        Value::Complex(a, b) => Ok((*a, *b)),
        other => Err(format!("expected a number, got {}", other.kind_name())),
    }
}

/// `_helpers.circle_integral`: the counter-clockwise integral of g(z) dz over a circle.
fn circle_integral<G: Fn(C) -> Result<C, String>>(g: G, center: f64, radius: f64) -> Result<C, String> {
    let n = 2000;
    let mut total: C = (0.0, 0.0);
    for k in 0..n {
        let t = 2.0 * PI * (k as f64 + 0.5) / n as f64;
        let e = (t.cos(), t.sin());
        let z = (center + radius * e.0, radius * e.1);
        let fz = g(z)?;
        // f(z) * 1j * radius * exp(1j t) * (2 pi / n), multiplied left to right as Python does
        let step = cm(cm(cm(fz, (0.0, 1.0)), (radius, 0.0)), e);
        let step = (step.0 * (2.0 * PI / n as f64), step.1 * (2.0 * PI / n as f64));
        total = (total.0 + step.0, total.1 + step.1);
    }
    Ok(total)
}

fn csub(a: C, b: C) -> C {
    (a.0 - b.0, a.1 - b.1)
}

fn cdivide(a: C, b: C) -> Result<C, String> {
    let d = b.0 * b.0 + b.1 * b.1;
    if d == 0.0 {
        return Err("division by zero".into());
    }
    Ok(((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d))
}

fn cresult(c: C) -> R {
    Ok(Value::Complex(c.0, c.1))
}

// ---------------------------------------------------------------- derivatives, integrals, limits
pub fn derivative(a: &[Value]) -> R {
    // f, x, h
    let (f, x, h) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n((call(&f, &[x + h])? - call(&f, &[x - h])?) / (2.0 * h))
}

pub fn second_derivative(a: &[Value]) -> R {
    let (f, x, h) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n((call(&f, &[x + h])? - 2.0 * call(&f, &[x])? + call(&f, &[x - h])?) / h.powi(2))
}

fn simpson_op(a: &[Value]) -> R {
    // f, a, b, n
    let (f, lo, hi, nn) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, whole(num(&a[3])?)?);
    let nn = nn + nn.rem_euclid(2);
    let h = (hi - lo) / nn as f64;
    let mut total = call(&f, &[lo])? + call(&f, &[hi])?;
    for i in 1..nn {
        total += (if i % 2 == 1 { 4.0 } else { 2.0 }) * call(&f, &[lo + i as f64 * h])?;
    }
    n(total * h / 3.0)
}

pub fn definite_integral(a: &[Value]) -> R {
    simpson_op(a)
}

pub fn simpson_s_rule(a: &[Value]) -> R {
    simpson_op(a)
}

pub fn indefinite_integral(a: &[Value]) -> R {
    // f, x, C
    let (f, x, c) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n(simpson(|t| call(&f, &[t]), 0.0, x, 1000)? + c)
}

pub fn lebesgue_integral(a: &[Value]) -> R {
    let (f, lo, hi) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n(simpson(|t| call(&f, &[t]), lo, hi, 2000)?)
}

pub fn trapezoid_rule(a: &[Value]) -> R {
    let (f, lo, hi, nn) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, whole(num(&a[3])?)?);
    let mut s = 0.0;
    for i in 1..nn {
        s += call(&f, &[lo + i as f64 * (hi - lo) / nn as f64])?;
    }
    n(div(hi - lo, nn as f64)? * ((call(&f, &[lo])? + call(&f, &[hi])?) / 2.0 + s))
}

pub fn riemann_sum(a: &[Value]) -> R {
    let (f, lo, hi, nn) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, whole(num(&a[3])?)?);
    let mut s = 0.0;
    for i in 0..nn.max(0) {
        s += call(&f, &[lo + i as f64 * (hi - lo) / nn as f64])?;
    }
    n(div(s * (hi - lo), nn as f64)?)
}

pub fn riemannstieltjes_sum(a: &[Value]) -> R {
    let (f, g, xs) = (func(&a[0])?, func(&a[1])?, nums(&a[2])?);
    let mut s = 0.0;
    for i in 0..xs.len().saturating_sub(1) {
        s += call(&f, &[xs[i]])? * (call(&g, &[xs[i + 1]])? - call(&g, &[xs[i]])?);
    }
    n(s)
}

pub fn lp_norm(a: &[Value]) -> R {
    // f, a, b, p, n
    let (f, lo, hi, p, nn) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    let mut s = 0.0;
    for i in 0..nn.max(0) {
        s += pypow(call(&f, &[lo + i as f64 * (hi - lo) / nn as f64])?.abs(), p)?;
    }
    n(pypow(s * (hi - lo) / nn as f64, div(1.0, p)?)?)
}

pub fn mean_value_theorem(a: &[Value]) -> R {
    let (f, lo, hi) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n(div(call(&f, &[hi])? - call(&f, &[lo])?, hi - lo)?)
}

pub fn lipschitz_ratio(a: &[Value]) -> R {
    let (f, x, y) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    n(div((call(&f, &[x])? - call(&f, &[y])?).abs(), (x - y).abs())?)
}

pub fn limit(a: &[Value]) -> R {
    let (f, at_) = (func(&a[0])?, num(&a[1])?);
    let h = 1e-6;
    let (left, right) = (call(&f, &[at_ - h])?, call(&f, &[at_ + h])?);
    need((left - right).abs() <= 1e-3 * 1.0_f64.max(left.abs()).max(right.abs()), "the left and right limits differ")?;
    n((left + right) / 2.0)
}

pub fn l_hopital_s_rule(a: &[Value]) -> R {
    let (f, g, at_) = (func(&a[0])?, func(&a[1])?, num(&a[2])?);
    let d = |ff: &Callable, x: f64| -> Result<f64, String> {
        let h = 1e-5;
        Ok((call(ff, &[x + h])? - call(ff, &[x - h])?) / (2.0 * h))
    };
    let h = 1e-4;
    let left = div(d(&f, at_ - h)?, d(&g, at_ - h)?)?;
    let right = div(d(&f, at_ + h)?, d(&g, at_ + h)?)?;
    need((left - right).abs() <= 1e-2 * 1.0_f64.max(left.abs()).max(right.abs()), "the one-sided limits of f′/g′ differ")?;
    n((left + right) / 2.0)
}

pub fn integral_test(a: &[Value]) -> R {
    let (f, nn) = (func(&a[0])?, int_of(num(&a[1])?)?);
    let mut s = 0.0;
    for k in 1..=nn.max(0) {
        s += call(&f, &[k as f64])?;
    }
    Ok(Value::Tuple(vec![nv(s), nv(simpson(|t| call(&f, &[t]), 1.0, nn as f64, 2000)?)]))
}

pub fn epsilon_delta_continuity(a: &[Value]) -> R {
    let (f, at_, eps) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    let mut delta = 1.0;
    for _ in 0..60 {
        let mut ok = true;
        for k in -199..200 {
            if (call(&f, &[at_ + delta * k as f64 / 200.0])? - call(&f, &[at_])?).abs() >= eps {
                ok = false;
                break;
            }
        }
        if ok {
            return n(delta);
        }
        delta /= 2.0;
    }
    Err("no δ found: f may not be continuous at a".into())
}

// ---------------------------------------------------------------- several variables
fn multi_at(f: &Callable, p: &[f64]) -> Result<f64, String> {
    call(f, &point_args(p))
}

pub fn gradient(a: &[Value]) -> R {
    // f, point, h
    let (f, pt, h) = (func(&a[0])?, nums(&a[1])?, num(&a[2])?);
    let mut out = vec![];
    for i in 0..pt.len() {
        out.push((multi_at(&f, &shifted(&pt, i as f64, h))? - multi_at(&f, &shifted(&pt, i as f64, -h))?) / (2.0 * h));
    }
    Ok(vlist(out))
}

pub fn partial_derivative(a: &[Value]) -> R {
    // f, point, i, h
    let (f, pt, i, h) = (func(&a[0])?, nums(&a[1])?, num(&a[2])?, num(&a[3])?);
    n((multi_at(&f, &shifted(&pt, i, h))? - multi_at(&f, &shifted(&pt, i, -h))?) / (2.0 * h))
}

pub fn second_partial_derivative(a: &[Value]) -> R {
    let (f, pt, i, h) = (func(&a[0])?, nums(&a[1])?, num(&a[2])?, num(&a[3])?);
    let up = multi_at(&f, &shifted(&pt, i, h))?;
    let mid = multi_at(&f, &pt)?;
    let dn = multi_at(&f, &shifted(&pt, i, -h))?;
    n((up - 2.0 * mid + dn) / (h * h))
}

fn component_fn<'a>(f: &'a Callable, comp: usize) -> impl Fn(&[f64]) -> Result<f64, String> + 'a {
    move |p: &[f64]| {
        let v = callv(f, &p.iter().map(|x| nv(*x)).collect::<Vec<_>>())?;
        let items = seqf(&v)?;
        at(&items, comp as i64)
    }
}

pub fn divergence(a: &[Value]) -> R {
    let (f, pt) = (func(&a[0])?, nums(&a[1])?);
    let mut s = 0.0;
    for i in 0..pt.len() {
        s += partial(&component_fn(&f, i), &pt, i as f64, 1e-5)?;
    }
    n(s)
}

pub fn curl(a: &[Value]) -> R {
    let (f, pt) = (func(&a[0])?, nums(&a[1])?);
    need(pt.len() == 3, "curl needs a 3-D point")?;
    let d = |comp: usize, var: usize| partial(&component_fn(&f, comp), &pt, var as f64, 1e-5);
    Ok(vlist(vec![d(2, 1)? - d(1, 2)?, d(0, 2)? - d(2, 0)?, d(1, 0)? - d(0, 1)?]))
}

pub fn laplacian(a: &[Value]) -> R {
    let (f, pt) = (func(&a[0])?, nums(&a[1])?);
    let ff = |p: &[f64]| multi_at(&f, p);
    let mut s = 0.0;
    for i in 0..pt.len() {
        s += partial2(&ff, &pt, i as f64, i as f64, 1e-4)?;
    }
    n(s)
}

pub fn hessian(a: &[Value]) -> R {
    let (f, pt) = (func(&a[0])?, nums(&a[1])?);
    let ff = |p: &[f64]| multi_at(&f, p);
    let mut out = vec![];
    for i in 0..pt.len() {
        let mut row = vec![];
        for j in 0..pt.len() {
            row.push(partial2(&ff, &pt, i as f64, j as f64, 1e-4)?);
        }
        out.push(row);
    }
    Ok(vmat(out))
}

pub fn lie_bracket(a: &[Value]) -> R {
    let (x, y, pt) = (func(&a[0])?, func(&a[1])?, nums(&a[2])?);
    let args: Vec<Value> = pt.iter().map(|v| nv(*v)).collect();
    let (xp, yp) = (seqf(&callv(&x, &args)?)?, seqf(&callv(&y, &args)?)?);
    let mut out = vec![];
    for i in 0..pt.len() {
        let mut s = 0.0;
        for j in 0..pt.len() {
            s += at(&xp, j as i64)? * partial(&component_fn(&y, i), &pt, j as f64, 1e-5)?
                - at(&yp, j as i64)? * partial(&component_fn(&x, i), &pt, j as f64, 1e-5)?;
        }
        out.push(s);
    }
    Ok(vlist(out))
}

pub fn lagrange_multipliers(a: &[Value]) -> R {
    let (f, g, pt) = (func(&a[0])?, func(&a[1])?, nums(&a[2])?);
    let (ff, gg) = (|p: &[f64]| multi_at(&f, p), |p: &[f64]| multi_at(&g, p));
    let mut gf = vec![];
    let mut gv = vec![];
    for i in 0..pt.len() {
        gf.push(partial(&ff, &pt, i as f64, 1e-5)?);
    }
    for i in 0..pt.len() {
        gv.push(partial(&gg, &pt, i as f64, 1e-5)?);
    }
    let denom: f64 = gv.iter().map(|v| v * v).sum();
    need(denom != 0.0, "∇g is zero here")?;
    let lam = gf.iter().zip(&gv).map(|(p, q)| p * q).sum::<f64>() / denom;
    let err = gf.iter().zip(&gv).map(|(p, q)| (p - lam * q).powi(2)).sum::<f64>().sqrt();
    Ok(Value::Tuple(vec![nv(lam), nv(err)]))
}

pub fn cauchyriemann_equations(a: &[Value]) -> R {
    let (u, v, x0, y0) = (func(&a[0])?, func(&a[1])?, num(&a[2])?, num(&a[3])?);
    let p = [x0, y0];
    let (uf, vf) = (|q: &[f64]| multi_at(&u, q), |q: &[f64]| multi_at(&v, q));
    let e1 = partial(&uf, &p, 0.0, 1e-5)? - partial(&vf, &p, 1.0, 1e-5)?;
    let e2 = partial(&uf, &p, 1.0, 1e-5)? + partial(&vf, &p, 0.0, 1e-5)?;
    Ok(Value::Tuple(vec![Value::Bool(e1.abs() < 1e-4 && e2.abs() < 1e-4), nv(e1), nv(e2)]))
}

pub fn double_integral(a: &[Value]) -> R {
    let (f, x0, x1, y0, y1) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, num(&a[4])?);
    n(simpson(|x| simpson(|y| call(&f, &[x, y]), y0, y1, 100), x0, x1, 100)?)
}

pub fn triple_integral(a: &[Value]) -> R {
    let f = func(&a[0])?;
    let v: Result<Vec<f64>, String> = a[1..7].iter().map(num).collect();
    let v = v?;
    n(simpson(|x| simpson(|y| simpson(|z| call(&f, &[x, y, z]), v[4], v[5], 24), v[2], v[3], 24), v[0], v[1], 24)?)
}

pub fn line_integral(a: &[Value]) -> R {
    // F, r, a, b
    let (ff, rr, lo, hi) = (func(&a[0])?, func(&a[1])?, num(&a[2])?, num(&a[3])?);
    let integrand = |t: f64| -> Result<f64, String> {
        let h = 1e-6;
        let pos = seqf(&callv(&rr, &[nv(t)])?)?;
        let (up, dn) = (seqf(&callv(&rr, &[nv(t + h)])?)?, seqf(&callv(&rr, &[nv(t - h)])?)?);
        let vel: Vec<f64> = up.iter().zip(&dn).map(|(p, m)| (p - m) / (2.0 * h)).collect();
        let fv = seqf(&callv(&ff, &pos.iter().map(|x| nv(*x)).collect::<Vec<_>>())?)?;
        Ok(fv.iter().zip(&vel).map(|(p, q)| p * q).sum())
    };
    n(simpson(integrand, lo, hi, 1000)?)
}

pub fn tangent_vector(a: &[Value]) -> R {
    let (curve, t) = (func(&a[0])?, num(&a[1])?);
    let h = 1e-6;
    let (up, dn) = (seqf(&callv(&curve, &[nv(t + h)])?)?, seqf(&callv(&curve, &[nv(t - h)])?)?);
    Ok(vlist(up.iter().zip(&dn).map(|(p, m)| (p - m) / (2.0 * h)).collect()))
}

pub fn open_ball(a: &[Value]) -> R {
    // d, x, y, r
    let (d, x, y, r) = (func(&a[0])?, a[1].clone(), a[2].clone(), num(&a[3])?);
    b(real(&callv(&d, &[x, y])?)? < r)
}

// ---------------------------------------------------------------- differential equations
pub fn differential_equation(a: &[Value]) -> R {
    let (f, x0, y0, xe) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?);
    n(rk4_scalar(&f, x0, y0, xe)?)
}

pub fn first_order_ode(a: &[Value]) -> R {
    differential_equation(a)
}

pub fn separable_equation(a: &[Value]) -> R {
    // g, h, x0, y0, x_end
    let (g, h, x0, y0, xe) = (func(&a[0])?, func(&a[1])?, num(&a[2])?, num(&a[3])?, num(&a[4])?);
    let steps = 1000;
    let hh = (xe - x0) / steps as f64;
    let rhs = |x: f64, y: f64| -> Result<f64, String> { Ok(call(&g, &[x])? * call(&h, &[y])?) };
    let (mut x, mut y) = (x0, y0);
    for _ in 0..steps {
        let k1 = rhs(x, y)?;
        let k2 = rhs(x + hh / 2.0, y + hh / 2.0 * k1)?;
        let k3 = rhs(x + hh / 2.0, y + hh / 2.0 * k2)?;
        let k4 = rhs(x + hh, y + hh * k3)?;
        y += hh / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        x += hh;
    }
    n(y)
}

pub fn second_order_ode(a: &[Value]) -> R {
    // f, x0, y0, v0, x_end
    let (f, x0, y0, v0, xe) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, num(&a[4])?);
    let steps = 1000;
    let h = (xe - x0) / steps as f64;
    let rhs = |x: f64, s: &[f64]| -> Result<Vec<f64>, String> { Ok(vec![s[1], call(&f, &[x, s[0], s[1]])?]) };
    let add = |y: &[f64], k: &[f64], sc: f64| -> Vec<f64> { y.iter().zip(k).map(|(p, q)| p + sc * q).collect() };
    let (mut x, mut y) = (x0, vec![y0, v0]);
    for _ in 0..steps {
        let k1 = rhs(x, &y)?;
        let k2 = rhs(x + h / 2.0, &add(&y, &k1, h / 2.0))?;
        let k3 = rhs(x + h / 2.0, &add(&y, &k2, h / 2.0))?;
        let k4 = rhs(x + h, &add(&y, &k3, h))?;
        y = (0..y.len()).map(|i| y[i] + h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])).collect();
        x += h;
    }
    n(y[0])
}

pub fn euler_s_method(a: &[Value]) -> R {
    // f, x0, y0, h, steps
    let (f, x0, y0, h, steps) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    let mut pts = vec![Value::Tuple(vec![nv(x0), nv(y0)])];
    let (mut x, mut y) = (x0, y0);
    for _ in 0..steps.max(0) {
        y += h * call(&f, &[x, y])?;
        x += h;
        pts.push(Value::Tuple(vec![nv(x), nv(y)]));
    }
    Ok(Value::List(pts))
}

pub fn runge_kutta_rk4(a: &[Value]) -> R {
    let (f, x0, y0, h, steps) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    let mut pts = vec![Value::Tuple(vec![nv(x0), nv(y0)])];
    let (mut x, mut y) = (x0, y0);
    for _ in 0..steps.max(0) {
        let k1 = call(&f, &[x, y])?;
        let k2 = call(&f, &[x + h / 2.0, y + h * k1 / 2.0])?;
        let k3 = call(&f, &[x + h / 2.0, y + h * k2 / 2.0])?;
        let k4 = call(&f, &[x + h, y + h * k3])?;
        y += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        x += h;
        pts.push(Value::Tuple(vec![nv(x), nv(y)]));
    }
    Ok(Value::List(pts))
}

pub fn fourier_series(a: &[Value]) -> R {
    let (f, k) = (func(&a[0])?, int_of(num(&a[1])?)? as f64);
    let ca = simpson(|x| Ok(call(&f, &[x])? * (k * x).cos()), -PI, PI, 4000)? / PI;
    let cb = simpson(|x| Ok(call(&f, &[x])? * (k * x).sin()), -PI, PI, 4000)? / PI;
    Ok(Value::Tuple(vec![nv(ca), nv(cb)]))
}

pub fn integrating_factor(a: &[Value]) -> R {
    let (p, x0, x) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    let e = simpson(|t| call(&p, &[t]), x0, x, 1000)?;
    let r = e.exp();
    need(r.is_finite(), "Numerical result out of range")?;
    n(r)
}

pub fn laplace_transform(a: &[Value]) -> R {
    let (f, s) = (func(&a[0])?, num(&a[1])?);
    need(s > 0.0, "s must be positive")?;
    let t_max = 20.0_f64.max(40.0 / s);
    n(simpson(|t| Ok(call(&f, &[t])? * (-s * t).exp()), 0.0, t_max, 20000)?)
}

pub fn variation_of_parameters(a: &[Value]) -> R {
    // y1, y2, g, x0, x
    let (y1, y2, g, x0, x) = (func(&a[0])?, func(&a[1])?, func(&a[2])?, num(&a[3])?, num(&a[4])?);
    let d = |ff: &Callable, t: f64| -> Result<f64, String> {
        let h = 1e-5;
        Ok((call(ff, &[t + h])? - call(ff, &[t - h])?) / (2.0 * h))
    };
    let w = |t: f64| -> Result<f64, String> { Ok(call(&y1, &[t])? * d(&y2, t)? - d(&y1, t)? * call(&y2, &[t])?) };
    let i1 = simpson(|t| div(call(&y2, &[t])? * call(&g, &[t])?, w(t)?), x0, x, 1000)?;
    let i2 = simpson(|t| div(call(&y1, &[t])? * call(&g, &[t])?, w(t)?), x0, x, 1000)?;
    n(-call(&y1, &[x])? * i1 + call(&y2, &[x])? * i2)
}

// ---------------------------------------------------------------- root finding and optimisation
pub fn bisection_method(a: &[Value]) -> R {
    // f, a, b, tol, max_iter
    let (f, mut lo, mut hi, tol, max_iter) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    let mut fa = call(&f, &[lo])?;
    need(!(fa * call(&f, &[hi])? > 0.0), "f(a) and f(b) must have opposite signs")?;
    for _ in 0..max_iter.max(0) {
        let mid = (lo + hi) / 2.0;
        let fm = call(&f, &[mid])?;
        if fm.abs() < tol || (hi - lo) / 2.0 < tol {
            return n(mid);
        }
        if fa * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            fa = fm;
        }
    }
    n((lo + hi) / 2.0)
}

pub fn intermediate_value_theorem(a: &[Value]) -> R {
    let (f, mut lo, mut hi, y) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?);
    let (fa, fb) = (call(&f, &[lo])? - y, call(&f, &[hi])? - y);
    need(!(fa * fb > 0.0), "y is not between f(a) and f(b)")?;
    for _ in 0..100 {
        let m = (lo + hi) / 2.0;
        if (call(&f, &[lo])? - y) * (call(&f, &[m])? - y) <= 0.0 {
            hi = m;
        } else {
            lo = m;
        }
    }
    n((lo + hi) / 2.0)
}

pub fn fixed_point_iteration(a: &[Value]) -> R {
    // g, x0, tol, max_iter
    let (g, x0, tol, max_iter) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, whole(num(&a[3])?)?);
    let mut x = x0;
    for _ in 0..max_iter.max(0) {
        let nx = call(&g, &[x])?;
        if (nx - x).abs() < tol {
            return n(nx);
        }
        x = nx;
    }
    Err("fixed-point iteration did not converge".into())
}

pub fn newton_s_method(a: &[Value]) -> R {
    // f, fprime, x0, tol, max_iter
    let (f, fp, x0, tol, max_iter) = (func(&a[0])?, func(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    let mut x = x0;
    for _ in 0..max_iter.max(0) {
        let step = div(call(&f, &[x])?, call(&fp, &[x])?)?;
        x -= step;
        if step.abs() < tol {
            return n(x);
        }
    }
    Err("Newton's method did not converge".into())
}

pub fn secant_method(a: &[Value]) -> R {
    // f, x0, x1, tol, max_iter
    let (f, mut x0, mut x1, tol, max_iter) = (func(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?, whole(num(&a[4])?)?);
    for _ in 0..max_iter.max(0) {
        let (f0, f1) = (call(&f, &[x0])?, call(&f, &[x1])?);
        if f1 == f0 {
            break;
        }
        let nx = x1 - div(f1 * (x1 - x0), f1 - f0)?;
        x0 = x1;
        x1 = nx;
        if (x1 - x0).abs() < tol {
            return n(x1);
        }
    }
    n(x1)
}

fn golden(a: &[Value], maximise: bool) -> R {
    let (f, lo, hi) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    let g = (5.0_f64.sqrt() - 1.0) / 2.0;
    let (mut p, mut q) = (lo, hi);
    for _ in 0..200 {
        let (c, d) = (q - g * (q - p), p + g * (q - p));
        let (fc, fd) = (call(&f, &[c])?, call(&f, &[d])?);
        let left_better = if maximise { -fc < -fd } else { fc < fd };
        if left_better {
            q = d;
        } else {
            p = c;
        }
    }
    let x = (p + q) / 2.0;
    Ok(Value::Tuple(vec![nv(x), nv(call(&f, &[x])?)]))
}

pub fn minimize(a: &[Value]) -> R {
    golden(a, false)
}

pub fn maximize(a: &[Value]) -> R {
    golden(a, true)
}

pub fn gradient_descent(a: &[Value]) -> R {
    // grad_f, x0, lr, steps
    let (g, x0, lr, steps) = (func(&a[0])?, nums(&a[1])?, num(&a[2])?, whole(num(&a[3])?)?);
    let mut x = x0;
    for _ in 0..steps.max(0) {
        let gv = seqf(&callv(&g, &[vlist(x.clone())])?)?;
        x = x.iter().zip(&gv).map(|(xi, gi)| xi - lr * gi).collect();
    }
    Ok(vlist(x))
}

pub fn convex_function(a: &[Value]) -> R {
    let (f, x, y) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    for k in 0..101 {
        let t = k as f64 / 100.0;
        if call(&f, &[t * x + (1.0 - t) * y])? > t * call(&f, &[x])? + (1.0 - t) * call(&f, &[y])? + 1e-9 {
            return b(false);
        }
    }
    b(true)
}

pub fn solve_for_x(a: &[Value]) -> R {
    let f = func(&a[0])?;
    let (nn, lo, hi) = (40000, -100.0_f64, 100.0_f64);
    let mut roots: Vec<f64> = vec![];
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..=nn {
        let x = lo + (hi - lo) * i as f64 / nn as f64;
        let yv = match callv(&f, &[nv(x)]) {
            Ok(v) => v,
            Err(_) => {
                prev = None;
                continue;
            }
        };
        let y = match real(&yv) {
            Ok(y) => y,
            Err(e) => {
                // a complex value: |y| can still be tested, but ordering it is an error
                let m = match yv {
                    Value::Complex(re, im) => re.hypot(im),
                    _ => return Err(e),
                };
                if m < 1e-12 {
                    roots.push(x);
                    prev = None;
                    continue;
                }
                if prev.is_some() {
                    return Err(e);
                }
                prev = None;
                continue;
            }
        };
        if y.abs() < 1e-12 {
            roots.push(x);
        } else if let Some((px, py)) = prev {
            if py * y < 0.0 {
                let (mut p, mut q) = (px, x);
                for _ in 0..80 {
                    let m = (p + q) / 2.0;
                    if call(&f, &[p])? * call(&f, &[m])? <= 0.0 {
                        q = m;
                    } else {
                        p = m;
                    }
                }
                roots.push((p + q) / 2.0);
            }
        }
        prev = Some((x, y));
    }
    let mut out: Vec<f64> = vec![];
    for r in roots {
        let r = format!("{:.9}", r).parse::<f64>().unwrap_or(r);
        if out.is_empty() || (r - out[out.len() - 1]).abs() > 1e-6 {
            out.push(r);
        }
    }
    Ok(vlist(out))
}

// ---------------------------------------------------------------- complex contour integrals
pub fn contour_integral(a: &[Value]) -> R {
    let (f, center, radius) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    cresult(circle_integral(|z| cplx_of(&callv(&f, &[Value::Complex(z.0, z.1)])?), center, radius)?)
}

pub fn residue_theorem(a: &[Value]) -> R {
    contour_integral(a)
}

pub fn residue(a: &[Value]) -> R {
    // f, a, radius
    let (f, center, radius) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    let t = circle_integral(|z| cplx_of(&callv(&f, &[Value::Complex(z.0, z.1)])?), center, radius)?;
    cresult(cdivide(t, (0.0, 2.0 * PI))?)
}

pub fn cauchy_integral_formula(a: &[Value]) -> R {
    let (f, center, radius) = (func(&a[0])?, num(&a[1])?, num(&a[2])?);
    let g = |z: C| -> Result<C, String> { cdivide(cplx_of(&callv(&f, &[Value::Complex(z.0, z.1)])?)?, csub(z, (center, 0.0))) };
    let t = circle_integral(g, center, radius)?;
    cresult(cdivide(t, (0.0, 2.0 * PI))?)
}

// ---------------------------------------------------------------- functions on sets, and group tests
fn set_of_items(v: &Value) -> Result<Vec<Value>, String> {
    match v {
        Value::Set(a) | Value::List(a) | Value::Tuple(a) => Ok(a.clone()),
        other => Err(format!("expected a set, got {}", other.kind_name())),
    }
}

fn in_list(l: &[Value], x: &Value) -> bool {
    l.iter().any(|y| y.same_item(x))
}

fn apply1(f: &Callable, x: &Value) -> Result<Value, String> {
    callv(f, &[x.clone()])
}

fn apply2(f: &Callable, x: &Value, y: &Value) -> Result<Value, String> {
    callv(f, &[x.clone(), y.clone()])
}

/// Python can only put hashable things in a set: numbers, text, tuples of those; a list is a TypeError.
fn hashable(v: Value) -> Result<Value, String> {
    match &v {
        Value::List(_) | Value::Set(_) | Value::Dict(_) | Value::Func(_) => Err("unhashable type".into()),
        Value::Tuple(t) => t.iter().try_for_each(|x| hashable(x.clone()).map(|_| ())).map(|_| v.clone()),
        _ => Ok(v),
    }
}

fn image_set(f: &Callable, dom: &[Value]) -> Result<Vec<Value>, String> {
    let mut v = vec![];
    for x in dom {
        v.push(hashable(apply1(f, x)?)?);
    }
    match Value::set_of(v) {
        Value::Set(items) => Ok(items),
        _ => unreachable!(),
    }
}

fn same_set(a: &[Value], c: &[Value]) -> bool {
    a.len() == c.len() && a.iter().all(|x| in_list(c, x))
}

pub fn function(a: &[Value]) -> R {
    let f = func(&a[0])?;
    callv(&f, &[a[1].clone()])
}

pub fn injective(a: &[Value]) -> R {
    let (f, dom) = (func(&a[0])?, set_of_items(&a[1])?);
    b(image_set(&f, &dom)?.len() == dom.len())
}

pub fn surjective(a: &[Value]) -> R {
    // f, domain, codomain
    let (f, dom, cod) = (func(&a[0])?, set_of_items(&a[1])?, set_of_items(&a[2])?);
    b(same_set(&image_set(&f, &dom)?, &cod))
}

pub fn bijective(a: &[Value]) -> R {
    let (f, dom, cod) = (func(&a[0])?, set_of_items(&a[1])?, set_of_items(&a[2])?);
    let img = image_set(&f, &dom)?;
    b(img.len() == dom.len() && same_set(&img, &cod))
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Number(x) => *x != 0.0,
        Value::Bool(t) => *t,
        Value::Complex(p, q) => *p != 0.0 || *q != 0.0,
        Value::List(x) | Value::Tuple(x) | Value::Set(x) => !x.is_empty(),
        Value::Text(s) => !s.is_empty(),
        _ => true,
    }
}

pub fn for_all(a: &[Value]) -> R {
    let (p, dom) = (func(&a[0])?, set_of_items(&a[1])?);
    for x in &dom {
        if !truthy(&apply1(&p, x)?) {
            return b(false);
        }
    }
    b(true)
}

pub fn there_exists(a: &[Value]) -> R {
    let (p, dom) = (func(&a[0])?, set_of_items(&a[1])?);
    for x in &dom {
        if truthy(&apply1(&p, x)?) {
            return b(true);
        }
    }
    b(false)
}

pub fn coset(a: &[Value]) -> R {
    // g, H, op
    let (h, op) = (set_of_items(&a[1])?, func(&a[2])?);
    let mut v = vec![];
    for x in &h {
        v.push(hashable(apply2(&op, &a[0], x)?)?);
    }
    Ok(Value::set_of(v))
}

pub fn image(a: &[Value]) -> R {
    // phi, G
    let (phi, g) = (func(&a[0])?, set_of_items(&a[1])?);
    Ok(Value::Set(image_set(&phi, &g)?))
}

pub fn kernel(a: &[Value]) -> R {
    // phi, G, identity
    let (phi, g) = (func(&a[0])?, set_of_items(&a[1])?);
    let mut out = vec![];
    for x in &g {
        if apply1(&phi, x)?.same_item(&a[2]) {
            out.push(x.clone());
        }
    }
    Ok(Value::set_of(out))
}

pub fn cyclic_group(a: &[Value]) -> R {
    // g, op, identity
    let (op, ident) = (func(&a[1])?, &a[2]);
    let mut out = vec![a[0].clone()];
    let mut x = a[0].clone();
    for _ in 0..10000 {
        if x.same_item(ident) {
            return Ok(Value::List(out));
        }
        x = apply2(&op, &x, &a[0])?;
        // a group element is a number or a name; a list that keeps growing would never return to the identity
        need(matches!(x, Value::Number(_) | Value::Text(_) | Value::Bool(_) | Value::Complex(..)), "g never returns to the identity")?;
        out.push(x.clone());
    }
    Err("g never returns to the identity".into())
}

pub fn order_of_element(a: &[Value]) -> R {
    // g, op, identity
    let (op, ident) = (func(&a[1])?, &a[2]);
    let (mut x, mut k) = (a[0].clone(), 1.0);
    let mut guard = 0;
    while !x.same_item(ident) {
        x = apply2(&op, &x, &a[0])?;
        need(matches!(x, Value::Number(_) | Value::Text(_) | Value::Bool(_) | Value::Complex(..)), "g never returns to the identity")?;
        k += 1.0;
        guard += 1;
        need(guard < 1_000_000, "g never returns to the identity")?;
    }
    n(k)
}

fn identities(g: &[Value], op: &Callable) -> Result<Vec<Value>, String> {
    let mut ids = vec![];
    for e in g {
        let mut ok = true;
        for x in g {
            if !(apply2(op, e, x)?.same_item(x) && apply2(op, x, e)?.same_item(x)) {
                ok = false;
                break;
            }
        }
        if ok {
            ids.push(e.clone());
        }
    }
    Ok(ids)
}

pub fn group(a: &[Value]) -> R {
    // G, op
    let (g, op) = (set_of_items(&a[0])?, func(&a[1])?);
    for x in &g {
        for y in &g {
            if !in_list(&g, &apply2(&op, x, y)?) {
                return b(false);
            }
        }
    }
    for x in &g {
        for y in &g {
            for z in &g {
                if !apply2(&op, &apply2(&op, x, y)?, z)?.same_item(&apply2(&op, x, &apply2(&op, y, z)?)?) {
                    return b(false);
                }
            }
        }
    }
    let ids = identities(&g, &op)?;
    if ids.is_empty() {
        return b(false);
    }
    for x in &g {
        let mut has_inverse = false;
        for y in &g {
            if apply2(&op, x, y)?.same_item(&ids[0]) {
                has_inverse = true;
                break;
            }
        }
        if !has_inverse {
            return b(false);
        }
    }
    b(true)
}

pub fn subgroup(a: &[Value]) -> R {
    // H, G, op
    let (h, g, op) = (set_of_items(&a[0])?, set_of_items(&a[1])?, func(&a[2])?);
    if h.is_empty() || !h.iter().all(|x| in_list(&g, x)) {
        return b(false);
    }
    for x in &h {
        for y in &h {
            if !in_list(&h, &apply2(&op, x, y)?) {
                return b(false);
            }
        }
    }
    let ids = identities(&g, &op)?;
    if ids.is_empty() || !in_list(&h, &ids[0]) {
        return b(false);
    }
    for x in &h {
        let mut has = false;
        for y in &h {
            if apply2(&op, x, y)?.same_item(&ids[0]) {
                has = true;
                break;
            }
        }
        if !has {
            return b(false);
        }
    }
    b(true)
}

pub fn normal_subgroup(a: &[Value]) -> R {
    // N, G, op
    let (nn, g, op) = (set_of_items(&a[0])?, set_of_items(&a[1])?, func(&a[2])?);
    let ids = identities(&g, &op)?;
    if ids.is_empty() || !in_list(&nn, &ids[0]) {
        return b(false);
    }
    for x in &nn {
        for y in &nn {
            if !in_list(&nn, &apply2(&op, x, y)?) {
                return b(false);
            }
        }
    }
    for x in &g {
        let mut inv = None;
        for h in &g {
            if apply2(&op, x, h)?.same_item(&ids[0]) {
                inv = Some(h.clone());
                break;
            }
        }
        for y in &nn {
            let inv = inv.clone().ok_or("no inverse")?;
            if !in_list(&nn, &apply2(&op, &apply2(&op, x, y)?, &inv)?) {
                return b(false);
            }
        }
    }
    b(true)
}

pub fn homomorphism(a: &[Value]) -> R {
    // phi, G, op1, op2
    let (phi, g, op1, op2) = (func(&a[0])?, set_of_items(&a[1])?, func(&a[2])?, func(&a[3])?);
    for x in &g {
        for y in &g {
            let lhs = apply1(&phi, &apply2(&op1, x, y)?)?;
            let rhs = apply2(&op2, &apply1(&phi, x)?, &apply1(&phi, y)?)?;
            if !lhs.same_item(&rhs) {
                return b(false);
            }
        }
    }
    b(true)
}

pub fn isomorphism(a: &[Value]) -> R {
    // phi, G, H_, op1, op2
    let (phi, g, h, op1, op2) = (func(&a[0])?, set_of_items(&a[1])?, set_of_items(&a[2])?, func(&a[3])?, func(&a[4])?);
    let images: Vec<Value> = g.iter().map(|x| apply1(&phi, x)).collect::<Result<_, _>>()?;
    let img_set = match Value::set_of(images) {
        Value::Set(i) => i,
        _ => unreachable!(),
    };
    if img_set.len() != g.len() || !same_set(&img_set, &h) {
        return b(false);
    }
    homomorphism(&[a[0].clone(), a[1].clone(), a[3].clone(), a[4].clone()]).map(|v| {
        let _ = (&op1, &op2);
        v
    })
}
