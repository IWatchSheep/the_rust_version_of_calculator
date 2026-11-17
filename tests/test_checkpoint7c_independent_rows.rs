//! Dedicated Checkpoint 7C Test Suite: Independent Mathematical Rows (Rows 1, 2, 3+)
//!
//! Verifies all 12 Checkpoint 7C requirements:
//! 1. Creating Row 2 from Row 1.
//! 2. Creating Row 3 from Row 2.
//! 3. Correct 18-column structure for every row.
//! 4. Independent mathematical inputs.
//! 5. Independent descriptions.
//! 6. Independent parentheses selections.
//! 7. Independent exponent values.
//! 8. Correct shared column positions.
//! 9. Expanding only one row.
//! 10. Preserving existing mathematical operations.
//! 11. Clearing back to one blank row.
//! 12. Existing tests remain passing and no crosstalk occurs.

use rust_calculator::gui_coordinates::*;
use rust_calculator::ops;
use rust_calculator::row::{Document, Join};

#[test]
fn test_req1_and_req2_creating_rows_sequence() {
    let mut doc = Document::new();
    assert_eq!(doc.rows.len(), 1, "Document initializes with Row 1 only");

    // Start on Row 1
    doc.target_row = 0;
    doc.rows[0].inputs = vec!["10".to_string()];
    assert_eq!(doc.target_row, 0);

    // 1. Create/advance to Row 2 from Row 1
    doc.next_row();
    assert_eq!(doc.target_row, 1, "target_row advanced to Row 2");
    assert_eq!(doc.rows.len(), 2, "Row 2 created dynamically");
    assert_eq!(doc.rows[1].paren_pairs_count, 1);

    // 2. Create Row 3 from Row 2
    doc.next_row();
    assert_eq!(doc.target_row, 2, "target_row advanced to Row 3");
    assert_eq!(doc.rows.len(), 3, "Row 3 created dynamically");
    assert_eq!(doc.rows[2].paren_pairs_count, 1);
    assert!(doc.rows[2].is_blank());
}

#[test]
fn test_req3_correct_18col_structure_for_every_row() {
    let mut doc = Document::new();
    doc.next_row(); // create Row 2
    doc.next_row(); // create Row 3
    for (i, row) in doc.rows.iter().enumerate() {
        assert_eq!(row.paren_pairs_count, 1, "Row {} starts with 1 paren pair", i + 1);
        assert_eq!(row.paren_open_states, vec![false]);
        assert_eq!(row.paren_close_states, vec![false]);
        assert!(row.exponent_1.is_empty());
        assert_eq!(row.exponent_2_boxes.len(), 1);
        assert!(row.exponent_2_boxes[0].is_empty());
        assert_eq!(row.math_descriptions.len(), 1);
        assert!(row.math_descriptions[0].is_empty());
        assert_eq!(row.result_descriptions.len(), 1);
        assert!(row.result_descriptions[0].is_empty());
    }

    // Verify workspace positions for Row 1, Row 2, Row 3
    for k in 1..=3 {
        let pos_lbl = excel_18col_workspace_position(k, "row_label", false).unwrap();
        assert_eq!(pos_lbl.column, COL_A_ROW_LABEL);
        let pos_join = excel_18col_workspace_position(k, "math_sign", false).unwrap();
        assert_eq!(pos_join.column, COL_C_MATH_SIGN);
        let pos_num = excel_18col_workspace_position(k, "number_math", false).unwrap();
        assert_eq!(pos_num.column, COL_E_NUMBER_MATH);
        let pos_res = excel_18col_workspace_position(k, "result_box", false).unwrap();
        assert_eq!(pos_res.column, COL_M_RESULT_BOX);
    }
}

#[test]
fn test_req4_independent_mathematical_inputs() {
    let mut doc = Document::new();
    doc.rows[0].inputs = vec!["100".to_string()];
    doc.next_row(); // creates Row 2
    doc.rows[1].inputs = vec!["200".to_string()];

    // Advance to Row 2, then Next Row creates Row 3
    doc.target_row = 1;
    doc.next_row(); // creates and advances to Row 3
    assert_eq!(doc.rows.len(), 3);
    doc.rows[2].inputs = vec!["300".to_string()];

    assert_eq!(doc.rows[0].inputs[0], "100");
    assert_eq!(doc.rows[1].inputs[0], "200");
    assert_eq!(doc.rows[2].inputs[0], "300");

    // Editing Row 2 input does not modify Row 1 or Row 3
    doc.rows[1].inputs[0] = "999".to_string();
    assert_eq!(doc.rows[0].inputs[0], "100");
    assert_eq!(doc.rows[1].inputs[0], "999");
    assert_eq!(doc.rows[2].inputs[0], "300");
}

#[test]
fn test_req5_independent_descriptions() {
    let mut doc = Document::new();
    doc.rows[0].math_descriptions[0] = "Initial velocity".to_string();
    doc.rows[0].add_math_description();
    doc.rows[0].math_descriptions[1] = "in m/s".to_string();

    doc.next_row(); // creates Row 2
    doc.rows[1].math_descriptions[0] = "Acceleration".to_string();

    assert_eq!(doc.rows[0].math_descriptions.len(), 2);
    assert_eq!(doc.rows[1].math_descriptions.len(), 1);
    assert_eq!(doc.rows[0].math_descriptions[0], "Initial velocity");
    assert_eq!(doc.rows[0].math_descriptions[1], "in m/s");
    assert_eq!(doc.rows[1].math_descriptions[0], "Acceleration");
}

#[test]
fn test_req6_independent_parentheses_selections() {
    let mut doc = Document::new();
    // Row 1: toggle open parenthesis
    doc.rows[0].toggle_paren_open(0);
    assert_eq!(doc.rows[0].paren_open_states, vec![true]);
    assert_eq!(doc.rows[0].paren_close_states, vec![false]);

    // Create Row 2
    doc.next_row();

    // Row 2 parentheses remain unaffected
    assert_eq!(doc.rows[1].paren_open_states, vec![false]);
    assert_eq!(doc.rows[1].paren_close_states, vec![false]);

    // Row 2: toggle close parenthesis
    doc.rows[1].toggle_paren_close(0);
    assert_eq!(doc.rows[0].paren_open_states, vec![true]);
    assert_eq!(doc.rows[0].paren_close_states, vec![false]);
    assert_eq!(doc.rows[1].paren_open_states, vec![false]);
    assert_eq!(doc.rows[1].paren_close_states, vec![true]);
}

#[test]
fn test_req7_independent_exponent_values() {
    let mut doc = Document::new();
    doc.rows[0].exponent_1 = "2".to_string();
    doc.rows[0].exponent_2_boxes[0] = "3".to_string();

    doc.next_row(); // creates Row 2
    doc.rows[1].exponent_1 = "4".to_string();
    doc.rows[1].exponent_2_boxes[0] = "5".to_string();

    assert_eq!(doc.rows[0].exponent_1, "2");
    assert_eq!(doc.rows[0].exponent_2_boxes[0], "3");
    assert_eq!(doc.rows[1].exponent_1, "4");
    assert_eq!(doc.rows[1].exponent_2_boxes[0], "5");
}

#[test]
fn test_req8_shared_column_positions_and_widths() {
    // Columns A-R must share exact widths across rows
    for c in 1..=18 {
        let w_base = excel_18col_column_width(c);
        let w_eff = excel_18col_effective_column_width(c);
        assert!(w_base > 0.0);
        assert!(w_eff >= w_base);
    }
    // Verify Column C (Join Region) width is constant
    assert_eq!(excel_18col_effective_column_width(COL_C_MATH_SIGN), 42.0);
    assert_eq!(excel_18col_effective_column_width(COL_E_NUMBER_MATH), 114.0);
    assert_eq!(excel_18col_effective_column_width(COL_M_RESULT_BOX), 110.0);
}

#[test]
fn test_req9_expanding_only_one_row() {
    let mut doc = Document::new();
    // Expand Row 1 by adding 2 parentheses pairs (total 3 pairs)
    doc.rows[0].add_paren_pair();
    doc.rows[0].add_paren_pair();
    assert_eq!(doc.rows[0].paren_pairs_count, 3);
    assert_eq!(doc.rows[0].paren_open_states.len(), 3);
    assert_eq!(doc.rows[0].paren_close_states.len(), 3);
    assert_eq!(doc.rows[0].exponent_2_boxes.len(), 3);

    // Create Row 2
    doc.next_row();

    // Row 2 remains at baseline 1 pair
    assert_eq!(doc.rows[1].paren_pairs_count, 1);
    assert_eq!(doc.rows[1].paren_open_states.len(), 1);
    assert_eq!(doc.rows[1].paren_close_states.len(), 1);
    assert_eq!(doc.rows[1].exponent_2_boxes.len(), 1);
}

#[test]
fn test_req10_preserving_existing_mathematical_operations() {
    let mut doc = Document::new();

    // Row 1: addition 2 + 3 = 5
    let op_add = ops::get("math/arithmetic/addition").unwrap();
    doc.target_row = 0;
    doc.place_operator(op_add);
    doc.rows[0].inputs[0] = "2".to_string();
    doc.rows[0].inputs[1] = "3".to_string();
    doc.rows[0].compute();
    assert_eq!(doc.rows[0].result, "5");

    // Row 2: multiplication 5 * 4 = 20
    let op_mul = ops::get("math/arithmetic/multiplication").unwrap();
    doc.next_row(); // creates Row 2
    doc.target_row = 1;
    doc.place_operator(op_mul);
    doc.rows[1].inputs[0] = "5".to_string();
    doc.rows[1].inputs[1] = "4".to_string();
    doc.rows[1].join = Join::Add;
    doc.rows[1].compute();
    assert_eq!(doc.rows[1].result, "20");

    // Compute all rows joined (5 + 20 = 25)
    let msg = doc.equals_all().unwrap();
    assert!(msg.contains("25"));
    assert_eq!(doc.rows.len(), 3, "Result row added as Row 3");
    assert!(doc.rows[2].is_result);
    assert_eq!(doc.rows[2].result, "25");
}

#[test]
fn test_req11_clearing_back_to_initial_state() {
    let mut doc = Document::new();
    doc.rows[0].inputs = vec!["10".to_string()];
    doc.rows[0].add_paren_pair();
    doc.rows[0].add_math_description();

    doc.next_row(); // creates Row 2
    doc.rows[1].inputs = vec!["20".to_string()];
    doc.rows[1].join = Join::Add;

    doc.next_row(); // creates Row 3
    assert_eq!(doc.rows.len(), 3);

    // Clear all
    doc.clear();
    assert_eq!(doc.target_row, 0);
    assert_eq!(doc.rows.len(), 1, "Default reset state is clean 1 row (Row 1 only)");
    assert!(doc.rows[0].is_blank());
    assert_eq!(doc.rows[0].paren_pairs_count, 1);
    assert_eq!(doc.rows[0].math_descriptions.len(), 1);
}

#[test]
fn test_parentheses_propagation_readiness() {
    let mut doc = Document::new();
    // Row 1 activates two open parentheses ((
    doc.rows[0].add_paren_pair();
    doc.rows[0].toggle_paren_open(0);
    doc.rows[0].toggle_paren_open(1);

    // Create Row 2
    doc.next_row();

    // Sync capacity: Row 2 automatically receives at least 2 closing controls
    doc.sync_closing_parentheses_capacity();
    assert_eq!(doc.rows[1].paren_pairs_count, 2);
    assert_eq!(doc.rows[1].paren_close_states.len(), 2);
    // Newly provided closing parentheses remain inactive by default
    assert_eq!(doc.rows[1].paren_close_states, vec![false, false]);
}
