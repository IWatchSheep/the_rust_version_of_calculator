//! How an operator's row is drawn: its picture written as cards on a grid.
//!
//! The design rules (the same as the Python program's README):
//!   1. Math that can be written on several lines is always written on several lines: a fraction has its numerator on the row above its bar and its denominator on
//!      the row below, an exponent is one line up, a subscript (or the lower limit of a sigma) one line down.
//!   2. Those lines are ONE calculator row.  They are not Row 2, Row 3 ...; Next Row goes from Row 1 to Row 2.
//!   3. The answer is shown to the right of the `=`, on the same row.
//!   4. Row 1 is joined to Row 2 by a math sign; the answer for both is on Row 3.
//!   5. The row follows the operator's picture.
//!
//! `layouts.rs` (generated from the Python row files) holds the cards; this module places them on the grid.

use std::collections::HashMap;
use std::sync::OnceLock;

/// What a card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardKind {
    /// A symbol, letter, number or word: an ordinary card.
    Op,
    /// An input box: the person types here.  `slot` is which input it shows (a variable written twice has two boxes with one slot).
    Box,
    /// The formula's answer box: it cannot be typed into and shows the answer after `=`.
    Out,
    /// A box the picture draws that is not an input: an ordinary card (a square, or a letter).
    Dec,
}

pub const O: CardKind = CardKind::Op;
pub const B: CardKind = CardKind::Box;
pub const U: CardKind = CardKind::Out;
pub const D: CardKind = CardKind::Dec;

/// The parts of a fraction a card can be.
pub const NUM: u8 = 0;
pub const DEN: u8 = 1;
/// The "/" card: not drawn (the bar is drawn instead).
pub const BAR: u8 = 2;
/// Brackets that only wrapped a whole numerator or denominator: not drawn.
pub const HIDE: u8 = 3;

pub const N: &[(u16, u8)] = &[];

#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub kind: CardKind,
    pub text: &'static str,
    /// Which input a Box card shows; -1 for the other kinds.
    pub slot: i32,
    /// Grid row relative to the line: -1 = an exponent, +1 = a subscript, 0 = on the line.
    pub level: i8,
    /// The fractions this card is inside, outermost first: (group, NUM / DEN / BAR / HIDE).
    pub frac: &'static [(u16, u8)],
}

pub const fn card(kind: CardKind, text: &'static str, slot: i32, level: i8, frac: &'static [(u16, u8)]) -> Card {
    Card { kind, text, slot, level, frac }
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub key: &'static str,
    /// The picture ends with its own "= [ ]" result box.
    pub right_box: bool,
    pub cards: &'static [Card],
}

/// One card of a row once the operators inside its boxes are written out in place (Python `row_tree.flatten`).
#[derive(Clone, Debug)]
pub struct FlatCard {
    pub kind: CardKind,
    /// Operator cards: the symbol; a box that holds an operator it copies: the operator's name.
    pub text: String,
    /// Box cards: the path of the slot it shows (the first number is the slot of the row's own operator, the next one a slot of the operator inside it ...).
    pub path: Vec<usize>,
    pub level: i8,
    pub frac: Vec<(u32, u8)>,
    /// Out cards: which answer box of its operator this is.
    pub out_k: usize,
    /// Out cards: which operator (by walking order) owns this answer box, so the screen can ask for its answer.
    pub owner: Vec<usize>,
}

impl FlatCard {
    pub fn pc(&self) -> PC {
        PC { level: self.level, frac: self.frac.clone() }
    }
}

/// The layout of the plain number `_number` (a row around a bracket: one box, nothing else).
static NUMBER_LAYOUT: Layout = Layout { key: "_number", right_box: false, cards: &[card(B, "", 0, 0, N)] };

pub fn get(key: &str) -> Option<&'static Layout> {
    if key == "_number" {
        return Some(&NUMBER_LAYOUT);
    }
    static MAP: OnceLock<HashMap<&'static str, &'static Layout>> = OnceLock::new();
    MAP.get_or_init(|| crate::layouts::LAYOUTS.iter().chain(crate::word_layouts::WORD_LAYOUTS.iter()).map(|l| (l.key, l)).collect()).get(key).copied()
}

/// Where every card goes on the grid.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// For each card: Some((grid row, column)), or None for a card that is not drawn (a "/" or brackets that only wrapped a numerator).
    pub placed: Vec<Option<(i32, i32)>>,
    /// Every fraction bar: (grid row, first column, width in columns).
    pub bars: Vec<(i32, i32, i32)>,
    /// How many columns the formula takes.
    pub columns: i32,
}

/// What the placing code needs to know about a card: its grid level and the fractions it is inside (outermost first).
#[derive(Clone, Debug)]
pub struct PC {
    pub level: i8,
    pub frac: Vec<(u32, u8)>,
}

enum Item {
    Card(usize),
    Frac(Vec<Item>, Vec<Item>),
}

fn parse(cards: &[PC], indices: &[usize], depth: usize) -> Vec<Item> {
    let mut items = Vec::new();
    let mut k = 0;
    while k < indices.len() {
        let i = indices[k];
        let path = &cards[i].frac;
        if path.len() > depth {
            let gid = path[depth].0;
            let mut num = Vec::new();
            let mut den = Vec::new();
            while k < indices.len() && cards[indices[k]].frac.len() > depth && cards[indices[k]].frac[depth].0 == gid {
                let j = indices[k];
                match cards[j].frac[depth].1 {
                    NUM => num.push(j),
                    DEN => den.push(j),
                    _ => {}
                }
                k += 1;
            }
            items.push(Item::Frac(parse(cards, &num, depth + 1), parse(cards, &den, depth + 1)));
        } else {
            items.push(Item::Card(i));
            k += 1;
        }
    }
    items
}

/// With `stack`, an exponent card straight after the card it powers shares that card's column (one grid row up) instead of having a column of its own.
fn stacked(cards: &[PC], items: &[Item], k: usize, stack: bool) -> bool {
    stack && k > 0 && matches!(items[k], Item::Card(i) if cards[i].level == -1) && matches!(items[k - 1], Item::Card(_))
}

fn width(cards: &[PC], items: &[Item], stack: bool) -> i32 {
    items
        .iter()
        .enumerate()
        .map(|(k, it)| match it {
            Item::Card(_) => {
                if stacked(cards, items, k, stack) {
                    0
                } else {
                    1
                }
            }
            Item::Frac(n, d) => width(cards, n, stack).max(width(cards, d, stack)).max(1),
        })
        .sum()
}

/// (highest, lowest) grid offset of an item list relative to its own line: exponents and nested fractions included.
fn reach(cards: &[PC], items: &[Item]) -> (i32, i32) {
    let (mut hi, mut lo) = (0, 0);
    for it in items {
        match it {
            Item::Card(i) => {
                hi = hi.min(cards[*i].level as i32);
                lo = lo.max(cards[*i].level as i32);
            }
            Item::Frac(n, d) => {
                let (nb, db) = bases(cards, n, d);
                hi = hi.min(nb + reach(cards, n).0);
                lo = lo.max(db + reach(cards, d).1);
            }
        }
    }
    (hi, lo)
}

/// Where a fraction's numerator and denominator lines are, relative to its bar: one row off, or more when an exponent / nested fraction needs the room.
fn bases(cards: &[PC], n: &[Item], d: &[Item]) -> (i32, i32) {
    (-1 - reach(cards, n).1.max(0), 1 + (-reach(cards, d).0).max(0))
}

fn place(cards: &[PC], items: &[Item], mut col: i32, base: i32, stack: bool, plan: &mut Plan) -> i32 {
    for (k, it) in items.iter().enumerate() {
        match it {
            Item::Card(i) => {
                if stacked(cards, items, k, stack) {
                    plan.placed[*i] = Some((base + cards[*i].level as i32, col - 1));
                    continue;
                }
                plan.placed[*i] = Some((base + cards[*i].level as i32, col));
                col += 1;
            }
            Item::Frac(n, d) => {
                let w = width(cards, n, stack).max(width(cards, d, stack)).max(1);
                let (nb, db) = bases(cards, n, d);
                place(cards, n, col + (w - width(cards, n, stack)) / 2, base + nb, stack, plan);
                place(cards, d, col + (w - width(cards, d, stack)) / 2, base + db, stack, plan);
                plan.bars.push((base, col, w));
                col += w;
            }
        }
    }
    col
}

/// Place the cards of a layout: every card is its own column; a numerator is on the row above its bar and a denominator on the row below, centred over the bar.
/// The grid rows are shifted so the highest one is 0.
pub fn plan(cards: &[Card]) -> Plan {
    plan_stack(cards, false)
}

/// `plan`, with exponents sharing the column of the symbol they power when `stack` is true.
pub fn plan_stack(cards: &[Card], stack: bool) -> Plan {
    let pcs: Vec<PC> = cards.iter().map(|c| PC { level: c.level, frac: c.frac.iter().map(|&(g, p)| (g as u32, p)).collect() }).collect();
    plan_pc(&pcs, stack)
}

/// The words the cards of this operator's row carry by default: (card index, label, context) (Python `WORDS`).
pub fn words(key: &str) -> &'static [(usize, &'static str, &'static str)] {
    crate::layouts::WORDS.iter().find(|(k, _)| *k == key).map_or(&[], |(_, w)| *w)
}

/// The one sentence a formula-only operator's row says ("Formula only. ..."), or "" for any other operator (Python `NOTE`).
pub fn note(key: &str) -> &'static str {
    crate::layouts::NOTES.iter().find(|(k, _)| *k == key).map_or("", |(_, n)| *n)
}

/// Does this operator draw an exponent in the column of the symbol it powers (Python `STACK_EXPONENTS`)?
pub fn stacks(key: &str) -> bool {
    crate::layouts::STACKED.contains(&key)
}

/// Place cards given only by level and fractions (see `plan`).  `stack`: exponents share the column of the symbol they power (see `stacked`).
pub fn plan_pc(cards: &[PC], stack: bool) -> Plan {
    let shown: Vec<usize> = (0..cards.len())
        .filter(|&i| !matches!(cards[i].frac.last(), Some(&(_, p)) if p == BAR || p == HIDE))
        .collect();
    let items = parse(cards, &shown, 0);
    let mut plan = Plan { placed: vec![None; cards.len()], bars: Vec::new(), columns: 0 };
    plan.columns = place(cards, &items, 0, 0, stack, &mut plan);
    let top = plan
        .placed
        .iter()
        .flatten()
        .map(|&(r, _)| r)
        .chain(plan.bars.iter().map(|&(r, _, _)| r))
        .min()
        .unwrap_or(0);
    for p in plan.placed.iter_mut().flatten() {
        p.0 -= top;
    }
    for b in plan.bars.iter_mut() {
        b.0 -= top;
    }
    plan
}

impl Plan {
    /// How many grid rows the formula takes.
    pub fn rows(&self) -> i32 {
        let cards = self.placed.iter().flatten().map(|&(r, _)| r).max().unwrap_or(0);
        let bars = self.bars.iter().map(|&(r, _, _)| r).max().unwrap_or(0);
        cards.max(bars) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const F_NUM: &[(u16, u8)] = &[(1, NUM)];
    const F_DEN: &[(u16, u8)] = &[(1, DEN)];
    const F_BAR: &[(u16, u8)] = &[(1, BAR)];

    #[test]
    fn a_stacked_exponent_shares_the_column_of_the_symbol_it_powers() {
        // x ^2 (an exponent card after x): its own column normally, x's column when exponents are stacked
        let cards = [card(O, "x", -1, 0, N), card(O, "2", -1, -1, N), card(O, "+", -1, 0, N)];
        assert_eq!(plan(&cards).placed, vec![Some((1, 0)), Some((0, 1)), Some((1, 2))]);
        let p = plan_stack(&cards, true);
        assert_eq!(p.placed, vec![Some((1, 0)), Some((0, 0)), Some((1, 1))]);
        assert_eq!(p.columns, 2);
    }

    #[test]
    fn the_navier_stokes_rows_have_the_columns_and_grid_rows_of_their_diagram() {
        // (operator, columns, grid rows) as drawn in operators_one_operator_per_file/physics/physics/<operator>/<operator>.jpg (the Python row files in operator_row_math/ hold the same cards)
        for (key, columns, rows) in [
            ("physics/physics/navier_stokes_equation", 22, 3),
            ("physics/physics/navier_stokes_x_momentum", 36, 5),
            ("physics/physics/navier_stokes_y_momentum", 36, 5),
            ("physics/physics/navier_stokes_z_momentum", 36, 5),
            ("physics/physics/navier_stokes_continuity", 10, 3),
        ] {
            assert!(stacks(key), "{key} must stack its exponents");
            let p = plan_stack(get(key).unwrap().cards, true);
            assert_eq!((p.columns, p.rows()), (columns, rows), "{key}");
        }
    }

    #[test]
    fn a_fraction_is_numerator_over_bar_over_denominator() {
        // B B / B  ->  numerator of two cards over a denominator of one card
        let cards = [card(B, "", 0, 0, F_NUM), card(B, "", 1, 0, F_NUM), card(O, "/", -1, 0, F_BAR), card(B, "", 2, 0, F_DEN)];
        let p = plan(&cards);
        assert_eq!(p.placed, vec![Some((0, 0)), Some((0, 1)), None, Some((2, 0))]);
        assert_eq!(p.bars, vec![(1, 0, 2)]);
        assert_eq!(p.columns, 2);
        assert_eq!(p.rows(), 3);
    }

    #[test]
    fn an_exponent_is_its_own_card_one_row_up() {
        let cards = [card(U, "", -1, 0, N), card(O, "=", -1, 0, N), card(B, "", 0, 0, N), card(O, "2", -1, -1, N)];
        let p = plan(&cards);
        assert_eq!(p.placed, vec![Some((1, 0)), Some((1, 1)), Some((1, 2)), Some((0, 3))]);
    }

    #[test]
    fn an_exponent_in_a_denominator_stays_below_the_bar() {
        // 1 / r^2 : the 2 is one row up from the r, which is below the bar
        let n1: &[(u16, u8)] = &[(1, NUM)];
        let cards = [card(O, "1", -1, 0, n1), card(O, "/", -1, 0, F_BAR), card(B, "", 0, 0, F_DEN), card(O, "2", -1, -1, F_DEN)];
        let p = plan(&cards);
        let bar_row = p.bars[0].0;
        let (r_row, _) = p.placed[2].unwrap();
        let (two_row, _) = p.placed[3].unwrap();
        assert!(two_row > bar_row, "the exponent of the denominator must be below the bar");
        assert_eq!(two_row, r_row - 1);
    }

    #[test]
    fn every_operator_has_a_layout_whose_boxes_are_its_inputs() {
        assert_eq!(crate::layouts::LAYOUTS.len(), 631);
        for l in crate::layouts::LAYOUTS {
            let op = crate::ops::get(l.key).unwrap_or_else(|| panic!("no operator for layout {}", l.key));
            for c in l.cards {
                if c.kind == CardKind::Box {
                    assert!(c.slot >= 0, "{}", l.key);
                    // a calculating operator draws every input; a reference one may draw fewer (its picture is only a picture)
                    assert!((c.slot as usize) < op.arity().max(1) || op.calc == crate::operator::CalcKind::Reference, "{}", l.key);
                }
            }
            if op.calc == crate::operator::CalcKind::Ready {
                for s in 0..op.arity() {
                    assert!(l.cards.iter().any(|c| c.kind == CardKind::Box && c.slot == s as i32), "{}: input {} has no box", l.key, s);
                }
            }
            let p = plan(l.cards);
            assert_eq!(p.placed.len(), l.cards.len());
        }
    }
}
