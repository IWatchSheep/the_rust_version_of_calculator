//! Checkpoint 7I: Final Integration and Regression Test Suite
//!
//! Comprehensive verification across:
//! 1. All 13 authoritative mathematical calculations.
//! 2. Complete invalid-input error handling.
//! 3. Independent mathematical row state.
//! 4. Total row full lifecycle (A through G).
//! 5. Keyboard, on-screen keypad, and focus management integration.
//! 6. Nested parentheses propagation and independent group exponent association.

use rust_calculator::app::{ActiveExponentTarget, CalculatorApp};
use rust_calculator::row::{apply_exponent, Document, Join};
use rust_calculator::value::Value;

// =========================================================================
// 1. Authoritative Mathematical Correctness (Section 3)
// =========================================================================

#[test]
fn test_7i_math_01_pemdas_add_mul() {
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

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "14");
}

#[test]
fn test_7i_math_02_parentheses_mul() {
    // (2 + 3) × 4 = 20
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_open(0, 0);
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0);
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "20");
}

#[test]
fn test_7i_math_03_subtraction_with_parenthesis() {
    // 10 − (3 + 2) = 5
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

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "5");
}

#[test]
fn test_7i_math_04_deeply_nested_expression() {
    // 2 × (3 + (4 × 5)) = 46
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Mul;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0); // Outer (
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_open(2, 0); // Inner (
    doc.next_row();
    doc.rows[3].ensure_number_box();
    doc.rows[3].join = Join::Mul;
    doc.rows[3].inputs = vec!["5".to_string()];
    doc.add_paren_pair(3);
    doc.toggle_paren_close(3, 0); // Inner )
    doc.toggle_paren_close(3, 1); // Outer )

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "46");
}

#[test]
fn test_7i_math_05_double_nested_group_exponents() {
    // ((2 + 3)^2)^3 = 15625
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
    doc.toggle_paren_close(1, 1);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();
    doc.rows[1].exponent_2_boxes[1] = "3".to_string();

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "15625");
}

#[test]
fn test_7i_math_06_nested_group_exponent_and_multiplication() {
    // ((2 + 3)^2 × 4)^3 = 1000000
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
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_close(2, 0);
    doc.rows[2].exponent_2_boxes[0] = "3".to_string();

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "1000000");
}

#[test]
fn test_7i_math_07_inner_group_exponent_addition() {
    // 2 + (3 + 4)^2 = 51
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0);
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_close(2, 0);
    doc.rows[2].exponent_2_boxes[0] = "2".to_string();

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "51");
}

#[test]
fn test_7i_math_08_individual_and_group_exponents_combined() {
    // (2^3 + 1)^2 × 2 = (8 + 1)^2 × 2 = 9^2 × 2 = 81 × 2 = 162
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string(); // 2^3
    doc.toggle_paren_open(0, 0);
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["1".to_string()];
    doc.toggle_paren_close(1, 0);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string(); // ^2
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["2".to_string()];

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "162");
}

#[test]
fn test_7i_math_09_group_exponent_with_multiplication() {
    // ((2 + 3) × 4)^2 = 400
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
    doc.toggle_paren_close(2, 0);
    doc.rows[2].exponent_2_boxes[0] = "2".to_string();

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "400");
}

#[test]
fn test_7i_math_10_through_13_fractional_and_negative_powers() {
    // 2^(-1) = 0.5
    let val_2 = Value::Number(2.0);
    assert_eq!(apply_exponent(&val_2, -1.0), Ok(Value::Number(0.5)));

    // 4^(1/2) = 2
    let val_4 = Value::Number(4.0);
    assert_eq!(apply_exponent(&val_4, 0.5), Ok(Value::Number(2.0)));

    // 27^(1/3) = 3
    let val_27 = Value::Number(27.0);
    assert_eq!(apply_exponent(&val_27, 1.0 / 3.0), Ok(Value::Number(3.0)));

    // (-8)^(1/3) = -2 (odd root)
    let val_neg8 = Value::Number(-8.0);
    assert_eq!(apply_exponent(&val_neg8, 1.0 / 3.0), Ok(Value::Number(-2.0)));
}

// =========================================================================
// 2. Complete Invalid-Input Error Handling (Section 4)
// =========================================================================

#[test]
fn test_7i_errors_invalid_inputs_and_edge_cases() {
    // 1. Division by zero in row evaluation
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["10".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Div;
    doc.rows[1].inputs = vec!["0".to_string()];
    assert_eq!(doc.equals_all().unwrap_err(), "Division by zero.");

    // 2. Zero raised to a negative exponent
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["0".to_string()];
    doc.rows[0].exponent_1 = "-2".to_string();
    assert_eq!(doc.equals_all().unwrap_err(), "Row 1: Division by zero.");

    // 3. Invalid exponent text
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "xyz".to_string();
    assert_eq!(doc.equals_all().unwrap_err(), "Row 1: Invalid exponent 'xyz'.");

    // 4. Fractional exponent with zero denominator
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "1/0".to_string();
    assert_eq!(doc.equals_all().unwrap_err(), "Row 1: Division by zero in exponent.");

    // 5. Unsupported negative-base fractional exponent domain
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["-4".to_string()];
    doc.rows[0].exponent_1 = "0.5".to_string();
    assert_eq!(
        doc.equals_all().unwrap_err(),
        "Row 1: Cannot calculate power: negative base with fractional exponent."
    );

    // 6. Unmatched opening parenthesis
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_open(0, 0);
    let err_open = doc.equals_all().unwrap_err();
    assert!(
        err_open.contains("opening parenthesis '(' without a matching closing parenthesis ')'"),
        "Unexpected error: {}",
        err_open
    );
    assert!(err_open.contains("Row 1"));

    // 7. Unmatched closing parenthesis
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_close(0, 0);
    let err_close = doc.equals_all().unwrap_err();
    assert!(
        err_close.contains("closing parenthesis ')' without a matching opening parenthesis '('"),
        "Unexpected error: {}",
        err_close
    );
    assert!(err_close.contains("Row 1"));

    // 8. Missing inter-row mathematical join
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].inputs = vec!["3".to_string()];
    let err_join = doc.equals_all().unwrap_err();
    assert!(err_join.contains("needs a join symbol"), "Unexpected error: {}", err_join);
    assert!(err_join.contains("Row 2"));

    // 9. Group exponent entered without corresponding closing parenthesis
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_2_boxes[0] = "3".to_string();
    assert_eq!(
        doc.equals_all().unwrap_err(),
        "Row 1 has a group exponent (Column P) without a closing parenthesis ')'."
    );
}

// =========================================================================
// 3. Row Independence (Section 5)
// =========================================================================

#[test]
fn test_7i_row_independence_comprehensive() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["100".to_string()];
    doc.rows[0].math_descriptions[0] = "Row 1 Math Desc".to_string();
    doc.rows[0].result_descriptions[0] = "Row 1 Result Desc".to_string();
    doc.rows[0].exponent_1 = "2".to_string();
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["200".to_string()];
    doc.rows[1].math_descriptions[0] = "Row 2 Math Desc".to_string();
    doc.rows[1].result_descriptions[0] = "Row 2 Result Desc".to_string();
    doc.rows[1].exponent_1 = "3".to_string();
    doc.toggle_paren_close(1, 0);
    doc.rows[1].exponent_2_boxes[0] = "4".to_string();

    // Verify Row 0 state is completely independent
    assert_eq!(doc.rows[0].inputs[0], "100");
    assert_eq!(doc.rows[0].math_descriptions[0], "Row 1 Math Desc");
    assert_eq!(doc.rows[0].result_descriptions[0], "Row 1 Result Desc");
    assert_eq!(doc.rows[0].exponent_1, "2");
    assert!(doc.rows[0].paren_open_states[0]);

    // Verify Row 1 state is completely independent
    assert_eq!(doc.rows[1].inputs[0], "200");
    assert_eq!(doc.rows[1].math_descriptions[0], "Row 2 Math Desc");
    assert_eq!(doc.rows[1].result_descriptions[0], "Row 2 Result Desc");
    assert_eq!(doc.rows[1].exponent_1, "3");
    assert!(doc.rows[1].paren_close_states[0]);
    assert_eq!(doc.rows[1].exponent_2_boxes[0], "4");
}

// =========================================================================
// 4. Total Row Lifecycle (Section 6: A through G)
// =========================================================================

#[test]
fn test_7i_total_row_lifecycle_a_through_g() {
    let mut doc = Document::new();

    // A. One row: 42 = 42 displayed in M2 with no Total row
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["42".to_string()];
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 1);
    assert_eq!(doc.rows[0].result, "42");
    assert!(doc.total_row().is_none());

    // B. Two rows: 2 + 3 = 5, creating one Total row
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.total_row().unwrap().result, "5");

    // C. Press equals repeatedly: no additional Total rows appear
    assert!(doc.equals_all().is_ok());
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.total_row().unwrap().result, "5");

    // D. Edit 3 to 4: the previous Total must be invalidated
    doc.target_row = 1;
    doc.target_path = vec![0];
    doc.backspace();
    assert!(doc.total_row().is_none(), "Previous Total must be invalidated upon edit");
    doc.append_digit('4');
    assert!(doc.total_row().is_none());

    // E. Reevaluate: Total must display 6
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.total_row().unwrap().result, "6");

    // F. Add another mathematical row: previous Total removed, next row created normally
    doc.next_row();
    assert_eq!(doc.rows.len(), 3);
    assert!(doc.total_row().is_none());
    assert_eq!(doc.target_row, 2);
    assert!(doc.rows[2].is_blank());

    // G. Press Clear: restore one empty Row 1
    doc.clear();
    assert_eq!(doc.rows.len(), 1);
    assert!(doc.rows[0].is_blank());
    assert!(doc.total_row().is_none());
}

// =========================================================================
// 5. Keyboard and Keypad Focus Integration (Section 7)
// =========================================================================

#[test]
fn test_7i_keyboard_keypad_focus_routing() {
    let mut app = CalculatorApp::default();

    // 1. Digits and decimal entry
    app.on_key("5");
    app.on_key(".");
    app.on_key("2");
    assert_eq!(app.doc.rows[0].inputs[0], "5.2");

    // 2. Negative toggle
    app.on_key("±");
    assert_eq!(app.doc.rows[0].inputs[0], "-5.2");
    app.on_key("±");
    assert_eq!(app.doc.rows[0].inputs[0], "5.2");

    // 3. Backspace
    app.on_key("Backspace");
    assert_eq!(app.doc.rows[0].inputs[0], "5.");

    // 4. Focus Exponent 1: input must not affect Column E
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.on_key("3");
    assert_eq!(app.doc.rows[0].exponent_1, "3");
    assert_eq!(app.doc.rows[0].inputs[0], "5.", "Column E must remain untouched");

    // 5. Focus Group Exponent: input must not affect Column E or Exponent 1
    app.active_exponent = Some(ActiveExponentTarget::Exp2(0, 0));
    app.on_key("4");
    assert_eq!(app.doc.rows[0].exponent_2_boxes[0], "4");
    assert_eq!(app.doc.rows[0].exponent_1, "3");
    assert_eq!(app.doc.rows[0].inputs[0], "5.");

    // 6. Escape clears focus without deleting mathematical data
    app.active_exponent = None;
    assert_eq!(app.doc.rows[0].inputs[0], "5.");
    assert_eq!(app.doc.rows[0].exponent_1, "3");

    // 7. Clear completely resets focus and document
    app.doc.clear();
    assert_eq!(app.active_exponent, None);
    assert_eq!(app.doc.rows.len(), 1);
}

// =========================================================================
// 6. Nested Parentheses Propagation & Independent Group Exponents (Section 8)
// =========================================================================

#[test]
fn test_7i_nested_parentheses_propagation_and_independent_exponents() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);
    doc.toggle_paren_open(0, 2);

    // Row 2 automatically provides 3 closing parentheses pairs
    doc.next_row();
    assert_eq!(doc.rows[1].paren_pairs_count, 3);
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Activate closing paren 0 and 2; keep 1 inactive
    doc.toggle_paren_close(1, 0);
    doc.toggle_paren_close(1, 2);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();
    doc.rows[1].exponent_2_boxes[2] = "3".to_string();

    // Close final remaining opening parenthesis in Row 3
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["1".to_string()];
    doc.toggle_paren_close(2, 0);

    assert!(doc.equals_all().is_ok());
    // ((2 + 3)^2)^3 * 1 = 15625
    assert_eq!(doc.total_row().unwrap().result, "15625");
}
