//! Code Autopsy — root cause analysis of codebase problems.

use crate::analyzer::{CodeAnalyzer, CodeIssue};
use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use crate::signals::{classify_rule, StyleSignal};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

/// A contributing factor to codebase death.
#[derive(Debug, Clone)]
pub struct ContributingFactor {
    pub name: &'static str,
    pub description: String,
    pub severity: &'static str,
}

/// The autopsy report.
#[derive(Debug, Clone)]
pub struct AutopsyReport {
    pub primary_cause: String,
    pub secondary_cause: String,
    pub contributing_factors: Vec<ContributingFactor>,
    pub estimated_lifespan: String,
    pub death_certificate: String,
}

/// Run autopsy analysis on a codebase.
pub fn run(path: &Path, format: &OutputFormat, lang: &str) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);
    let report = generate_autopsy(&issues);

    let output = match format {
        OutputFormat::Terminal => display_terminal(&report, lang),
        OutputFormat::Json => display_json(&report),
    };

    Ok(output)
}

fn generate_autopsy(issues: &[CodeIssue]) -> AutopsyReport {
    let mut category_counts: HashMap<&str, usize> = HashMap::new();

    for issue in issues {
        let cat = categorize(&issue.rule_name);
        *category_counts.entry(cat).or_insert(0) += 1;
    }

    // Find primary and secondary causes
    let mut sorted: Vec<_> = category_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let primary = sorted.first().map(|(k, _)| **k).unwrap_or("Unknown");
    let secondary = sorted.get(1).map(|(k, _)| **k).unwrap_or("Unknown");

    let total = issues.len();

    // Contributing factors
    let mut factors = Vec::new();

    if total > 50 {
        factors.push(ContributingFactor {
            name: "Accumulated Technical Debt",
            description: format!("{} issues found — debt has been accumulating", total),
            severity: "Critical",
        });
    }

    // Check for deadline-driven patterns
    let quick_fix_count = issues
        .iter()
        .filter(|i| i.rule_name.to_lowercase().contains("unwrap"))
        .count();
    if quick_fix_count > 10 {
        factors.push(ContributingFactor {
            name: "Deadline Pressure",
            description: format!(
                "{} unwrap() calls suggest 'ship now, fix never' mentality",
                quick_fix_count
            ),
            severity: "High",
        });
    }

    let naming_count = issues
        .iter()
        .filter(|i| {
            let r = i.rule_name.to_lowercase();
            r.contains("name") || r.contains("single_letter")
        })
        .count();
    if naming_count > 5 {
        factors.push(ContributingFactor {
            name: "Rapid Prototyping Syndrome",
            description: format!(
                "{} naming issues suggest code was written in a hurry",
                naming_count
            ),
            severity: "Medium",
        });
    }

    if factors.is_empty() {
        factors.push(ContributingFactor {
            name: "Natural Aging",
            description: "Code naturally degrades over time without maintenance".to_string(),
            severity: "Low",
        });
    }

    // Estimate lifespan based on issue count
    let lifespan = if total > 200 {
        "3 months — code is on life support".to_string()
    } else if total > 80 {
        "6 months — symptoms are treatable".to_string()
    } else if total > 30 {
        "12 months — early intervention recommended".to_string()
    } else {
        "24+ months — prognosis is good".to_string()
    };

    // Death certificate quote
    let certificate = match primary {
        "Copy-Paste Driven Development" => {
            "The codebase was pronounced dead after 47 identical implementations \
             of the same function were found across 12 files."
                .to_string()
        }
        "Uncontrolled unwrap() Proliferation" => "Cause of death: acute panic attack. \
             The code believed everything would always work. It was wrong."
            .to_string(),
        "Pyramid of Doom" => "Cause of death: suffocation under layers of nested code. \
             The indentation level exceeded human comprehension."
            .to_string(),
        _ => {
            format!(
                "Cause of death: {} — the codebase could not withstand \
                 the accumulated weight of its own technical debt.",
                primary
            )
        }
    };

    AutopsyReport {
        primary_cause: primary.to_string(),
        secondary_cause: secondary.to_string(),
        contributing_factors: factors,
        estimated_lifespan: lifespan,
        death_certificate: certificate,
    }
}

fn categorize(rule_name: &str) -> &'static str {
    match classify_rule(rule_name) {
        StyleSignal::Duplication => "Copy-Paste Driven Development",
        StyleSignal::PanicAddiction => "Uncontrolled unwrap() Proliferation",
        StyleSignal::NamingChaos => "Naming Atrophy",
        StyleSignal::NestedHell => "Pyramid of Doom",
        StyleSignal::CodeSmells => "Magic Number Syndrome",
        StyleSignal::OverEngineering => "Function Obesity",
        StyleSignal::HotfixCulture => "Chronic Code Smell",
        StyleSignal::LegacyCode => "Cemetary of Dead Code",
        StyleSignal::TodoMountain => "Unfinished Symphony",
        StyleSignal::LineCountSmell => "File Bloat",
    }
}

fn display_terminal(report: &AutopsyReport, lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        t(
            lang,
            "\u{2620}\u{fe0f} 代码尸检报告",
            "\u{2620}\u{fe0f} Code Autopsy Report"
        )
        .bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    out.push_str(&format!(
        "{}\n",
        t(
            lang,
            "\u{1f480} 代码库死亡的根本原因",
            "\u{1f480} Root Cause of Codebase Death"
        )
        .bold()
    ));
    out.push_str(&format!(
        "  {}\n    {}\n\n",
        t(lang, "主要原因：", "Primary Cause:"),
        report.primary_cause.red().bold()
    ));
    out.push_str(&format!(
        "  {}\n    {}\n\n",
        t(lang, "次要原因：", "Secondary Cause:"),
        report.secondary_cause.yellow()
    ));

    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f4a7} 促成因素", "\u{1f4a7} Contributing Factors").bold()
    ));
    for factor in &report.contributing_factors {
        let sev = match factor.severity {
            "Critical" => factor.severity.red().bold(),
            "High" => factor.severity.red(),
            "Medium" => factor.severity.yellow(),
            _ => factor.severity.green(),
        };
        out.push_str(&format!(
            "  \u{2022} [{}] {}: {}\n",
            sev,
            factor.name.bold(),
            factor.description
        ));
    }
    out.push('\n');

    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f52e} 预估寿命", "\u{1f52e} Estimated Lifespan").bold()
    ));
    out.push_str(&format!("  {}\n\n", report.estimated_lifespan));

    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f4dc} 死亡证明", "\u{1f4dc} Death Certificate").bold()
    ));
    out.push_str(&format!("  \"{}\"\n", report.death_certificate.italic()));

    out
}

fn display_json(report: &AutopsyReport) -> String {
    serde_json::json!({
        "primary_cause": report.primary_cause,
        "secondary_cause": report.secondary_cause,
        "contributing_factors": report.contributing_factors.iter().map(|f| {
            serde_json::json!({
                "name": f.name,
                "description": f.description,
                "severity": f.severity,
            })
        }).collect::<Vec<_>>(),
        "estimated_lifespan": report.estimated_lifespan,
        "death_certificate": report.death_certificate,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize() {
        assert_eq!(
            categorize("unwrap-abuse"),
            "Uncontrolled unwrap() Proliferation"
        );
        assert_eq!(categorize("deep-nesting"), "Pyramid of Doom");
        assert_eq!(
            categorize("code-duplication"),
            "Copy-Paste Driven Development"
        );
    }

    #[test]
    fn test_run_on_current_dir() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "en-US");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_on_current_dir_chinese() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "zh-CN");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_json_format() {
        let result = run(std::path::Path::new("."), &OutputFormat::Json, "en-US");
        assert!(result.is_ok());
        let json = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["primary_cause"].is_string());
    }
}
