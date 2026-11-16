//! Logical GUI coordinate convention and layout definitions for the 18-column calculator layout.
//!
//! Mirrors `gui_coordinates.py` from the upgraded Python reference calculator.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutRegion {
    ExcelWorkspace,
    CardComponent,
    ArrowGrid,
    MainRow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    /// 1-based logical row (>= 1)
    pub row: usize,
    /// 1-based logical column (>= 1)
    pub column: usize,
    /// Number of rows spanned (>= 1)
    pub row_span: usize,
    /// Number of columns spanned (>= 1)
    pub column_span: usize,
    /// Region this position belongs to
    pub region: LayoutRegion,
}

impl GridPosition {
    pub fn new(row: usize, column: usize) -> Self {
        assert!(row >= 1, "row must be >= 1");
        assert!(column >= 1, "column must be >= 1");
        Self {
            row,
            column,
            row_span: 1,
            column_span: 1,
            region: LayoutRegion::ExcelWorkspace,
        }
    }

    pub fn with_span(row: usize, column: usize, row_span: usize, column_span: usize) -> Self {
        assert!(row >= 1 && column >= 1 && row_span >= 1 && column_span >= 1);
        Self {
            row,
            column,
            row_span,
            column_span,
            region: LayoutRegion::ExcelWorkspace,
        }
    }

    pub fn with_region(mut self, region: LayoutRegion) -> Self {
        self.region = region;
        self
    }

    pub fn to_zero_based(&self) -> (usize, usize) {
        (self.row - 1, self.column - 1)
    }

    pub fn from_zero_based(zero_row: usize, zero_col: usize) -> Self {
        Self::new(zero_row + 1, zero_col + 1)
    }
}

// 18 Column Constants (1-based)
pub const EXCEL_18COL_TOTAL_COLUMNS: usize = 18;
pub const EXCEL_18COL_ROWS_PER_CALCULATOR_ROW: usize = 3;
pub const EXCEL_18COL_CALCULATOR_ROW_STRIDE: usize = 6;
pub const EXCEL_18COL_EXAMPLE_START_ROW: usize = 27;

pub const COL_A_ROW_LABEL: usize = 1;
pub const COL_B_SPACER_AB: usize = 2;
pub const COL_C_MATH_SIGN: usize = 3;
pub const COL_D_SPACER_CD: usize = 4;
pub const COL_E_NUMBER_MATH: usize = 5;
pub const COL_F_MATH_BOX1: usize = 6;
pub const COL_G_MATH_TRAILING: usize = 7;
pub const COL_H_SPACER_GH: usize = 8;
pub const COL_I_ACTIONS: usize = 9;
pub const COL_J_RESULT_EQUALS: usize = 10;
pub const COL_K_PAREN_PLUS: usize = 11;
pub const COL_L_PAREN_OPEN: usize = 12;
pub const COL_M_RESULT_BOX: usize = 13;
pub const COL_N_EXPONENT_1: usize = 14;
pub const COL_O_PAREN_CLOSE: usize = 15;
pub const COL_P_EXPONENT_2: usize = 16;
pub const COL_Q_RESULT_BOX1: usize = 17;
pub const COL_R_RESULT_TRAILING: usize = 18;

pub const EXCEL_18COL_LETTERS: [&str; 18] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
];

pub fn excel_18col_column_width(col: usize) -> f32 {
    match col {
        1 => 160.0,
        2 => 10.0,
        3 => 42.0,
        4 => 10.0,
        5 => 110.0,
        6 => 90.0,
        7 => 42.0,
        8 => 10.0,
        9 => 130.0,
        10 => 42.0,
        11 => 36.0,
        12 => 36.0,
        13 => 110.0,
        14 => 48.0,
        15 => 36.0,
        16 => 48.0,
        17 => 90.0,
        18 => 42.0,
        _ => 40.0,
    }
}

pub fn excel_18col_effective_column_width(col: usize) -> f32 {
    match col {
        1 => 160.0,
        2 => 10.0,
        3 => 42.0,
        4 => 10.0,
        5 => 114.0,
        6 => 140.0,
        7 => 93.0,
        8 => 10.0,
        9 => 130.0,
        10 => 42.0,
        11 => 36.0,
        12 => 36.0,
        13 => 110.0,
        14 => 48.0,
        15 => 36.0,
        16 => 48.0,
        17 => 140.0,
        18 => 93.0,
        _ => 40.0,
    }
}

pub fn excel_18col_total_width() -> f32 {
    (1..=18).map(excel_18col_column_width).sum()
}

pub fn excel_18col_effective_total_width() -> f32 {
    (1..=18).map(excel_18col_effective_column_width).sum()
}

/// Convert 1-based column index to standard Excel column lettering:
/// 1 = A, 26 = Z, 27 = AA, 28 = AB, 29 = AC, 52 = AZ, 53 = BA, 702 = ZZ, 703 = AAA.
pub fn excel_18col_column_letter(column_1based: usize) -> String {
    assert!(column_1based >= 1, "column must be >= 1");
    let mut result = String::new();
    let mut col = column_1based;
    while col > 0 {
        let rem = (col - 1) % 26;
        result.push((b'A' + rem as u8) as char);
        col = (col - 1) / 26;
    }
    result.chars().rev().collect()
}

/// Convert standard Excel column lettering (A..Z, AA..ZZ, AAA..) to 1-based column index.
pub fn excel_18col_column_from_letter(letter: &str) -> Option<usize> {
    let clean = letter.trim().to_ascii_uppercase();
    if clean.is_empty() {
        return None;
    }
    let mut col: usize = 0;
    for ch in clean.chars() {
        if !ch.is_ascii_uppercase() {
            return None;
        }
        let val = (ch as u8 - b'A' + 1) as usize;
        col = col.checked_mul(26)?.checked_add(val)?;
    }
    Some(col)
}

pub fn excel_18col_start_row(calc_row: usize, use_example_offset: bool) -> usize {
    assert!(calc_row >= 1);
    let base = (calc_row - 1) * EXCEL_18COL_CALCULATOR_ROW_STRIDE;
    if use_example_offset {
        EXCEL_18COL_EXAMPLE_START_ROW + base
    } else {
        1 + base
    }
}

pub fn excel_18col_workspace_position(
    calc_row: usize,
    name: &str,
    use_example_offset: bool,
) -> Option<GridPosition> {
    let r0 = excel_18col_start_row(calc_row, use_example_offset);
    match name {
        "row_label" => Some(GridPosition::with_span(r0, COL_A_ROW_LABEL, 3, 1)),
        "spacer_ab" => Some(GridPosition::new(r0 + 1, COL_B_SPACER_AB)),
        "math_sign" => Some(GridPosition::new(r0 + 1, COL_C_MATH_SIGN)),
        "spacer_cd" => Some(GridPosition::new(r0 + 1, COL_D_SPACER_CD)),
        "number_math" | "math_slot" => Some(GridPosition::new(r0 + 1, COL_E_NUMBER_MATH)),
        "math_box1" | "math_desc" => Some(GridPosition::new(r0 + 1, COL_F_MATH_BOX1)),
        "math_trailing" | "math_orange_plus" => Some(GridPosition::new(r0 + 1, COL_G_MATH_TRAILING)),
        "spacer_gh" => Some(GridPosition::new(r0 + 1, COL_H_SPACER_GH)),
        "actions_frame" => Some(GridPosition::with_span(r0, COL_I_ACTIONS, 3, 1)),
        "action_math_left" => Some(GridPosition::new(r0, COL_I_ACTIONS)),
        "action_clear_left" => Some(GridPosition::new(r0 + 2, COL_I_ACTIONS)),
        "result_equals" | "equals" => Some(GridPosition::new(r0 + 1, COL_J_RESULT_EQUALS)),
        "paren_plus" | "blue_plus" => Some(GridPosition::new(r0 + 1, COL_K_PAREN_PLUS)),
        "paren_open" => Some(GridPosition::new(r0 + 1, COL_L_PAREN_OPEN)),
        "result_box" => Some(GridPosition::new(r0 + 1, COL_M_RESULT_BOX)),
        "exponent_1" => Some(GridPosition::new(r0, COL_N_EXPONENT_1)),
        "paren_close" => Some(GridPosition::new(r0 + 1, COL_O_PAREN_CLOSE)),
        "exponent_2" => Some(GridPosition::new(r0, COL_P_EXPONENT_2)),
        "result_box1" | "result_desc" => Some(GridPosition::new(r0 + 1, COL_Q_RESULT_BOX1)),
        "result_trailing" | "result_orange_plus" => Some(GridPosition::new(r0 + 1, COL_R_RESULT_TRAILING)),
        _ => None,
    }
}
