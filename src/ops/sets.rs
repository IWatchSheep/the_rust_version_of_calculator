//! Set operators (logic and sets, topology on a finite space, products) and the interval-measure operators.
//! A set is `Value::Set` (items in a fixed order, no duplicates); a topology is a list of sets.

use super::vals::*;
use crate::value::Value;

fn set_items(v: &Value) -> Result<Vec<Value>, String> {
    match v {
        Value::Set(a) => Ok(a.clone()),
        Value::List(a) | Value::Tuple(a) => Ok(a.clone()),
        other => Err(format!("expected a set, got {}", other.kind_name())),
    }
}

fn contains(s: &[Value], x: &Value) -> bool {
    s.iter().any(|y| y.same_item(x))
}

fn subset(a: &[Value], b: &[Value]) -> bool {
    a.iter().all(|x| contains(b, x))
}

fn set_items_of(v: Vec<Value>) -> Vec<Value> {
    match Value::set_of(v) {
        Value::Set(items) => items,
        _ => unreachable!(),
    }
}

fn difference(a: &[Value], b: &[Value]) -> Vec<Value> {
    a.iter().filter(|x| !contains(b, x)).cloned().collect()
}

fn same_set(a: &[Value], b: &[Value]) -> bool {
    a.len() == b.len() && subset(a, b)
}

fn s(v: Vec<Value>) -> Value {
    Value::Set(v)
}

// ---------------------------------------------------------------- basic set operations
pub fn union(a: &[Value]) -> R {
    Ok(s(union_fn(&set_items(&a[0])?, &set_items(&a[1])?)))
}

fn union_fn(a: &[Value], b: &[Value]) -> Vec<Value> {
    let mut v = a.to_vec();
    v.extend(b.iter().cloned());
    set_items_of(v)
}

pub fn intersection(a: &[Value]) -> R {
    Ok(s(set_items_of(intersection_fn(&set_items(&a[0])?, &set_items(&a[1])?))))
}

fn intersection_fn(a: &[Value], b: &[Value]) -> Vec<Value> {
    a.iter().filter(|x| contains(b, x)).cloned().collect()
}

pub fn set_difference(a: &[Value]) -> R {
    Ok(s(difference(&set_items(&a[0])?, &set_items(&a[1])?)))
}

pub fn complement(a: &[Value]) -> R {
    // a, universe
    Ok(s(difference(&set_items(&a[1])?, &set_items(&a[0])?)))
}

pub fn subset_or_equal(a: &[Value]) -> R {
    b(subset(&set_items(&a[0])?, &set_items(&a[1])?))
}

pub fn proper_subset(a: &[Value]) -> R {
    let (x, y) = (set_items(&a[0])?, set_items(&a[1])?);
    b(subset(&x, &y) && x.len() < y.len())
}

pub fn element_of(a: &[Value]) -> R {
    b(contains(&set_items(&a[1])?, &a[0]))
}

pub fn not_element_of(a: &[Value]) -> R {
    b(!contains(&set_items(&a[1])?, &a[0]))
}

pub fn cardinality(a: &[Value]) -> R {
    n(set_items_of(set_items(&a[0])?).len() as f64)
}

pub fn order_of_group(a: &[Value]) -> R {
    cardinality(a)
}

pub fn cartesian_product(a: &[Value]) -> R {
    let (x, y) = (set_items(&a[0])?, set_items(&a[1])?);
    Ok(Value::List(x.iter().flat_map(|p| y.iter().map(move |q| Value::Tuple(vec![p.clone(), q.clone()]))).collect()))
}

pub fn direct_product(a: &[Value]) -> R {
    cartesian_product(a)
}

fn combinations(items: &[Value], size: usize, start: usize, cur: &mut Vec<Value>, out: &mut Vec<Value>) {
    if cur.len() == size {
        out.push(Value::Set(cur.clone()));
        return;
    }
    for i in start..items.len() {
        cur.push(items[i].clone());
        combinations(items, size, i + 1, cur, out);
        cur.pop();
    }
}

pub fn power_set(a: &[Value]) -> R {
    let items = set_items(&a[0])?;
    need(items.len() <= 16, "the set is too large for a power set")?;
    let mut out: Vec<Value> = vec![];
    for size in 0..=items.len() {
        combinations(&items, size, 0, &mut vec![], &mut out);
    }
    Ok(Value::List(out))
}

pub fn axiom_of_choice(a: &[Value]) -> R {
    let fam = items(&a[0])?;
    let mut out = vec![];
    for set in fam {
        let it = set_items(set)?;
        need(!it.is_empty(), "every set must be non-empty")?;
        let first = it.iter().min_by_key(|v| v.display()).unwrap().clone();
        out.push(first);
    }
    Ok(Value::List(out))
}

pub fn adjacency_matrix(a: &[Value]) -> R {
    // n, edges
    let nn = whole(num(&a[0])?)?;
    let edges = items(&a[1])?;
    let has = |i: i64, j: i64| {
        edges.iter().any(|e| match e {
            Value::Tuple(t) if t.len() == 2 => t[0].same_item(&Value::Number(i as f64)) && t[1].same_item(&Value::Number(j as f64)),
            _ => false,
        })
    };
    let mut out = vec![];
    for i in 0..nn.max(0) {
        out.push(vlist((0..nn).map(|j| if has(i, j) || has(j, i) { 1.0 } else { 0.0 }).collect()));
    }
    Ok(Value::List(out))
}

// ---------------------------------------------------------------- topology on a finite set
fn opens(tau: &Value) -> Result<Vec<Vec<Value>>, String> {
    items(tau)?.iter().map(set_items).collect()
}

fn space(tau: &Value) -> Result<Vec<Value>, String> {
    let mut all = vec![];
    for u in opens(tau)? {
        all.extend(u);
    }
    Ok(set_items_of(all))
}

pub fn open_set(a: &[Value]) -> R {
    let u = set_items(&a[1])?;
    b(opens(&a[0])?.iter().any(|o| same_set(o, &u)))
}

pub fn closed_set(a: &[Value]) -> R {
    let comp = difference(&space(&a[0])?, &set_items(&a[1])?);
    b(opens(&a[0])?.iter().any(|o| same_set(o, &comp)))
}

pub fn interior(a: &[Value]) -> R {
    let am = set_items(&a[1])?;
    let mut out: Vec<Value> = vec![];
    for u in opens(&a[0])? {
        if subset(&u, &am) {
            out = union_fn(&out, &u);
        }
    }
    Ok(s(out))
}

pub fn closure(a: &[Value]) -> R {
    let x = space(&a[0])?;
    let am = set_items(&a[1])?;
    let mut out = x.clone();
    for u in opens(&a[0])? {
        let c = difference(&x, &u);
        if subset(&am, &c) {
            out = intersection_fn(&out, &c);
        }
    }
    Ok(s(out))
}

pub fn boundary(a: &[Value]) -> R {
    let x = space(&a[0])?;
    let am = set_items(&a[1])?;
    let mut out = x.clone();
    for u in opens(&a[0])? {
        let c = difference(&x, &u);
        if subset(&am, &c) {
            out = intersection_fn(&out, &c);
        }
    }
    let mut inner: Vec<Value> = vec![];
    for u in opens(&a[0])? {
        if subset(&u, &am) {
            inner = union_fn(&inner, &u);
        }
    }
    Ok(s(difference(&out, &inner)))
}

pub fn neighborhood(a: &[Value]) -> R {
    // tau, x, N
    let nn = set_items(&a[2])?;
    b(opens(&a[0])?.iter().any(|u| contains(u, &a[1]) && subset(u, &nn)))
}

pub fn subspace_topology(a: &[Value]) -> R {
    let am = set_items(&a[1])?;
    let sets: Vec<Value> = opens(&a[0])?.iter().map(|u| Value::Set(intersection_fn(u, &am))).collect();
    Ok(Value::set_of(sets))
}

pub fn connected(a: &[Value]) -> R {
    let x = space(&a[0])?;
    let os: Vec<Vec<Value>> = opens(&a[0])?.into_iter().filter(|u| !u.is_empty() && !same_set(u, &x)).collect();
    for u in &os {
        for v in &os {
            if same_set(&union_fn(u, v), &x) && intersection_fn(u, v).is_empty() {
                return b(false);
            }
        }
    }
    b(true)
}

pub fn hausdorff(a: &[Value]) -> R {
    let mut x = space(&a[0])?;
    x.sort_by_key(|v| v.display());
    let os = opens(&a[0])?;
    for i in 0..x.len() {
        for j in i + 1..x.len() {
            let mut found = false;
            'search: for u in &os {
                for v in &os {
                    if contains(u, &x[i]) && contains(v, &x[j]) && intersection_fn(u, v).is_empty() {
                        found = true;
                        break 'search;
                    }
                }
            }
            if !found {
                return b(false);
            }
        }
    }
    b(true)
}

pub fn basis_of_topology(a: &[Value]) -> R {
    // tau, B
    let mut os: Vec<Vec<Value>> = vec![];
    for u in opens(&a[0])? {
        if !os.iter().any(|o| same_set(o, &u)) {
            os.push(u);
        }
    }
    let bs = opens(&a[1])?;
    if !bs.iter().all(|bb| os.iter().any(|o| same_set(o, bb))) {
        return b(false);
    }
    for u in &os {
        let mut un: Vec<Value> = vec![];
        for bb in &bs {
            if subset(bb, u) {
                un = union_fn(&un, bb);
            }
        }
        if !same_set(&un, u) {
            return b(false);
        }
    }
    b(true)
}

// ---------------------------------------------------------------- measure of a union of intervals
fn measure_of(v: &Value) -> R {
    let mut iv: Vec<(f64, f64)> = vec![];
    for e in items(v)? {
        match e {
            Value::Tuple(t) if t.len() == 2 => iv.push((num(&t[0])?, num(&t[1])?)),
            _ => return Err("each interval needs two ends".into()),
        }
    }
    iv.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    let (mut total, mut end): (f64, Option<f64>) = (0.0, None);
    for (lo, hi) in iv {
        match end {
            None => {
                total += hi - lo;
                end = Some(hi);
            }
            Some(e) if lo > e => {
                total += hi - lo;
                end = Some(hi);
            }
            Some(e) if hi > e => {
                total += hi - e;
                end = Some(hi);
            }
            _ => {}
        }
    }
    n(total)
}

pub fn measure(a: &[Value]) -> R {
    measure_of(&a[0])
}

pub fn outer_measure(a: &[Value]) -> R {
    measure_of(&a[0])
}
