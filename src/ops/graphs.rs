//! Graph operators.  A graph arrives as `Value::Dict` (vertex -> list of neighbours, or of (neighbour, weight)),
//! exactly what the Advanced Input window builds from  `A: B, C; B: C; C: A`.

use super::vals::*;
use crate::value::Value;

/// An undirected view of a graph (`_helpers.undirected`): vertices in first-seen order, each with its neighbours.
struct Adj {
    verts: Vec<Value>,
    nbrs: Vec<Vec<usize>>,
}

impl Adj {
    fn index_of(&self, v: &Value) -> Option<usize> {
        self.verts.iter().position(|x| x.same_item(v))
    }
    fn vertex(&mut self, v: &Value) -> usize {
        match self.index_of(v) {
            Some(i) => i,
            None => {
                self.verts.push(v.clone());
                self.nbrs.push(vec![]);
                self.verts.len() - 1
            }
        }
    }
    fn len(&self) -> usize {
        self.verts.len()
    }
    /// edges + self-loops counted the way `_helpers.edge_count` does (with its integer divisions)
    fn edge_count(&self) -> i64 {
        let degrees: i64 = self.nbrs.iter().map(|n| n.len() as i64).sum();
        let loops = (0..self.len()).filter(|u| self.nbrs[*u].contains(u)).count() as i64;
        degrees / 2 + loops / 2
    }
    fn components(&self) -> Vec<Vec<usize>> {
        let mut seen = vec![false; self.len()];
        let mut comps = vec![];
        for s in 0..self.len() {
            if seen[s] {
                continue;
            }
            seen[s] = true;
            let mut comp = vec![s];
            let mut queue = std::collections::VecDeque::from(vec![s]);
            while let Some(u) = queue.pop_front() {
                for &v in &self.nbrs[u] {
                    if !seen[v] {
                        seen[v] = true;
                        comp.push(v);
                        queue.push_back(v);
                    }
                }
            }
            comps.push(comp);
        }
        comps
    }
}

fn dict(v: &Value) -> Result<&Vec<(Value, Value)>, String> {
    match v {
        Value::Dict(d) => Ok(d),
        other => Err(format!("expected a graph, got {}", other.kind_name())),
    }
}

fn undirected(g: &Value) -> Result<Adj, String> {
    let mut adj = Adj { verts: vec![], nbrs: vec![] };
    for (u, nb) in dict(g)? {
        let ui = adj.vertex(u);
        for v in items(nb)? {
            let vv = match v {
                Value::Tuple(t) if !t.is_empty() => &t[0],
                other => other,
            };
            let vi = adj.vertex(vv);
            if !adj.nbrs[ui].contains(&vi) {
                adj.nbrs[ui].push(vi);
            }
            if !adj.nbrs[vi].contains(&ui) {
                adj.nbrs[vi].push(ui);
            }
        }
    }
    Ok(adj)
}

// ---------------------------------------------------------------- yes / no questions and counts
pub fn bipartite_graph(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    let mut colour: Vec<Option<u8>> = vec![None; adj.len()];
    for s in 0..adj.len() {
        if colour[s].is_some() {
            continue;
        }
        colour[s] = Some(0);
        let mut stack = vec![s];
        while let Some(u) = stack.pop() {
            for &v in &adj.nbrs[u] {
                match colour[v] {
                    None => {
                        colour[v] = Some(1 - colour[u].unwrap());
                        stack.push(v);
                    }
                    Some(c) if Some(c) == colour[u] => return b(false),
                    _ => {}
                }
            }
        }
    }
    b(true)
}

pub fn chromatic_number(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    let mut verts: Vec<usize> = (0..adj.len()).collect();
    verts.sort_by_key(|v| std::cmp::Reverse(adj.nbrs[*v].len()));
    need(verts.len() <= 14, "too many vertices for the exact search (limit 14)")?;
    fn go(i: usize, k: usize, verts: &[usize], adj: &Adj, colour: &mut Vec<Option<usize>>) -> bool {
        if i == verts.len() {
            return true;
        }
        let v = verts[i];
        for c in 0..k {
            if adj.nbrs[v].iter().all(|u| colour[*u] != Some(c)) {
                colour[v] = Some(c);
                if go(i + 1, k, verts, adj, colour) {
                    return true;
                }
                colour[v] = None;
            }
        }
        false
    }
    let mut k = 1;
    loop {
        let mut colour = vec![None; adj.len()];
        if go(0, k, &verts, &adj, &mut colour) {
            return n(k as f64);
        }
        k += 1;
    }
}

pub fn cycle(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    b(adj.edge_count() > adj.len() as i64 - adj.components().len() as i64)
}

pub fn euler_path(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    let comps = adj.components().into_iter().filter(|c| c.iter().any(|v| !adj.nbrs[*v].is_empty())).count();
    let odd = adj.nbrs.iter().filter(|nb| nb.len() % 2 == 1).count();
    b(comps <= 1 && (odd == 0 || odd == 2))
}

pub fn graph(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    Ok(Value::Tuple(vec![Value::Number(adj.len() as f64), Value::Number(adj.edge_count() as f64)]))
}

pub fn planar_graph(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    let (v, e) = (adj.len() as i64, adj.edge_count());
    b(v < 3 || e <= 3 * v - 6)
}

pub fn tree(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    b(adj.components().len() == 1 && adj.edge_count() == adj.len() as i64 - 1)
}

pub fn vertex_degree(a: &[Value]) -> R {
    // graph, v
    let d = dict(&a[0])?;
    let (_, nb) = d.iter().find(|(k, _)| k.same_item(&a[1])).ok_or("that vertex is not in the graph")?;
    n(items(nb)?.len() as f64)
}

pub fn hamiltonian_path(a: &[Value]) -> R {
    let adj = undirected(&a[0])?;
    need(adj.len() <= 10, "too many vertices for the exact search (limit 10)")?;
    fn go(path: &mut Vec<usize>, adj: &Adj) -> bool {
        if path.len() == adj.len() {
            return true;
        }
        let last = *path.last().unwrap();
        for &v in &adj.nbrs[last] {
            if !path.contains(&v) {
                path.push(v);
                if go(path, adj) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }
    for s in 0..adj.len() {
        let mut path = vec![s];
        if go(&mut path, &adj) {
            return Ok(Value::List(path.into_iter().map(|i| adj.verts[i].clone()).collect()));
        }
    }
    b(false)
}

pub fn path(a: &[Value]) -> R {
    // graph, start, end
    let adj = undirected(&a[0])?;
    let mut verts: Vec<Value> = vec![a[1].clone()];
    let mut prev: Vec<Option<usize>> = vec![None];
    let mut queue = 0;
    while queue < verts.len() {
        let u = verts[queue].clone();
        if let Some(ui) = adj.index_of(&u) {
            let mut nb: Vec<&Value> = adj.nbrs[ui].iter().map(|i| &adj.verts[*i]).collect();
            nb.sort_by_key(|v| v.display());
            for v in nb {
                if !verts.iter().any(|x| x.same_item(v)) {
                    verts.push(v.clone());
                    prev.push(Some(queue));
                }
            }
        }
        queue += 1;
    }
    let mut cur = verts.iter().position(|x| x.same_item(&a[2])).ok_or("no path from start to end")?;
    let mut out = vec![verts[cur].clone()];
    while let Some(p) = prev[cur] {
        out.push(verts[p].clone());
        cur = p;
    }
    out.reverse();
    Ok(Value::List(out))
}

// ---------------------------------------------------------------- weighted graphs
fn weighted(g: &Value) -> Result<Vec<(Value, Vec<(Value, f64)>)>, String> {
    let mut out = vec![];
    for (u, nb) in dict(g)? {
        let mut list = vec![];
        for e in items(nb)? {
            match e {
                Value::Tuple(t) if t.len() == 2 => list.push((t[0].clone(), num(&t[1])?)),
                _ => return Err("a weighted graph needs neighbour=weight".into()),
            }
        }
        out.push((u.clone(), list));
    }
    Ok(out)
}

pub fn shortest_path_dijkstra(a: &[Value]) -> R {
    let g = weighted(&a[0])?;
    let mut dist: Vec<(Value, f64)> = vec![(a[1].clone(), 0.0)];
    let mut heap: Vec<(f64, Value)> = vec![(0.0, a[1].clone())];
    let mut steps = 0;
    while !heap.is_empty() {
        steps += 1;
        need(steps < 1_000_000, "this graph does not settle (negative weights?)")?;
        // pop the smallest (distance, then name)
        let mut best = 0;
        for i in 1..heap.len() {
            let (x, y) = (&heap[i], &heap[best]);
            if x.0 < y.0 || (x.0 == y.0 && x.1.sort_key() < y.1.sort_key()) {
                best = i;
            }
        }
        let (d, u) = heap.swap_remove(best);
        let known = |dist: &Vec<(Value, f64)>, v: &Value| dist.iter().position(|(k, _)| k.same_item(v));
        if let Some(i) = known(&dist, &u) {
            if d > dist[i].1 {
                continue;
            }
        }
        if let Some((_, nb)) = g.iter().find(|(k, _)| k.same_item(&u)) {
            for (v, w) in nb {
                let nd = d + w;
                match known(&dist, v) {
                    Some(i) if nd < dist[i].1 => {
                        dist[i].1 = nd;
                        heap.push((nd, v.clone()));
                    }
                    None => {
                        dist.push((v.clone(), nd));
                        heap.push((nd, v.clone()));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(Value::Dict(dist.into_iter().map(|(k, d)| (k, Value::Number(d))).collect()))
}

pub fn spanning_tree(a: &[Value]) -> R {
    let g = weighted(&a[0])?;
    let mut edges: Vec<(f64, Value, Value)> = vec![];
    for (u, nb) in &g {
        for (v, w) in nb {
            edges.push((*w, u.clone(), v.clone()));
        }
    }
    edges.sort_by(|p, q| {
        p.0.partial_cmp(&q.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| p.1.sort_key().cmp(&q.1.sort_key())).then_with(|| p.2.sort_key().cmp(&q.2.sort_key()))
    });
    let mut names: Vec<Value> = vec![];
    let mut parent: Vec<usize> = vec![];
    let id = |v: &Value, names: &mut Vec<Value>, parent: &mut Vec<usize>| -> usize {
        match names.iter().position(|x| x.same_item(v)) {
            Some(i) => i,
            None => {
                names.push(v.clone());
                parent.push(names.len() - 1);
                names.len() - 1
            }
        }
    };
    fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    let (mut out, mut total) = (vec![], 0.0);
    for (w, u, v) in edges {
        let (ui, vi) = (id(&u, &mut names, &mut parent), id(&v, &mut names, &mut parent));
        let (ru, rv) = (find(&mut parent, ui), find(&mut parent, vi));
        if ru != rv {
            parent[ru] = rv;
            out.push(Value::Tuple(vec![u, v, Value::Number(w)]));
            total += w;
        }
    }
    Ok(Value::Tuple(vec![Value::List(out), Value::Number(total)]))
}

pub fn max_flow_min_cut(a: &[Value]) -> R {
    // graph, source, sink
    let g = weighted(&a[0])?;
    let (source, sink) = (&a[1], &a[2]);
    let mut names: Vec<Value> = vec![];
    let mut cap: Vec<Vec<(usize, f64)>> = vec![]; // per vertex, (neighbour, remaining capacity) in insertion order
    fn id(names: &mut Vec<Value>, cap: &mut Vec<Vec<(usize, f64)>>, v: &Value) -> usize {
        match names.iter().position(|x| x.same_item(v)) {
            Some(i) => i,
            None => {
                names.push(v.clone());
                cap.push(vec![]);
                names.len() - 1
            }
        }
    }
    fn slot(cap: &mut Vec<Vec<(usize, f64)>>, u: usize, v: usize) -> &mut f64 {
        if let Some(i) = cap[u].iter().position(|(x, _)| *x == v) {
            return &mut cap[u][i].1;
        }
        cap[u].push((v, 0.0));
        &mut cap[u].last_mut().unwrap().1
    }
    for (u, nb) in &g {
        for (v, w) in nb {
            let ui = id(&mut names, &mut cap, u);
            let vi = id(&mut names, &mut cap, v);
            *slot(&mut cap, ui, vi) += w;
            slot(&mut cap, vi, ui); // make sure the reverse entry exists
        }
    }
    let mut flow = 0.0;
    loop {
        // breadth-first search over edges with capacity left
        let mut prev: Vec<(Value, Option<Value>)> = vec![(source.clone(), None)];
        let mut order: Vec<Value> = vec![source.clone()];
        let mut qi = 0;
        while qi < order.len() {
            let u = order[qi].clone();
            qi += 1;
            if let Some(ui) = names.iter().position(|x| x.same_item(&u)) {
                for (vi, c) in &cap[ui] {
                    let v = &names[*vi];
                    if *c > 1e-12 && !prev.iter().any(|(k, _)| k.same_item(v)) {
                        prev.push((v.clone(), Some(u.clone())));
                        order.push(v.clone());
                    }
                }
            }
        }
        if !prev.iter().any(|(k, _)| k.same_item(sink)) {
            return n(flow);
        }
        let mut path: Vec<(Value, Value)> = vec![];
        let mut v = sink.clone();
        loop {
            let p = prev.iter().find(|(k, _)| k.same_item(&v)).and_then(|(_, p)| p.clone());
            match p {
                Some(u) => {
                    path.push((u.clone(), v.clone()));
                    v = u;
                }
                None => break,
            }
        }
        need(!path.is_empty(), "min() of an empty path")?;
        let idx = |names: &Vec<Value>, v: &Value| names.iter().position(|x| x.same_item(v)).unwrap();
        let mut push = f64::INFINITY;
        for (u, v) in &path {
            let (ui, vi) = (idx(&names, u), idx(&names, v));
            push = push.min(*slot(&mut cap, ui, vi));
        }
        for (u, v) in &path {
            let (ui, vi) = (idx(&names, u), idx(&names, v));
            *slot(&mut cap, ui, vi) -= push;
            *slot(&mut cap, vi, ui) += push;
        }
        flow += push;
    }
}

pub fn a_star(a: &[Value]) -> R {
    // graph, positions, start, goal
    let g = weighted(&a[0])?;
    need(g.iter().all(|(_, nb)| nb.iter().all(|(_, w)| *w >= 0.0)), "a weight cannot be negative")?;
    let positions = dict(&a[1])?;
    let where_is = |node: &Value| -> Result<(f64, f64), String> {
        let p = positions.iter().find(|(k, _)| k.same_item(node)).and_then(|(_, v)| nums(v).ok());
        match p {
            Some(p) if p.len() == 2 => Ok((p[0], p[1])),
            _ => Err("each node needs a position".to_string()),
        }
    };
    let (start, goal) = (a[2].clone(), a[3].clone());
    let (gx, gy) = where_is(&goal)?;
    let heuristic = |node: &Value| -> Result<f64, String> {
        let (x, y) = where_is(node)?;
        Ok(((gx - x) * (gx - x) + (gy - y) * (gy - y)).sqrt())
    };
    // cost[i] and came[i] belong to nodes[i]
    let mut nodes: Vec<Value> = vec![start.clone()];
    let mut cost: Vec<f64> = vec![0.0];
    let mut came: Vec<Option<usize>> = vec![None];
    let index = |nodes: &Vec<Value>, v: &Value| nodes.iter().position(|k| k.same_item(v));
    let mut count = 0usize;
    let mut heap: Vec<(f64, usize, usize)> = vec![(heuristic(&start)?, count, 0)];
    while !heap.is_empty() {
        // the smallest f, then the earliest pushed
        let mut best = 0;
        for i in 1..heap.len() {
            if heap[i].0 < heap[best].0 || (heap[i].0 == heap[best].0 && heap[i].1 < heap[best].1) {
                best = i;
            }
        }
        let (f, _, u) = heap.swap_remove(best);
        if f > cost[u] + heuristic(&nodes[u])? {
            continue;
        }
        if nodes[u].same_item(&goal) {
            let mut path = vec![nodes[u].clone()];
            let mut at = u;
            while let Some(p) = came[at] {
                path.push(nodes[p].clone());
                at = p;
            }
            path.reverse();
            return Ok(Value::List(path));
        }
        if let Some((_, nb)) = g.iter().find(|(k, _)| k.same_item(&nodes[u])) {
            for (v, w) in nb {
                let ng = cost[u] + w;
                let vi = match index(&nodes, v) {
                    Some(i) => i,
                    None => {
                        nodes.push(v.clone());
                        cost.push(f64::INFINITY);
                        came.push(None);
                        nodes.len() - 1
                    }
                };
                if ng < cost[vi] {
                    cost[vi] = ng;
                    came[vi] = Some(u);
                    count += 1;
                    heap.push((ng + heuristic(v)?, count, vi));
                }
            }
        }
    }
    Err("no path".into())
}
