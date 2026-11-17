//! Checkpoint 7E: PEMDAS Mathematical Evaluation Regression Test Suite
//!
//! Verifies:
//! 1. 2 + 3 × 4 = 14 (Multiplication evaluated before addition)
//! 2. 2 × 3 + 4 = 10 (Multiplication evaluated before addition)
//! 3. 20 ÷ 5 × 2 = 8 (Equal precedence ÷ and × evaluated left to right)
//! 4. 10 − 3 − 2 = 5 (Equal precedence − evaluated left to right)
//! 5. 10 − 3 × 2 = 4 (Multiplication evaluated before subtraction)
//! 6. 8 ÷ 2 ÷ 2 = 2 (Equal precedence ÷ evaluated left to right)
//! 7. 2 + 3 × 4 − 5 = 9 (Multiplication evaluated before addition and subtraction)
//! 8. 8 ÷ 0 produces a division-by-zero error.
//! 9. Single-row calculation displays in M2 without Total row.
//! 10. Both numerical results and mathematical analysis status text are verified.
//! 11. Preserves compact number input without 8-arrow legacy interface.

use rust_calculator::app::CalculatorApp;
use rust_calculator::layout::CardKind;
use rust_calculator::row::{Document, Join};

#[test]
fn test_pemdas_case1_add_mul_precedence() {
    // 2 + 3 × 4 = 14
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok(), "Calculation should succeed");
    let analysis_msg = res.unwrap();
    assert_eq!(analysis_msg, "All 3 rows joined = 14  (shown in Row 4)");

    // Verify row results
    assert_eq!(doc.rows[0].result, "2");
    assert_eq!(doc.rows[1].result, "3");
    assert_eq!(doc.rows[2].result, "4");

    // Verify Total row
    assert_eq!(doc.rows.len(), 4);
    assert!(doc.rows[3].is_result);
    assert_eq!(doc.rows[3].result, "14");
}

#[test]
fn test_pemdas_case2_mul_add_precedence() {
    // 2 × 3 + 4 = 10
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Mul;
    doc.rows[1].inputs = vec!["3".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 3 rows joined = 10  (shown in Row 4)");
    assert_eq!(doc.rows[3].result, "10");
}

#[test]
fn test_pemdas_case3_div_mul_left_to_right() {
    // 20 ÷ 5 × 2 = 8
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["20".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Div;
    doc.rows[1].inputs = vec!["5".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["2".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 3 rows joined = 8  (shown in Row 4)");
    assert_eq!(doc.rows[3].result, "8");
}

#[test]
fn test_pemdas_case4_sub_sub_left_to_right() {
    // 10 − 3 − 2 = 5
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["10".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Sub;
    doc.rows[1].inputs = vec!["3".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Sub;
    doc.rows[2].inputs = vec!["2".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 3 rows joined = 5  (shown in Row 4)");
    assert_eq!(doc.rows[3].result, "5");
}

#[test]
fn test_pemdas_case5_sub_mul_precedence() {
    // 10 − 3 × 2 = 4
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["10".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Sub;
    doc.rows[1].inputs = vec!["3".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["2".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 3 rows joined = 4  (shown in Row 4)");
    assert_eq!(doc.rows[3].result, "4");
}

#[test]
fn test_pemdas_case6_div_div_left_to_right() {
    // 8 ÷ 2 ÷ 2 = 2
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["8".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Div;
    doc.rows[1].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Div;
    doc.rows[2].inputs = vec!["2".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 3 rows joined = 2  (shown in Row 4)");
    assert_eq!(doc.rows[3].result, "2");
}

#[test]
fn test_pemdas_case7_mixed_four_operations() {
    // 2 + 3 × 4 − 5 = 9
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];

    doc.next_row();
    doc.rows[3].ensure_number_box();
    doc.rows[3].join = Join::Sub;
    doc.rows[3].inputs = vec!["5".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "All 4 rows joined = 9  (shown in Row 5)");
    assert_eq!(doc.rows[4].result, "9");
}

#[test]
fn test_pemdas_case8_division_by_zero_error() {
    // 8 ÷ 0 produces a division-by-zero error
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["8".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Div;
    doc.rows[1].inputs = vec!["0".to_string()];

    let res = doc.equals_all();
    assert!(res.is_err(), "Division by zero should return Err");
    assert_eq!(res.unwrap_err(), "Division by zero.");

    // In App integration, self.status should reflect this error
    let mut app = CalculatorApp::default();
    app.on_key("8");
    let _ = app.doc.join_key("÷");
    app.on_key("0");
    app.equals();
    assert_eq!(app.status, "Division by zero.");
}

#[test]
fn test_single_row_evaluated_without_total_row() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["42".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "Row 1 = 42");
    assert_eq!(doc.rows.len(), 1, "Single row does not create Total row");
    assert_eq!(doc.rows[0].result, "42");
    assert!(!doc.rows[0].is_result);
}

#[test]
fn test_gui_controls_compact_and_no_legacy_arrows() {
    let mut app = CalculatorApp::default();
    app.on_key("2");
    let _ = app.doc.join_key("+");
    app.on_key("3");
    let _ = app.doc.join_key("*");
    app.on_key("4");
    app.equals();

    assert_eq!(app.doc.rows.len(), 4);
    assert_eq!(app.doc.rows[3].result, "14");

    for (k, row) in app.doc.rows.iter().enumerate() {
        if row.is_result {
            continue;
        }
        let cards = row.flatten();
        let is_box = cards.is_empty() || (cards.len() == 1 && cards[0].kind == CardKind::Box);
        assert!(is_box, "Row {} must be a compact single number box without arrows", k + 1);
    }
}
