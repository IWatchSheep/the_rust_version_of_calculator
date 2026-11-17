//! Checkpoint 7F: Nested Parentheses Across Mathematical Rows Regression Test Suite
//!
//! Verifies:
//! 1. ((2 + 3)) = 5
//! 2. ((2 + 3) × 4) = 20
//! 3. 2 × (3 + (4 × 5)) = 46
//! 4. ((2 + 3) × (4 + 1)) = 25
//! 5. 10 − (3 + 2) = 5
//! 6. 2 + 3 × 4 = 14 (PEMDAS regression without parentheses)
//! 7. Automatic closing-parenthesis capacity propagation from Row 1 to Row 2 and Row 3
//! 8. Unmatched opening parentheses detection and error reporting
//! 9. Unmatched closing parentheses detection and error reporting
//! 10. Missing join operator error reporting
//! 11. Division by zero within parenthesized scopes error reporting
//! 12. Single row calculation with parentheses evaluated without Total row
//! 13. Clear restores single empty row state
//! 14. Compact mathematical number boxes in Column E preserved

use rust_calculator::app::CalculatorApp;
use rust_calculator::layout::CardKind;
use rust_calculator::row::{Document, Join};

#[test]
fn test_nested_case1_double_parens_single_scope() {
    // ((2 + 3)) = 5
    // Row 1: (( 2
    // Row 2: + 3 ))
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0); // Row 1 now has 2 pairs
    doc.toggle_paren_open(0, 0); // Active '(' 1
    doc.toggle_paren_open(0, 1); // Active '(' 2

    // Adding Row 2 automatically provides 2 closing parentheses
    doc.next_row();
    assert_eq!(doc.rows[1].paren_pairs_count, 2, "Row 2 must automatically have 2 pairs");
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0); // Active ')' 1
    doc.toggle_paren_close(1, 1); // Active ')' 2

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows[0].result, "2");
    assert_eq!(doc.rows[1].result, "3");
    // Total row is row 3 (index 2)
    assert_eq!(doc.rows.len(), 3);
    assert!(doc.rows[2].is_result);
    assert_eq!(doc.rows[2].result, "5");
    let msg = res.unwrap();
    assert!(msg.contains("All 2 rows joined = 5"));
}

#[test]
fn test_nested_case2_distributed_closing_parens() {
    // ((2 + 3) × 4) = 20
    // Row 1: (( 2
    // Row 2: + 3 )
    // Row 3: × 4 )
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0); // 2 pairs on Row 1
    doc.toggle_paren_open(0, 0); // Active '(' 1
    doc.toggle_paren_open(0, 1); // Active '(' 2

    doc.next_row();
    assert_eq!(doc.rows[1].paren_pairs_count, 2);
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0); // Closes innermost scope (2 + 3)

    // After closing 1 paren on Row 2, 1 unmatched paren remains, so Row 3 gets capacity
    doc.next_row();
    assert!(doc.rows[2].paren_pairs_count >= 1);
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_close(2, 0); // Closes outer scope (* 4)

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows[0].result, "2");
    assert_eq!(doc.rows[1].result, "3");
    assert_eq!(doc.rows[2].result, "4");
    assert_eq!(doc.rows.len(), 4);
    assert!(doc.rows[3].is_result);
    assert_eq!(doc.rows[3].result, "20");
}

#[test]
fn test_nested_case3_deeply_nested_expression() {
    // 2 × (3 + (4 × 5)) = 46
    // Row 1: 2
    // Row 2: × ( 3
    // Row 3: + ( 4
    // Row 4: × 5 ))
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Mul;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0); // '('

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_open(2, 0); // '('

    doc.next_row();
    // Two scopes currently open from rows 1 and 2, so Row 4 gets 2 pairs
    assert_eq!(doc.rows[3].paren_pairs_count, 2);
    doc.rows[3].ensure_number_box();
    doc.rows[3].join = Join::Mul;
    doc.rows[3].inputs = vec!["5".to_string()];
    doc.toggle_paren_close(3, 0); // ')'
    doc.toggle_paren_close(3, 1); // ')'

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows.len(), 5);
    assert!(doc.rows[4].is_result);
    assert_eq!(doc.rows[4].result, "46");
}

#[test]
fn test_nested_case4_two_sibling_parentheses_groups() {
    // ((2 + 3) × (4 + 1)) = 25
    // Row 1: (( 2
    // Row 2: + 3 )
    // Row 3: × ( 4
    // Row 4: + 1 ))
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0);

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_open(2, 0);

    doc.next_row();
    doc.rows[3].ensure_number_box();
    doc.rows[3].join = Join::Add;
    doc.rows[3].inputs = vec!["1".to_string()];
    doc.toggle_paren_close(3, 0);
    doc.toggle_paren_close(3, 1);

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows.len(), 5);
    assert!(doc.rows[4].is_result);
    assert_eq!(doc.rows[4].result, "25");
}

#[test]
fn test_nested_case5_subtraction_with_parenthesis() {
    // 10 − (3 + 2) = 5
    // Row 1: 10
    // Row 2: − ( 3
    // Row 3: + 2 )
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["10".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Sub;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0);

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["2".to_string()];
    doc.toggle_paren_close(2, 0);

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows.len(), 4);
    assert!(doc.rows[3].is_result);
    assert_eq!(doc.rows[3].result, "5");
}

#[test]
fn test_nested_case6_pemdas_regression_no_parentheses() {
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
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows.len(), 4);
    assert!(doc.rows[3].is_result);
    assert_eq!(doc.rows[3].result, "14");
}

#[test]
fn test_closing_parenthesis_propagation_behavior() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["1".to_string()];
    assert_eq!(doc.rows[0].paren_pairs_count, 1);

    // Click blue + on Row 1 -> 2 pairs
    doc.add_paren_pair(0);
    assert_eq!(doc.rows[0].paren_pairs_count, 2);
    assert_eq!(doc.rows[0].paren_open_states, vec![false, false]);
    assert_eq!(doc.rows[0].paren_close_states, vec![false, false]);

    // Activate both open parentheses on Row 1
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);
    assert_eq!(doc.rows[0].paren_open_states, vec![true, true]);

    // Add Row 2 -> must automatically have 2 pairs
    doc.next_row();
    assert_eq!(doc.rows[1].paren_pairs_count, 2);
    // User chooses which to close; by default they are inactive
    assert_eq!(doc.rows[1].paren_close_states, vec![false, false]);

    // Row 2 closes only 1 paren
    doc.toggle_paren_close(1, 0);
    assert_eq!(doc.rows[1].paren_close_states, vec![true, false]);

    // Add Row 3 -> 1 unmatched paren remains from preceding rows, so Row 3 has capacity
    doc.next_row();
    assert!(doc.rows[2].paren_pairs_count >= 1);

    // Row 2 closes the 2nd paren as well
    doc.toggle_paren_close(1, 1);
    assert_eq!(doc.rows[1].paren_close_states, vec![true, true]);
    doc.sync_closing_parentheses_capacity();

    // Now all parens closed by Row 2; adding Row 4 needs only baseline 1 pair
    doc.next_row();
    assert_eq!(doc.rows[3].paren_pairs_count, 1);
}

#[test]
fn test_unmatched_opening_parenthesis_error() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_open(0, 0); // '(' without matching ')'

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    let res = doc.equals_all();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("opening parenthesis '(' without a matching closing parenthesis ')'"),
        "Unexpected error: {}",
        err
    );
    assert!(err.contains("Row 1"));
}

#[test]
fn test_unmatched_closing_parenthesis_error() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0); // ')' without matching '('

    let res = doc.equals_all();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("closing parenthesis ')' without a matching opening parenthesis '('"),
        "Unexpected error: {}",
        err
    );
    assert!(err.contains("Row 2"));
}

#[test]
fn test_missing_join_operator_error() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::None;
    doc.rows[1].inputs = vec!["3".to_string()];

    let res = doc.equals_all();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("needs a join symbol"), "Unexpected error: {}", err);
    assert!(err.contains("Row 2"));
}

#[test]
fn test_division_by_zero_within_parentheses() {
    // 8 ÷ (5 − 5) = division by zero error
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["8".to_string()];

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Div;
    doc.rows[1].inputs = vec!["5".to_string()];
    doc.toggle_paren_open(1, 0);

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Sub;
    doc.rows[2].inputs = vec!["5".to_string()];
    doc.toggle_paren_close(2, 0);

    let res = doc.equals_all();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err, "Division by zero.");
}

#[test]
fn test_single_row_parentheses_evaluated_without_total_row() {
    // Row 1: (( 5 )) on a single row (or with parens on single value)
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["5".to_string()];
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);
    doc.toggle_paren_close(0, 0);
    doc.toggle_paren_close(0, 1);

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    assert_eq!(doc.rows.len(), 1, "Single row calculation must not generate a Total row");
    assert_eq!(doc.rows[0].result, "5");
    assert_eq!(res.unwrap(), "Row 1 = 5");
}

#[test]
fn test_gui_compact_number_boxes_and_clear_preservation() {
    let mut app = CalculatorApp::default();
    assert_eq!(app.doc.rows.len(), 1);

    // Verify Row 1 has compact box, no 8-arrow cards
    let flat_0 = app.doc.rows[0].flatten();
    assert!(
        flat_0.len() <= 1 && flat_0.iter().all(|c| c.kind == CardKind::Box),
        "Row 1 must be compact single number box"
    );

    // Add nested parentheses across rows
    app.doc.rows[0].ensure_number_box();
    app.doc.rows[0].inputs = vec!["2".to_string()];
    app.doc.add_paren_pair(0);
    app.doc.toggle_paren_open(0, 0);
    app.doc.toggle_paren_open(0, 1);

    app.doc.next_row();
    app.doc.rows[1].ensure_number_box();
    app.doc.rows[1].join = Join::Add;
    app.doc.rows[1].inputs = vec!["3".to_string()];
    app.doc.toggle_paren_close(1, 0);
    app.doc.toggle_paren_close(1, 1);

    app.equals();
    assert!(app.status.contains("All 2 rows joined = 5"));
    assert_eq!(app.doc.rows.len(), 3);
    assert!(app.doc.rows[2].is_result);
    assert_eq!(app.doc.rows[2].result, "5");

    // Clear restores only Row 1
    app.doc.clear();
    assert_eq!(app.doc.rows.len(), 1);
    assert_eq!(app.doc.rows[0].paren_pairs_count, 1);
    assert_eq!(app.doc.rows[0].paren_open_states, vec![false]);
    assert_eq!(app.doc.rows[0].paren_close_states, vec![false]);
}
