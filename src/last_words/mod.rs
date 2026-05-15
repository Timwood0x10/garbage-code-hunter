//! Code Last Words — find and report legacy TODO/FIXME/HACK comments.

pub mod display;
pub mod scanner;

use crate::common::OutputFormat;
use anyhow::Result;
use std::path::Path;

/// Run the last-words scan on a path.
pub fn run(path: &Path, format: &OutputFormat, with_age: bool, lang: &str) -> Result<String> {
    let mut words = scanner::scan(path);

    // Optionally try to get ages via git blame (slower)
    if with_age {
        for word in &mut words {
            word.age_days = scanner::try_get_age(&word.file, word.line);
        }
    }

    let output = match format {
        OutputFormat::Terminal => display::format_terminal(&words, lang),
        OutputFormat::Json => display::format_json(&words),
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_on_temp_dir() {
        let dir = std::env::temp_dir().join("gch_lw_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// TODO: test\nfn main() {}\n").unwrap();

        let result = run(&dir, &OutputFormat::Terminal, false, "en-US");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("TODO"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
