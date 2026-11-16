//! Checkpoint 7D: Appearance and Alignment Verification Tests
//!
//! Verifies:
//! 1. Shared spreadsheet coordinate model across rows.
//! 2. Excel column lettering beyond Column R (bijective base-26: A..Z, AA..ZZ, AAA..).
//! 3. Total row structure: "Total" badge in Column A, empty Columns B..I reserving width,
//!    and full result controls across Columns J through R.
//! 4. Multi-row appearance consistency (1 row, 2 rows, 3 rows, Total row).
//! 5. Independent styling and states for parentheses, exponents, and descriptions.

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
fn test_req3_total_row_creation_and_layout() {
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["1".to_string()];

    // Row 2 joined with +
    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["1".to_string()];

    // Equals creates Row 3 as the Total row
    let res = doc.equals_all();
    assert!(res.is_ok(), "Calculation should succeed: {:?}", res);
    assert_eq!(doc.rows.len(), 3);

    let row3 = &doc.rows[2];
    assert!(row3.is_result, "Row 3 must be marked as result / Total row");
    assert_eq!(row3.result, "2", "Total result should be 1 + 1 = 2");

    // Total row maintains independent paren pairs, exponents, and descriptions
    assert_eq!(row3.paren_pairs_count, 1);
    assert_eq!(row3.exponent_1, "");
    assert_eq!(row3.result_descriptions.len(), 1);
}

#[test]
fn test_req4_total_row_parentheses_and_descriptions() {
    let mut doc = Document::default();
    doc.rows[0].ensure_number_box();
    doc.rows[0].inputs = vec!["5".to_string()];

    doc.rows[1].ensure_number_box();
    doc.rows[1].join = Join::Add;
    doc.rows[1].inputs = vec!["10".to_string()];

    let res = doc.equals_all();
    assert!(res.is_ok(), "Calculation should succeed: {:?}", res);
    assert_eq!(doc.rows.len(), 3);

    // Expand parentheses on the Total row
    doc.rows[2].add_paren_pair();
    assert_eq!(doc.rows[2].paren_pairs_count, 2);
    assert_eq!(doc.rows[0].paren_pairs_count, 1, "Row 1 should not expand");
    assert_eq!(doc.rows[1].paren_pairs_count, 1, "Row 2 should not expand");

    // Add description on Total row
    doc.rows[2].add_result_description();
    assert_eq!(doc.rows[2].result_descriptions.len(), 2);
    assert_eq!(doc.rows[0].result_descriptions.len(), 1);
    assert_eq!(doc.rows[1].result_descriptions.len(), 1);
}

#[test]
fn test_req5_app_state_and_row_count_tracking() {
    let mut app = CalculatorApp::default();
    // Default document starts with 2 rows ready
    assert_eq!(app.doc.rows.len(), 2);
    assert_eq!(app.doc.target_row, 0);

    // Type 3 into Row 1
    app.on_key("3");
    assert_eq!(app.doc.rows[0].inputs.first().map(|s| s.as_str()), Some("3"));

    // Advance to Row 2
    app.doc.target_row = 1;
    app.doc.rows[1].join = Join::Add;
    app.on_key("4");
    assert_eq!(app.doc.rows[1].inputs.first().map(|s| s.as_str()), Some("4"));

    // Equals produces Row 3 as the Total row
    let res = app.doc.equals_all();
    assert!(res.is_ok());
    assert_eq!(app.doc.rows.len(), 3);
    assert!(app.doc.rows[2].is_result);
    assert_eq!(app.doc.rows[2].result, "7");
}
