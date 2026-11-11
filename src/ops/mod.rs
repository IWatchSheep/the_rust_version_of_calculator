//! Phase-1 operator registry: arithmetic, algebra, trig, geometry, discrete, physics.

use crate::operator::{CalcKind, Operator};
use crate::value::Value;

macro_rules! op1 {
    ($f:expr) => {
        |a: &[f64]| -> Result<Value, String> { Ok(Value::Number(($f)(a[0]))) }
    };
}
macro_rules! op2 {
    ($f:expr) => {
        |a: &[f64]| -> Result<Value, String> { Ok(Value::Number(($f)(a[0], a[1]))) }
    };
}
macro_rules! op3 {
    ($f:expr) => {
        |a: &[f64]| -> Result<Value, String> { Ok(Value::Number(($f)(a[0], a[1], a[2]))) }
    };
}

fn need(cond: bool, msg: &str) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(msg.into())
    }
}

mod generated;
mod geometry;
mod calculus;
mod graphs;
mod lists;
mod math;
mod matrices;
mod misc;
mod sets;
mod vals;
mod physics;
pub mod words;
mod words_data;

/// Every operator: the hand-written ones below, then the ones generated from the catalog
/// (`operator_keys/make_rust_ops.py`).
pub fn all_operators() -> &'static [Operator] {
    static ALL: std::sync::OnceLock<Vec<Operator>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| OPS.iter().chain(generated::GENERATED.iter()).copied().collect())
}

/// The plain number a row can hold around a bracket: one box, any value passes through.
static NUMBER_OP: Operator = Operator {
    key: "_number",
    name: "Number",
    symbol: "",
    category: "",
    subcategory: "",
    formula: "□",
    pattern: "□",
    inputs: &["value"],
    calc: CalcKind::Ready,
    extra: &[],
    variadic: false,
    kinds: &[],
    compute: |a: &[f64]| -> Result<Value, String> { Ok(Value::Number(a[0])) },
    compute_v: Some(|a: &[Value]| -> Result<Value, String> { Ok(a[0].clone()) }),
};

pub fn get(key: &str) -> Option<&'static Operator> {
    if key == "_number" {
        return Some(&NUMBER_OP);
    }
    all_operators().iter().find(|o| o.key == key).or_else(|| if key.starts_with("word:") { words::get(key) } else { None })
}

const OPS: &[Operator] = &[
    // ---- Arithmetic ----
    Operator {
        key: "math/arithmetic/addition",
        name: "Addition",
        symbol: "+",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "a + b",
        pattern: "□ + □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|a, b| a + b),
    },
    Operator {
        key: "math/arithmetic/subtraction",
        name: "Subtraction",
        symbol: "−",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "a − b",
        pattern: "□ − □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|a, b| a - b),
    },
    Operator {
        key: "math/arithmetic/multiplication",
        name: "Multiplication",
        symbol: "×",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "a × b",
        pattern: "□ × □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|a, b| a * b),
    },
    Operator {
        key: "math/arithmetic/division",
        name: "Division",
        symbol: "÷",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "a ÷ b",
        pattern: "□ ÷ □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "division by zero")?;
            Ok(Value::Number(a[0] / a[1]))
        },
    },
    Operator {
        key: "math/arithmetic/modulo",
        name: "Modulo",
        symbol: "%",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "a mod b = a − b·⌊a/b⌋",
        pattern: "□ mod □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "modulo by zero")?;
            // Python's % : the result takes the sign of the divisor
            let r = a[0] % a[1];
            Ok(Value::Number(if r != 0.0 && (r < 0.0) != (a[1] < 0.0) { r + a[1] } else { r }))
        },
    },
    Operator {
        key: "math/arithmetic/fraction",
        name: "Fraction",
        symbol: "a/b",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "numerator / denominator",
        pattern: "□ / □ = □",
        inputs: &["numerator", "denominator"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "denominator is zero")?;
            Ok(Value::Number(a[0] / a[1]))
        },
    },
    Operator {
        key: "math/arithmetic/absolute_value",
        name: "Absolute value",
        symbol: "|x|",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "|x| = x if x ≥ 0, otherwise −x",
        pattern: "|□| = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.abs()),
    },
    Operator {
        key: "math/arithmetic/negate",
        name: "Negate",
        symbol: "±",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "−x",
        pattern: "−□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| -x),
    },
    Operator {
        key: "math/arithmetic/reciprocal",
        name: "Reciprocal",
        symbol: "1/x",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "1/x",
        pattern: "1/□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] != 0.0, "reciprocal of zero")?;
            Ok(Value::Number(1.0 / a[0]))
        },
    },
    Operator {
        key: "math/arithmetic/round_down",
        name: "Round down",
        symbol: "floor",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "⌊x⌋ = greatest integer ≤ x",
        pattern: "⌊□⌋ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.floor()),
    },
    Operator {
        key: "math/arithmetic/round_up",
        name: "Round up",
        symbol: "ceil",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "⌈x⌉ = smallest integer ≥ x",
        pattern: "⌈□⌉ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.ceil()),
    },
    Operator {
        key: "math/arithmetic/round_nearest",
        name: "Round nearest",
        symbol: "round",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "round(x) = nearest integer to x",
        pattern: "round(□) = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.round_ties_even()),   // Python round(): halves go to the even neighbour
    },
    Operator {
        key: "math/arithmetic/minimum",
        name: "Minimum",
        symbol: "min",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "min(a, b, …) = smallest value",
        pattern: "min(□, □, □) = □",
        inputs: &["x1", "x2", "x3"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: true,
        kinds: &[],
        compute_v: None,
        compute: |a| Ok(Value::Number(a.iter().cloned().fold(f64::INFINITY, f64::min))),
    },
    Operator {
        key: "math/arithmetic/maximum",
        name: "Maximum",
        symbol: "max",
        category: "Math",
        subcategory: "Arithmetic",
        formula: "max(a, b, …) = largest value",
        pattern: "max(□, □, □) = □",
        inputs: &["x1", "x2", "x3"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: true,
        kinds: &[],
        compute_v: None,
        compute: |a| Ok(Value::Number(a.iter().cloned().fold(f64::NEG_INFINITY, f64::max))),
    },
    // ---- Algebra ----
    Operator {
        key: "math/algebra/power",
        name: "Power",
        symbol: "^",
        category: "Math",
        subcategory: "Algebra",
        formula: "aᵇ",
        pattern: "□ ^ □ = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|a: f64, b: f64| a.powf(b)),
    },
    Operator {
        key: "math/algebra/square",
        name: "Square",
        symbol: "x²",
        category: "Math",
        subcategory: "Algebra",
        formula: "x² = x · x",
        pattern: "□² = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x * x),
    },
    Operator {
        key: "math/algebra/cube",
        name: "Cube",
        symbol: "x³",
        category: "Math",
        subcategory: "Algebra",
        formula: "x³ = x · x · x",
        pattern: "□³ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x * x * x),
    },
    Operator {
        key: "math/algebra/square_root",
        name: "Square root",
        symbol: "√",
        category: "Math",
        subcategory: "Algebra",
        formula: "√x",
        pattern: "√□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] >= 0.0, "square root of a negative number")?;
            Ok(Value::Number(a[0].sqrt()))
        },
    },
    Operator {
        key: "math/algebra/nth_root",
        name: "Nth root",
        symbol: "ⁿ√",
        category: "Math",
        subcategory: "Algebra",
        formula: "ⁿ√x = x^(1/n)",
        pattern: "□√□ = □",
        inputs: &["x", "n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "root degree cannot be zero")?;
            Ok(Value::Number(a[0].powf(1.0 / a[1])))
        },
    },
    Operator {
        key: "math/algebra/cube_root",
        name: "Cube root",
        symbol: "∛",
        category: "Math",
        subcategory: "Algebra",
        formula: "∛x",
        pattern: "∛□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.cbrt()),
    },
    Operator {
        key: "math/algebra/exponential",
        name: "Exponential",
        symbol: "eˣ",
        category: "Math",
        subcategory: "Algebra",
        formula: "eˣ",
        pattern: "e^□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.exp()),
    },
    Operator {
        key: "math/algebra/power_of_ten",
        name: "Power of ten",
        symbol: "10ˣ",
        category: "Math",
        subcategory: "Algebra",
        formula: "10ˣ",
        pattern: "10^□ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| 10f64.powf(x)),
    },
    Operator {
        key: "math/algebra/log_base_10",
        name: "Log base 10",
        symbol: "log",
        category: "Math",
        subcategory: "Algebra",
        formula: "log₁₀(x)",
        pattern: "log □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] > 0.0, "log of a non-positive number")?;
            Ok(Value::Number(a[0].log10()))
        },
    },
    Operator {
        key: "math/algebra/natural_log",
        name: "Natural log",
        symbol: "ln",
        category: "Math",
        subcategory: "Algebra",
        formula: "ln(x) = logₑ(x)",
        pattern: "ln □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] > 0.0, "ln of a non-positive number")?;
            Ok(Value::Number(a[0].ln()))
        },
    },
    Operator {
        key: "math/algebra/log_any_base",
        name: "Log any base",
        symbol: "logₐ",
        category: "Math",
        subcategory: "Algebra",
        formula: "logₐ(x) = ln(x) / ln(a)",
        pattern: "log_□(□) = □",
        inputs: &["x", "base"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] > 0.0 && a[1] != 1.0, "base must be positive and ≠ 1")?;
            need(a[0] > 0.0, "log of a non-positive number")?;
            Ok(Value::Number(a[0].ln() / a[1].ln()))
        },
    },
    Operator {
        key: "math/algebra/quadratic_formula",
        name: "Quadratic formula",
        symbol: "ax²+bx+c=0",
        category: "Math",
        subcategory: "Algebra",
        formula: "x = (−b ± √(b² − 4ac)) / 2a",
        pattern: "□x²+□x+□=0 → □",
        inputs: &["a", "b", "c"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] != 0.0, "a cannot be zero")?;
            let disc = a[1] * a[1] - 4.0 * a[0] * a[2];
            need(disc >= 0.0, "complex roots not shown yet (discriminant < 0)")?;
            let s = disc.sqrt();
            let x1 = (-a[1] + s) / (2.0 * a[0]);
            let x2 = (-a[1] - s) / (2.0 * a[0]);
            Ok(Value::Pair(x1, x2))
        },
    },
    Operator {
        key: "math/algebra/discriminant",
        name: "Discriminant",
        symbol: "b²−4ac",
        category: "Math",
        subcategory: "Algebra",
        formula: "b² − 4ac",
        pattern: "□²−4□□ = □",
        inputs: &["a", "b", "c"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op3!(|a, b, c| b * b - 4.0 * a * c),
    },
    Operator {
        key: "math/algebra/slope_intercept_form",
        name: "Slope-intercept",
        symbol: "y=mx+b",
        category: "Math",
        subcategory: "Algebra",
        formula: "y = mx + b",
        pattern: "□·□+□ = □",
        inputs: &["m", "x", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op3!(|m, x, b| m * x + b),
    },
    Operator {
        key: "math/algebra/arithmetic_sequence",
        name: "Arithmetic sequence",
        symbol: "aₙ",
        category: "Math",
        subcategory: "Algebra",
        formula: "aₙ = a₁ + (n−1)d",
        pattern: "a₁=□, d=□, n=□ → □",
        inputs: &["a1", "d", "n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op3!(|a1, d, n| a1 + (n - 1.0) * d),
    },
    Operator {
        key: "math/algebra/geometric_sequence",
        name: "Geometric sequence",
        symbol: "aₙ=a₁rⁿ⁻¹",
        category: "Math",
        subcategory: "Algebra",
        formula: "aₙ = a₁ rⁿ⁻¹",
        pattern: "a₁=□, r=□, n=□ → □",
        inputs: &["a1", "r", "n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op3!(|a1: f64, r: f64, n: f64| a1 * r.powf(n - 1.0)),
    },
    Operator {
        key: "math/algebra/arithmetic_series_sum",
        name: "Arithmetic series sum",
        symbol: "Σ arith",
        category: "Math",
        subcategory: "Algebra",
        formula: "Sₙ = n/2 · (2a₁ + (n−1)d)",
        pattern: "a₁=□, d=□, n=□ → □",
        inputs: &["a1", "d", "n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op3!(|a1, d, n| n / 2.0 * (2.0 * a1 + (n - 1.0) * d)),
    },
    Operator {
        key: "math/algebra/geometric_series_sum",
        name: "Geometric series sum",
        symbol: "Σ geom",
        category: "Math",
        subcategory: "Algebra",
        formula: "Sₙ = a₁ (1−rⁿ)/(1−r)  (r≠1);  n·a₁ if r=1",
        pattern: "a₁=□, r=□, n=□ → □",
        inputs: &["a1", "r", "n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            let (a1, r, n) = (a[0], a[1], a[2]);
            if (r - 1.0).abs() < 1e-15 {
                Ok(Value::Number(n * a1))
            } else {
                Ok(Value::Number(a1 * (1.0 - r.powf(n)) / (1.0 - r)))
            }
        },
    },
    Operator {
        key: "math/algebra/difference_of_squares",
        name: "Difference of squares",
        symbol: "a²−b²",
        category: "Math",
        subcategory: "Algebra",
        formula: "a² − b² = (a−b)(a+b)",
        pattern: "□² − □² = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| Ok(Value::Pair(a[0] - a[1], a[0] + a[1])),
    },
    // ---- Trigonometry (radians) ----
    Operator {
        key: "math/trigonometry/sine",
        name: "Sine",
        symbol: "sin",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "sin θ",
        pattern: "sin □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|t: f64| t.sin()),
    },
    Operator {
        key: "math/trigonometry/cosine",
        name: "Cosine",
        symbol: "cos",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "cos θ",
        pattern: "cos □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|t: f64| t.cos()),
    },
    Operator {
        key: "math/trigonometry/tangent",
        name: "Tangent",
        symbol: "tan",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "tan θ = sin θ / cos θ",
        pattern: "tan □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|t: f64| t.tan()),
    },
    Operator {
        key: "math/trigonometry/cosecant",
        name: "Cosecant",
        symbol: "csc",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "csc θ = 1 / sin θ",
        pattern: "csc □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            let s = a[0].sin();
            need(s != 0.0, "csc undefined")?;
            Ok(Value::Number(1.0 / s))
        },
    },
    Operator {
        key: "math/trigonometry/secant",
        name: "Secant",
        symbol: "sec",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "sec θ = 1 / cos θ",
        pattern: "sec □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            let c = a[0].cos();
            need(c != 0.0, "sec undefined")?;
            Ok(Value::Number(1.0 / c))
        },
    },
    Operator {
        key: "math/trigonometry/cotangent",
        name: "Cotangent",
        symbol: "cot",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "cot θ = cos θ / sin θ",
        pattern: "cot □ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            let s = a[0].sin();
            need(s != 0.0, "cot undefined")?;
            Ok(Value::Number(a[0].cos() / s))
        },
    },
    Operator {
        key: "math/trigonometry/inverse_sine",
        name: "Inverse sine",
        symbol: "asin",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "θ = arcsin(x), −1 ≤ x ≤ 1",
        pattern: "asin □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need((-1.0..=1.0).contains(&a[0]), "asin domain is [-1, 1]")?;
            Ok(Value::Number(a[0].asin()))
        },
    },
    Operator {
        key: "math/trigonometry/inverse_cosine",
        name: "Inverse cosine",
        symbol: "acos",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "θ = arccos(x), −1 ≤ x ≤ 1",
        pattern: "acos □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need((-1.0..=1.0).contains(&a[0]), "acos domain is [-1, 1]")?;
            Ok(Value::Number(a[0].acos()))
        },
    },
    Operator {
        key: "math/trigonometry/inverse_tangent",
        name: "Inverse tangent",
        symbol: "atan",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "θ = arctan(x)",
        pattern: "atan □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.atan()),
    },
    Operator {
        key: "math/trigonometry/two_argument_arctangent",
        name: "Two-argument arctangent",
        symbol: "atan2",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "atan2(y, x)",
        pattern: "atan2(□, □) = □",
        inputs: &["y", "x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|y: f64, x: f64| y.atan2(x)),
    },
    Operator {
        key: "math/trigonometry/hyperbolic_sine",
        name: "Hyperbolic sine",
        symbol: "sinh",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "sinh x = (eˣ − e⁻ˣ) / 2",
        pattern: "sinh □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.sinh()),
    },
    Operator {
        key: "math/trigonometry/hyperbolic_cosine",
        name: "Hyperbolic cosine",
        symbol: "cosh",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "cosh x = (eˣ + e⁻ˣ) / 2",
        pattern: "cosh □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.cosh()),
    },
    Operator {
        key: "math/trigonometry/hyperbolic_tangent",
        name: "Hyperbolic tangent",
        symbol: "tanh",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "tanh x = sinh x / cosh x",
        pattern: "tanh □ = □",
        inputs: &["x"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|x: f64| x.tanh()),
    },
    Operator {
        key: "math/trigonometry/degrees_to_radians",
        name: "Degrees to radians",
        symbol: "deg→rad",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "radians = degrees × π / 180",
        pattern: "□° → □ rad",
        inputs: &["degrees"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|d: f64| d.to_radians()),
    },
    Operator {
        key: "math/trigonometry/radians_to_degrees",
        name: "Radians to degrees",
        symbol: "rad→deg",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "degrees = radians × 180 / π",
        pattern: "□ rad → □°",
        inputs: &["radians"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|r: f64| r.to_degrees()),
    },
    Operator {
        key: "math/trigonometry/double_angle",
        name: "Double angle",
        symbol: "sin 2θ",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "sin 2θ = 2 sin θ cos θ",
        pattern: "sin 2·□ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|t: f64| (2.0 * t).sin()),
    },
    Operator {
        key: "math/trigonometry/pythagorean_identity",
        name: "Pythagorean identity",
        symbol: "sin²θ+cos²θ=1",
        category: "Math",
        subcategory: "Trigonometry",
        formula: "sin²θ + cos²θ = 1",
        pattern: "sin²□+cos²□ = □",
        inputs: &["theta"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|t: f64| t.sin().powi(2) + t.cos().powi(2)),
    },
    // ---- Discrete ----
    Operator {
        key: "math/discrete_and_number_theory/factorial",
        name: "Factorial",
        symbol: "n!",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "n!",
        pattern: "□! = □",
        inputs: &["n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] >= 0.0 && a[0] == a[0].floor() && a[0] <= 170.0, "n must be an integer 0..170")?;
            let n = a[0] as u32;
            let mut r = 1.0;
            for k in 2..=n {
                r *= k as f64;
            }
            Ok(Value::Number(r))
        },
    },
    Operator {
        key: "math/discrete_and_number_theory/combinations",
        name: "Combinations",
        symbol: "nCr",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "C(n,r) = n! / (r!(n−r)!)",
        pattern: "C(□, □) = □",
        inputs: &["n", "r"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] >= 0.0 && a[1] >= 0.0 && a[0] == a[0].floor() && a[1] == a[1].floor(), "n,r integers")?;
            let n = a[0] as u64;
            let r = a[1] as u64;
            if r > n {
                return Ok(Value::Number(0.0)); // Python math.comb gives 0
            }
            let r = r.min(n - r);
            let mut c = 1.0;
            for i in 0..r {
                c = c * (n - i) as f64 / (i + 1) as f64;
            }
            Ok(Value::Number(c))
        },
    },
    Operator {
        key: "math/discrete_and_number_theory/permutations",
        name: "Permutations",
        symbol: "nPr",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "P(n,r) = n! / (n−r)!",
        pattern: "P(□, □) = □",
        inputs: &["n", "r"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] >= 0.0 && a[1] >= 0.0 && a[0] == a[0].floor() && a[1] == a[1].floor(), "n,r integers")?;
            let n = a[0] as u64;
            let r = a[1] as u64;
            if r > n {
                return Ok(Value::Number(0.0)); // Python math.perm gives 0
            }
            let mut p = 1.0;
            for i in 0..r {
                p *= (n - i) as f64;
            }
            Ok(Value::Number(p))
        },
    },
    Operator {
        key: "math/discrete_and_number_theory/greatest_common_divisor",
        name: "Greatest common divisor",
        symbol: "gcd",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "gcd(a, b)",
        pattern: "gcd(□, □) = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0].fract() == 0.0 && a[1].fract() == 0.0, "gcd needs whole numbers")?;
            let mut x = a[0].abs() as i64;
            let mut y = a[1].abs() as i64;
            while y != 0 {
                let t = x % y;
                x = y;
                y = t;
            }
            Ok(Value::Number(x as f64))
        },
    },
    Operator {
        key: "math/discrete_and_number_theory/least_common_multiple",
        name: "Least common multiple",
        symbol: "lcm",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "lcm(a, b) = |a b| / gcd(a,b)",
        pattern: "lcm(□, □) = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0].fract() == 0.0 && a[1].fract() == 0.0, "lcm needs whole numbers")?;
            let mut x = a[0].abs() as i64;
            let mut y = a[1].abs() as i64;
            let aa = x;
            let bb = y;
            while y != 0 {
                let t = x % y;
                x = y;
                y = t;
            }
            if x == 0 {
                return Ok(Value::Number(0.0)); // lcm(0, 0) = 0 in Python
            }
            Ok(Value::Number((aa / x * bb).abs() as f64))
        },
    },
    Operator {
        key: "math/discrete_and_number_theory/fibonacci",
        name: "Fibonacci",
        symbol: "Fₙ",
        category: "Math",
        subcategory: "Discrete and number theory",
        formula: "F₀=0, F₁=1, Fₙ=Fₙ₋₁+Fₙ₋₂",
        pattern: "F_□ = □",
        inputs: &["n"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0] >= 0.0 && a[0] == a[0].floor() && a[0] <= 90.0, "n integer 0..90")?;
            let n = a[0] as u32;
            if n == 0 {
                return Ok(Value::Number(0.0));
            }
            let (mut a0, mut a1) = (0.0, 1.0);
            for _ in 1..n {
                let t = a0 + a1;
                a0 = a1;
                a1 = t;
            }
            Ok(Value::Number(a1))
        },
    },
    // ---- Geometry ----
    Operator {
        key: "geometry/geometry/circle_area",
        name: "Circle area",
        symbol: "πr²",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "A = π r²",
        pattern: "π □² = □",
        inputs: &["r"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|r: f64| std::f64::consts::PI * r * r),
    },
    Operator {
        key: "geometry/geometry/circle_circumference",
        name: "Circle circumference",
        symbol: "2πr",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "C = 2 π r",
        pattern: "2π □ = □",
        inputs: &["r"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|r: f64| 2.0 * std::f64::consts::PI * r),
    },
    Operator {
        key: "geometry/geometry/rectangle_area",
        name: "Rectangle area",
        symbol: "l×w",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "A = l × w",
        pattern: "□ × □ = □",
        inputs: &["l", "w"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|l, w| l * w),
    },
    Operator {
        key: "geometry/geometry/triangle_area",
        name: "Triangle area",
        symbol: "½bh",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "A = ½ b h",
        pattern: "½ □ □ = □",
        inputs: &["b", "h"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|b, h| 0.5 * b * h),
    },
    Operator {
        key: "geometry/geometry/pythagorean_theorem",
        name: "Pythagorean theorem",
        symbol: "a²+b²=c²",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "c = √(a² + b²)",
        pattern: "√(□²+□²) = □",
        inputs: &["a", "b"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|a: f64, b: f64| (a * a + b * b).sqrt()),
    },
    Operator {
        key: "geometry/geometry/distance_formula",
        name: "Distance formula",
        symbol: "d",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "d = √((x₂−x₁)² + (y₂−y₁)²)",
        pattern: "d(□,□ → □,□) = □",
        inputs: &["x1", "y1", "x2", "y2"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            let dx = a[2] - a[0];
            let dy = a[3] - a[1];
            Ok(Value::Number((dx * dx + dy * dy).sqrt()))
        },
    },
    Operator {
        key: "geometry/geometry/sphere_volume",
        name: "Sphere volume",
        symbol: "⁴⁄₃πr³",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "V = ⁴⁄₃ π r³",
        pattern: "⁴⁄₃π □³ = □",
        inputs: &["r"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op1!(|r: f64| 4.0 / 3.0 * std::f64::consts::PI * r * r * r),
    },
    Operator {
        key: "geometry/geometry/cylinder_volume",
        name: "Cylinder volume",
        symbol: "πr²h",
        category: "Geometry",
        subcategory: "Geometry",
        formula: "V = π r² h",
        pattern: "π □² □ = □",
        inputs: &["r", "h"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|r, h| std::f64::consts::PI * r * r * h),
    },
    // ---- Physics (common) ----
    Operator {
        key: "physics/physics/velocity",
        name: "Velocity",
        symbol: "v=d/t",
        category: "Physics",
        subcategory: "Physics",
        formula: "v = d / t",
        pattern: "□ / □ = □",
        inputs: &["d", "t"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "time cannot be zero")?;
            Ok(Value::Number(a[0] / a[1]))
        },
    },
    Operator {
        key: "physics/physics/acceleration",
        name: "Acceleration",
        symbol: "a=Δv/t",
        category: "Physics",
        subcategory: "Physics",
        formula: "a = Δv / t",
        pattern: "□ / □ = □",
        inputs: &["delta_v", "t"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[1] != 0.0, "time cannot be zero")?;
            Ok(Value::Number(a[0] / a[1]))
        },
    },
    Operator {
        key: "physics/physics/newton_s_second_law",
        name: "Newton's 2nd law",
        symbol: "F=ma",
        category: "Physics",
        subcategory: "Physics",
        formula: "F = m a",
        pattern: "□ · □ = □",
        inputs: &["m", "a"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|m, a| m * a),
    },
    Operator {
        key: "physics/physics/kinetic_energy",
        name: "Kinetic energy",
        symbol: "½mv²",
        category: "Physics",
        subcategory: "Physics",
        formula: "K = ½ m v²",
        pattern: "½ □ □² = □",
        inputs: &["m", "v"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|m, v| 0.5 * m * v * v),
    },
    Operator {
        key: "physics/physics/potential_energy",
        name: "Potential energy",
        symbol: "mgh",
        category: "Physics",
        subcategory: "Physics",
        formula: "U = m g h",
        pattern: "□ · □ · □ = □",
        inputs: &["m", "h"],
        calc: CalcKind::Ready,
        extra: &[("g", 9.81)],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| Ok(Value::Number(a[0] * a[2] * a[1])),
    },
    Operator {
        key: "physics/physics/ohm_s_law",
        name: "Ohm's law",
        symbol: "V=IR",
        category: "Physics",
        subcategory: "Physics",
        formula: "V = I R",
        pattern: "□ · □ = □",
        inputs: &["I", "R"],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: op2!(|i, r| i * r),
    },
    Operator {
        key: "physics/physics/time_dilation",
        name: "Time dilation",
        symbol: "γΔt₀",
        category: "Physics",
        subcategory: "Physics",
        formula: "Δt = Δt₀ / √(1 − v²/c²)",
        pattern: "Δt₀=□, v=□ → □",
        inputs: &["dt0", "v"],
        calc: CalcKind::Ready,
        extra: &[("c", 299_792_458.0)],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            // the same expression and the same message as Python's compute(), so the two programs give the same float (also near the speed of light)
            need(a[1].abs() < a[2], "speed must be less than the speed of light")?;
            Ok(Value::Number(a[0] / (1.0 - a[1].powi(2) / a[2].powi(2)).sqrt()))
        },
    },
    Operator {
        key: "physics/physics/lorentz_factor",
        name: "Lorentz factor",
        symbol: "γ",
        category: "Physics",
        subcategory: "Physics",
        formula: "γ = 1 / √(1 − v²/c²)",
        pattern: "γ(□) = □",
        inputs: &["v"],
        calc: CalcKind::Ready,
        extra: &[("c", 299_792_458.0)],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |a| {
            need(a[0].abs() < a[1], "speed must be less than the speed of light")?;
            Ok(Value::Number(1.0 / (1.0 - a[0].powi(2) / a[1].powi(2)).sqrt()))
        },
    },
    // ---- Constants ----
    Operator {
        key: "math/constants_and_relations/pi",
        name: "Pi",
        symbol: "π",
        category: "Math",
        subcategory: "Constants and relations",
        formula: "π ≈ 3.14159…",
        pattern: "π = □",
        inputs: &[],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |_| Ok(Value::Number(std::f64::consts::PI)),
    },
    Operator {
        key: "math/constants_and_relations/euler_s_number",
        name: "Euler's number",
        symbol: "e",
        category: "Math",
        subcategory: "Constants and relations",
        formula: "e ≈ 2.71828…",
        pattern: "e = □",
        inputs: &[],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |_| Ok(Value::Number(std::f64::consts::E)),
    },
    Operator {
        key: "math/constants_and_relations/golden_ratio",
        name: "Golden ratio",
        symbol: "φ",
        category: "Math",
        subcategory: "Constants and relations",
        formula: "φ = (1 + √5) / 2",
        pattern: "φ = □",
        inputs: &[],
        calc: CalcKind::Ready,
        extra: &[],
        variadic: false,
        kinds: &[],
        compute_v: None,
        compute: |_| Ok(Value::Number((1.0 + 5f64.sqrt()) / 2.0)),
    },
];
