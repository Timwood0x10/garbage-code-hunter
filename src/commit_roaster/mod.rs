//! Commit Roaster module.
//!
//! Scans git history and roasts bad commit messages with
//! sharp but humorous feedback.

pub mod analyzer;
pub mod report;
pub mod rules;

use analyzer::AnalyzerConfig;
use anyhow::Result;
use report::ScoredCommit;
use rules::{match_rules, RuleSet};
use std::path::Path;

/// Output format for the commit-roaster report.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Terminal,
    Json,
}

/// Run the commit-roaster analysis on a repository.
pub fn run(repo_path: &Path, config: &AnalyzerConfig, format: &OutputFormat) -> Result<String> {
    // Load default rules
    let ruleset = RuleSet::from_toml(rules::default_rules_toml())?;
    let active_rules = ruleset.active_rules();

    // Analyze commits
    let commits = analyzer::analyze_repo(repo_path, config)?;

    // Score each commit
    let scored_commits: Vec<ScoredCommit> = commits
        .into_iter()
        .map(|commit| {
            let first = analyzer::first_line(&commit.message);
            let issues = match_rules(first, &active_rules);
            ScoredCommit { commit, issues }
        })
        .collect();

    // Build report
    let report = report::build_report(scored_commits);

    // Format output
    match format {
        OutputFormat::Terminal => Ok(report::format_terminal(&report)),
        OutputFormat::Json => Ok(report::format_json(&report)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_run_on_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/repo");
        let config = AnalyzerConfig::default();
        let result = run(&path, &config, &OutputFormat::Terminal);
        assert!(result.is_err(), "Should fail on nonexistent path");
    }
}
