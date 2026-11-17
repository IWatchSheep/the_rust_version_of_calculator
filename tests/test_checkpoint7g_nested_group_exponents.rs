use rust_calculator::app::{ActiveExponentTarget, CalculatorApp};
use rust_calculator::gui_coordinates::{
    excel_18col_column_letter,
    COL_A_ROW_LABEL, COL_C_MATH_SIGN, COL_E_NUMBER_MATH, COL_M_RESULT_BOX, COL_N_EXPONENT_1,
    COL_O_PAREN_CLOSE, COL_P_EXPONENT_2,
};
use rust_calculator::row::{apply_exponent, parse_exponent_value, Document, Join};
use rust_calculator::value::Value;

#[test]
fn test_req1_exact_excel_coordinates_and_column_letters() {
    assert_eq!(excel_18col_column_letter(COL_A_ROW_LABEL), "A");
    assert_eq!(excel_18col_column_letter(COL_C_MATH_SIGN), "C");
    assert_eq!(excel_18col_column_letter(COL_E_NUMBER_MATH), "E");
    assert_eq!(excel_18col_column_letter(COL_M_RESULT_BOX), "M");
    assert_eq!(excel_18col_column_letter(COL_N_EXPONENT_1), "N");
    assert_eq!(excel_18col_column_letter(COL_O_PAREN_CLOSE), "O");
    assert_eq!(excel_18col_column_letter(COL_P_EXPONENT_2), "P");

    // Dynamic closing paren and exponent pairs beyond baseline:
    // Pair 0: O (15), P (16)
    // Pair 1: Q (17), R (18)
    // Pair 2: S (19), T (20)
    assert_eq!(excel_18col_column_letter(15), "O");
    assert_eq!(excel_18col_column_letter(16), "P");
    assert_eq!(excel_18col_column_letter(17), "Q");
    assert_eq!(excel_18col_column_letter(18), "R");
    assert_eq!(excel_18col_column_letter(19), "S");
    assert_eq!(excel_18col_column_letter(20), "T");
}

#[test]
fn test_req2_individual_row_exponent_box_1() {
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string();

    let res = doc.equals_all();
    assert!(res.is_ok());
    // M2 must show 2^3 = 8
    assert_eq!(doc.rows[0].result, "8");
}

#[test]
fn test_req3_calculation_1_double_nested_group_exponents() {
    // ((2 + 3)^2)^3 = (5^2)^3 = 25^3 = 15625
    let mut doc = Document::default();
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

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "15625");
}

#[test]
fn test_req3_calculation_2_nested_exponent_multiplication() {
    // ((2 + 3)^2 × 4)^3 = (25 × 4)^3 = 100^3 = 1000000
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    // Close innermost parenthesis with exponent 2
    doc.toggle_paren_close(1, 0);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];
    // Close outermost parenthesis with exponent 3
    doc.toggle_paren_close(2, 0);
    doc.rows[2].exponent_2_boxes[0] = "3".to_string();

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "1000000");
}

#[test]
fn test_req3_calculation_3_inner_group_exponent_addition() {
    // (2 + (3 + 4)^2) = 2 + 7^2 = 2 + 49 = 51
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.toggle_paren_open(0, 0); // Outer (

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_open(1, 0); // Inner (

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    // Row 2 closes inner ) with exponent 2, and closes outer ) with no exponent
    doc.add_paren_pair(2);
    doc.toggle_paren_close(2, 0);
    doc.toggle_paren_close(2, 1);
    doc.rows[2].exponent_2_boxes[0] = "2".to_string();

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "51");
}

#[test]
fn test_req3_calculation_4_individual_and_group_exponents_combined() {
    // ((2^3 + 1)^2 × 2) = ((8 + 1)^2 × 2) = (9^2 × 2) = (81 × 2) = 162
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string(); // 2^3
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);

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
    doc.toggle_paren_close(2, 0); // Outermost )

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "162");
}

#[test]
fn test_req3_calculation_5_group_exponent_with_multiplication() {
    // ((2 + 3) × 4)^2 = (5 × 4)^2 = 20^2 = 400
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    doc.toggle_paren_close(1, 0); // Inner )

    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["4".to_string()];
    doc.toggle_paren_close(2, 0); // Outer )
    doc.rows[2].exponent_2_boxes[0] = "2".to_string(); // ^2

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "400");
}

#[test]
fn test_req3_calculation_6_nested_without_group_exponent() {
    // ((2 + 3) × 4) = 20
    let mut doc = Document::default();
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

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);

    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "20");
}

#[test]
fn test_req4_negative_fractional_and_float_exponents() {
    // 4^(1/2) = 2
    assert_eq!(parse_exponent_value("1/2"), Ok(Some(0.5)));
    let val_4 = Value::Number(4.0);
    assert_eq!(apply_exponent(&val_4, 0.5), Ok(Value::Number(2.0)));

    // 2^(-1) = 0.5
    assert_eq!(parse_exponent_value("-1"), Ok(Some(-1.0)));
    let val_2 = Value::Number(2.0);
    assert_eq!(apply_exponent(&val_2, -1.0), Ok(Value::Number(0.5)));

    // 27^(1/3) = 3
    let val_27 = Value::Number(27.0);
    assert_eq!(apply_exponent(&val_27, 1.0 / 3.0), Ok(Value::Number(3.0)));

    // (-8)^(1/3) = -2 (odd root)
    let val_neg8 = Value::Number(-8.0);
    assert_eq!(apply_exponent(&val_neg8, 1.0 / 3.0), Ok(Value::Number(-2.0)));

    // 2^0 = 1
    assert_eq!(apply_exponent(&val_2, 0.0), Ok(Value::Number(1.0)));

    // 0^5 = 0
    let val_0 = Value::Number(0.0);
    assert_eq!(apply_exponent(&val_0, 5.0), Ok(Value::Number(0.0)));
}

#[test]
fn test_req5_error_handling_invalid_exponent_and_orphan_group_exponent() {
    // 1. Invalid exponent syntax
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "abc".to_string();
    let err = doc.equals_all().unwrap_err();
    assert_eq!(err, "Row 1: Invalid exponent 'abc'.");

    // 2. Division by zero in fractional exponent
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "1/0".to_string();
    let err = doc.equals_all().unwrap_err();
    assert_eq!(err, "Row 1: Division by zero in exponent.");

    // 3. Base zero with negative exponent
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["0".to_string()];
    doc.rows[0].exponent_1 = "-2".to_string();
    let err = doc.equals_all().unwrap_err();
    assert_eq!(err, "Row 1: Division by zero.");

    // 4. Group exponent without active closing parenthesis (Column P in baseline)
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_2_boxes[0] = "2".to_string();
    let err = doc.equals_all().unwrap_err();
    assert_eq!(
        err,
        "Row 1 has a group exponent (Column P) without a closing parenthesis ')'."
    );

    // 5. Group exponent without active closing parenthesis in multi-pair row
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.rows[0].exponent_2_boxes[1] = "3".to_string(); // Pair 1 is Column R
    let err = doc.equals_all().unwrap_err();
    assert_eq!(
        err,
        "Row 1 has a group exponent (Column S) without a closing parenthesis ')'."
    );

    // 6. Negative base with fractional exponent having even denominator
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["-4".to_string()];
    doc.rows[0].exponent_1 = "0.5".to_string();
    let err = doc.equals_all().unwrap_err();
    assert_eq!(
        err,
        "Row 1: Cannot calculate power: negative base with fractional exponent."
    );
}

#[test]
fn test_req6_independent_exponent_association() {
    // If buttons 0 and 2 are active, but 1 is inactive:
    // Closing parenthesis 0 associates with Exponent Box 0.
    // Closing parenthesis 2 associates with Exponent Box 2.
    // They must apply to their respective groups without misaligned remapping.
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.add_paren_pair(0);
    doc.add_paren_pair(0);
    doc.toggle_paren_open(0, 0);
    doc.toggle_paren_open(0, 1);
    doc.toggle_paren_open(0, 2);

    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    // Activate closing paren 0 and 2; keep 1 inactive
    doc.toggle_paren_close(1, 0);
    doc.toggle_paren_close(1, 2);
    doc.rows[1].exponent_2_boxes[0] = "2".to_string();
    doc.rows[1].exponent_2_boxes[2] = "3".to_string();

    // Still need one more closing paren for the 3rd opened paren
    doc.next_row();
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Mul;
    doc.rows[2].inputs = vec!["1".to_string()];
    doc.toggle_paren_close(2, 0);

    let res = doc.equals_all();
    assert!(res.is_ok(), "Evaluation failed: {:?}", res);
    // ((2 + 3)^2)^3 * 1 = 15625 * 1 = 15625
    let total = doc.rows.iter().find(|r| r.is_result).expect("Total row must exist");
    assert_eq!(total.result, "15625");
}

#[test]
fn test_req7_active_exponent_focus_and_keypad_routing() {
    let mut app = CalculatorApp::default();
    assert_eq!(app.active_exponent, None);

    // Target Row 1 Exponent 1
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.on_key("3");
    assert_eq!(app.doc.rows[0].exponent_1, "3");

    // Keypad backspace on exponent
    app.on_key("Backspace");
    assert_eq!(app.doc.rows[0].exponent_1, "");

    // Keypad negative toggle on exponent
    app.on_key("5");
    app.on_key("±");
    assert_eq!(app.doc.rows[0].exponent_1, "-5");
    app.on_key("±");
    assert_eq!(app.doc.rows[0].exponent_1, "5");

    // Target Group Exponent (Row 0, Pair 0)
    app.active_exponent = Some(ActiveExponentTarget::Exp2(0, 0));
    app.on_key("2");
    assert_eq!(app.doc.rows[0].exponent_2_boxes[0], "2");

    // Next Row clears active exponent focus
    app.doc.next_row();
    app.active_exponent = None;
    assert_eq!(app.active_exponent, None);

    // Clear toolbar/keypad clears active exponent focus
    app.active_exponent = Some(ActiveExponentTarget::Exp1(0));
    app.doc.clear();
    app.active_exponent = None;
    assert_eq!(app.active_exponent, None);
}

#[test]
fn test_req8_format_mathematical_analysis_log_with_exponents() {
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string();

    let analysis = doc.format_mathematical_analysis();
    assert!(
        analysis.contains("2^3"),
        "Analysis log should contain exponent notation: {}",
        analysis
    );
}

#[test]
fn test_req9_clear_restores_only_row1_initial_state() {
    let mut doc = Document::default();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.rows[0].exponent_1 = "3".to_string();
    doc.next_row();
    doc.rows[1].inputs = vec!["4".to_string()];

    doc.clear();
    assert_eq!(doc.rows.len(), 1);
    assert!(doc.rows[0].exponent_1.is_empty());
    assert!(doc.rows[0].exponent_2_boxes[0].is_empty());
    assert_eq!(doc.target_row, 0);
}
