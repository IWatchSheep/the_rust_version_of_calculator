//! egui desktop UI — same jobs and pixel-accurate layout as the Python Tkinter shell.

use eframe::egui::{self, Color32, RichText, Ui};
use egui::{Button, ScrollArea, TextEdit};

use crate::ops;
use crate::row::{Document, Join, Row};

const CREAM: Color32 = Color32::from_rgb(0xff, 0xfb, 0xe6);
const BLUE: Color32 = Color32::from_rgb(0x1e, 0x6f, 0xd9);
const GREEN: Color32 = Color32::from_rgb(0x3c, 0xb0, 0x43);
const RED: Color32 = Color32::from_rgb(0xe0, 0x20, 0x20);
#[allow(dead_code)]
const ORANGE: Color32 = Color32::from_rgb(0xf0, 0x70, 0x30);
const PALE_BLUE: Color32 = Color32::from_rgb(0xd6, 0xe6, 0xfa);
const PALE_ORANGE: Color32 = Color32::from_rgb(0xfb, 0xe4, 0xcf);
const ARROW_BG: Color32 = Color32::from_rgb(0xea, 0xea, 0xea);
const ARROW_BORDER: Color32 = Color32::from_rgb(0xc8, 0xc8, 0xc8);
const HELPER_YELLOW: Color32 = Color32::from_rgb(0xfb, 0xd3, 0x4d);
const TARGET_BLUE: Color32 = Color32::from_rgb(0x5b, 0x9b, 0xf0);
const ROW_BADGE_BG: Color32 = Color32::from_rgb(0xb5, 0xdc, 0xa8);
const ROW_BADGE_ACTIVE: Color32 = Color32::from_rgb(0x6f, 0xcf, 0x5a);
const ROW_BADGE_FG: Color32 = Color32::from_rgb(0x2a, 0x6a, 0x20);
const ORANGE_TEXT: Color32 = Color32::from_rgb(0xc0, 0x70, 0x00);
const ORANGE_BORDER: Color32 = Color32::from_rgb(0xe0, 0xa0, 0x60);
const CLEAR_MATH_BG: Color32 = Color32::from_rgb(0xf4, 0xd0, 0xd0);
const CLEAR_MATH_FG: Color32 = Color32::from_rgb(0xa0, 0x20, 0x20);

// Tkinter `area.grid_columnconfigure` minsizes / canvas height from calculator/main.py
const COL_ROW: f32 = 170.0;
const COL_HELPER: f32 = 430.0;
const COL_ACTIONS: f32 = 160.0;
const COL_RESULT: f32 = 410.0;
const ROWS_CANVAS_H: f32 = 290.0;
const MATH_SLOT_H: f32 = 118.0;
const MATH_SLOT_MIN_W: f32 = 420.0;
const MATH_SLOT_MAX_W: f32 = 560.0;
/// Fixed row-grid content width at the math-slot floor (for horizontal scroll when the window is narrow).
const ROWS_CONTENT_MIN_W: f32 = COL_ROW + COL_HELPER + MATH_SLOT_MIN_W + COL_ACTIONS + COL_RESULT + 16.0;
const ROW_CARD_TOP: f32 = 19.0;
const CARD_FRAME_INSET: f32 = 2.0;
const MATH_SLOT_INSET: f32 = 4.0;
/// 3×3 arrow grid height (24 + 2 + 36 + 2 + 24). Lead/trailing span this full height
/// like Tkinter `make_component` rowspan=3, so vertical middles and bottoms stay aligned.
const ARROW_CELL: f32 = 24.0;
const ARROW_CENTER_H: f32 = 36.0;
const ARROW_GAP: f32 = 2.0;
const CARD_H: f32 = ARROW_CELL + ARROW_GAP + ARROW_CENTER_H + ARROW_GAP + ARROW_CELL;
const LEAD_TRAIL_W: f32 = 36.0;
const ITEM_W: f32 = 90.0;
const CONTEXT_W: f32 = 140.0;
const ITEM_W_MIN: f32 = 48.0;
const CONTEXT_W_MIN: f32 = 72.0;

pub struct CalculatorApp {
    doc: Document,
    status: String,
    show_ops: bool,
    op_filter: String,
    selected_category: String,
    show_words: bool,
    word_category: String,
    show_card_words: bool,
    show_adv: bool,
    adv_kind: String,
    adv_text: String,
    adv_msg: String,
    adv_filter: String,
    preview_key: Option<String>,
    diagram: Option<&'static ops::words::Item>,
    tpl_text: String,
    tpl_saved: Vec<String>,
    tpl_pick: String,
    tpl_hide_guide: bool,
    tpl_guide: bool,
    tpl_previewed: Option<String>,
    tpl_applied: bool,
    tpl_last_text: String,
    tpl_preview: Option<TplPreview>,
    tpl_path: std::path::PathBuf,
    custom_word: String,
    /// When set, the rows canvas scrolls so this row is in view (Tkinter `_scroll_to_row`).
    rows_scroll_to: Option<usize>,
    /// Last seen target/count — used to auto-request scroll when join/`Next Row` adds rows.
    rows_follow_target: usize,
    rows_follow_count: usize,
}

struct TplPreview {
    row: usize,
    snapshot: Row,
    shown: Vec<String>,
}

fn row_signature(r: &Row) -> Vec<String> {
    r.flatten().into_iter().map(|c| c.text).collect()
}

const GUIDE_STEPS: [(&str, &str); 4] = [
    (
        "Type your math in the text box",
        "Use ? or a letter such as x for a box you will fill in later. Examples:   (? + 3) × 2     sqrt(x) + 4     Time dilation(1, x)",
    ),
    (
        "Press  Preview Posted",
        "Your math appears in the target row, just like an operator from the Operators list, so you can see it. You can change the text and preview again.",
    ),
    (
        "Like it?  Press  Save Template",
        "It is kept on this computer, so it will be there next time too. Not happy? Change the text and press Preview Posted again.",
    ),
    (
        "Use it whenever you want",
        "Pick it in the list next to Apply Template, then press Apply Template. It goes into the row. Then fill the empty boxes and press the keypad = key.",
    ),
];

impl Default for CalculatorApp {
    fn default() -> Self {
        let tpl_path = crate::pictures::root().cloned().unwrap_or_else(|| std::path::PathBuf::from(".")).join("saved_templates.json");
        let tpl_loaded = crate::template_math::load_data(&tpl_path);
        Self {
            doc: Document::new(),
            status: "Pick Operators, choose Addition, type numbers, press =.".into(),
            show_ops: false,
            op_filter: String::new(),
            selected_category: "All".into(),
            show_words: false,
            show_card_words: true,
            word_category: "All".into(),
            show_adv: false,
            adv_kind: "list".into(),
            adv_text: String::new(),
            adv_msg: String::new(),
            adv_filter: String::new(),
            preview_key: None,
            diagram: None,
            tpl_text: String::new(),
            tpl_saved: tpl_loaded.0,
            tpl_pick: String::new(),
            tpl_hide_guide: tpl_loaded.1,
            tpl_guide: false,
            tpl_previewed: None,
            tpl_applied: false,
            tpl_last_text: String::new(),
            tpl_preview: None,
            tpl_path,
            custom_word: String::new(),
            rows_scroll_to: None,
            rows_follow_target: 0,
            rows_follow_count: 2,
        }
    }
}

impl CalculatorApp {
    pub fn with_demo(key: &str, inputs: &[String]) -> Self {
        let mut app = Self::default();
        if let Some(op) = ops::get(key) {
            app.preview_key = Some(key.to_string());
            app.diagram = key.strip_prefix("word:").and_then(|k| ops::words::items().iter().find(|i| i.key == k));
            app.doc.rows[0].set_operator(op);
            for (i, t) in inputs.iter().enumerate() {
                if let Some(slot) = app.doc.rows[0].inputs.get_mut(i) {
                    *slot = t.clone();
                }
            }
            app.equals();
        } else {
            app.status = format!("no operator {key}");
        }
        app
    }

    fn apply_word_preset(&mut self, which: usize, word: &str) {
        let r = self.doc.target_row;
        if r < self.doc.rows.len() {
            let card_idx = self.doc.target_path.first().copied().unwrap_or(0);
            let entry = self.doc.rows[r].words.entry(card_idx).or_insert_with(|| (String::new(), String::new()));
            if which == 0 {
                entry.0 = word.to_string();
            } else {
                entry.1 = word.to_string();
            }
        }
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = Color32::from_rgb(0xf0, 0xf4, 0xf4);
        style.visuals.window_fill = Color32::WHITE;
        ctx.set_style(style);

        // Keyboard handling when not typing into a TextEdit
        if !ctx.wants_keyboard_input() {
            ctx.input(|input| {
                for event in &input.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        match key {
                            egui::Key::Num0 => self.on_key("0"),
                            egui::Key::Num1 => self.on_key("1"),
                            egui::Key::Num2 => self.on_key("2"),
                            egui::Key::Num3 => self.on_key("3"),
                            egui::Key::Num4 => self.on_key("4"),
                            egui::Key::Num5 => self.on_key("5"),
                            egui::Key::Num6 => self.on_key("6"),
                            egui::Key::Num7 => self.on_key("7"),
                            egui::Key::Num8 => self.on_key("8"),
                            egui::Key::Num9 => self.on_key("9"),
                            egui::Key::Period => self.on_key("."),
                            egui::Key::Backspace => self.doc.backspace(),
                            egui::Key::Enter => self.equals(),
                            _ => {}
                        }
                    }
                }
            });
        }

        // Top Toolbar matching Tkinter:
        // Smart Token Expression Builder (Math Templates) | Clear (C) | Next Row (↓)
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Smart Token Expression Builder (Math Templates)").strong().size(13.0));
                ui.add_space(8.0);
                if colored_btn(ui, "Clear (C)", RED).clicked() {
                    self.doc.clear();
                    self.status = "Cleared. Click a square box and type a number, or open Operators.".into();
                }
                if colored_btn(ui, "Next Row (↓)", GREEN).clicked() {
                    self.doc.next_row();
                    self.status = format!("Row {} ready.", self.doc.target_row + 1);
                }
            });
            ui.add_space(4.0);
            // Selected Target Bar (white frame with light grey border, orange text)
            egui::Frame::none()
                .fill(Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xc0, 0xc0, 0xc0)))
                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Selected Target: Row {} | Addend {} | Col 1: Number ('o')",
                            self.doc.target_row + 1,
                            self.doc.target_path.first().map_or(1, |x| x + 1)
                        ))
                        .color(Color32::from_rgb(0xd0, 0x50, 0x10))
                        .size(11.0)
                        .strong(),
                    );
                });
            ui.add_space(4.0);
        });

        // Floating popup windows (Operators & Advanced Input & Diagram)
        if self.show_ops {
            let mut open = true;
            egui::Window::new("Operators")
                .open(&mut open)
                .resizable(true)
                .default_width(380.0)
                .default_height(520.0)
                .min_width(240.0)
                .min_height(200.0)
                .scroll([true, true])
                .show(ctx, |ui| {
                    self.ui_operators(ui);
                });
            self.show_ops = open;
        }

        self.ui_diagram_window(ctx);
        self.ui_advanced(ctx);
        self.ui_templates_guide(ctx);

        // Keep the active row in the rows canvas (e.g. 1 + 1 + 1 creates Row 3 below the fold).
        self.follow_rows_scroll();

        // Central Area matching Tkinter vertical structure.
        // Both-axis scroll so shrinking the window still reaches rows / bottom panels.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Viewport size before entering the scroll area (inside both-scroll, width is unbounded).
            let viewport_w = ui.available_width();
            let bottom_min_w = 265.0 + 10.0 + 340.0 + 10.0 + 320.0;
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(viewport_w.max(ROWS_CONTENT_MIN_W).max(bottom_min_w));

                    // Section 2: Mathematical Addend Sequence (Rows Area)
                    self.ui_rows(ui, viewport_w);

                    ui.add_space(8.0);

                    // Section 3: Detailed Results & Mathematical Analysis
                    self.ui_results_bar(ui);

                    ui.add_space(8.0);

                    // Section 4: Bottom 3-Column Area (Keypad | Templates & Console | Inspector)
                    ui.horizontal_top(|ui| {
                        // Column 1: Keypad
                        ui.allocate_ui_with_layout(egui::vec2(265.0, 260.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                            self.ui_bottom_keypad(ui);
                        });

                        ui.add_space(10.0);

                        // Column 2: Templates & Descriptive Columns Console
                        ui.allocate_ui_with_layout(egui::vec2(340.0, 260.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                            self.ui_bottom_middle(ui);
                        });

                        ui.add_space(10.0);

                        // Column 3: Addend Representations Inspector
                        let insp_w = (viewport_w - 265.0 - 10.0 - 340.0 - 10.0).max(320.0);
                        ui.allocate_ui_with_layout(egui::vec2(insp_w, 260.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                            self.ui_bottom_inspector(ui);
                        });
                    });
                });
        });
    }
}

impl CalculatorApp {
    /// Match Tkinter `_scroll_to_row`: when the target row or row count changes, scroll that row into view.
    fn follow_rows_scroll(&mut self) {
        let target = self.doc.target_row;
        let count = self.doc.rows.len();
        if target != self.rows_follow_target || count != self.rows_follow_count {
            // A new result row appears after the last used row — prefer scrolling to it.
            let scroll_i = if count > self.rows_follow_count
                && self.doc.rows.last().map_or(false, |r| r.is_result)
            {
                count - 1
            } else {
                target.min(count.saturating_sub(1))
            };
            self.rows_scroll_to = Some(scroll_i);
            self.rows_follow_target = target;
            self.rows_follow_count = count;
        }
    }

    fn ui_operators(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.show_words, false, "Operators");
            ui.selectable_value(&mut self.show_words, true, "Algebra word problems");
        });
        self.ui_picture(ui);
        if self.show_words {
            self.ui_word_problems(ui);
            return;
        }
        ui.heading("Operators");
        ui.label("Click an operator to put it on the target row.");
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.op_filter);
        });

        let cats = {
            let mut c = vec!["All".to_string()];
            for op in ops::all_operators() {
                let lab = if op.subcategory.is_empty() {
                    op.category.to_string()
                } else {
                    format!("{} / {}", op.category, op.subcategory)
                };
                if !c.contains(&lab) {
                    c.push(lab);
                }
            }
            c
        };
        egui::ComboBox::from_id_salt("cat")
            .selected_text(&self.selected_category)
            .show_ui(ui, |ui| {
                for c in &cats {
                    ui.selectable_value(&mut self.selected_category, c.clone(), c);
                }
            });

        ui.separator();
        // Window already scrolls; draw the list directly so one scrollbar covers the whole panel.
        let filter = self.op_filter.to_ascii_lowercase();
        let mut pick: Option<&'static str> = None;
        for op in ops::all_operators() {
            let lab = if op.subcategory.is_empty() {
                op.category.to_string()
            } else {
                format!("{} / {}", op.category, op.subcategory)
            };
            if self.selected_category != "All" && lab != self.selected_category {
                continue;
            }
            if !filter.is_empty() {
                let blob = format!("{} {} {} {}", op.name, op.symbol, op.key, op.formula).to_ascii_lowercase();
                if !blob.contains(&filter) {
                    continue;
                }
            }
            let label = format!("{}   {}", op.symbol, op.name);
            if ui.selectable_label(false, label).clicked() {
                pick = Some(op.key);
            }
            ui.weak(op.formula);
        }
        if let Some(key) = pick {
            self.apply_operator(key);
        }
    }

    fn ui_word_problems(&mut self, ui: &mut Ui) {
        ui.heading("Algebra word problems");
        ui.label("Click a problem. Ready ones go into the target row as their equation.");
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.op_filter);
        });
        let items = ops::words::items();
        let cats = {
            let mut c = vec!["All".to_string()];
            for it in items {
                if !c.iter().any(|x| x == it.category) {
                    c.push(it.category.to_string());
                }
            }
            c
        };
        egui::ComboBox::from_id_salt("wordcat").selected_text(&self.word_category).show_ui(ui, |ui| {
            for c in &cats {
                ui.selectable_value(&mut self.word_category, c.clone(), c);
            }
        });
        let ready = items.iter().filter(|i| i.ready).count();
        ui.weak(format!("{} problems: {} ready, {} reference only.", items.len(), ready, items.len() - ready));
        ui.separator();
        // Window already scrolls; draw the list directly so one scrollbar covers the whole panel.
        let filter = self.op_filter.to_ascii_lowercase();
        let mut pick: Option<&'static ops::words::Item> = None;
        for it in items {
            if self.word_category != "All" && it.category != self.word_category {
                continue;
            }
            if !filter.is_empty() && !format!("{} {} {}", it.id, it.category, it.problem).to_ascii_lowercase().contains(&filter) {
                continue;
            }
            let tag = if it.ready { "Ready" } else { "Reference" };
            if ui.selectable_label(false, format!("{}  {}  [{}]", it.id, it.category, tag)).clicked() {
                pick = Some(it);
            }
            ui.weak(it.problem);
        }
        if let Some(it) = pick {
            self.apply_word_problem(it);
        }
    }

    fn ui_picture(&mut self, ui: &mut Ui) {
        let Some(key) = self.preview_key.clone() else {
            ui.weak("Click an operator or a problem to see its picture.");
            return;
        };
        match crate::pictures::picture_for(&key) {
            Some(path) => {
                ui.add(egui::Image::new(crate::pictures::uri(&path)).max_width(300.0).max_height(170.0));
            }
            None => {
                ui.weak("(picture unavailable)");
            }
        }
        ui.separator();
    }

    fn ui_diagram_window(&mut self, ctx: &egui::Context) {
        let Some(it) = self.diagram else { return };
        let mut close = false;
        let builder = egui::ViewportBuilder::default()
            .with_title(format!("{}  {}", it.id, it.category))
            .with_inner_size([1000.0, 640.0])
            .with_min_inner_size([320.0, 240.0]);
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("word_diagram"), builder, |ctx, _class| {
            egui::CentralPanel::default().frame(egui::Frame::none().fill(Color32::WHITE).inner_margin(egui::Margin::same(12.0))).show(ctx, |ui| {
                ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_min_width(960.0);
                    ui.label(RichText::new(format!("{}    {}    {}", it.id, it.category, if it.ready { "Ready" } else { "Reference" })).color(BLUE).strong().size(18.0));
                    ui.add_space(4.0);
                    ui.label(RichText::new(it.problem).color(Color32::BLACK).size(16.0));
                    ui.add_space(8.0);
                    match crate::pictures::word_picture(it.key) {
                        Some(path) => {
                            ui.add(egui::Image::new(crate::pictures::uri(&path)).max_width(960.0).max_height(480.0));
                        }
                        None => {
                            ui.label("The diagram file is not in the folder.");
                        }
                    }
                });
            });
            if ctx.input(|i| i.viewport().close_requested()) {
                close = true;
            }
        });
        if close {
            self.diagram = None;
        }
    }

    fn apply_word_problem(&mut self, it: &'static ops::words::Item) {
        self.diagram = Some(it);
        self.preview_key = Some(format!("word:{}", it.key));
        if !it.ready {
            self.status = format!(
                "{} {} is reference only: its text and steps are stored, but no solver answers it yet, so it is not placed in a row.  {}",
                it.category, it.id, it.problem
            );
            return;
        }
        self.apply_operator(&format!("word:{}", it.key));
        self.status = format!(
            "{}  {} {}: the row is its equation. Each box holds a number from the text, already filled in: press =, or change a box first.",
            it.problem, it.category, it.id
        );
    }

    fn tpl_progress(&self) -> [bool; 4] {
        let text = self.tpl_text.trim();
        let ok = crate::template_math::parse_math(text).is_ok();
        [
            ok,
            ok && self.tpl_previewed.as_deref() == Some(text),
            ok && self.tpl_saved.iter().any(|t| t == text),
            ok && self.tpl_applied && self.tpl_pick.trim() == text,
        ]
    }

    fn tpl_write_file(&mut self) {
        if let Err(e) = crate::template_math::save_data(&self.tpl_path, &self.tpl_saved, self.tpl_hide_guide) {
            self.status = format!("(Could not write {}: {}. The templates will be forgotten when the window closes.)", self.tpl_path.display(), e);
        }
    }

    fn tpl_restore_preview(&mut self) {
        let Some(state) = self.tpl_preview.take() else { return };
        if state.row < self.doc.rows.len() && row_signature(&self.doc.rows[state.row]) == state.shown {
            self.doc.rows[state.row] = state.snapshot;
        }
    }

    fn tpl_do_preview(&mut self) {
        let text = self.tpl_text.trim().to_string();
        let row = match crate::template_math::parse_math(&text) {
            Ok(r) => r,
            Err(e) => {
                self.status = format!("Template: {}", e);
                return;
            }
        };
        self.tpl_restore_preview();
        self.doc.ensure_target();
        let i = self.doc.target_row;
        let snapshot = if self.doc.rows[i].is_result { Row::blank() } else { self.doc.rows[i].clone() };
        let (describe, n) = (crate::template_math::describe(&row), crate::template_math::empty_boxes(&row));
        self.doc.place_row(row);
        self.tpl_preview = Some(TplPreview { row: i, snapshot, shown: row_signature(&self.doc.rows[i]) });
        self.tpl_previewed = Some(text);
        self.status = format!(
            "Preview - posted in Row {}:\n  {}\n{}\nLike it? Press Save Template. Not right? Change the text and press Preview Posted again.",
            i + 1,
            describe,
            if n > 0 { format!("{} empty square box{} to fill in.", n, if n == 1 { "" } else { "es" }) } else { "No empty boxes, so press the keypad = key to calculate it.".into() }
        );
    }

    fn tpl_do_save(&mut self) {
        let text = self.tpl_text.trim().to_string();
        if let Err(e) = crate::template_math::parse_math(&text) {
            self.status = format!("Template: {}", e);
            return;
        }
        if self.tpl_saved.contains(&text) {
            self.status = format!("Already saved: {}\nPick it in the list next to Apply Template.", text);
        } else {
            self.tpl_saved.push(text.clone());
            self.tpl_write_file();
            if !self.status.starts_with("(Could not") {
                self.status = format!("Saved template: {}\nPick it in the list next to Apply Template and press Apply Template whenever you want it in a row. It is kept for next time too.", text);
            }
        }
        self.tpl_pick = text;
    }

    fn tpl_do_apply(&mut self) {
        let text = self.tpl_pick.trim().to_string();
        if text.is_empty() {
            self.status = "No template picked yet. Type some math in the text box, press Preview Posted, then Save Template, then pick it in this list.".into();
            return;
        }
        let row = match crate::template_math::parse_math(&text) {
            Ok(r) => r,
            Err(e) => {
                self.status = format!("Template: {}", e);
                return;
            }
        };
        self.tpl_restore_preview();
        let (describe, n) = (crate::template_math::describe(&row), crate::template_math::empty_boxes(&row));
        self.doc.place_row(row);
        self.tpl_applied = true;
        self.status = format!(
            "Template placed in Row {}:  {}\n{}",
            self.doc.target_row + 1,
            describe,
            if n > 0 { format!("Fill the {} empty square box{} (click one, then use the number pad), then press =.", n, if n == 1 { "" } else { "es" }) } else { "Nothing to fill in: press =".into() }
        );
    }

    fn ui_templates_guide(&mut self, ctx: &egui::Context) {
        if !self.tpl_guide {
            return;
        }
        let mut open = true;
        let done = self.tpl_progress();
        let current = done.iter().position(|d| !d);
        let text = self.tpl_text.trim().to_string();
        let target_row = self.doc.target_row;
        let mut hide = self.tpl_hide_guide;
        egui::Window::new("Make your own template - 4 steps")
            .open(&mut open)
            .resizable(true)
            .default_width(900.0)
            .default_height(280.0)
            .min_width(280.0)
            .min_height(160.0)
            .scroll([true, true])
            .show(ctx, |ui| {
                ui.label("Follow the steps in order, left to right. The step you are on is yellow, finished steps are green with a tick.");
                ui.horizontal_top(|ui| {
                    for (k, (title, detail)) in GUIDE_STEPS.iter().enumerate() {
                        let (fill, mark) = if done[k] {
                            (Color32::from_rgb(0xdf, 0xf3, 0xdc), "✓".to_string())
                        } else if Some(k) == current {
                            (Color32::from_rgb(0xff, 0xf3, 0xc4), (k + 1).to_string())
                        } else {
                            (Color32::WHITE, (k + 1).to_string())
                        };
                        egui::Frame::none().fill(fill).stroke(egui::Stroke::new(2.0, Color32::LIGHT_GRAY)).inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
                            ui.set_width(200.0);
                            ui.set_min_height(150.0);
                            ui.label(RichText::new(mark).size(22.0).strong());
                            ui.label(RichText::new(*title).strong());
                            ui.label(detail.replace("the target row", &format!("Row {}", target_row + 1)));
                            if k == 0 {
                                if text.is_empty() {
                                    ui.weak("Nothing typed yet.");
                                } else {
                                    match crate::template_math::parse_math(&text) {
                                        Ok(r) => {
                                            let n = crate::template_math::empty_boxes(&r);
                                            ui.colored_label(
                                                Color32::from_rgb(0x2a, 0x6a, 0x20),
                                                format!("✓ Understood:  {}   ({} empty box{})", crate::template_math::describe(&r), n, if n == 1 { "" } else { "es" }),
                                            );
                                        }
                                        Err(e) if e.incomplete => {
                                            ui.weak(format!("… keep typing.  {}", e));
                                        }
                                        Err(e) => {
                                            ui.colored_label(RED, format!("✗ {}", e));
                                        }
                                    }
                                }
                            }
                        });
                    }
                });
                ui.checkbox(&mut hide, "Don't show this again  (the ? button opens it any time)");
            });
        if hide != self.tpl_hide_guide {
            self.tpl_hide_guide = hide;
            self.tpl_write_file();
        }
        self.tpl_guide = open;
    }

    fn selected_box_kind(&self) -> (String, String, String) {
        let row = &self.doc.rows[self.doc.target_row.min(self.doc.rows.len() - 1)];
        match row.operator() {
            Some(op) => {
                let _ = op;
                match row.owner_of(&self.doc.target_path) {
                    Some((owner, j)) => {
                        let kind = owner.kinds.get(j).copied().unwrap_or("number").to_string();
                        let name = owner.inputs.get(j).copied().unwrap_or("input").to_string();
                        (owner.name.to_string(), name, kind)
                    }
                    None => (op.name.to_string(), "input".into(), "number".into()),
                }
            }
            None => (String::new(), String::new(), "number".into()),
        }
    }

    fn adv_pick_suggested(&mut self) {
        let (_, _, kind) = self.selected_box_kind();
        self.adv_kind = kind;
        self.adv_msg.clear();
    }

    fn ui_advanced(&mut self, ctx: &egui::Context) {
        if !self.show_adv {
            return;
        }
        let mut open = true;
        let (opname, boxname, sug) = self.selected_box_kind();
        egui::Window::new("Advanced Input")
            .open(&mut open)
            .resizable(true)
            .default_width(380.0)
            .default_height(420.0)
            .min_width(240.0)
            .min_height(200.0)
            .scroll([true, true])
            .show(ctx, |ui| {
            if opname.is_empty() {
                ui.colored_label(RED, "No box selected - pick an operator, then click a box in the rows.");
            } else {
                ui.colored_label(
                    Color32::from_rgb(0x10, 0x60, 0x10),
                    format!("Box: {} > {}      suggested: {}", opname, boxname, crate::kinds::kind_title(&sug)),
                );
            }
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.adv_filter);
                if ui.button("✕").clicked() {
                    self.adv_filter.clear();
                }
            });
            let q = self.adv_filter.to_ascii_lowercase();
            // Kinds list stays height-capped; the window scroll covers the rest when resized small.
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                let mut shown = 0;
                for (id, title, how, ex) in crate::kinds::KINDS {
                    if !q.is_empty() && !format!("{title} {how} {ex}").to_ascii_lowercase().contains(&q) {
                        continue;
                    }
                    shown += 1;
                    if ui.selectable_label(self.adv_kind == *id, format!("{title}    e.g.  {ex}")).clicked() {
                        self.adv_kind = id.to_string();
                        self.adv_text.clear();
                        self.adv_msg.clear();
                    }
                }
                if shown == 0 {
                    ui.weak("Nothing matches");
                }
            });
            ui.separator();
            ui.label(RichText::new(format!("Typing: {}", crate::kinds::kind_title(&self.adv_kind))).strong());
            ui.weak(format!("{}   Example: {}", crate::kinds::how_to_type(&self.adv_kind), crate::kinds::example(&self.adv_kind)));
            ui.add(TextEdit::multiline(&mut self.adv_text).desired_rows(4).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Put in box").strong()).clicked() {
                    self.adv_put_in_box();
                }
                if ui.button("Use example").clicked() {
                    self.adv_text = crate::kinds::example(&self.adv_kind).to_string();
                }
                if ui.button("Clear box").clicked() {
                    self.doc.ensure_target();
                    let (r, b) = (self.doc.target_row, self.doc.target_path.clone());
                    if let Some(slot) = self.doc.rows[r].slot_mut(&b) {
                        slot.clear();
                        self.doc.rows[r].clear_result();
                    }
                    self.adv_text.clear();
                    self.adv_msg.clear();
                }
            });
            if !self.adv_msg.is_empty() {
                ui.label(RichText::new(&self.adv_msg).strong());
            }
        });
        self.show_adv = open;
    }

    fn adv_put_in_box(&mut self) {
        self.doc.ensure_target();
        let (r, b) = (self.doc.target_row, self.doc.target_path.clone());
        if self.doc.rows[r].slot_text(&b).is_none() {
            self.adv_msg = "Click a square box in the rows first.".into();
            return;
        }
        let raw = self.adv_text.trim().to_string();
        match crate::kinds::parse_kind(&self.adv_kind, &raw) {
            Ok(_) => {
                if let Some(slot) = self.doc.rows[r].slot_mut(&b) {
                    *slot = raw;
                }
                self.doc.rows[r].clear_result();
                self.doc.rows[r].error = None;
                self.adv_msg = "Put in the box. Press = on the number pad to calculate.".into();
            }
            Err(e) => self.adv_msg = e,
        }
    }

    fn apply_operator(&mut self, key: &str) {
        self.apply_operator_inner(key);
        if self.doc.rows.iter().any(|r| r.card_words().iter().any(|w| !w.0.is_empty() || !w.1.is_empty())) {
            self.show_card_words = true;
        }
    }

    fn apply_operator_inner(&mut self, key: &str) {
        self.preview_key = Some(key.to_string());
        self.doc.ensure_target();
        if let Some(spec) = crate::multirow::get(key) {
            let name = ops::get(key).map_or("This operator", |o| o.name);
            let static_key = ops::get(key).map_or("", |o| o.key);
            match self.doc.apply_template(static_key, spec) {
                Ok(()) => {
                    let steps: Vec<String> = spec.rows.iter().enumerate().map(|(k, r)| format!("Row {}: {}.", k + 1, r.title)).collect();
                    self.status = format!(
                        "{} needs {} rows.\n{}\n{}\nJoined with {}. Fill the boxes in every row, then press =.",
                        name,
                        spec.rows.len(),
                        steps.join("   "),
                        spec.title,
                        spec.joins.join(" ")
                    );
                }
                Err(e) => self.status = e,
            }
            return;
        }
        if let Some(op) = ops::get(key) {
            let nests = self.doc.target_row == 0 && self.doc.rows[0].operator().is_some();
            self.doc.place_operator(op);
            self.status = if nests {
                format!("{} is now inside the selected box of Row 1 — fill the boxes, press =.  {}", op.name, op.formula)
            } else {
                format!("Row {}: {} — fill the boxes, press =.  {}", self.doc.target_row + 1, op.name, op.formula)
            };
        }
    }

    fn ui_rows(&mut self, ui: &mut Ui, viewport_w: f32) {
        // Fixed grid like Tkinter:
        // col0 minsize=170 | col1 minsize=430 | col2 weight=1 (math) | col3 actions | col4 result
        // Size math from the visible viewport (not unbounded scroll content width), and keep a
        // fixed content_w so a narrow window gets a horizontal scrollbar instead of clipping.
        let result_card_w = COL_RESULT;
        let math_w = (viewport_w - COL_ROW - COL_HELPER - COL_ACTIONS - result_card_w - 16.0)
            .clamp(MATH_SLOT_MIN_W, MATH_SLOT_MAX_W);
        let content_w = COL_ROW + COL_HELPER + math_w + COL_ACTIONS + result_card_w + 8.0;

        // Header bar matching Tkinter:
        // header = tk.Frame(self, bg="#e4e4e4")
        // Row label padx=(8, 70) | Mathematical Addend Sequence ...
        egui::Frame::none()
            .fill(Color32::from_rgb(0xe4, 0xe4, 0xe4))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.set_min_width(content_w);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Row").size(11.0).strong().color(Color32::BLACK));
                    ui.add_space(70.0);
                    ui.label(
                        RichText::new(
                            "Mathematical Addend Sequence (each number owns its Item, Context and extra fields)",
                        )
                        .size(11.0)
                        .strong()
                        .color(Color32::BLACK),
                    );
                });
            });

        egui::Frame::none()
            .fill(CREAM)
            .stroke(egui::Stroke::new(2.0, Color32::BLACK))
            .inner_margin(egui::Margin::same(4.0))
            .show(ui, |ui| {
                ui.set_min_width(content_w);
                // Scroll only this canvas (not the outer page). Explicit Y offset avoids nested
                // scroll_to_me leaking into ScrollArea::both above.
                let scroll_y = self.rows_scroll_to.take().map(|i| {
                    let mut y = 0.0;
                    for k in 0..i {
                        y += if k == 0 { MATH_SLOT_H + 8.0 } else { MATH_SLOT_H + 24.0 };
                    }
                    y
                });
                let mut rows_scroll = ScrollArea::both()
                    .id_salt("rows_canvas")
                    .max_height(ROWS_CANVAS_H)
                    .auto_shrink([false, false]);
                if let Some(y) = scroll_y {
                    rows_scroll = rows_scroll.vertical_scroll_offset(y);
                }
                rows_scroll.show(ui, |ui| {
                        ui.set_min_width(content_w - 8.0);
                        for i in 0..self.doc.rows.len() {
                            let is_target = i == self.doc.target_row;
                            let row_h = if i == 0 { MATH_SLOT_H + 8.0 } else { MATH_SLOT_H + 24.0 };
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                                // Column 0: Row badge — exact width
                                fixed_col(ui, COL_ROW, row_h, egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.add_space(10.0);
                                    let badge_bg = if is_target { ROW_BADGE_ACTIVE } else { ROW_BADGE_BG };
                                    let title = if self.doc.rows[i].is_result {
                                        "Result".to_string()
                                    } else {
                                        format!("Row {}", i + 1)
                                    };
                                    let btn = Button::new(RichText::new(title).size(14.0).strong().color(ROW_BADGE_FG))
                                        .fill(badge_bg)
                                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0x8a, 0xb8, 0x80)))
                                        .rounding(2.0);
                                    if ui.add_sized(egui::vec2(78.0, 28.0), btn).clicked() {
                                        self.doc.target_row = i;
                                        self.doc.target_path = self.doc.rows[i].first_box();
                                    }
                                    if let Some(t) = self.doc.template {
                                        if let Some(rs) = t.spec.rows.get(i) {
                                            ui.label(RichText::new(rs.title).size(10.0).italics().color(ROW_BADGE_FG));
                                        }
                                    }
                                    if self.doc.rows[i].is_result {
                                        ui.label(RichText::new("(the answer)").size(9.0).italics().color(ROW_BADGE_FG));
                                    } else if let Some(op) = self.doc.rows[i].operator() {
                                        ui.label(RichText::new(op.name).size(9.0).italics().color(ROW_BADGE_FG));
                                    }
                                });

                                // Column 1: helper / join — exact width so Row 1 and Row 2 math start at the same X
                                fixed_col(ui, COL_HELPER, row_h, egui::Layout::top_down(egui::Align::Min), |ui| {
                                    if i == 0 {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new("math operator to join rows 1 and 2")
                                                .size(12.0)
                                                .color(Color32::from_rgb(0x33, 0x33, 0x33)),
                                        );
                                        ui.add_space(2.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(40.0);
                                            draw_down_arrow(ui);
                                        });
                                    } else if !self.doc.rows[i].is_result {
                                        fixed_col(
                                            ui,
                                            COL_HELPER,
                                            ROW_CARD_TOP - CARD_FRAME_INSET,
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                ui.label(
                                                    RichText::new("helper card (slot 1)")
                                                        .size(13.0)
                                                        .strong()
                                                        .color(Color32::from_rgb(0xb0, 0x30, 0x10)),
                                                );
                                            },
                                        );
                                        let join_sym = self.doc.rows[i].join.symbol();
                                        let center = if join_sym.is_empty() { "□" } else { join_sym };
                                        let mut l = String::new();
                                        let mut c = String::new();
                                        let (lead_clk, center_clk) = draw_standalone_card(
                                            ui,
                                            "+",
                                            center,
                                            "helper",
                                            false,
                                            &mut l,
                                            "[Label]",
                                            &mut c,
                                            "[Context]",
                                        );
                                        if lead_clk || center_clk {
                                            self.doc.target_row = i;
                                            if center_clk {
                                                self.doc.rows[i].join = match self.doc.rows[i].join {
                                                    Join::None => Join::Add,
                                                    Join::Add => Join::Sub,
                                                    Join::Sub => Join::Mul,
                                                    Join::Mul => Join::Div,
                                                    Join::Div => Join::Eq,
                                                    Join::Eq => Join::Then,
                                                    _ => Join::None,
                                                };
                                                self.doc.rows[i].clear_result();
                                            }
                                        }
                                    } else {
                                        ui.label(RichText::new("result").size(11.0).strong().color(ROW_BADGE_FG));
                                        let mut l = String::new();
                                        let mut c = String::new();
                                        draw_standalone_card(
                                            ui,
                                            "=",
                                            "=",
                                            "helper",
                                            false,
                                            &mut l,
                                            "[Label]",
                                            &mut c,
                                            "[Context]",
                                        );
                                    }
                                });

                                // Column 2: white math slot — fixed column width; inner H-scroll so wide formulas aren't clipped
                                let target_path = self.doc.target_path.clone();
                                let mut focused = None;
                                fixed_col(ui, math_w, row_h, egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    egui::Frame::none()
                                        .fill(Color32::WHITE)
                                        .inner_margin(egui::Margin::same(MATH_SLOT_INSET))
                                        .show(ui, |ui| {
                                            ui.set_min_height(MATH_SLOT_H - 8.0);
                                            if !self.doc.rows[i].is_result {
                                                ui.add_space(ROW_CARD_TOP - MATH_SLOT_INSET - CARD_FRAME_INSET);
                                            }
                                            ScrollArea::horizontal()
                                                .id_salt(("math_slot", i))
                                                .auto_shrink([false, false])
                                                .show(ui, |ui| {
                                            if self.doc.rows[i].is_result {
                                                let mut l = String::new();
                                                let mut c = String::new();
                                                draw_standalone_card(
                                                    ui,
                                                    "=",
                                                    &self.doc.rows[i].result,
                                                    "number",
                                                    false,
                                                    &mut l,
                                                    "[Item]",
                                                    &mut c,
                                                    "[Context]",
                                                );
                                            } else if self.doc.rows[i].flatten().is_empty() {
                                                let mut l = String::new();
                                                let mut c = String::new();
                                                let (lead_clk, center_clk) = draw_standalone_card(
                                                    ui,
                                                    "+",
                                                    self.doc.rows[i].inputs.first().map(|s| s.as_str()).unwrap_or("□"),
                                                    "number",
                                                    is_target,
                                                    &mut l,
                                                    "[Item]",
                                                    &mut c,
                                                    "[Context]",
                                                );
                                                if lead_clk || center_clk {
                                                    self.doc.target_row = i;
                                                    self.doc.target_path = vec![0];
                                                    self.doc.rows[i].ensure_number_box();
                                                }
                                            } else {
                                                focused = self.ui_formula(ui, i, is_target, &target_path);
                                            }
                                                });
                                        });
                                });

                                if let Some(b) = focused {
                                    self.doc.target_row = i;
                                    self.doc.target_path = b;
                                }

                                // Column 3: actions
                                fixed_col(ui, COL_ACTIONS, row_h, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("=").size(26.0).strong().color(Color32::BLACK));
                                        ui.vertical(|ui| {
                                            egui::Frame::none()
                                                .fill(Color32::BLACK)
                                                .inner_margin(egui::Margin::same(3.0))
                                                .show(ui, |ui| {
                                                    let btn = Button::new(
                                                        RichText::new("math to put\non the left")
                                                            .size(10.0)
                                                            .strong()
                                                            .color(Color32::BLACK),
                                                    )
                                                    .fill(Color32::WHITE)
                                                    .rounding(0.0);
                                                    if ui.add_sized(egui::vec2(118.0, 36.0), btn).clicked() {
                                                        self.doc.target_row = i;
                                                        self.show_ops = true;
                                                    }
                                                });
                                            ui.add_space(6.0);
                                            let clr_btn = Button::new(
                                                RichText::new("clear math on the left")
                                                    .size(10.0)
                                                    .strong()
                                                    .color(CLEAR_MATH_FG),
                                            )
                                            .fill(CLEAR_MATH_BG)
                                            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xe0, 0xa0, 0xa0)))
                                            .rounding(2.0);
                                            if ui.add_sized(egui::vec2(124.0, 28.0), clr_btn).clicked() {
                                                self.doc.clear_row_math(i);
                                                self.doc.template = None;
                                            }
                                        });
                                    });
                                });

                                // Column 4: result card (=)
                                if !self.doc.rows[i].is_result {
                                    let mut l = String::new();
                                    let mut c = String::new();
                                    fixed_col(ui, result_card_w, row_h, egui::Layout::top_down(egui::Align::Min), |ui| {
                                        ui.add_space(ROW_CARD_TOP - CARD_FRAME_INSET);
                                        draw_standalone_card(
                                            ui,
                                            "=",
                                            &self.doc.rows[i].result,
                                            "number",
                                            false,
                                            &mut l,
                                            "[Item]",
                                            &mut c,
                                            "[Context]",
                                        );
                                    });
                                }
                            });

                            if let Some(note) = self.doc.rows[i].op_key.as_deref().map(crate::layout::note).filter(|n| !n.is_empty()) {
                                ui.label(RichText::new(note).italics().weak());
                            }
                        }
                    });
            });
    }

    fn ui_formula(&mut self, ui: &mut Ui, i: usize, is_target: bool, target_path: &[usize]) -> Option<Vec<usize>> {
        use crate::layout::{self};
        let mut focused = None;
        let cards = self.doc.rows[i].flatten();
        if cards.is_empty() {
            return None;
        }
        let words = self.doc.rows[i].card_words();
        let mut word_edits: Vec<(usize, String, String)> = Vec::new();
        let pcs: Vec<layout::PC> = cards.iter().map(|c| c.pc()).collect();
        let stack = self.doc.rows[i].op_key.as_deref().map_or(false, layout::stacks);
        let plan = layout::plan_pc(&pcs, stack);
        let (n_rows, n_cols) = (plan.rows().max(1) as usize, plan.columns.max(1) as usize);

        enum Cell {
            Empty,
            Card(usize),
            Bar,
        }
        let mut grid: Vec<Vec<Cell>> = (0..n_rows).map(|_| (0..n_cols).map(|_| Cell::Empty).collect()).collect();
        for (idx, p) in plan.placed.iter().enumerate() {
            if let Some((r, c)) = p {
                grid[*r as usize][*c as usize] = Cell::Card(idx);
            }
        }
        for &(r, c, w) in &plan.bars {
            for k in c..c + w {
                if let Cell::Empty = grid[r as usize][k as usize] {
                    grid[r as usize][k as usize] = Cell::Bar;
                }
            }
        }

        ui.horizontal(|ui| {
            egui::Grid::new(("formula", i)).spacing([8.0, 10.0]).show(ui, |ui| {
                for r in 0..n_rows {
                    for c in 0..n_cols {
                        match grid[r][c] {
                            Cell::Empty => {
                                ui.label("");
                            }
                            Cell::Bar => {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width().max(350.0), 4.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
                            }
                            Cell::Card(idx) => {
                                let card = &cards[idx];
                                let (mut label, mut context) = words.get(idx).cloned().unwrap_or_default();
                                if let Some(p) = draw_formula_card(
                                    ui,
                                    card,
                                    &self.doc.rows[i],
                                    is_target,
                                    target_path,
                                    &mut label,
                                    &mut context,
                                    idx,
                                    &mut word_edits,
                                ) {
                                    focused = Some(p);
                                }
                            }
                        }
                    }
                    ui.end_row();
                }
            });
        });
        if let Some(err) = &self.doc.rows[i].error {
            ui.colored_label(RED, err);
        }
        for (idx, l, c) in word_edits {
            self.doc.rows[i].words.insert(idx, (l, c));
        }
        focused
    }

    fn ui_results_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Detailed Results & Mathematical Analysis:").size(11.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for t in ["A -", "100%", "A +", "Wrap: ON", "Copy Results", "Expand ↑"] {
                    let _ = ui.add(Button::new(RichText::new(t).size(9.0).strong().color(Color32::BLACK)).fill(Color32::from_rgb(0xf4, 0xe8, 0xc0)));
                }
            });
        });
        ui.add_space(2.0);
        // Tkinter: Text height=5, Consolas 8, fg #106010
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xcc, 0xcc, 0xcc)))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.set_min_height(80.0);
                ui.set_width(ui.available_width());
                let msg = if self.status.is_empty() {
                    "Mathematical and descriptive calculation results will appear here."
                } else {
                    &self.status
                };
                ui.label(RichText::new(msg).monospace().size(11.0).color(Color32::from_rgb(0x10, 0x60, 0x10)));
            });
    }

    fn ui_bottom_keypad(&mut self, ui: &mut Ui) {
        let btn_sz = egui::vec2(54.0, 26.0);
        let wide_sz = egui::vec2(90.0, 26.0);
        let tall_btn_sz = egui::vec2(90.0, 86.0); // Spans 3 rows!

        // Row 0: (, ), C, Operators
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            if ui.add_sized(btn_sz, Button::new("(").fill(PALE_BLUE)).clicked() { self.on_key("("); }
            if ui.add_sized(btn_sz, Button::new(")").fill(PALE_BLUE)).clicked() { self.on_key(")"); }
            if ui.add_sized(btn_sz, Button::new(RichText::new("C").color(Color32::WHITE).strong()).fill(RED)).clicked() { self.doc.clear_entry(); }
            if ui.add_sized(wide_sz, Button::new("Operators").fill(PALE_BLUE)).clicked() { self.show_ops = !self.show_ops; }
        });

        // Row 1: 7, 8, 9, Next Row (↓)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            if ui.add_sized(btn_sz, Button::new("7").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("7"); }
            if ui.add_sized(btn_sz, Button::new("8").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("8"); }
            if ui.add_sized(btn_sz, Button::new("9").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("9"); }
            if ui.add_sized(wide_sz, Button::new("Next Row (↓)").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() {
                self.doc.next_row();
                self.status = format!("Row {} ready.", self.doc.target_row + 1);
            }
        });

        // Rows 2, 3, 4 with tall = key on the right
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                // Row 2: 4, 5, 6
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                    if ui.add_sized(btn_sz, Button::new("4").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("4"); }
                    if ui.add_sized(btn_sz, Button::new("5").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("5"); }
                    if ui.add_sized(btn_sz, Button::new("6").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("6"); }
                });
                // Row 3: 1, 2, 3
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                    if ui.add_sized(btn_sz, Button::new("1").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("1"); }
                    if ui.add_sized(btn_sz, Button::new("2").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("2"); }
                    if ui.add_sized(btn_sz, Button::new("3").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("3"); }
                });
                // Row 4: 0, ., Backspace
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
                    if ui.add_sized(btn_sz, Button::new("0").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("0"); }
                    if ui.add_sized(btn_sz, Button::new(".").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("."); }
                    if ui.add_sized(btn_sz, Button::new("Backspace").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.doc.backspace(); }
                });
            });

            // Tall = key spanning 3 rows
            let eq_btn = Button::new(RichText::new("=").size(22.0).strong().color(Color32::WHITE)).fill(GREEN);
            if ui.add_sized(tall_btn_sz, eq_btn).clicked() {
                self.equals();
            }
        });

        // Row 5: ±, EE
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            if ui.add_sized(btn_sz, Button::new("±").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("±"); }
            if ui.add_sized(btn_sz, Button::new("EE").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() { self.on_key("EE"); }
        });

        ui.add_space(4.0);

        // Join keys section
        let join_row = self.doc.target_row.max(1) + 1;
        ui.label(RichText::new(format!("Join Row {} to the rows above (helper card):", join_row)).size(10.0).strong().color(Color32::from_rgb(0xb0, 0x30, 0x10)));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
            for sym in ["+", "−", "×", "÷", "=", "≠", "<", ">", "≤", "≥"] {
                if ui.add_sized(egui::vec2(23.0, 24.0), Button::new(RichText::new(sym).strong()).fill(HELPER_YELLOW)).clicked() {
                    self.status = self.doc.join_key(sym);
                }
            }
        });

        ui.add_space(4.0);

        // Advanced Input button
        let adv_btn = Button::new(RichText::new("Advanced Input").size(11.0).strong().color(Color32::BLACK)).fill(Color32::from_rgb(0xe6, 0xd6, 0xf6));
        if ui.add_sized(egui::vec2(260.0, 26.0), adv_btn).clicked() {
            self.show_adv = !self.show_adv;
            if self.show_adv {
                self.adv_pick_suggested();
            }
        }
    }

    fn ui_bottom_middle(&mut self, ui: &mut Ui) {
        // 1. Mathematical Templates (Column 1)
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xbd, 0xd0, 0xe8)))
            .fill(Color32::from_rgb(0xf0, 0xf4, 0xf4))
            .inner_margin(egui::Margin::same(6.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Mathematical Templates (Column 1)").size(11.0).strong().color(BLUE));
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("bottom_tpl_pick")
                        .width(180.0)
                        .selected_text(if self.tpl_pick.is_empty() { "(saved templates)" } else { self.tpl_pick.as_str() })
                        .show_ui(ui, |ui| {
                            for t in self.tpl_saved.clone() {
                                ui.selectable_value(&mut self.tpl_pick, t.clone(), t);
                            }
                        });
                    if ui.add(Button::new(RichText::new("Apply Template").color(BLUE)).fill(Color32::from_rgb(0xcf, 0xe0, 0xf7))).clicked() {
                        self.tpl_do_apply();
                    }
                    if ui.add(Button::new(RichText::new("?").color(Color32::from_rgb(0x7a, 0x5a, 0x00))).fill(Color32::from_rgb(0xff, 0xf3, 0xc4))).clicked() {
                        self.tpl_guide = true;
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let r = ui.add(TextEdit::singleline(&mut self.tpl_text).desired_width(170.0).hint_text("(? + 3) × 2"));
                    if r.changed() {
                        let text = self.tpl_text.trim().to_string();
                        if text != self.tpl_last_text {
                            self.tpl_applied = false;
                            if !text.is_empty() && self.tpl_last_text.is_empty() && !self.tpl_hide_guide {
                                self.tpl_guide = true;
                            }
                            self.tpl_last_text = text;
                        }
                    }
                    ui.label(RichText::new("=").strong());
                    if ui.add(Button::new(RichText::new("Preview Posted").color(Color32::from_rgb(0xc0, 0x50, 0x00))).fill(PALE_ORANGE)).clicked() {
                        self.tpl_do_preview();
                    }
                    if ui.add(Button::new("Save Template").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() {
                        self.tpl_do_save();
                    }
                });
            });

        ui.add_space(8.0);

        // 2. Descriptive Columns Console (Col 2 & 3)
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xd8, 0xb8, 0xe8)))
            .fill(Color32::from_rgb(0xf0, 0xf4, 0xf4))
            .inner_margin(egui::Margin::same(6.0))
            .show(ui, |ui| {
                ui.label(RichText::new("Descriptive Columns Console (Col 2 & 3)").size(11.0).strong().color(Color32::from_rgb(0x8a, 0x1f, 0xb0)));
                ui.add_space(2.0);
                // Unit row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Unit:").size(10.0).strong());
                    for u in ["pens", "pencils", "books", "apples", "oranges", "cars"] {
                        if ui.add(Button::new(RichText::new(u).size(10.0)).fill(Color32::from_rgb(0xe8, 0xe8, 0xf8))).clicked() {
                            self.apply_word_preset(0, u);
                        }
                    }
                });
                ui.add_space(2.0);
                // Context row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Context:").size(10.0).strong());
                    for c in ["in the drawer", "on the desk", "in the room", "in the basket"] {
                        if ui.add(Button::new(RichText::new(c).size(10.0).color(Color32::from_rgb(0xa0, 0x10, 0x40))).fill(Color32::from_rgb(0xfb, 0xe0, 0xe8))).clicked() {
                            self.apply_word_preset(1, c);
                        }
                    }
                });
                ui.add_space(2.0);
                // Custom text and Apply Word button
                ui.horizontal(|ui| {
                    ui.add_space(36.0);
                    ui.add(TextEdit::singleline(&mut self.custom_word).desired_width(170.0).hint_text("custom word..."));
                    if ui.add(Button::new("Apply Word").fill(Color32::from_rgb(0xe8, 0xe8, 0xe8))).clicked() {
                        let w = self.custom_word.trim().to_string();
                        if !w.is_empty() {
                            self.apply_word_preset(0, &w);
                        }
                    }
                });
            });
    }

    fn ui_bottom_inspector(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Addend Representations Inspector").size(11.0).strong().color(BLUE));
        ui.add_space(2.0);
        let insp_str = inspector_text(&self.doc);
        egui::Frame::none()
            .fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0xaa, 0xaa, 0xaa)))
            .inner_margin(egui::Margin::same(4.0))
            .show(ui, |ui| {
                ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    ui.label(RichText::new(insp_str).monospace().size(10.0).color(Color32::BLACK));
                });
            });
    }

    fn on_key(&mut self, key: &str) {
        match key {
            "±" => {
                self.doc.ensure_target();
                let path = self.doc.target_path.clone();
                let row = &mut self.doc.rows[self.doc.target_row];
                if let Some(s) = row.slot_mut(&path) {
                    if let Some(rest) = s.strip_prefix('-') {
                        *s = rest.to_string();
                    } else if !s.is_empty() {
                        s.insert(0, '-');
                    } else {
                        s.push('-');
                    }
                }
            }
            "EE" => {
                self.doc.ensure_target();
                let path = self.doc.target_path.clone();
                let row = &mut self.doc.rows[self.doc.target_row];
                if let Some(s) = row.slot_mut(&path) {
                    if !s.contains('e') && !s.is_empty() && s.chars().last().map_or(false, |c| c.is_ascii_digit()) {
                        s.push('e');
                    }
                }
            }
            "+" | "-" | "*" | "/" => {
                let sym = match key {
                    "-" => "−",
                    "*" => "×",
                    "/" => "÷",
                    k => k,
                };
                self.status = self.doc.join_key(sym);
            }
            "(" | ")" => {
                self.doc.ensure_target();
                let path = self.doc.target_path.clone();
                let row = &mut self.doc.rows[self.doc.target_row];
                if let Some(s) = row.slot_mut(&path) {
                    s.push_str(key);
                }
            }
            "." => self.doc.append_digit('.'),
            d if d.len() == 1 && d.chars().next().unwrap().is_ascii_digit() => {
                self.doc.append_digit(d.chars().next().unwrap());
            }
            _ => {}
        }
    }

    fn equals(&mut self) {
        self.status = match self.doc.equals_all() {
            Ok(msg) => msg,
            Err(e) => e,
        };
    }
}

fn inspector_text(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str("=== TOTAL SUMMARY ===\n");
    let total_str = doc.rows.iter().rev().find(|r| r.is_result).and_then(|r| (!r.result.is_empty()).then(|| r.result.as_str()))
        .or_else(|| doc.rows.first().and_then(|r| (!r.result.is_empty()).then(|| r.result.as_str())))
        .unwrap_or("(not available: not calculated yet)");
    out.push_str(&format!("Math Total: {}\n\n", total_str));
    out.push_str("=== ROWS ===\n\n");
    for (i, row) in doc.rows.iter().enumerate() {
        if row.is_result {
            out.push_str(&format!("--- Row {} (Result) ---\n", i + 1));
            out.push_str(&format!("  Total: {}\n\n", if row.result.is_empty() { "□" } else { &row.result }));
            continue;
        }
        let pat = row.operator().map_or("□", |o| o.pattern);
        out.push_str(&format!("--- Row {} (Addend): pattern {} ---\n", i + 1, pat));
        let tokens: Vec<String> = row.flatten().into_iter().map(|c| c.text).collect();
        out.push_str(&format!("  Editing tokens: {:?}\n", tokens));
        let words = row.card_words();
        for (j, input) in row.inputs.iter().enumerate() {
            let (item, ctx) = words.get(j).cloned().unwrap_or_default();
            let item_s = if item.is_empty() { "empty".to_string() } else { item };
            let ctx_s = if ctx.is_empty() { "empty".to_string() } else { ctx };
            let val = if input.is_empty() { "□" } else { input.as_str() };
            out.push_str(&format!("  Number Group {} (id {:08x}): {} | Item ({}) | Context ({})\n", j + 1, (i * 10 + j + 1) * 0x1234567, val, item_s, ctx_s));
        }
        let calc = if row.result.is_empty() { "(not available: not calculated yet)" } else { &row.result };
        out.push_str(&format!("  Calculated Terms: {}\n\n", calc));
    }
    out
}

#[allow(dead_code)]
fn answer_width(text: &str, min: f32) -> f32 {
    (text.chars().count() as f32 * 15.0 + 16.0).max(min)
}

fn colored_btn(ui: &mut Ui, text: &str, bg: Color32) -> egui::Response {
    ui.add(Button::new(RichText::new(text).color(Color32::WHITE).strong()).fill(bg))
}

/// Reserve an exact column width (Tkinter grid minsize). Content smaller than `w` still keeps the slot.
/// Uses `new_child` (not `allocate_new_ui`) so the parent cursor is not advanced a second time.
fn fixed_col<R>(ui: &mut Ui, w: f32, h: f32, layout: egui::Layout, add: impl FnOnce(&mut Ui) -> R) -> R {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
    add(&mut child)
}

fn draw_down_arrow(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(70.0, 80.0), egui::Sense::hover());
    let origin = rect.min;
    let pts = vec![
        origin + egui::vec2(22.0, 2.0),
        origin + egui::vec2(48.0, 2.0),
        origin + egui::vec2(48.0, 40.0),
        origin + egui::vec2(66.0, 40.0),
        origin + egui::vec2(35.0, 78.0),
        origin + egui::vec2(4.0, 40.0),
        origin + egui::vec2(22.0, 40.0),
    ];
    ui.painter().add(egui::Shape::convex_polygon(
        pts,
        Color32::WHITE,
        egui::Stroke::new(3.0, Color32::BLACK),
    ));
}

fn draw_arrow_button(ui: &mut Ui, sym: &str) {
    let btn = Button::new(RichText::new(sym).size(12.0).strong().color(Color32::from_rgb(0x33, 0x33, 0x33)))
        .fill(ARROW_BG)
        .stroke(egui::Stroke::new(1.0, ARROW_BORDER))
        .rounding(2.0);
    let _ = ui.add_sized(egui::vec2(ARROW_CELL, ARROW_CELL), btn);
}

fn draw_arrow_grid(ui: &mut Ui, center_width: f32, draw_center: impl FnOnce(&mut Ui)) {
    let centered = egui::Layout::centered_and_justified(egui::Direction::LeftToRight);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, ARROW_GAP);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(ARROW_GAP, 0.0);
            fixed_col(ui, ARROW_CELL, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↖"));
            fixed_col(ui, center_width, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↑"));
            fixed_col(ui, ARROW_CELL, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↗"));
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(ARROW_GAP, 0.0);
            fixed_col(ui, ARROW_CELL, ARROW_CENTER_H, centered, |ui| draw_arrow_button(ui, "←"));
            fixed_col(ui, center_width, ARROW_CENTER_H, centered, draw_center);
            fixed_col(ui, ARROW_CELL, ARROW_CENTER_H, centered, |ui| draw_arrow_button(ui, "→"));
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(ARROW_GAP, 0.0);
            fixed_col(ui, ARROW_CELL, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↙"));
            fixed_col(ui, center_width, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↓"));
            fixed_col(ui, ARROW_CELL, ARROW_CELL, centered, |ui| draw_arrow_button(ui, "↘"));
        });
    });
}

fn draw_lead_button(ui: &mut Ui, lead: &str) -> egui::Response {
    let is_plus = lead == "+";
    let fg = if is_plus { BLUE } else { Color32::BLACK };
    let stroke = if is_plus {
        egui::Stroke::new(1.5, BLUE)
    } else {
        egui::Stroke::new(1.5, Color32::BLACK)
    };
    let btn = Button::new(RichText::new(lead).size(18.0).strong().color(fg))
        .fill(PALE_BLUE)
        .stroke(stroke)
        .rounding(2.0);
    ui.add_sized(egui::vec2(LEAD_TRAIL_W, CARD_H), btn)
}

fn draw_trailing_button(ui: &mut Ui) -> egui::Response {
    let btn = Button::new(RichText::new("+").size(18.0).strong().color(ORANGE_TEXT))
        .fill(PALE_ORANGE)
        .stroke(egui::Stroke::new(1.5, ORANGE_BORDER))
        .rounding(2.0);
    ui.add_sized(egui::vec2(LEAD_TRAIL_W, CARD_H), btn)
}

/// Shrink Item/Context widths so the middle strip fits between lead and trailing.
fn fit_word_widths(budget: f32) -> (f32, f32) {
    let gap = 6.0;
    let preferred = ITEM_W + gap + CONTEXT_W;
    if budget <= 0.0 || budget >= preferred {
        return (ITEM_W, CONTEXT_W);
    }
    let usable = (budget - gap).max(0.0);
    let total = ITEM_W + CONTEXT_W;
    let mut item = (ITEM_W * usable / total).clamp(ITEM_W_MIN, ITEM_W);
    let mut context = (usable - item).clamp(CONTEXT_W_MIN, CONTEXT_W);
    if item + gap + context > budget {
        context = (budget - gap - item).max(CONTEXT_W_MIN);
    }
    if item + gap + context > budget {
        item = (budget - gap - context).max(ITEM_W_MIN);
    }
    (item, context)
}

/// Item/Context side-by-side and vertically centred with the 3×3 arrows (Tkinter `make_component`).
fn draw_word_boxes(
    ui: &mut Ui,
    label: &mut String,
    label_hint: &str,
    context: &mut String,
    context_hint: &str,
    item_w: f32,
    context_w: f32,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(item_w + 6.0 + context_w, CARD_H),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add(
                TextEdit::singleline(label)
                    .desired_width(item_w)
                    .hint_text(label_hint)
                    .font(egui::TextStyle::Body),
            );
            ui.add_space(6.0);
            ui.add(
                TextEdit::singleline(context)
                    .desired_width(context_w)
                    .hint_text(context_hint)
                    .font(egui::TextStyle::Body),
            );
        },
    );
}

/// Card shell: lead | middle | trailing at one shared height.
/// Middle content is max-width clamped into the gap so it cannot overlap the end buttons;
/// the card keeps its natural width (trailing stays next to the words, not column-right).
/// Lead/trailing share CARD_H so tops, vertical middles, and bottoms stay aligned.
fn draw_card_shell(
    ui: &mut Ui,
    lead: &str,
    add_middle: impl FnOnce(&mut Ui, /* max middle width */ f32),
) -> egui::Response {
    let mut lead_resp = None;
    ui.horizontal(|ui| {
        ui.set_height(CARD_H);
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        lead_resp = Some(draw_lead_button(ui, lead));
        ui.add_space(6.0);

        let mid_max = (ui.available_width() - LEAD_TRAIL_W - 4.0).max(0.0);
        ui.scope(|ui| {
            ui.set_max_width(mid_max);
            ui.set_height(CARD_H);
            ui.horizontal(|ui| {
                ui.set_height(CARD_H);
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                add_middle(ui, mid_max);
            });
        });

        ui.add_space(4.0);
        let _ = draw_trailing_button(ui);
    });
    lead_resp.expect("lead button response")
}

fn draw_standalone_card(
    ui: &mut Ui,
    lead: &str,
    center_text: &str,
    kind: &str,
    is_selected: bool,
    label: &mut String,
    label_hint: &str,
    context: &mut String,
    context_hint: &str,
) -> (bool, bool) {
    let mut lead_clicked = false;
    let mut center_clicked = false;
    egui::Frame::none()
        .fill(CREAM)
        .inner_margin(egui::Margin::same(CARD_FRAME_INSET))
        .show(ui, |ui| {
            let resp_lead = draw_card_shell(ui, lead, |ui, _mid_w| {
                let bg = if is_selected {
                    TARGET_BLUE
                } else if kind == "helper" {
                    HELPER_YELLOW
                } else {
                    PALE_BLUE
                };
                let stroke = if is_selected {
                    egui::Stroke::new(2.5, BLUE)
                } else if kind == "helper" {
                    egui::Stroke::new(1.0, Color32::from_rgb(0xd0, 0xa0, 0x20))
                } else {
                    egui::Stroke::new(1.0, BLUE)
                };
                let disp = if center_text.is_empty() { "□" } else { center_text };
                let w = (disp.chars().count() as f32 * 9.0 + 16.0).max(36.0);
                draw_arrow_grid(ui, w, |ui| {
                    let btn = Button::new(RichText::new(disp).size(14.0).strong().color(Color32::BLACK))
                        .fill(bg)
                        .stroke(stroke)
                        .rounding(2.0);
                    if ui.add_sized(egui::vec2(w, ARROW_CENTER_H), btn).clicked() {
                        center_clicked = true;
                    }
                });
                ui.add_space(6.0);
                let (item_w, context_w) = fit_word_widths(ui.available_width());
                draw_word_boxes(ui, label, label_hint, context, context_hint, item_w, context_w);
            });
            if resp_lead.clicked() {
                lead_clicked = true;
            }
        });
    (lead_clicked, center_clicked)
}

fn draw_formula_card(
    ui: &mut Ui,
    card: &crate::layout::FlatCard,
    row: &Row,
    is_target_row: bool,
    target_path: &[usize],
    label: &mut String,
    context: &mut String,
    card_idx: usize,
    word_edits: &mut Vec<(usize, String, String)>,
) -> Option<Vec<usize>> {
    use crate::layout::CardKind;
    let mut focused = None;
    egui::Frame::none()
        .fill(CREAM)
        .inner_margin(egui::Margin::same(CARD_FRAME_INSET))
        .show(ui, |ui| {
            let is_box = card.kind == CardKind::Box;
            let resp_lead = draw_card_shell(ui, "+", |ui, _mid_w| {
                let (display_text, bg, stroke) = match card.kind {
                    CardKind::Box => {
                        let is_selected = is_target_row && card.path == target_path;
                        let text = row.slot_text(&card.path).map_or("", |s| s.as_str());
                        let disp = if text.is_empty() { "□" } else { text };
                        let fill = if is_selected { TARGET_BLUE } else { PALE_BLUE };
                        let st = if is_selected {
                            egui::Stroke::new(2.5, BLUE)
                        } else {
                            egui::Stroke::new(1.0, BLUE)
                        };
                        (disp.to_string(), fill, st)
                    }
                    CardKind::Op => (
                        card.text.clone(),
                        HELPER_YELLOW,
                        egui::Stroke::new(1.0, Color32::from_rgb(0xd0, 0xa0, 0x20)),
                    ),
                    CardKind::Dec => {
                        let disp = if card.text.is_empty() { "□" } else { card.text.as_str() };
                        (
                            disp.to_string(),
                            HELPER_YELLOW,
                            egui::Stroke::new(1.0, Color32::from_rgb(0xd0, 0xa0, 0x20)),
                        )
                    }
                    CardKind::Out => {
                        let disp = if card.text.is_empty() { "□" } else { card.text.as_str() };
                        (disp.to_string(), PALE_BLUE, egui::Stroke::new(1.0, BLUE))
                    }
                };

                let w = (display_text.chars().count() as f32 * 9.0 + 16.0).max(36.0);
                draw_arrow_grid(ui, w, |ui| {
                    let btn = Button::new(
                        RichText::new(&display_text).size(14.0).strong().color(Color32::BLACK),
                    )
                    .fill(bg)
                    .stroke(stroke)
                    .rounding(2.0);
                    if ui.add_sized(egui::vec2(w, ARROW_CENTER_H), btn).clicked() && is_box {
                        focused = Some(card.path.clone());
                    }
                });
                ui.add_space(6.0);
                // Word boxes side-by-side (Tkinter make_component columns 2 & 3)
                let hint = if is_box { "[Item]" } else { "[Label]" };
                let (mut l, mut c) = (label.clone(), context.clone());
                let (item_w, context_w) = fit_word_widths(ui.available_width());
                let mut changed = false;
                ui.allocate_ui_with_layout(
                    egui::vec2(item_w + 6.0 + context_w, CARD_H),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let a = ui.add(
                            TextEdit::singleline(&mut l)
                                .desired_width(item_w)
                                .hint_text(hint)
                                .font(egui::TextStyle::Body),
                        );
                        ui.add_space(6.0);
                        let b = ui.add(
                            TextEdit::singleline(&mut c)
                                .desired_width(context_w)
                                .hint_text("[Context]")
                                .font(egui::TextStyle::Body),
                        );
                        changed = a.changed() || b.changed();
                    },
                );
                if changed {
                    word_edits.push((card_idx, l, c));
                }
            });
            if is_box && resp_lead.clicked() {
                focused = Some(card.path.clone());
            }
        });
    focused
}
