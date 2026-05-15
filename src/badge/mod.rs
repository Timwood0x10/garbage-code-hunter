//! Badge generation module.
//!
//! Generates SVG score badges for embedding in README files.

pub mod generator;

use anyhow::Result;
use generator::{generate_svg, BadgeStyle};
use std::fs;
use std::path::Path;

/// Run badge generation with a pre-computed score.
pub fn run(score: f64, output_path: &Path, style: &BadgeStyle) -> Result<String> {
    let svg = generate_svg(score, style);
    fs::write(output_path, &svg)?;
    Ok(format!(
        "Badge written to {} (score: {:.0}/100)",
        output_path.display(),
        score
    ))
}

/// Generate SVG string without writing to file.
pub fn render(score: f64, style: &BadgeStyle) -> String {
    generate_svg(score, style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_writes_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("badge.svg");
        let result = run(85.0, &path, &BadgeStyle::Flat);
        assert!(result.is_ok());
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("85"));
    }

    #[test]
    fn test_render_returns_svg() {
        let svg = render(72.0, &BadgeStyle::Flat);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_render_plastic_style() {
        let svg = render(90.0, &BadgeStyle::Plastic);
        assert!(svg.contains("90"));
    }
}
