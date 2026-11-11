//! Desktop calculator (Rust port of the Python Tkinter app).
//!
//! Phase 1: operator rows, number pad, Operators browser, ready compute for a
//! core library. Same program as `main.py`, built toward full parity.

use rust_calculator::app::CalculatorApp;

/// egui's built-in fonts have no ∇ (nabla) and some other math symbols, so they would show as an empty box.  When the system has a symbol font it is added as a
/// fallback (Windows: Segoe UI Symbol, then Cambria); without one the program still runs.
fn add_symbol_font(ctx: &egui::Context) {
    let windows = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let mut fonts = egui::FontDefinitions::default();
    for name in ["seguisym.ttf", "cambria.ttc"] {
        if let Ok(bytes) = std::fs::read(std::path::Path::new(&windows).join("Fonts").join(name)) {
            let key = format!("symbols-{name}");
            fonts.font_data.insert(key.clone(), egui::FontData::from_owned(bytes));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(key.clone());
            }
        }
    }
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1700.0, 850.0])
            .with_min_inner_size([480.0, 320.0])
            .with_title("Smart Token Addition Calculator (Math Templates & Central Input)"),
        ..Default::default()
    };
    eframe::run_native(
        "Smart Token Addition Calculator (Math Templates & Central Input)",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            add_symbol_font(&cc.egui_ctx);
            // `--demo <operator key> <numbers...>` opens with that operator in Row 1, filled in and calculated
            let args: Vec<String> = std::env::args().skip(1).collect();
            if args.first().map(|a| a == "--demo").unwrap_or(false) && args.len() >= 2 {
                return Ok(Box::new(CalculatorApp::with_demo(&args[1], &args[2..])));
            }
            Ok(Box::new(CalculatorApp::default()))
        }),
    )
}
