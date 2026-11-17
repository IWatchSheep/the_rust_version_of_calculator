//! Regression Tests: Verify No Legacy 8-Arrow GUI Appears During Regular Number Input and Calculation
//!
//! Verifies:
//! 1. Mathematical Row 1 at startup is a compact single number box (no legacy directional controls).
//! 2. Entering a number (e.g. 1 in E2) remains a compact single number box.
//! 3. Adding Row 2 (+ 1 in E8) keeps both rows compact and aligned.
//! 4. Calculating 1 + 1 = 2 displays Total 2 in M13, and no directional arrow grids appear.
//! 5. Focus navigation between E2 and E8 works via keyboard and keypad.
//! 6. Clear (C) restores a single empty row without arrows.
//! 7. Advanced operators placed from operator menu still function properly.

use rust_calculator::app::CalculatorApp;
use rust_calculator::layout::CardKind;
use rust_calculator::ops;
use rust_calculator::row::{Document, Join};

#[test]
fn test_startup_row1_is_compact_number_box() {
    let app = CalculatorApp::default();
    assert_eq!(app.doc.rows.len(), 1, "Startup has exactly 1 mathematical row");
    assert_eq!(app.doc.target_row, 0);

    let cards = app.doc.rows[0].flatten();
    // In startup state, cards is empty or a single Box
    let is_single_number_box = cards.is_empty()
        || (cards.len() == 1 && cards[0].kind == CardKind::Box);
    assert!(is_single_number_box, "Startup Row 1 must be a compact single number box, not legacy arrow grid");
}

#[test]
fn test_entering_number_preserves_compact_number_box() {
    let mut app = CalculatorApp::default();
    app.on_key("1");

    assert_eq!(app.doc.rows.len(), 1);
    assert_eq!(app.doc.rows[0].inputs.first().map(|s| s.as_str()), Some("1"));

    let cards = app.doc.rows[0].flatten();
    assert_eq!(cards.len(), 1, "Entering a number produces exactly 1 box card");
    assert_eq!(cards[0].kind, CardKind::Box);

    let is_single_number_box = cards.is_empty()
        || (cards.len() == 1 && cards[0].kind == CardKind::Box);
    assert!(is_single_number_box, "Entering digits must remain a compact single number box without 8 arrows");
}

#[test]
fn test_adding_row2_preserves_compact_number_boxes() {
    let mut app = CalculatorApp::default();
    app.on_key("1");

    // Add Row 2 with +
    let _ = app.doc.join_key("+");
    assert_eq!(app.doc.rows.len(), 2);
    assert_eq!(app.doc.target_row, 1);
    assert_eq!(app.doc.rows[1].join, Join::Add);

    // Enter 1 in Row 2
    app.on_key("1");
    assert_eq!(app.doc.rows[1].inputs.first().map(|s| s.as_str()), Some("1"));

    // Check Row 1
    let cards1 = app.doc.rows[0].flatten();
    let is_box1 = cards1.is_empty() || (cards1.len() == 1 && cards1[0].kind == CardKind::Box);
    assert!(is_box1, "Row 1 must remain a compact single number box");

    // Check Row 2
    let cards2 = app.doc.rows[1].flatten();
    let is_box2 = cards2.is_empty() || (cards2.len() == 1 && cards2[0].kind == CardKind::Box);
    assert!(is_box2, "Row 2 must be a compact single number box");

    // No Total row exists yet
    assert!(app.doc.rows.iter().all(|r| !r.is_result));
}

#[test]
fn test_equals_calculation_1_plus_1_equals_2_no_arrows() {
    let mut app = CalculatorApp::default();
    app.on_key("1");
    let _ = app.doc.join_key("+");
    app.on_key("1");

    // Press =
    app.equals();

    assert_eq!(app.doc.rows.len(), 3, "Rows 1, 2, and Total");
    assert_eq!(app.doc.rows[0].result, "1");
    assert_eq!(app.doc.rows[1].result, "1");
    assert!(app.doc.rows[2].is_result);
    assert_eq!(app.doc.rows[2].result, "2", "1 + 1 = 2");

    // Ensure mathematical rows remain single number boxes after calculation
    let cards1 = app.doc.rows[0].flatten();
    let is_box1 = cards1.is_empty() || (cards1.len() == 1 && cards1[0].kind == CardKind::Box);
    assert!(is_box1, "Row 1 remains a compact number box after =");

    let cards2 = app.doc.rows[1].flatten();
    let is_box2 = cards2.is_empty() || (cards2.len() == 1 && cards2[0].kind == CardKind::Box);
    assert!(is_box2, "Row 2 remains a compact number box after =");
}

#[test]
fn test_focus_navigation_between_e2_and_e8() {
    let mut app = CalculatorApp::default();
    app.on_key("1");
    let _ = app.doc.join_key("+");
    app.on_key("5");

    // Initially target is Row 2 (index 1)
    assert_eq!(app.doc.target_row, 1);

    // Switch focus to Row 1 (e.g. user clicks E2)
    app.doc.target_row = 0;
    app.doc.target_path = vec![0];
    assert_eq!(app.doc.target_row, 0);

    // Append digit to Row 1
    app.on_key("0");
    assert_eq!(app.doc.rows[0].inputs.first().map(|s| s.as_str()), Some("10"));
    assert_eq!(app.doc.rows[1].inputs.first().map(|s| s.as_str()), Some("5"));

    // Switch focus back to Row 2 (e.g. user clicks E8 or keypad Next Row)
    app.doc.target_row = 1;
    app.doc.target_path = vec![0];
    app.on_key("2");
    assert_eq!(app.doc.rows[0].inputs.first().map(|s| s.as_str()), Some("10"));
    assert_eq!(app.doc.rows[1].inputs.first().map(|s| s.as_str()), Some("52"));
}

#[test]
fn test_clear_restores_single_empty_row() {
    let mut app = CalculatorApp::default();
    app.on_key("1");
    let _ = app.doc.join_key("+");
    app.on_key("1");
    app.equals();
    assert_eq!(app.doc.rows.len(), 3);

    // Clear
    app.doc.clear();
    assert_eq!(app.doc.rows.len(), 1, "Clear restores exactly 1 row");
    assert_eq!(app.doc.target_row, 0);
    assert!(app.doc.rows[0].is_blank());

    let cards = app.doc.rows[0].flatten();
    let is_box = cards.is_empty() || (cards.len() == 1 && cards[0].kind == CardKind::Box);
    assert!(is_box, "Cleared row is a compact number box");
}

#[test]
fn test_advanced_operator_preserves_functionality() {
    let mut doc = Document::new();
    let op = ops::get("math/arithmetic/addition").expect("addition operator exists");
    doc.place_operator(op);

    // a + b has multiple cards (box, op, box)
    let cards = doc.rows[0].flatten();
    assert_eq!(cards.len(), 3, "add operator produces 3 cards: a, +, b");
    assert_eq!(cards[0].kind, CardKind::Box);
    assert_eq!(cards[1].kind, CardKind::Op);
    assert_eq!(cards[2].kind, CardKind::Box);

    // Multi-card operator is correctly detected as non-single-number-box
    let is_single = cards.is_empty() || (cards.len() == 1 && cards[0].kind == CardKind::Box);
    assert!(!is_single, "Multi-card formula operator is not treated as a plain number box");
}
