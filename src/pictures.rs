//! The pictures that live beside the Python files: an operator's formula picture
//! (`operators_one_operator_per_file/<key>/<name>.jpg`) and a word problem's diagram
//! (`operators_one_algebra_word_problem_per_file/<category>/<id>/<id>.jpg`).
//! They are read from disk when shown (58 MB of JPGs are not built into the program).

use std::path::PathBuf;
use std::sync::OnceLock;

const OPS_DIR: &str = "operators_one_operator_per_file";
const WORD_DIR: &str = "operators_one_algebra_word_problem_per_file";

/// The project folder (the one holding `operators_one_operator_per_file`), found from the current folder or the program's own folder; the
/// `CALCULATOR_ROOT` environment variable overrides it.
pub fn root() -> Option<&'static PathBuf> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        if let Ok(r) = std::env::var("CALCULATOR_ROOT") {
            return Some(PathBuf::from(r));
        }
        let starts = [std::env::current_dir().ok(), std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()))];
        for start in starts.into_iter().flatten() {
            let mut dir = Some(start.as_path());
            while let Some(d) = dir {
                if d.join(OPS_DIR).is_dir() {
                    return Some(d.to_path_buf());
                }
                dir = d.parent();
            }
        }
        None
    })
    .as_ref()
}

fn existing(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

/// The formula picture of an operator key such as `geometry/geometry/arc_length`.
pub fn operator_picture(key: &str) -> Option<PathBuf> {
    let last = key.rsplit('/').next()?;
    existing(root()?.join(OPS_DIR).join(key).join(format!("{last}.jpg")))
}

/// The diagram of a word problem key such as `age/0001` (also given as `word:age/0001`).
pub fn word_picture(key: &str) -> Option<PathBuf> {
    let key = key.strip_prefix("word:").unwrap_or(key);
    let last = key.rsplit('/').next()?;
    existing(root()?.join(WORD_DIR).join(key).join(format!("{last}.jpg")))
}

/// The picture for any key an operator row can hold.
pub fn picture_for(key: &str) -> Option<PathBuf> {
    if key.starts_with("word:") {
        word_picture(key)
    } else {
        operator_picture(key)
    }
}

/// `file://` address that egui's file loader reads.
pub fn uri(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_operator_and_a_word_problem_have_their_pictures() {
        if root().is_none() {
            return; // built outside the project folder
        }
        assert!(operator_picture("geometry/geometry/arc_length").is_some());
        assert!(word_picture("age/0001").is_some());
        assert!(picture_for("word:age/0001").is_some());
    }

    #[test]
    fn every_operator_picture_opens_and_decodes() {
        if root().is_none() {
            return; // built outside the project folder
        }
        let mut count = 0;
        for layout in crate::layouts::LAYOUTS {
            let path = operator_picture(layout.key).unwrap_or_else(|| panic!("no picture for {}", layout.key));
            let img = image::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(img.width() > 100 && img.height() > 100, "{} is {}x{}", layout.key, img.width(), img.height());
            count += 1;
        }
        assert_eq!(count, 631);
    }
}
