// Rust calculator GUI: eframe/egui native application.
//
// Matches the visual and structural layout of the Python Tkinter calculator:
// same window size (1380x920), colors, header, rows canvas, operations bar,
// descriptive columns console, and keypad.

use eframe::egui;
use rust_calculator::app::CalculatorApp;
use rust_calculator::row::Join;

fn add_symbol_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // egui doesn't ship math symbols (√, ×, ÷, ≤, ≥, etc.) in its default font.
    // Try to load Segoe UI Symbol (standard on Windows) and append it as a fallback.
    let candidates = [
        "C:\\Windows\\Fonts\\seguisym.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    for path in &candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "symbol_fallback".to_owned(),
                egui::FontData::from_owned(bytes),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push("symbol_fallback".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1380.0, 920.0])
            .with_title("Smart Token Addition Calculator (Math Templates & Central Input)"),
        ..Default::default()
    };
    eframe::run_native(
        "Smart Token Addition Calculator (Math Templates & Central Input)",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            add_symbol_font(&cc.egui_ctx);
            let args: Vec<String> = std::env::args().skip(1).collect();
            let mut screenshot_target = None;
            if let Some(pos) = args.iter().position(|a| a == "--screenshot") {
                if let Some(target) = args.get(pos + 1) {
                    screenshot_target = Some(std::path::PathBuf::from(target));
                }
            }
            let mut demo_config = None;
            if let Some(demo_pos) = args.iter().position(|a| a == "--demo") {
                let demo_args: Vec<String> = args.iter().skip(demo_pos + 1).take_while(|a| !a.starts_with("--")).cloned().collect();
                if demo_args.len() >= 2 {
                    demo_config = Some((demo_args[0].clone(), demo_args[1..].to_vec()));
                }
            }
            let mut scenario = None;
            if let Some(sc_pos) = args.iter().position(|a| a == "--scenario") {
                if let Some(name) = args.get(sc_pos + 1) {
                    scenario = Some(name.clone());
                }
            }

            let mut app = if let Some(sc) = scenario {
                let mut app = CalculatorApp::default();
                match sc.as_str() {
                    "3_rows_aligned" => {
                        // Row 1: 10
                        app.doc.rows[0].inputs = vec!["10".to_string()];
                        // Row 2: Join +, input 20
                        app.doc.rows[1].join = Join::Add;
                        app.doc.rows[1].inputs = vec!["20".to_string()];
                        // Row 3: Next Row, Join *, input 5
                        app.doc.target_row = 1;
                        app.doc.next_row();
                        app.doc.rows[2].join = Join::Mul;
                        app.doc.rows[2].inputs = vec!["5".to_string()];
                        app.doc.target_row = 2;
                    }
                    "row1_expanded_parens" => {
                        // Row 1: Expand to 3 paren pairs
                        app.doc.rows[0].inputs = vec!["2".to_string()];
                        app.doc.rows[0].add_paren_pair();
                        app.doc.rows[0].add_paren_pair();
                        app.doc.rows[0].toggle_paren_open(0);
                        app.doc.rows[0].toggle_paren_open(1);
                        app.doc.rows[0].exponent_1 = "3".to_string();
                        app.doc.rows[0].exponent_2_boxes[0] = "2".to_string();
                        // Row 2: Join +, baseline unexpanded (1 pair)
                        app.doc.rows[1].join = Join::Add;
                        app.doc.rows[1].inputs = vec!["4".to_string()];
                    }
                    "expanded_descriptions" => {
                        // Row 1: multiple math descriptions
                        app.doc.rows[0].inputs = vec!["50".to_string()];
                        app.doc.rows[0].math_descriptions[0] = "Base Length".to_string();
                        app.doc.rows[0].add_math_description();
                        app.doc.rows[0].math_descriptions[1] = "in meters".to_string();
                        app.doc.rows[0].result_descriptions[0] = "Area".to_string();
                        app.doc.rows[0].add_result_description();
                        app.doc.rows[0].result_descriptions[1] = "sq meters".to_string();
                        // Row 2: Join +, single description
                        app.doc.rows[1].join = Join::Add;
                        app.doc.rows[1].inputs = vec!["75".to_string()];
                        app.doc.rows[1].math_descriptions[0] = "Height".to_string();
                        app.doc.rows[1].result_descriptions[0] = "Volume".to_string();
                    }
                    _ => {}
                }
                app
            } else if let Some((op, nums)) = demo_config {
                CalculatorApp::with_demo(&op, &nums)
            } else {
                CalculatorApp::default()
            };
            if let Some(target) = screenshot_target {
                app.screenshot_path = Some(target);
            }
            Ok(Box::new(app))
        }),
    )
}
