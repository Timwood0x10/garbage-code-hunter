//! Code Smell Radar — SVG radar chart of code quality dimensions.

use crate::analyzer::{CodeAnalyzer, CodeIssue};
use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Radar chart dimensions.
#[derive(Debug, Clone)]
pub struct RadarData {
    pub complexity: f64,
    pub duplication: f64,
    pub naming: f64,
    pub panic_risk: f64,
    pub dependency_hell: f64,
    pub legacy_smell: f64,
}

impl RadarData {
    pub fn dimensions(&self) -> [(&str, f64); 6] {
        [
            ("Complexity", self.complexity),
            ("Duplication", self.duplication),
            ("Naming", self.naming),
            ("Panic Risk", self.panic_risk),
            ("Dep Hell", self.dependency_hell),
            ("Legacy Smell", self.legacy_smell),
        ]
    }
}

/// Run radar analysis and generate SVG.
pub fn run(
    path: &Path,
    format: &OutputFormat,
    lang: &str,
    output_path: Option<&Path>,
) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);
    let data = analyze_dimensions(&issues);

    match format {
        OutputFormat::Json => Ok(display_json(&data)),
        OutputFormat::Terminal => {
            let svg = generate_svg(&data);
            if let Some(out_path) = output_path {
                std::fs::write(out_path, &svg)?;
                Ok(format!(
                    "\n  Radar chart written to {}\n",
                    out_path.display()
                ))
            } else {
                Ok(display_terminal(&data, lang))
            }
        }
    }
}

fn analyze_dimensions(issues: &[CodeIssue]) -> RadarData {
    let mut counts: HashMap<&str, f64> = HashMap::new();

    for issue in issues {
        let cat = categorize(&issue.rule_name);
        *counts.entry(cat).or_insert(0.0) += 1.0;
    }

    // Normalize to 0-100 scale (higher = worse smell)
    let normalize = |key: &str| -> f64 {
        let count = counts.get(key).copied().unwrap_or(0.0);
        (count * 5.0).min(100.0)
    };

    RadarData {
        complexity: normalize("complexity"),
        duplication: normalize("duplication"),
        naming: normalize("naming"),
        panic_risk: normalize("panic_risk"),
        dependency_hell: normalize("dependency_hell"),
        legacy_smell: normalize("legacy_smell"),
    }
}

fn categorize(rule_name: &str) -> &'static str {
    let lower = rule_name.to_lowercase();
    if lower.contains("nest") || lower.contains("complex") || lower.contains("long") {
        "complexity"
    } else if lower.contains("duplicat") || lower.contains("copy") {
        "duplication"
    } else if lower.contains("name")
        || lower.contains("single_letter")
        || lower.contains("meaningless")
    {
        "naming"
    } else if lower.contains("unwrap") {
        "panic_risk"
    } else {
        "legacy_smell"
    }
}

/// Generate SVG radar chart.
fn generate_svg(data: &RadarData) -> String {
    let dims = data.dimensions();
    let center_x: f64 = 150.0;
    let center_y: f64 = 150.0;
    let radius: f64 = 120.0;
    let n = dims.len() as f64;
    let angle_step = 2.0 * std::f64::consts::PI / n;

    let mut svg = String::new();
    svg.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"300\" height=\"340\" viewBox=\"0 0 300 340\">\n");
    svg.push_str("  <style>\n");
    svg.push_str("    text { font-family: sans-serif; font-size: 11px; fill: #333; }\n");
    svg.push_str("    .title { font-size: 14px; font-weight: bold; fill: #111; }\n");
    svg.push_str("  </style>\n");

    // Title
    svg.push_str("  <text x=\"150\" y=\"20\" text-anchor=\"middle\" class=\"title\">Code Smell Radar</text>\n");

    // Background circles
    for i in 1..=4 {
        let r = radius * i as f64 / 4.0;
        svg.push_str(&format!(
            "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\" stroke=\"#ddd\" stroke-width=\"0.5\"/>\n",
            center_x, center_y, r
        ));
    }

    // Axis lines and labels
    for (i, (label, _)) in dims.iter().enumerate() {
        let angle = angle_step * i as f64 - std::f64::consts::PI / 2.0;
        let x2 = center_x + radius * angle.cos();
        let y2 = center_y + radius * angle.sin();
        svg.push_str(&format!(
            "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ccc\" stroke-width=\"0.5\"/>\n",
            center_x, center_y, x2, y2
        ));

        // Label
        let label_r = radius + 18.0;
        let lx = center_x + label_r * angle.cos();
        let ly = center_y + label_r * angle.sin();
        let anchor = if lx < center_x - 10.0 {
            "end"
        } else if lx > center_x + 10.0 {
            "start"
        } else {
            "middle"
        };
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"{}\">{}</text>\n",
            lx, ly, anchor, label
        ));
    }

    // Data polygon
    let mut points = Vec::new();
    for (i, (_, value)) in dims.iter().enumerate() {
        let angle = angle_step * i as f64 - std::f64::consts::PI / 2.0;
        let r = radius * (value / 100.0);
        let x = center_x + r * angle.cos();
        let y = center_y + r * angle.sin();
        points.push(format!("{},{}", x, y));
    }
    svg.push_str(&format!(
        "  <polygon points=\"{}\" fill=\"rgba(255,99,71,0.25)\" stroke=\"#e74c3c\" stroke-width=\"2\"/>\n",
        points.join(" ")
    ));

    // Data points
    for (i, (_, value)) in dims.iter().enumerate() {
        let angle = angle_step * i as f64 - std::f64::consts::PI / 2.0;
        let r = radius * (value / 100.0);
        let x = center_x + r * angle.cos();
        let y = center_y + r * angle.sin();
        svg.push_str(&format!(
            "  <circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"#e74c3c\"/>\n",
            x, y
        ));
        // Value label
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"9\" fill=\"#666\">{:.0}</text>\n",
            x, y - 8.0, value
        ));
    }

    // Legend
    svg.push_str("  <text x=\"150\" y=\"330\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">Higher = worse smell</text>\n");

    svg.push_str("</svg>");
    svg
}

fn display_terminal(data: &RadarData, lang: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        t(lang, "\u{1f4e1} 代码气味雷达", "\u{1f4e1} Code Smell Radar").bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    for (label, value) in data.dimensions() {
        let bar_len = (value / 5.0) as usize;
        let bar: String = "\u{2588}".repeat(bar_len);
        let bar_colored = if value >= 70.0 {
            bar.red()
        } else if value >= 40.0 {
            bar.yellow()
        } else {
            bar.green()
        };
        out.push_str(&format!("  {:<16} {:>5.0} {}\n", label, value, bar_colored));
    }

    out.push_str(&format!(
        "\n  {}\n",
        t(
            lang,
            "使用 --output <file.svg> 生成雷达图 SVG",
            "Use --output <file.svg> to generate radar chart SVG"
        )
    ));

    out
}

fn display_json(data: &RadarData) -> String {
    serde_json::json!({
        "complexity": data.complexity,
        "duplication": data.duplication,
        "naming": data.naming,
        "panic_risk": data.panic_risk,
        "dependency_hell": data.dependency_hell,
        "legacy_smell": data.legacy_smell,
    })
    .to_string()
}

use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize() {
        assert_eq!(categorize("deep_nesting"), "complexity");
        assert_eq!(categorize("unwrap_abuse"), "panic_risk");
        assert_eq!(categorize("code_duplication"), "duplication");
    }

    #[test]
    fn test_generate_svg() {
        let data = RadarData {
            complexity: 50.0,
            duplication: 30.0,
            naming: 70.0,
            panic_risk: 20.0,
            dependency_hell: 10.0,
            legacy_smell: 40.0,
        };
        let svg = generate_svg(&data);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("polygon"));
    }

    #[test]
    fn test_display_terminal() {
        let data = RadarData {
            complexity: 50.0,
            duplication: 30.0,
            naming: 70.0,
            panic_risk: 20.0,
            dependency_hell: 10.0,
            legacy_smell: 40.0,
        };
        let out = display_terminal(&data, "en-US");
        assert!(out.contains("Code Smell Radar"));
    }

    #[test]
    fn test_display_terminal_chinese() {
        let data = RadarData {
            complexity: 50.0,
            duplication: 30.0,
            naming: 70.0,
            panic_risk: 20.0,
            dependency_hell: 10.0,
            legacy_smell: 40.0,
        };
        let out = display_terminal(&data, "zh-CN");
        assert!(out.contains("代码气味雷达"));
    }

    #[test]
    fn test_generate_svg_all_zeros() {
        let data = RadarData {
            complexity: 0.0,
            duplication: 0.0,
            naming: 0.0,
            panic_risk: 0.0,
            dependency_hell: 0.0,
            legacy_smell: 0.0,
        };
        let svg = generate_svg(&data);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_svg_all_max() {
        let data = RadarData {
            complexity: 100.0,
            duplication: 100.0,
            naming: 100.0,
            panic_risk: 100.0,
            dependency_hell: 100.0,
            legacy_smell: 100.0,
        };
        let svg = generate_svg(&data);
        assert!(svg.contains("polygon"));
    }

    #[test]
    fn test_display_json() {
        let data = RadarData {
            complexity: 50.0,
            duplication: 30.0,
            naming: 70.0,
            panic_risk: 20.0,
            dependency_hell: 10.0,
            legacy_smell: 40.0,
        };
        let json = display_json(&data);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["complexity"], 50.0);
    }
}
