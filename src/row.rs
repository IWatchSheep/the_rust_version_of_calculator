//! One calculator row: operator, input boxes, join symbol, result.

use crate::operator::Operator;
use crate::ops;
use crate::value::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    None,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    /// "then" (→): the rows read each other's answers; the answer of the whole chain is the LAST row's answer.
    Then,
}

impl Join {
    pub fn from_symbol(sym: &str) -> Join {
        match sym {
            "+" => Join::Add,
            "−" | "-" => Join::Sub,
            "×" | "*" => Join::Mul,
            "÷" | "/" => Join::Div,
            "=" => Join::Eq,
            "≠" => Join::Ne,
            "<" => Join::Lt,
            ">" => Join::Gt,
            "≤" => Join::Le,
            "≥" => Join::Ge,
            "→" => Join::Then,
            _ => Join::None,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Join::None => "",
            Join::Add => "+",
            Join::Sub => "−",
            Join::Mul => "×",
            Join::Div => "÷",
            Join::Eq => "=",
            Join::Ne => "≠",
            Join::Lt => "<",
            Join::Gt => ">",
            Join::Le => "≤",
            Join::Ge => "≥",
            Join::Then => "→",
        }
    }

    /// Combine the answer so far with the next row's answer.  Lists and the like cannot be joined (the same rule as the Python program).
    pub fn apply(self, a: &Value, b: &Value) -> Result<Value, String> {
        if self == Join::Then {
            return Ok(b.clone());
        }
        let (x, y) = match (a, b) {
            (Value::Number(x), Value::Number(y)) => (*x, *y),
            _ => return Err(format!("Can't join those two results with {}.", self.symbol())),
        };
        let close = |p: f64, q: f64| (p - q).abs() <= 1e-3 * 1.0_f64.max(p.abs()).max(q.abs());
        Ok(match self {
            Join::None => return Err("pick a join symbol".into()),
            Join::Add => Value::Number(x + y),
            Join::Sub => Value::Number(x - y),
            Join::Mul => Value::Number(x * y),
            Join::Div => {
                if y == 0.0 {
                    return Err("Division by zero.".into());
                }
                Value::Number(x / y)
            }
            Join::Eq => Value::Bool(close(x, y)),
            Join::Ne => Value::Bool(!close(x, y)),
            Join::Lt => Value::Bool(x < y),
            Join::Gt => Value::Bool(x > y),
            Join::Le => Value::Bool(x <= y),
            Join::Ge => Value::Bool(x >= y),
            Join::Then => unreachable!(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub op_key: Option<String>,
    /// Text in each input box (one per input of the operator; a variable the picture writes twice is one input with two boxes).
    pub inputs: Vec<String>,
    /// An operator written inside a box (one per input, like `inputs`): its math takes the place of the box, e.g. the (a + b) in (a + b) × c.
    pub children: Vec<Option<Box<Row>>>,
    /// The row's own answer, shown in its answer card to the right of the `=`.
    pub result: String,
    /// The answer as a value (used when rows are joined).
    pub value: Option<Value>,
    /// What each of the formula's answer boxes shows (one for a single answer, one each for `x1, x2`).
    pub outs: Vec<String>,
    /// How this row is joined to the rows above it (Row 1 has none).
    pub join: Join,
    /// A row that only shows the answer for the rows above it (Row 3 when Rows 1 and 2 are joined).
    pub is_result: bool,
    pub error: Option<String>,
    /// Words typed into a card's Label / Item and Context boxes: flat card index -> (label, context).  They replace the operator's default words (`layout::words`).
    pub words: std::collections::HashMap<usize, (String, String)>,
}

impl Row {
    pub fn blank() -> Self {
        Self {
            op_key: None,
            inputs: Vec::new(),
            children: Vec::new(),
            result: String::new(),
            value: None,
            outs: Vec::new(),
            join: Join::None,
            is_result: false,
            error: None,
            words: std::collections::HashMap::new(),
        }
    }

    /// Forget the answer (something was changed).
    pub fn clear_result(&mut self) {
        self.result.clear();
        self.value = None;
        self.outs.clear();
        for c in self.children.iter_mut().flatten() {
            c.clear_result();
        }
    }

    /// A row holding just this operator, its boxes empty (or holding the problem's own numbers for an algebra word problem).
    pub fn with_operator(op: &Operator) -> Row {
        let mut r = Row::blank();
        r.set_operator(op);
        r
    }

    /// The row (the operator) at `path`'s parent and the input number the path ends at.
    fn walk_to_owner(&self, path: &[usize]) -> Option<(&Row, usize)> {
        let (last, init) = path.split_last()?;
        let mut node = self;
        for &j in init {
            node = node.children.get(j)?.as_deref()?;
        }
        Some((node, *last))
    }

    fn walk_to_owner_mut(&mut self, path: &[usize]) -> Option<(&mut Row, usize)> {
        let (last, init) = path.split_last()?;
        let mut node = self;
        for &j in init {
            node = node.children.get_mut(j)?.as_deref_mut()?;
        }
        Some((node, *last))
    }

    /// The operator that owns the box at `path`, and which of its inputs the box is.
    pub fn owner_of(&self, path: &[usize]) -> Option<(&'static Operator, usize)> {
        let (node, j) = self.walk_to_owner(path)?;
        Some((node.operator()?, j))
    }

    /// The text of the box at `path` (None when the path leads to an operator, or nowhere).
    pub fn slot_text(&self, path: &[usize]) -> Option<&String> {
        let (node, j) = self.walk_to_owner(path)?;
        if node.children.get(j)?.is_some() {
            return None;
        }
        node.inputs.get(j)
    }

    pub fn slot_mut(&mut self, path: &[usize]) -> Option<&mut String> {
        let (node, j) = self.walk_to_owner_mut(path)?;
        if node.children.get(j)?.is_some() {
            return None;
        }
        node.inputs.get_mut(j)
    }

    /// Write an operator's math into the box at `path` (what was in the box goes).  Returns false when there is no such box.
    pub fn expand(&mut self, path: &[usize], op: &Operator) -> bool {
        let Some((node, j)) = self.walk_to_owner_mut(path) else { return false };
        if j >= node.children.len() {
            return false;
        }
        node.children[j] = Some(Box::new(Row::with_operator(op)));
        node.inputs[j].clear();
        true
    }

    /// Put a ready-made tree into the box at `path`.
    pub fn put_row(&mut self, path: &[usize], row: Row) -> bool {
        let Some((node, j)) = self.walk_to_owner_mut(path) else { return false };
        if j >= node.children.len() {
            return false;
        }
        node.children[j] = Some(Box::new(row));
        node.inputs[j].clear();
        true
    }

    /// Put the box at `path` back to an empty number box.
    pub fn collapse(&mut self, path: &[usize]) {
        if let Some((node, j)) = self.walk_to_owner_mut(path) {
            if j < node.children.len() {
                node.children[j] = None;
                node.inputs[j].clear();
            }
        }
    }

    /// Every card of the row left to right with the operators inside the boxes written out in place (Python `row_tree.flatten`).
    pub fn flatten(&self) -> Vec<crate::layout::FlatCard> {
        let mut counter = 0u32;
        let mut out = Vec::new();
        self.walk_cards(&[], &[], 0, &mut counter, &mut out);
        out
    }

    fn walk_cards(&self, path: &[usize], outer: &[(u32, u8)], base_level: i8, counter: &mut u32, out: &mut Vec<crate::layout::FlatCard>) {
        use crate::layout::{CardKind, FlatCard};
        let Some(layout) = self.operator().and_then(|op| crate::layout::get(op.key)) else { return };
        let node_id = *counter;
        *counter += 1;
        let mut seen: Vec<i32> = Vec::new();
        let mut out_k = 0usize;
        for card in layout.cards {
            let level = card.level + base_level;
            let mut own: Vec<(u32, u8)> = outer.to_vec();
            own.extend(card.frac.iter().map(|&(g, p)| (node_id * 256 + g as u32, p)));
            match card.kind {
                CardKind::Box => {
                    let j = card.slot as usize;
                    let mut here = path.to_vec();
                    here.push(j);
                    let child = self.children.get(j).and_then(|c| c.as_deref());
                    if seen.contains(&card.slot) {
                        // the same variable written again: a copy of the box, it shows (and selects) the same slot
                        let (kind, text) = match child {
                            Some(c) => (CardKind::Dec, c.operator().map_or(String::new(), |o| o.name.to_string())),
                            None => (CardKind::Box, self.inputs.get(j).cloned().unwrap_or_default()),
                        };
                        out.push(FlatCard { kind, text, path: here, level, frac: own, out_k: 0, owner: path.to_vec() });
                        continue;
                    }
                    seen.push(card.slot);
                    match child {
                        Some(c) => c.walk_cards(&here, &own, level, counter, out),
                        None => out.push(FlatCard { kind: CardKind::Box, text: self.inputs.get(j).cloned().unwrap_or_default(), path: here, level, frac: own, out_k: 0, owner: path.to_vec() }),
                    }
                }
                CardKind::Out => {
                    let shown = self.outs.get(out_k).cloned().unwrap_or_default();
                    out.push(FlatCard { kind: CardKind::Out, text: shown, path: Vec::new(), level, frac: own, out_k, owner: path.to_vec() });
                    out_k += 1;
                }
                kind => out.push(FlatCard { kind, text: card.text.to_string(), path: Vec::new(), level, frac: own, out_k: 0, owner: path.to_vec() }),
            }
        }
    }

    /// The (label, context) words of every card of `flatten()`, same order: the operator's default words (Python `WORDS`), with what was typed on top.
    pub fn card_words(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        self.walk_words(&mut out);
        for (k, w) in out.iter_mut().enumerate() {
            if let Some(typed) = self.words.get(&k) {
                *w = typed.clone();
            }
        }
        out
    }

    fn walk_words(&self, out: &mut Vec<(String, String)>) {
        use crate::layout::CardKind;
        let Some(op) = self.operator() else { return };
        let Some(layout) = crate::layout::get(op.key) else { return };
        let given = crate::layout::words(op.key);
        let mut seen: Vec<i32> = Vec::new();
        for (n, card) in layout.cards.iter().enumerate() {
            let w = given.iter().find(|(i, _, _)| *i == n).map_or((String::new(), String::new()), |(_, l, c)| (l.to_string(), c.to_string()));
            if card.kind == CardKind::Box {
                if seen.contains(&card.slot) {
                    out.push(w);
                    continue;
                }
                seen.push(card.slot);
                match self.children.get(card.slot as usize).and_then(|c| c.as_deref()) {
                    Some(c) => c.walk_words(out),
                    None => out.push(w),
                }
            } else {
                out.push(w);
            }
        }
    }

    /// The path of the first box of the row (where typing starts).
    pub fn first_box(&self) -> Vec<usize> {
        self.flatten().into_iter().find(|c| c.kind == crate::layout::CardKind::Box).map(|c| c.path).unwrap_or_else(|| vec![0])
    }

    /// The paths of all boxes in order, each once.
    pub fn box_paths(&self) -> Vec<Vec<usize>> {
        let mut v: Vec<Vec<usize>> = Vec::new();
        for c in self.flatten() {
            if c.kind == crate::layout::CardKind::Box && !v.contains(&c.path) {
                v.push(c.path);
            }
        }
        v
    }

    /// How many answer boxes the formula has.
    pub fn out_count(&self) -> usize {
        self.operator()
            .and_then(|op| crate::layout::get(op.key))
            .map(|l| l.cards.iter().filter(|c| c.kind == crate::layout::CardKind::Out).count())
            .unwrap_or(0)
    }

    /// A row with nothing in it: no operator, or just the plain number box with nothing typed (the default empty square box of Python's rows).
    pub fn is_blank(&self) -> bool {
        if self.is_result {
            return false;
        }
        match self.op_key.as_deref() {
            None => true,
            Some("_number") => self.inputs.first().map_or(true, |t| t.is_empty()) && self.children.first().map_or(true, |c| c.is_none()),
            Some(_) => false,
        }
    }

    /// Give a row with no operator the plain number box, so keypad digits have somewhere to go.
    pub fn ensure_number_box(&mut self) {
        if self.op_key.is_none() && !self.is_result {
            if let Some(op) = ops::get("_number") {
                self.set_operator(op);
            }
        }
    }

    pub fn set_operator(&mut self, op: &Operator) {
        self.op_key = Some(op.key.to_string());
        self.inputs = vec![String::new(); op.arity()];
        self.children = vec![None; op.arity()];
        if let Some(d) = crate::ops::words::defaults(op.key) {
            // a word problem starts with its own numbers in the boxes
            self.inputs = d.iter().map(|s| s.to_string()).collect();
        }
        self.clear_result();
        self.is_result = false;
        self.error = None;
    }

    pub fn operator(&self) -> Option<&'static Operator> {
        self.op_key.as_deref().and_then(ops::get)
    }

    pub fn compute(&mut self) {
        self.error = None;
        self.clear_result();
        match self.eval() {
            Ok(v) => {
                self.result = v.show();
                self.value = Some(v);
            }
            Err(e) => self.error = Some(e),
        }
    }

    /// Calculate this operator, the operators inside its boxes first, and remember the answer in its answer boxes (Python `row_tree.evaluate`).
    pub fn eval(&mut self) -> Result<Value, String> {
        let Some(op) = self.operator() else {
            return Err("pick an operator first".into());
        };
        // Each box is typed the way the Python Advanced Input window takes it: a number, `1, 2, 3` for a list,
        // `1 2; 3 4` for a matrix, `x**2 + 3*x` for a function, `A: B, C; B: C` for a graph ...
        let typed_parent = op.key == "_number" || op.kinds.iter().any(|k| *k != "number");
        let mut args = Vec::with_capacity(op.arity());
        for i in 0..self.inputs.len() {
            if let Some(child) = self.children.get_mut(i).and_then(|c| c.as_deref_mut()) {
                let v = child.eval()?;
                if !typed_parent && !matches!(v, Value::Number(_)) {
                    return Err(format!("{} gives {}, which can't be used as a single number inside {}.", child.operator().map_or("?", |o| o.name), v.show(), op.name));
                }
                args.push(v);
                continue;
            }
            let t = self.inputs[i].trim();
            if t.is_empty() {
                return Err(format!("fill box {} ({})", i + 1, op.inputs.get(i).unwrap_or(&"?")));
            }
            let kind = op.kinds.get(i).copied().unwrap_or("number");
            match crate::kinds::parse_kind(kind, t) {
                Ok(Value::Number(n)) if !n.is_finite() => return Err(format!("box {} is not finite", i + 1)),
                Ok(v) => args.push(v),
                Err(_) if kind == "number" => return Err(format!("box {} is not a number", i + 1)),
                Err(e) => return Err(format!("box {} ({}): {}", i + 1, op.inputs.get(i).unwrap_or(&"?"), e)),
            }
        }
        let v = if op.key == "_number" { args.into_iter().next().unwrap_or(Value::Number(0.0)) } else { op.run_values(&args)? };
        self.outs = out_texts(&v, self.out_count());
        Ok(v)
    }
}

/// What each answer box of the formula shows: a single answer in the one box, or each of `(x1, x2)` in its own box.
pub fn out_texts(v: &Value, outs: usize) -> Vec<String> {
    if outs == 0 {
        return Vec::new();
    }
    let parts: Option<Vec<String>> = match v {
        Value::Pair(a, b) => Some(vec![Value::Number(*a).show(), Value::Number(*b).show()]),
        Value::Tuple(items) | Value::List(items) => Some(items.iter().map(|x| x.show()).collect()),
        _ => None,
    };
    match parts {
        Some(p) if outs > 1 && p.len() == outs => p,
        _ => vec![v.show(); outs],
    }
}

#[derive(Clone, Debug)]
pub struct Document {
    pub rows: Vec<Row>,
    /// Which row receives the next operator / keypad digits.
    pub target_row: usize,
    /// Which box in that row is selected: the first number is the box of the row's own operator, a longer path goes into an operator written inside a box.
    pub target_path: Vec<usize>,
    /// The multi-row operator laid out in the rows right now (see multirow.rs).
    pub template: Option<crate::multirow::Template>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            rows: vec![Row::blank(), Row::blank()],
            target_row: 0,
            target_path: vec![0],
            template: None,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn ensure_target(&mut self) {
        if self.rows.is_empty() {
            self.rows.push(Row::blank());
        }
        if self.target_row >= self.rows.len() {
            self.target_row = self.rows.len() - 1;
        }
        let row = &self.rows[self.target_row];
        if !self.target_path.is_empty() && row.slot_text(&self.target_path).is_none() {
            self.target_path = row.first_box();
        }
        if self.target_path.is_empty() {
            self.target_path = vec![0];
        }
    }

    /// The selected box's operator input (operator, input number), if the target is a box.
    pub fn target_owner(&self) -> Option<(&'static Operator, usize)> {
        self.rows.get(self.target_row)?.owner_of(&self.target_path)
    }

    /// Put an operator's math into the target row.  Row 1 that already has math: the operator EXPANDS the selected box (the first box if none was
    /// selected) into that operator's math, so it nests.  Any other row: the operator replaces the row's math.
    pub fn place_operator(&mut self, op: &Operator) {
        self.place_row(Row::with_operator(op));
    }

    /// Put a ready-made tree (an operator, or a template) into the target row; same rules as `place_operator`.
    pub fn place_row(&mut self, new: Row) {
        self.ensure_target();
        if self.rows[self.target_row].is_result {
            self.rows[self.target_row] = Row::blank();
        }
        let i = self.target_row;
        let nest = i == 0 && self.rows[i].operator().is_some() && !self.rows[i].box_paths().is_empty();
        if nest {
            let boxes = self.rows[i].box_paths();
            let path = if boxes.contains(&self.target_path) { self.target_path.clone() } else { boxes[0].clone() };
            self.rows[i].put_row(&path, new);
            self.rows[i].words.clear();          // the cards moved: the words typed into the old ones no longer fit
            self.rows[i].clear_result();
            self.rows[i].error = None;
        } else {
            let join = self.rows[i].join;
            self.rows[i] = new;
            self.rows[i].join = join;
        }
        self.template = None;
        self.target_path = self.rows[i].first_box();
    }

    /// A multi-row operator: add the rows it needs, put its operator in each row (boxes pre-filled), write the joins.
    pub fn apply_template(&mut self, key: &'static str, spec: &'static crate::multirow::Spec) -> Result<(), String> {
        use crate::multirow::{Fill, NodeSpec};
        fn build(n: &NodeSpec) -> Result<Row, String> {
            let op = crate::ops::get(n.op).ok_or_else(|| format!("no operator {}", n.op))?;
            let mut row = Row::with_operator(op);
            for (j, fill) in n.fill {
                match fill {
                    Fill::Text(t) => {
                        if let Some(slot) = row.inputs.get_mut(*j) {
                            *slot = t.to_string();
                        }
                    }
                    Fill::Row(_) => {} // a chained box starts empty: `=` fills it from the earlier row's answer
                    Fill::Node(sub) => {
                        let child = build(sub)?;
                        if *j < row.children.len() {
                            row.children[*j] = Some(Box::new(child));
                        }
                    }
                }
            }
            Ok(row)
        }
        while self.rows.len() < spec.rows.len() {
            self.rows.push(Row::blank());
        }
        for r in self.rows.iter_mut() {
            *r = Row::blank();
        }
        for (k, rs) in spec.rows.iter().enumerate() {
            self.rows[k] = build(&rs.node)?;
            if k > 0 {
                self.rows[k].join = Join::from_symbol(spec.joins[k - 1]);
            }
        }
        self.template = Some(crate::multirow::Template { key, spec });
        self.target_row = 0;
        self.target_path = self.rows[0].first_box();
        Ok(())
    }

    /// Boxes that a multi-row operator links (the same u and point in every row) copy the value typed into their source box.
    pub fn sync_links(&mut self) {
        let Some(t) = self.template else { return };
        for ((dr, dp), (sr, sp)) in t.spec.links {
            let Some(src) = self.rows.get(*sr).and_then(|r| r.slot_text(sp)).cloned() else { continue };
            if let Some(dst) = self.rows.get_mut(*dr).and_then(|r| r.slot_mut(dp)) {
                *dst = src;
            }
        }
    }

    /// Empty every box that reads an earlier row's answer (they show "row k" until `=` fills them in).
    pub fn forget_chains(&mut self) {
        use crate::multirow::Fill;
        let Some(t) = self.template else { return };
        for (i, rs) in t.spec.rows.iter().enumerate() {
            for (j, fill) in rs.node.fill {
                if let Fill::Row(_) = fill {
                    if let Some(slot) = self.rows.get_mut(i).and_then(|r| r.inputs.get_mut(*j)) {
                        slot.clear();
                    }
                }
            }
        }
    }

    /// The row that the box at `path` of row `row` reads the answer of (a chained box), if it is one.
    pub fn chain_source(&self, row: usize, path: &[usize]) -> Option<usize> {
        use crate::multirow::Fill;
        let t = self.template?;
        if path.len() != 1 {
            return None;
        }
        t.spec.rows.get(row)?.node.fill.iter().find_map(|(j, f)| match f {
            Fill::Row(k) if *j == path[0] => Some(*k),
            _ => None,
        })
    }

    /// `=` for a multi-row operator: every row is calculated and the results are joined with the operator's joins.
    fn equals_template(&mut self, t: crate::multirow::Template) -> Result<String, String> {
        use crate::multirow::Fill;
        self.sync_links();
        self.forget_chains(); // a chained box never keeps an old answer
        let mut results: Vec<Value> = Vec::new();
        for (i, rs) in t.spec.rows.iter().enumerate() {
            if self.rows[i].is_blank() {
                return Err(format!("Row {} ({}) is empty: pick an operator for it.", i + 1, rs.title));
            }
            for (j, fill) in rs.node.fill {
                if let Fill::Row(k) = fill {
                    // the boxes that read an earlier row get its answer, as the exact number it is
                    match results.get(*k) {
                        Some(Value::Number(x)) => {
                            if let Some(slot) = self.rows[i].inputs.get_mut(*j) {
                                *slot = format!("{}", x);
                            }
                        }
                        Some(other) => {
                            return Err(format!("Row {} ({}) reads the answer of Row {}, which is {}: it needs a single number.", i + 1, rs.title, k + 1, other.show()))
                        }
                        None => return Err(format!("Row {} ({}) reads Row {}, which has no answer yet.", i + 1, rs.title, k + 1)),
                    }
                }
            }
            self.rows[i].compute();
            if let Some(e) = self.rows[i].error.clone() {
                return Err(format!("Row {} ({}): {}", i + 1, rs.title, e));
            }
            results.push(self.rows[i].value.clone().ok_or_else(|| format!("Row {} has no answer", i + 1))?);
        }
        let mut total = results[0].clone();
        for (k, sym) in t.spec.joins.iter().enumerate() {
            total = Join::from_symbol(sym).apply(&total, &results[k + 1])?;
        }
        let lines: Vec<String> = results.iter().enumerate().map(|(i, v)| format!("Row {} = {}", i + 1, v.show())).collect();
        Ok(format!("{}\n{}\njoined with {}  ->  {}", t.spec.title, lines.join("   |   "), t.spec.joins.join(" "), total.show()))
    }

    /// Take the math out of one row (the row stays, empty).
    pub fn clear_row_math(&mut self, i: usize) {
        if i < self.rows.len() {
            let join = self.rows[i].join;
            self.rows[i] = Row::blank();
            self.rows[i].join = join;
            if i == self.target_row {
                self.target_path = vec![0];
            }
        }
    }

    /// Next Row: from Row 1 to Row 2, and so on.  (The lines of one operator's math are not rows: only this button and the join signs make rows.)
    pub fn next_row(&mut self) {
        if self.target_row + 1 >= self.rows.len() {
            self.rows.push(Row::blank());
        }
        self.target_row += 1;
        self.target_path = vec![0];
        if self.rows[self.target_row].is_result {
            self.rows[self.target_row] = Row::blank(); // carry on from the answer row
        }
    }

    /// `=`: calculate every row that has math, join them with the join signs (Row 2 joins to Row 1, and so on), show each row's own answer in its own answer card
    /// and, when rows were joined, the answer for all of them in the row after the last one used (Row 3 for Rows 1 and 2).  Returns the message to show.
    pub fn equals_all(&mut self) -> Result<String, String> {
        if let Some(t) = self.template {
            return self.equals_template(t);
        }
        // a previous answer row is dropped; the rows go back to being ready for more math
        while self.rows.len() > 2 && self.rows.last().map_or(false, |r| r.is_result) {
            self.rows.pop();
        }
        for r in self.rows.iter_mut() {
            if r.is_result {
                *r = Row::blank();
            }
        }
        if self.rows[0].is_blank() {
            return Err("Pick an operator for Row 1 (open Operators), type its numbers, then press =.".into());
        }
        let mut total: Option<Value> = None;
        let (mut used, mut last) = (0usize, 0usize);
        for k in 0..self.rows.len() {
            if self.rows[k].is_blank() {
                continue;
            }
            self.rows[k].compute();
            if let Some(e) = self.rows[k].error.clone() {
                return Err(if k == 0 { e } else { format!("Row {}: {}", k + 1, e) });
            }
            let v = self.rows[k].value.clone().ok_or_else(|| format!("Row {} has no answer", k + 1))?;
            total = Some(match total {
                None => v,
                Some(t) => {
                    if self.rows[k].join == Join::None {
                        return Err(format!("Row {} needs a join symbol: pick + − × ÷ = above its boxes.", k + 1));
                    }
                    self.rows[k].join.apply(&t, &v)?
                }
            });
            used += 1;
            last = k;
        }
        let total = total.ok_or_else(|| "nothing to calculate".to_string())?;
        if used > 1 {
            if last + 1 >= self.rows.len() {
                self.rows.push(Row::blank());
            }
            let r = &mut self.rows[last + 1];
            *r = Row::blank();
            r.is_result = true;
            r.result = total.show();
            r.value = Some(total.clone());
            return Ok(format!("All {} rows joined = {}  (shown in Row {})", used, total.show(), last + 2));
        }
        Ok(format!("Row 1 = {}", total.show()))
    }

    /// A join key under the keypad (like pressing + on a calculator): put the symbol on the target row and carry on in that row, so the next digits are ITS number.
    /// A second join key after the number of an already joined row starts the NEXT row, so  1 + 1 + 1  makes three rows.
    pub fn join_key(&mut self, symbol: &str) -> String {
        self.ensure_target();
        let mut i = self.target_row.max(1);
        if i < self.rows.len() && self.rows[i].join != Join::None && !self.rows[i].is_result && !self.rows[i].is_blank() {
            i += 1;
        }
        while i >= self.rows.len() {
            self.rows.push(Row::blank());
        }
        if self.rows[i].is_result {
            self.rows[i] = Row::blank();
        }
        for r in self.rows.iter_mut() {
            r.clear_result();
            r.error = None;
        }
        self.rows[i].join = Join::from_symbol(symbol);
        self.target_row = i;
        self.target_path = vec![0];
        if self.rows[i].is_blank() {
            self.rows[i].ensure_number_box();
            format!("Row {} is joined to the rows above with  {} .  Type its number, then press =.", i + 1, symbol)
        } else {
            format!("Row {} is joined to the rows above with  {} .  Press = to calculate.", i + 1, symbol)
        }
    }

    pub fn append_digit(&mut self, ch: char) {
        self.ensure_target();
        self.rows[self.target_row].ensure_number_box();
        let path = self.target_path.clone();
        let row = &mut self.rows[self.target_row];
        if let Some(slot) = row.slot_mut(&path) {
            slot.push(ch);
            row.clear_result();
            row.error = None;
        }
    }

    pub fn backspace(&mut self) {
        self.ensure_target();
        let path = self.target_path.clone();
        let row = &mut self.rows[self.target_row];
        if let Some(slot) = row.slot_mut(&path) {
            slot.pop();
            row.clear_result();
            row.error = None;
        }
    }

    pub fn clear_entry(&mut self) {
        self.ensure_target();
        let path = self.target_path.clone();
        let row = &mut self.rows[self.target_row];
        if let Some(slot) = row.slot_mut(&path) {
            slot.clear();
            row.clear_result();
            row.error = None;
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
