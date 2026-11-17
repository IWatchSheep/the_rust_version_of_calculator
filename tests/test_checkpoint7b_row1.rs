//! Dedicated Checkpoint 7B Test Suite: Row 1 18-Column Structure & Independent Row Architecture
//!
//! Verifies:
//! 1. 18-column Excel-style coordinate specification (Columns A through R and beyond).
//! 2. Column widths, standard Excel letter mappings (A..Z, AA, AB.. AZ, BA.. ZZ, AAA), spans, and control positions.
//! 3. Row 1 initial state (parentheses, exponents, descriptions, and math inputs).
//! 4. Row 1 dynamic parentheses expansion (+ button) and independent toggle states.
//! 5. Row 1 expanding descriptions (orange + buttons).
//! 6. Independence of Row 1 state from other rows (architecture readiness for multi-row).
//! 7. Compatibility with existing calculation engine and operator evaluation.

use rust_calculator::gui_coordinates::*;
use rust_calculator::row::{Document, Join, Row};
use rust_calculator::ops;

#[test]
fn test_18col_coordinate_specification() {
    assert_eq!(EXCEL_18COL_TOTAL_COLUMNS, 18);
    assert_eq!(EXCEL_18COL_LETTERS.len(), 18);

    // Verify 1-based column constants
    assert_eq!(COL_A_ROW_LABEL, 1);
    assert_eq!(COL_B_SPACER_AB, 2);
    assert_eq!(COL_C_MATH_SIGN, 3);
    assert_eq!(COL_D_SPACER_CD, 4);
    assert_eq!(COL_E_NUMBER_MATH, 5);
    assert_eq!(COL_F_MATH_BOX1, 6);
    assert_eq!(COL_G_MATH_TRAILING, 7);
    assert_eq!(COL_H_SPACER_GH, 8);
    assert_eq!(COL_I_ACTIONS, 9);
    assert_eq!(COL_J_RESULT_EQUALS, 10);
    assert_eq!(COL_K_PAREN_PLUS, 11);
    assert_eq!(COL_L_PAREN_OPEN, 12);
    assert_eq!(COL_M_RESULT_BOX, 13);
    assert_eq!(COL_N_EXPONENT_1, 14);
    assert_eq!(COL_O_PAREN_CLOSE, 15);
    assert_eq!(COL_P_EXPONENT_2, 16);
    assert_eq!(COL_Q_RESULT_BOX1, 17);
    assert_eq!(COL_R_RESULT_TRAILING, 18);

    // Verify letter round-trip for baseline Columns A-R
    let expected_letters = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R"];
    for (idx, &letter) in expected_letters.iter().enumerate() {
        let col_1based = idx + 1;
        assert_eq!(excel_18col_column_letter(col_1based), letter);
        assert_eq!(excel_18col_column_from_letter(letter), Some(col_1based));
    }

    // Verify standard Excel column lettering beyond R (rule 1: 1=A, 26=Z, 27=AA, 28=AB, 29=AC, 52=AZ, 53=BA, 702=ZZ, 703=AAA)
    assert_eq!(excel_18col_column_letter(26), "Z");
    assert_eq!(excel_18col_column_from_letter("Z"), Some(26));
    assert_eq!(excel_18col_column_letter(27), "AA");
    assert_eq!(excel_18col_column_from_letter("AA"), Some(27));
    assert_eq!(excel_18col_column_letter(28), "AB");
    assert_eq!(excel_18col_column_from_letter("AB"), Some(28));
    assert_eq!(excel_18col_column_letter(29), "AC");
    assert_eq!(excel_18col_column_from_letter("AC"), Some(29));
    assert_eq!(excel_18col_column_letter(52), "AZ");
    assert_eq!(excel_18col_column_from_letter("AZ"), Some(52));
    assert_eq!(excel_18col_column_letter(53), "BA");
    assert_eq!(excel_18col_column_from_letter("BA"), Some(53));
    assert_eq!(excel_18col_column_letter(702), "ZZ");
    assert_eq!(excel_18col_column_from_letter("ZZ"), Some(702));
    assert_eq!(excel_18col_column_letter(703), "AAA");
    assert_eq!(excel_18col_column_from_letter("AAA"), Some(703));

    // Verify base column widths match specification
    assert_eq!(excel_18col_column_width(COL_A_ROW_LABEL), 160.0);
    assert_eq!(excel_18col_column_width(COL_B_SPACER_AB), 10.0);
    assert_eq!(excel_18col_column_width(COL_C_MATH_SIGN), 42.0);
    assert_eq!(excel_18col_column_width(COL_D_SPACER_CD), 10.0);
    assert_eq!(excel_18col_column_width(COL_E_NUMBER_MATH), 110.0);
    assert_eq!(excel_18col_column_width(COL_F_MATH_BOX1), 90.0);
    assert_eq!(excel_18col_column_width(COL_G_MATH_TRAILING), 42.0);
    assert_eq!(excel_18col_column_width(COL_H_SPACER_GH), 10.0);
    assert_eq!(excel_18col_column_width(COL_I_ACTIONS), 130.0);
    assert_eq!(excel_18col_column_width(COL_J_RESULT_EQUALS), 42.0);
    assert_eq!(excel_18col_column_width(COL_K_PAREN_PLUS), 36.0);
    assert_eq!(excel_18col_column_width(COL_L_PAREN_OPEN), 36.0);
    assert_eq!(excel_18col_column_width(COL_M_RESULT_BOX), 110.0);
    assert_eq!(excel_18col_column_width(COL_N_EXPONENT_1), 48.0);
    assert_eq!(excel_18col_column_width(COL_O_PAREN_CLOSE), 36.0);
    assert_eq!(excel_18col_column_width(COL_P_EXPONENT_2), 48.0);
    assert_eq!(excel_18col_column_width(COL_Q_RESULT_BOX1), 90.0);
    assert_eq!(excel_18col_column_width(COL_R_RESULT_TRAILING), 42.0);

    // Verify effective column widths
    assert_eq!(excel_18col_effective_column_width(COL_E_NUMBER_MATH), 114.0);
    assert_eq!(excel_18col_effective_column_width(COL_F_MATH_BOX1), 140.0);
    assert_eq!(excel_18col_effective_column_width(COL_G_MATH_TRAILING), 93.0);
    assert_eq!(excel_18col_effective_column_width(COL_Q_RESULT_BOX1), 140.0);
    assert_eq!(excel_18col_effective_column_width(COL_R_RESULT_TRAILING), 93.0);

    // Verify total width calculations
    assert_eq!(excel_18col_total_width(), 1092.0);
    assert_eq!(excel_18col_effective_total_width(), 1298.0);
}

#[test]
fn test_grid_position_conversions_and_spans() {
    let pos = GridPosition::with_span(1, 5, 3, 2);
    assert_eq!(pos.row, 1);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.row_span, 3);
    assert_eq!(pos.column_span, 2);
    assert_eq!(pos.to_zero_based(), (0, 4));

    let from_zero = GridPosition::from_zero_based(2, 8);
    assert_eq!(from_zero.row, 3);
    assert_eq!(from_zero.column, 9);
}

#[test]
fn test_18col_workspace_positions_for_row_1() {
    // Row 1 design example at Excel rows 27..29
    let r0 = EXCEL_18COL_EXAMPLE_START_ROW; // 27

    let pos_label = excel_18col_workspace_position(1, "row_label", true).unwrap();
    assert_eq!(pos_label.row, r0);
    assert_eq!(pos_label.column, COL_A_ROW_LABEL);
    assert_eq!(pos_label.row_span, 3);

    let pos_math_sign = excel_18col_workspace_position(1, "math_sign", true).unwrap();
    assert_eq!(pos_math_sign.row, r0 + 1); // 28
    assert_eq!(pos_math_sign.column, COL_C_MATH_SIGN);

    let pos_num = excel_18col_workspace_position(1, "number_math", true).unwrap();
    assert_eq!(pos_num.row, r0 + 1);
    assert_eq!(pos_num.column, COL_E_NUMBER_MATH);

    let pos_desc1 = excel_18col_workspace_position(1, "math_box1", true).unwrap();
    assert_eq!(pos_desc1.row, r0 + 1);
    assert_eq!(pos_desc1.column, COL_F_MATH_BOX1);

    let pos_orange_plus = excel_18col_workspace_position(1, "math_orange_plus", true).unwrap();
    assert_eq!(pos_orange_plus.row, r0 + 1);
    assert_eq!(pos_orange_plus.column, COL_G_MATH_TRAILING);

    let pos_actions = excel_18col_workspace_position(1, "actions_frame", true).unwrap();
    assert_eq!(pos_actions.row, r0);
    assert_eq!(pos_actions.column, COL_I_ACTIONS);
    assert_eq!(pos_actions.row_span, 3);

    let pos_equals = excel_18col_workspace_position(1, "equals", true).unwrap();
    assert_eq!(pos_equals.row, r0 + 1);
    assert_eq!(pos_equals.column, COL_J_RESULT_EQUALS);

    let pos_blue_plus = excel_18col_workspace_position(1, "paren_plus", true).unwrap();
    assert_eq!(pos_blue_plus.row, r0 + 1);
    assert_eq!(pos_blue_plus.column, COL_K_PAREN_PLUS);

    let pos_paren_open = excel_18col_workspace_position(1, "paren_open", true).unwrap();
    assert_eq!(pos_paren_open.row, r0 + 1);
    assert_eq!(pos_paren_open.column, COL_L_PAREN_OPEN);

    let pos_result = excel_18col_workspace_position(1, "result_box", true).unwrap();
    assert_eq!(pos_result.row, r0 + 1);
    assert_eq!(pos_result.column, COL_M_RESULT_BOX);

    let pos_exp1 = excel_18col_workspace_position(1, "exponent_1", true).unwrap();
    assert_eq!(pos_exp1.row, r0); // Superscript in row 0
    assert_eq!(pos_exp1.column, COL_N_EXPONENT_1);

    let pos_paren_close = excel_18col_workspace_position(1, "paren_close", true).unwrap();
    assert_eq!(pos_paren_close.row, r0 + 1);
    assert_eq!(pos_paren_close.column, COL_O_PAREN_CLOSE);

    let pos_exp2 = excel_18col_workspace_position(1, "exponent_2", true).unwrap();
    assert_eq!(pos_exp2.row, r0); // Superscript in row 0
    assert_eq!(pos_exp2.column, COL_P_EXPONENT_2);

    let pos_res_desc = excel_18col_workspace_position(1, "result_desc", true).unwrap();
    assert_eq!(pos_res_desc.row, r0 + 1);
    assert_eq!(pos_res_desc.column, COL_Q_RESULT_BOX1);

    let pos_res_orange_plus = excel_18col_workspace_position(1, "result_orange_plus", true).unwrap();
    assert_eq!(pos_res_orange_plus.row, r0 + 1);
    assert_eq!(pos_res_orange_plus.column, COL_R_RESULT_TRAILING);
}

#[test]
fn test_row1_initial_state_structure() {
    let row = Row::blank();

    // Verify Row 1 baseline state
    assert_eq!(row.paren_pairs_count, 1);
    assert_eq!(row.paren_open_states, vec![false]);
    assert_eq!(row.paren_close_states, vec![false]);
    assert!(row.exponent_1.is_empty());
    assert_eq!(row.exponent_2_boxes.len(), 1);
    assert!(row.exponent_2_boxes[0].is_empty());

    // Description initial state
    assert_eq!(row.math_descriptions.len(), 1);
    assert!(row.math_descriptions[0].is_empty());
    assert_eq!(row.result_descriptions.len(), 1);
    assert!(row.result_descriptions[0].is_empty());

    // Default join and results
    assert_eq!(row.join, Join::None);
    assert!(row.result.is_empty());
    assert!(row.is_blank());
}

#[test]
fn test_row1_parentheses_expansion_and_toggles() {
    let mut row = Row::blank();
    assert_eq!(row.paren_pairs_count, 1);

    // Toggle pair 0 open and close
    row.toggle_paren_open(0);
    assert_eq!(row.paren_open_states, vec![true]);
    row.toggle_paren_close(0);
    assert_eq!(row.paren_close_states, vec![true]);

    // Click blue + button to add pair 2
    row.add_paren_pair();
    assert_eq!(row.paren_pairs_count, 2);
    assert_eq!(row.paren_open_states, vec![true, false]);
    assert_eq!(row.paren_close_states, vec![true, false]);
    assert_eq!(row.exponent_2_boxes.len(), 2);

    // Toggle pair 1
    row.toggle_paren_open(1);
    assert_eq!(row.paren_open_states, vec![true, true]);

    // Click blue + button to add pair 3
    row.add_paren_pair();
    assert_eq!(row.paren_pairs_count, 3);
    assert_eq!(row.paren_open_states, vec![true, true, false]);
    assert_eq!(row.paren_close_states, vec![true, false, false]);
    assert_eq!(row.exponent_2_boxes.len(), 3);

    // Independent exponents for each group
    row.exponent_2_boxes[0] = "2".to_string();
    row.exponent_2_boxes[1] = "3".to_string();
    row.exponent_2_boxes[2] = "4".to_string();
    assert_eq!(row.exponent_2_boxes, vec!["2", "3", "4"]);
}

#[test]
fn test_row1_expanding_descriptions() {
    let mut row = Row::blank();
    assert_eq!(row.math_descriptions.len(), 1);
    assert_eq!(row.result_descriptions.len(), 1);

    row.math_descriptions[0] = "Length".to_string();
    row.result_descriptions[0] = "Area".to_string();

    // Click orange + button in math description region
    row.add_math_description();
    assert_eq!(row.math_descriptions.len(), 2);
    row.math_descriptions[1] = "in meters".to_string();

    // Click orange + button in result description region
    row.add_result_description();
    assert_eq!(row.result_descriptions.len(), 2);
    row.result_descriptions[1] = "in square meters".to_string();

    assert_eq!(row.math_descriptions, vec!["Length", "in meters"]);
    assert_eq!(row.result_descriptions, vec!["Area", "in square meters"]);
}

#[test]
fn test_independent_row_architecture() {
    let mut doc = Document::new();
    assert_eq!(doc.rows.len(), 1);
    doc.next_row();
    assert_eq!(doc.rows.len(), 2);

    // Configure Row 1 with distinct 18-column values
    doc.rows[0].inputs = vec!["10".to_string()];
    doc.rows[0].exponent_1 = "2".to_string();
    doc.rows[0].add_paren_pair();
    doc.rows[0].toggle_paren_open(0);
    doc.rows[0].toggle_paren_open(1);
    doc.rows[0].exponent_2_boxes[0] = "3".to_string();
    doc.rows[0].exponent_2_boxes[1] = "4".to_string();
    doc.rows[0].math_descriptions[0] = "Base Row 1".to_string();

    // Verify Row 2 is completely independent and retains default baseline state
    assert_eq!(doc.rows[1].paren_pairs_count, 1);
    assert_eq!(doc.rows[1].paren_open_states, vec![false]);
    assert_eq!(doc.rows[1].paren_close_states, vec![false]);
    assert!(doc.rows[1].exponent_1.is_empty());
    assert_eq!(doc.rows[1].exponent_2_boxes.len(), 1);
    assert!(doc.rows[1].exponent_2_boxes[0].is_empty());
    assert_eq!(doc.rows[1].math_descriptions.len(), 1);
    assert!(doc.rows[1].math_descriptions[0].is_empty());

    // Mutating Row 2 does not affect Row 1
    doc.rows[1].exponent_1 = "5".to_string();
    doc.rows[1].math_descriptions[0] = "Row 2 description".to_string();
    assert_eq!(doc.rows[0].exponent_1, "2");
    assert_eq!(doc.rows[0].math_descriptions[0], "Base Row 1");
    assert_eq!(doc.rows[0].paren_pairs_count, 2);
    assert_eq!(doc.rows[1].paren_pairs_count, 1);
}

#[test]
fn test_row1_calculation_compatibility() {
    let mut row = Row::blank();
    let op = ops::get("math/arithmetic/addition").unwrap();
    row.set_operator(op);

    // Operator loaded on Row 1
    assert_eq!(row.inputs.len(), 2);
    row.inputs[0] = "20".to_string();
    row.inputs[1] = "30".to_string();
    row.compute();

    assert_eq!(row.result, "50");
    assert_eq!(row.error, None);

    // Clear row math resets calculation while preserving blank 18-col row state
    row.clear_result();
    assert!(row.result.is_empty());
    assert_eq!(row.paren_pairs_count, 1);
}
