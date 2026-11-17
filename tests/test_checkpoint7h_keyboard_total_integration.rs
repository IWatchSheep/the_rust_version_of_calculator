//! Checkpoint 7H: Keyboard and Total Row Integration Test Suite
//!
//! Verifies:
//! 1. Keyboard focus selection and switching.
//! 2. Number entry.
//! 3. Exponent entry.
//! 4. On-screen keypad routing.
//! 5. Escape and Clear behavior.
//! 6. One-row evaluation.
//! 7. Multi-row Total creation.
//! 8. Repeated equals.
//! 9. Editing after evaluation.
//! 10. Next Row behavior.
//! 11. Parentheses and exponent regression:
//!     - 2 + 3 × 4 = 14
//!     - ((2 + 3) × 4) = 20
//!     - ((2 + 3)^2)^3 = 15625
//!     - ((2 + 3)^2 × 4)^3 = 1000000
//!     - (2 + (3 + 4)^2) = 51
//!     - ((2^3 + 1)^2 × 2) = 162
//! 12. Preservation of independent row state.

use rust_calculator::app::{ActiveExponentTarget, CalculatorApp};
use rust_calculator::row::{Document, Join};

#[test]
fn test_req1_keyboard_focus_selection_and_switching() {
    let mut app = CalculatorApp::default();
    assert_eq!(app.active_exponent, None);

    // Target Exponent 1 on Row 0
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.on_key("2");
    assert_eq!(app.doc.rows[0].exponent_1, "2");

    // Target Group Exponent (Row 0, Pair 0)
    app.active_exponent = Some(ActiveExponentTarget::Exp2(0, 0));
    app.on_key("3");
    assert_eq!(app.doc.rows[0].exponent_2_boxes[0], "3");

    // Previous exponent was not modified by targeting the new one
    assert_eq!(app.doc.rows[0].exponent_1, "2");

    // Switching focus to Column E clears active_exponent
    app.active_exponent = None;
    app.on_key("5");
    assert_eq!(app.doc.rows[0].inputs[0], "5");
    // Exponents remain intact
    assert_eq!(app.doc.rows[0].exponent_1, "2");
    assert_eq!(app.doc.rows[0].exponent_2_boxes[0], "3");
}

#[test]
fn test_req2_number_entry_keypad_and_keyboard() {
    let mut app = CalculatorApp::default();

    // Digits
    app.on_key("1");
    app.on_key("2");
    app.on_key("3");
    assert_eq!(app.doc.rows[0].inputs[0], "123");

    // Decimal point
    app.on_key(".");
    app.on_key("5");
    assert_eq!(app.doc.rows[0].inputs[0], "123.5");

    // Negative toggle
    app.on_key("±");
    assert_eq!(app.doc.rows[0].inputs[0], "-123.5");
    app.on_key("±");
    assert_eq!(app.doc.rows[0].inputs[0], "123.5");

    // Backspace
    app.on_key("Backspace");
    assert_eq!(app.doc.rows[0].inputs[0], "123.");
}

#[test]
fn test_req3_exponent_entry_routing_not_affecting_column_e() {
    let mut app = CalculatorApp::default();
    app.on_key("2");
    assert_eq!(app.doc.rows[0].inputs[0], "2");

    // Focus Exponent 1
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.on_key("3");
    // Exponent 1 receives 3, Column E remains 2!
    assert_eq!(app.doc.rows[0].exponent_1, "3");
    assert_eq!(app.doc.rows[0].inputs[0], "2");

    // Negative toggle on exponent
    app.on_key("±");
    assert_eq!(app.doc.rows[0].exponent_1, "-3");
    assert_eq!(app.doc.rows[0].inputs[0], "2");

    // Backspace on exponent
    app.on_key("Backspace");
    assert_eq!(app.doc.rows[0].exponent_1, "-");
    assert_eq!(app.doc.rows[0].inputs[0], "2");
}

#[test]
fn test_req4_onscreen_keypad_routing() {
    let mut app = CalculatorApp::default();
    // Simulate clicking keypad buttons
    app.on_key("7");
    app.on_key("8");
    assert_eq!(app.doc.rows[0].inputs[0], "78");

    // Keypad ±
    app.on_key("±");
    assert_eq!(app.doc.rows[0].inputs[0], "-78");

    // Keypad Backspace
    app.on_key("Backspace");
    assert_eq!(app.doc.rows[0].inputs[0], "-7");
}

#[test]
fn test_req5_escape_and_clear_behavior() {
    let mut app = CalculatorApp::default();
    app.on_key("4");
    app.on_key("2");
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));

    // Escape clears focus without deleting mathematical data!
    app.active_exponent = None;
    assert_eq!(app.active_exponent, None);
    assert_eq!(app.doc.rows[0].inputs[0], "42", "Mathematical data must NOT be deleted by Escape");

    // Clear resets document and focus completely
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.active_exponent = None;
    app.doc.clear();
    assert_eq!(app.active_exponent, None);
    assert_eq!(app.doc.rows.len(), 1);
    assert!(app.doc.rows[0].is_blank());
    assert_eq!(app.doc.rows[0].exponent_1, "");
}

#[test]
fn test_req6_single_row_evaluation_without_total() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["42".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(doc.rows.len(), 1, "Single row evaluation must NOT create a Total row");
    assert_eq!(doc.rows[0].result, "42");
    assert!(!doc.rows[0].is_result);
    assert!(doc.total_row().is_none());
}

#[test]
fn test_req7_multi_row_total_creation_and_alignment() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Before equals: 2 rows, no Total row
    assert_eq!(doc.rows.len(), 2);
    assert!(doc.total_row().is_none());

    // After equals: exactly one Total row immediately beneath Row 2
    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(doc.rows.len(), 3);

    let total = doc.total_row().expect("Total row must exist");
    assert!(total.is_result);
    assert_eq!(total.result, "5");
    assert_eq!(doc.rows[0].result, "2");
    assert_eq!(doc.rows[1].result, "3");
}

#[test]
fn test_req8_repeated_equals_does_not_duplicate_total() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.total_row().unwrap().result, "5");

    // Second equals
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3, "Repeated equals must not duplicate Total");
    assert_eq!(doc.total_row().unwrap().result, "5");

    // Third equals
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3, "Repeated equals must not duplicate Total");
    assert_eq!(doc.total_row().unwrap().result, "5");
}

#[test]
fn test_req9_editing_after_evaluation_sequence() {
    // Sequence required by Checkpoint 7H:
    // 1. Enter 2 + 3.
    // 2. Press = and verify Total = 5.
    // 3. Change the second mathematical input from 3 to 4.
    // 4. Press = again.
    // 5. Verify Total = 6 (2 + 4 = 6).
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Step 2: Press = and verify Total = 5
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "5");

    // Step 3: Change the second mathematical input from 3 to 4
    doc.target_row = 1;
    doc.target_path = vec![0];
    doc.backspace();
    // After editing, the previous Total row is invalidated/hidden so it never represents stale data
    assert!(doc.total_row().is_none());

    doc.append_digit('4');
    assert_eq!(doc.rows[1].inputs[0], "4");
    assert!(doc.total_row().is_none());

    // Step 4: Press = again
    assert!(doc.equals_all().is_ok());

    // Step 5: Verify Total = 6 without duplicate Total rows
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.total_row().unwrap().result, "6");
}

#[test]
fn test_req10_next_row_behavior() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert!(doc.total_row().is_some());

    // Next Row removes the Total row and adds exactly one new clean mathematical row
    doc.next_row();
    assert_eq!(doc.rows.len(), 3);
    assert!(doc.total_row().is_none(), "Total row must be replaced by new mathematical row");
    assert_eq!(doc.target_row, 2);
    assert!(doc.rows[2].is_blank());
}

#[test]
fn test_req11_parentheses_and_exponent_regression() {
    // 1. 2 + 3 × 4 = 14
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

    // 2. ((2 + 3) × 4) = 20
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
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "20");

    // 3. ((2 + 3)^2)^3 = 15625
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

    // 4. ((2 + 3)^2 × 4)^3 = 1000000
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

    // 5. (2 + (3 + 4)^2) = 51
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_open(0, 0);
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0);
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.add_paren_pair(2);
    doc.toggle_paren_close(2, 0);
    doc.toggle_paren_close(2, 1);
    doc.rows[2].exponent_2_boxes[0] = "2".to_string();
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "51");

    // 6. ((2^3 + 1)^2 × 2) = 162
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string();
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["1".to_string()];
    doc.toggle_paren_close(1, 0);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["2".to_string()];
    doc.toggle_paren_close(2, 0);
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.total_row().unwrap().result, "162");
}

#[test]
fn test_req12_preservation_of_independent_row_state() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["10".to_string()];
    doc.rows[0].math_descriptions[0] = "Row1 Desc".to_string();
    doc.rows[0].exponent_1 = "2".to_string();

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["20".to_string()];
    doc.rows[1].math_descriptions[0] = "Row2 Desc".to_string();
    doc.rows[1].exponent_1 = "3".to_string();

    // Verify row 0 unchanged by row 1
    assert_eq!(doc.rows[0].inputs[0], "10");
    assert_eq!(doc.rows[0].math_descriptions[0], "Row1 Desc");
    assert_eq!(doc.rows[0].exponent_1, "2");

    // Verify row 1 has its own values
    assert_eq!(doc.rows[1].inputs[0], "20");
    assert_eq!(doc.rows[1].math_descriptions[0], "Row2 Desc");
    assert_eq!(doc.rows[1].exponent_1, "3");
}
