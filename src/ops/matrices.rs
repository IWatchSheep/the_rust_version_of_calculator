//! Operators on matrices (and the vector-space operators that read a list of vectors as the rows of a matrix).
//! Written to follow the Python line by line, including what happens with shapes that do not fit: Python's
//! `zip` stops at the shorter list and an out-of-range index is an error, so the same is done here.

use super::vals::*;
use super::lists::{quat_multiply, quat_size, quaternion, vector3};
use crate::value::Value;

fn row_sub(a: &[f64], bb: &[f64], f: f64) -> Vec<f64> {
    a.iter().zip(bb).map(|(x, y)| x - f * y).collect()
}

fn py_round(x: f64, digits: usize) -> f64 {
    format!("{:.*}", digits, x).parse::<f64>().unwrap_or(x)
}

fn cell(m: &Mat, i: usize, j: usize) -> Result<f64, String> {
    m.get(i).and_then(|r| r.get(j)).copied().ok_or_else(|| "list index out of range".to_string())
}

fn index_of_first_max(range: std::ops::Range<usize>, key: impl Fn(usize) -> Result<f64, String>) -> Result<usize, String> {
    let mut best: Option<(usize, f64)> = None;
    for r in range {
        let k = key(r)?;
        if best.map_or(true, |(_, bk)| k > bk) {
            best = Some((r, k));
        }
    }
    best.map(|(r, _)| r).ok_or_else(|| "max() of an empty range".to_string())
}

// ---------------------------------------------------------------- vector spaces
pub fn basis(a: &[Value]) -> R {
    let v = mat(&a[0])?;
    let (_, piv) = rref(&transpose_m(&v))?;
    let mut out = vec![];
    for i in piv {
        out.push(at(&v, i as i64)?);
    }
    Ok(vmat(out))
}

pub fn dimension(a: &[Value]) -> R {
    let (_, piv) = rref(&mat(&a[0])?)?;
    n(piv.len() as f64)
}

pub fn column_space(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let (_, piv) = rref(&m)?;
    let cols = transpose_m(&m);
    let mut out = vec![];
    for p in piv {
        out.push(at(&cols, p as i64)?);
    }
    Ok(vmat(out))
}

pub fn null_space(a: &[Value]) -> R {
    let basis = nullspace(&mat(&a[0])?)?;
    Ok(vmat(basis.into_iter().map(|v| v.into_iter().map(|c| py_round(c, 10)).collect()).collect()))
}

// ---------------------------------------------------------------- arithmetic on matrices
fn zip_map(x: &Mat, y: &Mat, f: impl Fn(f64, f64) -> f64) -> Mat {
    x.iter().zip(y).map(|(r1, r2)| r1.iter().zip(r2).map(|(p, q)| f(*p, *q)).collect()).collect()
}

pub fn matrix_add(a: &[Value]) -> R {
    Ok(vmat(zip_map(&mat(&a[0])?, &mat(&a[1])?, |p, q| p + q)))
}

pub fn matrix_subtract(a: &[Value]) -> R {
    Ok(vmat(zip_map(&mat(&a[0])?, &mat(&a[1])?, |p, q| p - q)))
}

pub fn matrix_multiply(a: &[Value]) -> R {
    Ok(vmat(matmul(&mat(&a[0])?, &mat(&a[1])?)))
}

pub fn scalar_multiply(a: &[Value]) -> R {
    let k = num(&a[0])?;
    Ok(vmat(mat(&a[1])?.into_iter().map(|r| r.into_iter().map(|x| k * x).collect()).collect()))
}

pub fn transpose(a: &[Value]) -> R {
    Ok(vmat(transpose_m(&mat(&a[0])?)))
}

pub fn trace(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let mut s = 0.0;
    for i in 0..m.len() {
        s += cell(&m, i, i)?;
    }
    n(s)
}

pub fn tensor_contraction(a: &[Value]) -> R {
    trace(a)
}

pub fn metric_tensor(a: &[Value]) -> R {
    let (g, v, w) = (mat(&a[0])?, nums(&a[1])?, nums(&a[2])?);
    let mut s = 0.0;
    for i in 0..v.len() {
        for j in 0..w.len() {
            s += v[i] * cell(&g, i, j)? * w[j];
        }
    }
    n(s)
}

pub fn determinant(a: &[Value]) -> R {
    fn det(m: &Mat) -> Result<f64, String> {
        let nn = m.len();
        if nn == 1 {
            return Ok(cell(m, 0, 0)?);
        }
        let mut s = 0.0;
        for j in 0..nn {
            let minor: Mat = m[1..].iter().map(|row| row.iter().enumerate().filter(|(k, _)| *k != j).map(|(_, v)| *v).collect()).collect();
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            s += sign * cell(m, 0, j)? * det(&minor)?;
        }
        Ok(s)
    }
    n(det(&mat(&a[0])?)?)
}

pub fn characteristic_polynomial(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let (a00, a01, a10, a11) = (cell(&m, 0, 0)?, cell(&m, 0, 1)?, cell(&m, 1, 0)?, cell(&m, 1, 1)?);
    Ok(vlist(vec![1.0, -(a00 + a11), a00 * a11 - a01 * a10]))
}

pub fn eigenvalues(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let (a00, a01, a10, a11) = (cell(&m, 0, 0)?, cell(&m, 0, 1)?, cell(&m, 1, 0)?, cell(&m, 1, 1)?);
    let tr = a00 + a11;
    let det = a00 * a11 - a01 * a10;
    let d = tr * tr - 4.0 * det;
    // cmath.sqrt of a real number
    let (rre, rim) = if d >= 0.0 { (d.sqrt(), 0.0) } else { (0.0, (-d).sqrt()) };
    Ok(Value::Tuple(vec![Value::Complex((tr + rre) / 2.0, rim / 2.0), Value::Complex((tr - rre) / 2.0, -rim / 2.0)]))
}

pub fn eigenvectors(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let nn = m.len();
    let mut vals: Vec<f64> = vec![];
    let add_unique = |vals: &mut Vec<f64>, v: f64| {
        if !vals.contains(&v) {
            vals.push(v);
        }
    };
    if nn == 2 {
        let (a00, a01, a10, a11) = (cell(&m, 0, 0)?, cell(&m, 0, 1)?, cell(&m, 1, 0)?, cell(&m, 1, 1)?);
        let tr = a00 + a11;
        let det = a00 * a11 - a01 * a10;
        let disc = tr * tr / 4.0 - det;
        need(disc >= -1e-12, "the eigenvalues are complex")?;
        let s = disc.max(0.0).sqrt();
        add_unique(&mut vals, py_round(tr / 2.0 + s, 10));
        add_unique(&mut vals, py_round(tr / 2.0 - s, 10));
    } else {
        let mut sym = true;
        for i in 0..nn {
            for j in 0..nn {
                if (cell(&m, i, j)? - cell(&m, j, i)?).abs() >= 1e-12 {
                    sym = false;
                }
            }
        }
        need(sym, "only 2×2 or symmetric matrices")?;
        for v in jacobi_eigen(&m, 100) {
            add_unique(&mut vals, py_round(v, 8));
        }
    }
    vals.sort_by(|x, y| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = vec![];
    for lam in vals {
        let shifted: Mat = (0..nn)
            .map(|i| (0..nn).map(|j| Ok(cell(&m, i, j)? - if i == j { lam } else { 0.0 })).collect::<Result<Vec<f64>, String>>())
            .collect::<Result<_, String>>()?;
        let vecs = nullspace(&shifted)?;
        let rounded: Mat = vecs.into_iter().map(|v| v.into_iter().map(|c| py_round(c, 10)).collect()).collect();
        out.push(Value::Tuple(vec![Value::Number(lam), vmat(rounded)]));
    }
    Ok(Value::List(out))
}

// ---------------------------------------------------------------- decompositions
pub fn lu_decomposition(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let nn = m.len();
    let mut l = vec![vec![0.0; nn]; nn];
    let mut u = vec![vec![0.0; nn]; nn];
    for i in 0..nn {
        for k in i..nn {
            let mut s = 0.0;
            for j in 0..i {
                s += l[i][j] * u[j][k];
            }
            u[i][k] = cell(&m, i, k)? - s;
        }
        for k in i..nn {
            if i == k {
                l[i][i] = 1.0;
            } else {
                need(u[i][i].abs() >= 1e-12, "a zero pivot: this matrix needs row swaps")?;
                let mut s = 0.0;
                for j in 0..i {
                    s += l[k][j] * u[j][i];
                }
                l[k][i] = (cell(&m, k, i)? - s) / u[i][i];
            }
        }
    }
    Ok(Value::Tuple(vec![vmat(l), vmat(u)]))
}

pub fn qr_decomposition(a: &[Value]) -> R {
    let cols = transpose_m(&mat(&a[0])?);
    let nn = cols.len();
    let mut q: Mat = vec![];
    let mut r = vec![vec![0.0; nn]; nn];
    for (j, v) in cols.iter().enumerate() {
        let mut w = v.clone();
        for (i, qi) in q.iter().enumerate() {
            r[i][j] = qi.iter().zip(v).map(|(x, y)| x * y).sum();
            w = w.iter().zip(qi).map(|(x, y)| x - r[i][j] * y).collect();
        }
        r[j][j] = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        need(r[j][j] >= 1e-12, "the columns are not independent")?;
        let d = r[j][j];
        q.push(w.iter().map(|x| x / d).collect());
    }
    Ok(Value::Tuple(vec![vmat(transpose_m(&q)), vmat(r)]))
}

pub fn singular_value_decomposition(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let ata = matmul(&transpose_m(&m), &m);
    Ok(vlist(jacobi_eigen(&ata, 100).into_iter().map(|v| v.max(0.0).sqrt()).collect()))
}

// ---------------------------------------------------------------- inverse, rank, row reduction
pub fn matrix_inverse(a: &[Value]) -> R {
    let m = mat(&a[0])?;
    let nn = m.len();
    let mut mm: Mat = m
        .iter()
        .enumerate()
        .map(|(i, row)| row.iter().cloned().chain((0..nn).map(|j| if i == j { 1.0 } else { 0.0 })).collect())
        .collect();
    for c in 0..nn {
        let p = index_of_first_max(c..nn, |r| Ok(cell(&mm, r, c)?.abs()))?;
        need(cell(&mm, p, c)?.abs() >= 1e-12, "matrix is singular")?;
        mm.swap(c, p);
        let pivot = cell(&mm, c, c)?;
        mm[c] = mm[c].iter().map(|x| x / pivot).collect();
        for r in 0..nn {
            if r != c {
                let f = cell(&mm, r, c)?;
                let pc = mm[c].clone();
                mm[r] = row_sub(&mm[r], &pc, f);
            }
        }
    }
    Ok(vmat(mm.into_iter().map(|row| row.into_iter().skip(nn).collect()).collect()))
}

pub fn matrix_rank(a: &[Value]) -> R {
    let mut m = mat(&a[0])?;
    need(!m.is_empty(), "list index out of range")?;
    let (rows, cols) = (m.len(), m[0].len());
    let mut rank = 0;
    for c in 0..cols {
        let mut p = None;
        for r in rank..rows {
            if cell(&m, r, c)?.abs() > 1e-12 {
                p = Some(r);
                break;
            }
        }
        let Some(p) = p else { continue };
        m.swap(rank, p);
        for r in rank + 1..rows {
            let f = div(cell(&m, r, c)?, cell(&m, rank, c)?)?;
            let pr = m[rank].clone();
            m[r] = row_sub(&m[r], &pr, f);
        }
        rank += 1;
    }
    n(rank as f64)
}

pub fn row_reduce_rref(a: &[Value]) -> R {
    let mut m = mat(&a[0])?;
    need(!m.is_empty(), "list index out of range")?;
    let (rows, cols) = (m.len(), m[0].len());
    let mut r = 0;
    for c in 0..cols {
        let mut p = None;
        for i in r..rows {
            if cell(&m, i, c)?.abs() > 1e-12 {
                p = Some(i);
                break;
            }
        }
        let Some(p) = p else { continue };
        m.swap(r, p);
        let pivot = cell(&m, r, c)?;
        m[r] = m[r].iter().map(|x| x / pivot).collect();
        for i in 0..rows {
            if i != r {
                let f = cell(&m, i, c)?;
                let pr = m[r].clone();
                m[i] = row_sub(&m[i], &pr, f);
            }
        }
        r += 1;
        if r == rows {
            break;
        }
    }
    Ok(vmat(m))
}

// ---------------------------------------------------------------- linear systems
fn solve_by_elimination(m: &Mat, b: &[f64]) -> Result<Vec<f64>, String> {
    let nn = m.len();
    let mut mm: Mat = vec![];
    for (i, row) in m.iter().enumerate() {
        let mut r = row.clone();
        r.push(at(b, i as i64)?);
        mm.push(r);
    }
    for c in 0..nn {
        let p = index_of_first_max(c..nn, |r| Ok(cell(&mm, r, c)?.abs()))?;
        need(cell(&mm, p, c)?.abs() >= 1e-12, "matrix is singular")?;
        mm.swap(c, p);
        for r in c + 1..nn {
            let f = div(cell(&mm, r, c)?, cell(&mm, c, c)?)?;
            let pc = mm[c].clone();
            mm[r] = row_sub(&mm[r], &pc, f);
        }
    }
    let mut x = vec![0.0; nn];
    for i in (0..nn).rev() {
        let mut s = 0.0;
        for j in i + 1..nn {
            s += cell(&mm, i, j)? * x[j];
        }
        x[i] = div(cell(&mm, i, nn)? - s, cell(&mm, i, i)?)?;
    }
    Ok(x)
}

pub fn solve_ax_b(a: &[Value]) -> R {
    Ok(vlist(solve_by_elimination(&mat(&a[0])?, &nums(&a[1])?)?))
}

pub fn gaussian_elimination(a: &[Value]) -> R {
    solve_ax_b(a)
}

pub fn solve_system_of_equations(a: &[Value]) -> R {
    let (m, b) = (mat(&a[0])?, nums(&a[1])?);
    let nn = m.len();
    let rows: Mat = m.iter().zip(&b).map(|(row, bi)| row.iter().cloned().chain(std::iter::once(*bi)).collect()).collect();
    let (r, piv) = rref(&rows)?;
    need(piv == (0..nn).collect::<Vec<_>>(), "the system has no single solution")?;
    let mut out = vec![];
    for i in 0..nn {
        out.push(cell(&r, i, nn)?);
    }
    Ok(vlist(out))
}

pub fn jacobi_iteration(a: &[Value]) -> R {
    // A, b, iterations
    let (m, b, iters) = (mat(&a[0])?, nums(&a[1])?, whole(num(&a[2])?)?);
    let nn = b.len();
    let mut x = vec![0.0; nn];
    for _ in 0..iters.max(0) {
        let mut next = vec![];
        for i in 0..nn {
            let mut s = 0.0;
            for j in 0..nn {
                if j != i {
                    s += cell(&m, i, j)? * x[j];
                }
            }
            next.push(div(b[i] - s, cell(&m, i, i)?)?);
        }
        x = next;
    }
    Ok(vlist(x))
}

pub fn gauss_seidel_iteration(a: &[Value]) -> R {
    let (m, b, iters) = (mat(&a[0])?, nums(&a[1])?, whole(num(&a[2])?)?);
    let nn = b.len();
    let mut x = vec![0.0; nn];
    for _ in 0..iters.max(0) {
        for i in 0..nn {
            let mut s = 0.0;
            for j in 0..nn {
                if j != i {
                    s += cell(&m, i, j)? * x[j];
                }
            }
            x[i] = div(b[i] - s, cell(&m, i, i)?)?;
        }
    }
    Ok(vlist(x))
}

// ---------------------------------------------------------------- linear programming (simplex tableau)
fn simplex(a: &[Value]) -> R {
    let (c, am, b) = (nums(&a[0])?, mat(&a[1])?, nums(&a[2])?);
    let (m, nn) = (am.len(), c.len());
    need(!b.iter().any(|v| *v < 0.0), "every b must be ≥ 0")?;
    let mut t: Mat = vec![];
    for i in 0..m {
        let mut row = am[i].clone();
        row.extend((0..m).map(|j| if j == i { 1.0 } else { 0.0 }));
        row.push(at(&b, i as i64)?);
        t.push(row);
    }
    let mut obj: Vec<f64> = c.iter().map(|v| -v).collect();
    obj.extend(vec![0.0; m]);
    obj.push(0.0);
    t.push(obj);
    for _ in 0..500 {
        let last = t.len() - 1;
        let col = index_of_first_max(0..nn + m, |j| Ok(-cell(&t, last, j)?))?;
        if cell(&t, last, col)? >= -1e-12 {
            break;
        }
        let mut best: Option<(f64, usize)> = None;
        for i in 0..m {
            if cell(&t, i, col)? > 1e-12 {
                let ratio = div(*t[i].last().ok_or("empty row")?, cell(&t, i, col)?)?;
                if best.map_or(true, |(br, bi)| ratio < br || (ratio == br && i < bi)) {
                    best = Some((ratio, i));
                }
            }
        }
        let Some((_, r)) = best else { return Err("the problem is unbounded".into()) };
        let piv = cell(&t, r, col)?;
        t[r] = t[r].iter().map(|v| v / piv).collect();
        for i in 0..=m {
            if i != r {
                let f = cell(&t, i, col)?;
                let pr = t[r].clone();
                t[i] = row_sub(&t[i], &pr, f);
            }
        }
    }
    let last = t.len() - 1;
    let mut x = vec![0.0; nn];
    for j in 0..nn {
        let mut ones = vec![];
        for i in 0..m {
            if (cell(&t, i, j)? - 1.0).abs() < 1e-9 {
                let mut clean = true;
                for k in 0..m {
                    if k != i && cell(&t, k, j)?.abs() >= 1e-9 {
                        clean = false;
                    }
                }
                if clean {
                    ones.push(i);
                }
            }
        }
        if let Some(i) = ones.first() {
            x[j] = *t[*i].last().ok_or("empty row")?;
        }
    }
    Ok(Value::Tuple(vec![vlist(x), Value::Number(*t[last].last().ok_or("empty row")?)]))
}

pub fn linear_programming(a: &[Value]) -> R {
    simplex(a)
}

pub fn simplex_method(a: &[Value]) -> R {
    simplex(a)
}

// ---------- rotation matrices (the same operations in the same order as the Python files) ----------

fn matrix_of_quaternion(q: &[f64; 4]) -> Mat {
    let s = quat_size(q);
    let (w, x, y, z) = (q[0] / s, q[1] / s, q[2] / s, q[3] / s);
    vec![
        vec![1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w)],
        vec![2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w)],
        vec![2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

fn three_by_three(v: &Value) -> Result<[[f64; 3]; 3], String> {
    let m = mat(v)?;
    need(m.len() == 3 && m.iter().all(|r| r.len() == 3), "a rotation matrix has 3 rows of 3 numbers")?;
    Ok([[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]])
}

pub fn rotation_matrix_from_quaternion(a: &[Value]) -> R {
    let q = quaternion(&a[0])?;
    need(quat_size(&q) != 0.0, "the quaternion is zero, it cannot be a rotation")?;
    Ok(vmat(matrix_of_quaternion(&q)))
}

pub fn rotation_matrix_from_axis_angle(a: &[Value]) -> R {
    let axis = vector3(&a[0])?;
    let angle = num(&a[1])?;
    let size = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    need(size != 0.0, "the axis has length zero")?;
    let s = (angle / 2.0).sin();
    Ok(vmat(matrix_of_quaternion(&[(angle / 2.0).cos(), axis[0] / size * s, axis[1] / size * s, axis[2] / size * s])))
}

pub fn rotation_matrix_from_yaw_pitch_roll(a: &[Value]) -> R {
    let (yaw, pitch, roll) = (num(&a[0])?, num(&a[1])?, num(&a[2])?);
    let q_yaw = [(yaw / 2.0).cos(), 0.0, 0.0, (yaw / 2.0).sin()];
    let q_pitch = [(pitch / 2.0).cos(), 0.0, (pitch / 2.0).sin(), 0.0];
    let q_roll = [(roll / 2.0).cos(), (roll / 2.0).sin(), 0.0, 0.0];
    Ok(vmat(matrix_of_quaternion(&quat_multiply(&quat_multiply(&q_yaw, &q_pitch), &q_roll))))
}

pub fn rotate_vector_by_matrix(a: &[Value]) -> R {
    let r = three_by_three(&a[0])?;
    let v = vector3(&a[1])?;
    Ok(vlist(r.iter().map(|row| row[0] * v[0] + row[1] * v[1] + row[2] * v[2]).collect()))
}

pub fn rotation_matrix_to_quaternion(a: &[Value]) -> R {
    let r = three_by_three(&a[0])?;
    for i in 0..3 {
        for j in 0..3 {
            let dot = r[i][0] * r[j][0] + r[i][1] * r[j][1] + r[i][2] * r[j][2];
            need((dot - if i == j { 1.0 } else { 0.0 }).abs() <= 1e-8, "not a rotation matrix")?;
        }
    }
    let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1]) - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
        + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
    need((det - 1.0).abs() <= 1e-8, "not a rotation matrix")?;
    let trace = r[0][0] + r[1][1] + r[2][2];
    let mut q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [0.25 * s, (r[2][1] - r[1][2]) / s, (r[0][2] - r[2][0]) / s, (r[1][0] - r[0][1]) / s]
    } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
        let s = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
        [(r[2][1] - r[1][2]) / s, 0.25 * s, (r[0][1] + r[1][0]) / s, (r[0][2] + r[2][0]) / s]
    } else if r[1][1] > r[2][2] {
        let s = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
        [(r[0][2] - r[2][0]) / s, (r[0][1] + r[1][0]) / s, 0.25 * s, (r[1][2] + r[2][1]) / s]
    } else {
        let s = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
        [(r[1][0] - r[0][1]) / s, (r[0][2] + r[2][0]) / s, (r[1][2] + r[2][1]) / s, 0.25 * s]
    };
    if q[0] < 0.0 {
        q = [-q[0], -q[1], -q[2], -q[3]];
    }
    Ok(vlist(q.iter().map(|c| c + 0.0).collect()))
}
