//! Checkpoint 7D: Appearance, Alignment, and Dynamic Row Visibility Verification Tests
//!
//! Verifies:
//! 1. Shared spreadsheet coordinate model across rows.
//! 2. Excel column lettering beyond Column R (bijective base-26: A..Z, AA..ZZ, AAA..).
//! 3. State A (Startup): Only Mathematical Row 1 is visible (Excel Rows 1..3). No Row 2, no Total.
//! 4. State B (Add Row 2): Rows 1 and 2 visible. No Total.
//! 5. State C (Press =): Row 1 (2 in E2), Row 2 (+ in C8, 3 in E8), Total displayed with 5 in Column M (Excel Rows 12..13).
//! 6. Special Case: Single-row = displays result in Row 1 Column M without Total.
//! 7. Repeated = does not duplicate Total row.
//! 8. Additional rows do not automatically create Total.
//! 9. State D (Clear): Restores only Row 1, removes extra rows, removes Total.
//! 10. Independent row data (inputs, descriptions, parentheses, exponents) is strictly preserved.

use rust_calculator::app::CalculatorApp;
use rust_calculator::gui_coordinates::*;
use rust_calculator::row::{Document, Join};

#[test]
fn test_req1_excel_column_lettering_beyond_r() {
    // 1-based index to standard Excel column letters
    assert_eq!(excel_18col_column_letter(1), "A");
    assert_eq!(excel_18col_column_letter(18), "R");
    assert_eq!(excel_18col_column_letter(26), "Z");
    assert_eq!(excel_18col_column_letter(27), "AA");
    assert_eq!(excel_18col_column_letter(28), "AB");
    assert_eq!(excel_18col_column_letter(52), "AZ");
    assert_eq!(excel_18col_column_letter(53), "BA");
    assert_eq!(excel_18col_column_letter(702), "ZZ");
    assert_eq!(excel_18col_column_letter(703), "AAA");

    // Inverse mapping
    assert_eq!(excel_18col_column_from_letter("A"), Some(1));
    assert_eq!(excel_18col_column_from_letter("R"), Some(18));
    assert_eq!(excel_18col_column_from_letter("Z"), Some(26));
    assert_eq!(excel_18col_column_from_letter("AA"), Some(27));
    assert_eq!(excel_18col_column_from_letter("AB"), Some(28));
    assert_eq!(excel_18col_column_from_letter("ZZ"), Some(702));
    assert_eq!(excel_18col_column_from_letter("AAA"), Some(703));
}

#[test]
fn test_req2_shared_absolute_column_coordinates() {
    let w_a = excel_18col_effective_column_width(COL_A_ROW_LABEL);
    let w_b = excel_18col_effective_column_width(COL_B_SPACER_AB);
    let w_c = excel_18col_effective_column_width(COL_C_MATH_SIGN);
    let w_d = excel_18col_effective_column_width(COL_D_SPACER_CD);
    let w_e = excel_18col_effective_column_width(COL_E_NUMBER_MATH);
    let w_f = excel_18col_effective_column_width(COL_F_MATH_BOX1);
    let w_g = excel_18col_column_width(COL_G_MATH_TRAILING);
    let w_h = excel_18col_effective_column_width(COL_H_SPACER_GH);
    let w_i = excel_18col_effective_column_width(COL_I_ACTIONS);
    let w_j = excel_18col_effective_column_width(COL_J_RESULT_EQUALS);

    // Sum of widths up to Column I
    let empty_w = w_b + w_c + w_d + w_e + w_f + w_g + w_h + w_i;

    // Both mathematical rows and Total rows maintain exact column alignment to Column J
    assert!(w_a >= 160.0);
    assert_eq!(empty_w, 10.0 + 42.0 + 10.0 + 114.0 + 140.0 + 42.0 + 10.0 + 130.0);
    assert_eq!(w_j, 42.0);
}

#[test]
fn test_req3_state_a_startup_row1_only() {
    let doc = Document::new();
    // At startup, only Mathematical Row 1 exists
    assert_eq!(doc.rows.len(), 1, "Startup must show Row 1 only");
    assert_eq!(doc.target_row, 0);
    assert!(!doc.rows[0].is_result, "Row 1 is not a Total row");
    assert!(doc.rows[0].is_blank());
}

#[test]
fn test_req4_state_b_add_row2_no_total() {
    let mut doc = Document::new();
    assert_eq!(doc.rows.len(), 1);

    // Add Mathematical Row 2
    doc.next_row();
    assert_eq!(doc.rows.len(), 2, "Adding Row 2 creates exactly 2 mathematical rows");
    assert_eq!(doc.target_row, 1);
    assert!(!doc.rows[0].is_result);
    assert!(!doc.rows[1].is_result, "Row 2 is not a Total row");
    // Verify no Total row exists
    assert!(doc.rows.iter().all(|r| !r.is_result), "No Total row exists in State B");
}

#[test]
fn test_req5_state_c_calculation_with_total_row_5() {
    let mut doc = Document::new();
    // Enter 2 in E2 (Row 1)
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];

    // Add + in C8 and 3 in E8 (Row 2)
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Press =
    let res = doc.equals_all();
    assert!(res.is_ok(), "Calculation should succeed: {:?}", res);
    assert_eq!(doc.rows.len(), 3, "Rows 1, 2, and exactly one Total row");

    // Row 1 result is 2
    assert_eq!(doc.rows[0].result, "2");
    // Row 2 result is 3
    assert_eq!(doc.rows[1].result, "3");

    // Total row
    let total_row = &doc.rows[2];
    assert!(total_row.is_result, "Row 3 must be the Total row");
    assert_eq!(total_row.result, "5", "Total Column M must display 5 (2 + 3 = 5)");

    // Total row maintains independent paren pairs, exponents, and descriptions
    assert_eq!(total_row.paren_pairs_count, 1);
    assert_eq!(total_row.exponent_1, "");
    assert_eq!(total_row.result_descriptions.len(), 1);
}

#[test]
fn test_req6_special_case_single_row_no_total() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["42".to_string()];

    // Press = on single row
    let res = doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(doc.rows[0].result, "42", "Row 1 Column M displays evaluated answer");
    assert_eq!(doc.rows.len(), 1, "No separate Total row is created for single row");
    assert!(!doc.rows[0].is_result);
}

#[test]
fn test_req7_repeated_equals_does_not_duplicate_total() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Press = first time
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert_eq!(doc.rows[2].result, "5");

    // Press = second time
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3, "Repeated = must NOT duplicate Total row");
    assert_eq!(doc.rows[2].result, "5");

    // Press = third time
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3, "Repeated = must NOT duplicate Total row");
    assert_eq!(doc.rows[2].result, "5");
}

#[test]
fn test_req8_adding_rows_removes_previous_total_until_equals() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];

    // Calculate -> Total exists
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 3);
    assert!(doc.rows[2].is_result);

    // User adds Row 3 -> Total is removed, Row 3 is a clean mathematical row
    doc.next_row();
    assert_eq!(doc.rows.len(), 3, "Total row was replaced by new Mathematical Row 3");
    assert!(!doc.rows[2].is_result, "Row 3 is a mathematical row, not Total");
    assert_eq!(doc.target_row, 2);

    // Re-calculating with Row 3 adds Total immediately beneath Row 3
    doc.rows[2].ensure_number_box();
    doc.rows[2].join = Join::Add;
    doc.rows[2].inputs = vec!["4".to_string()];
    assert!(doc.equals_all().is_ok());
    assert_eq!(doc.rows.len(), 4, "Rows 1, 2, 3, and Total");
    assert!(doc.rows[3].is_result);
    assert_eq!(doc.rows[3].result, "9", "2 + 3 + 4 = 9");
}

#[test]
fn test_req9_state_d_clear_restores_only_row1() {
    let mut doc = Document::new();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["2".to_string()];
    doc.next_row();
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["3".to_string()];
    let _ = doc.equals_all();
    assert_eq!(doc.rows.len(), 3);

    // Clear restores only Row 1
    doc.clear();
    assert_eq!(doc.rows.len(), 1, "Clear restores exactly one empty Row 1");
    assert_eq!(doc.target_row, 0);
    assert!(doc.rows[0].is_blank());
    assert_eq!(doc.rows[0].paren_pairs_count, 1);
    assert_eq!(doc.rows[0].exponent_1, "");
    assert_eq!(doc.rows[0].result_descriptions.len(), 1);
}

#[test]
fn test_req10_app_integration_and_keypad_actions() {
    let mut app = CalculatorApp::default();
    // Startup has Row 1 only
    assert_eq!(app.doc.rows.len(), 1);
    assert_eq!(app.doc.target_row, 0);

    // Enter 2 into Row 1
    app.on_key("2");
    assert_eq!(app.doc.rows[0].inputs.first().map(|s| s.as_str()), Some("2"));

    // Add Row 2 using join key '+'
    let msg = app.doc.join_key("+");
    assert!(msg.contains("Row 2"));
    assert_eq!(app.doc.rows.len(), 2, "Row 2 added via join key");
    assert_eq!(app.doc.target_row, 1);
    assert_eq!(app.doc.rows[1].join, Join::Add);

    // Enter 3 into Row 2
    app.on_key("3");
    assert_eq!(app.doc.rows[1].inputs.first().map(|s| s.as_str()), Some("3"));

    // Equals produces Total row with 5
    app.equals();
    assert_eq!(app.doc.rows.len(), 3);
    assert!(app.doc.rows[2].is_result);
    assert_eq!(app.doc.rows[2].result, "5");
}
