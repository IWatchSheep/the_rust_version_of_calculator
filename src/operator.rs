//! Operator metadata (mirrors Python NAME / SYMBOL / FORMULA / CALC / INPUTS).

use crate::value::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcKind {
    /// Boxes feed `compute`; result appears right of `=`.
    Ready,
    /// Formula / definition only for now.
    Reference,
}

#[derive(Clone, Copy, Debug)]
pub struct Operator {
    pub key: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub category: &'static str,
    pub subcategory: &'static str,
    pub formula: &'static str,
    pub pattern: &'static str,
    pub inputs: &'static [&'static str],
    pub calc: CalcKind,
    /// Optional trailing arguments (name, default) such as `c` = 299792458 in the physics operators.
    /// The row boxes fill `inputs`; `compute` always receives `inputs.len() + extra.len()` numbers.
    pub extra: &'static [(&'static str, f64)],
    /// Takes any number of values (Python `compute(*values)`): the row boxes are only a starting point.
    pub variadic: bool,
    /// What each box holds, in Advanced Input terms (`number`, `list`, `matrix`, `function`, `set` ...).  Empty = all numbers.
    pub kinds: &'static [&'static str],
    /// Operators whose boxes are not all numbers (or whose answer is a list, matrix, set ...) compute here.
    pub compute_v: Option<fn(&[Value]) -> Result<Value, String>>,
    pub compute: fn(&[f64]) -> Result<Value, String>,
}

impl Operator {
    pub fn arity(&self) -> usize {
        self.inputs.len()
    }

    /// What `=` answers for a formula-only operator: the old sentence, then its row's note ("Formula only. Needs the curve. ...").
    fn reference_message(&self) -> String {
        let note = crate::layout::note(self.key);
        if note.is_empty() {
            "this operator is reference-only (no calculation yet)".into()
        } else {
            format!("this operator is reference-only (no calculation yet). {note}")
        }
    }

    pub fn run(&self, args: &[f64]) -> Result<Value, String> {
        if self.calc != CalcKind::Ready {
            return Err(self.reference_message());
        }
        let result = self.run_checked(args)?;
        if self.compute_v.is_some() {
            return Ok(result); // typed operators decide for themselves (infinity is a legitimate answer)
        }
        // Python raises on 1/0 and on math domain errors; in Rust those give inf / NaN, so report them as errors.
        if result.is_finite_result() {
            Ok(result)
        } else {
            Err("result is not a finite number (division by zero or out of range)".into())
        }
    }

    /// Run with typed values (lists, matrices, sets, functions ...).  Numbers-only operators accept numbers.
    pub fn run_values(&self, args: &[Value]) -> Result<Value, String> {
        if self.calc != CalcKind::Ready {
            return Err(self.reference_message());
        }
        let Some(f) = self.compute_v else {
            let nums: Result<Vec<f64>, String> = args.iter().map(|v| v.as_number()).collect();
            return self.run(&nums?);
        };
        let max = self.arity() + self.extra.len();
        if args.len() < self.arity() || args.len() > max {
            return Err(format!("{} needs {} input(s), got {}", self.name, self.arity(), args.len()));
        }
        let mut full = args.to_vec();
        for (_, default) in &self.extra[args.len() - self.arity()..] {
            full.push(Value::Number(*default));
        }
        f(&full)
    }

    fn run_checked(&self, args: &[f64]) -> Result<Value, String> {
        if self.compute_v.is_some() {
            let vals: Vec<Value> = args.iter().map(|a| Value::Number(*a)).collect();
            return self.run_values(&vals);
        }
        if self.variadic {
            if args.is_empty() {
                return Err(format!("{} needs at least one value", self.name));
            }
            return (self.compute)(args);
        }
        let max = self.arity() + self.extra.len();
        if args.len() < self.arity() || args.len() > max {
            return Err(format!(
                "{} needs {} input(s), got {}",
                self.name,
                self.arity(),
                args.len()
            ));
        }
        if self.extra.is_empty() {
            return (self.compute)(args);
        }
        let mut full = args.to_vec();
        for (_, default) in &self.extra[args.len() - self.arity()..] {
            full.push(*default);
        }
        (self.compute)(&full)
    }
}
