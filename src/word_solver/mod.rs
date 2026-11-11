//! Algebra word-problem solver: Rust port of `algebra_word_problems/algebra_matcher/{word_solver,number_puzzle}.py`.
//!
//! A family answers ONLY when (1) its wording pattern matches, (2) EVERY number in the problem was given a role by the pattern, and (3) the answer passes a
//! sanity check.  Anything else is a decline (None).  A decline is always safe; a wrong number is the failure.

pub mod common;
pub mod families_a;
pub mod families_b;
pub mod families_c;
pub mod families_d;
pub mod number_puzzle;
pub mod patterns;
pub mod q;
pub mod rx;

pub use common::{answer_text, Family, SolveResult};
pub use number_puzzle::{solve as solve_number_puzzle_raw, Puzzle};
pub use q::Q;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;

/// Every family in Python's `FAMILIES` order, with its name.
pub fn families() -> Vec<(&'static str, Family)> {
    let mut all = families_a::list();
    all.extend(families_b::list());
    all.extend(families_c::list());
    all.extend(families_d::list());
    all
}

static QUIET: Once = Once::new();

/// A decline (division by zero, a number too big, a bad group) unwinds with `q::Decline`; don't print those as panics.
fn quiet_declines() {
    QUIET.call_once(|| {
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if info.payload().downcast_ref::<q::Decline>().is_none() {
                default(info);
            }
        }));
    });
}

/// Run a family; a decline (Python's ZeroDivisionError / ValueError / KeyError / IndexError) is None.
pub fn run_family(f: Family, problem: &str) -> Option<SolveResult> {
    quiet_declines();
    catch_unwind(AssertUnwindSafe(|| f(problem))).unwrap_or(None)
}

/// The number-puzzle reader; a decline is None.
pub fn solve_number_puzzle(problem: &str) -> Option<Puzzle> {
    quiet_declines();
    catch_unwind(AssertUnwindSafe(|| number_puzzle::solve(problem))).unwrap_or(None)
}

/// The result of the first family that matches and passes every check; otherwise None.  If two different families both claim the problem with different
/// answers, decline.
pub fn solve(problem: &str) -> Option<SolveResult> {
    let found: Vec<SolveResult> = families().into_iter().filter_map(|(_, f)| run_family(f, problem)).collect();
    let first = found.first()?.clone();
    if found[1..].iter().any(|f| f.values != first.values) {
        return None;
    }
    Some(first)
}
