//! PR Title Hunter module.
//!
//! Scans PR titles from git merge commits or GitHub API,
//! checks them against quality rules, and generates a roasting report.

pub mod report;
pub mod rules;
pub mod types;

use anyhow::{Context, Result};
use git2::Repository;
use report::{format_json, format_terminal};
use rules::check_prs;
use std::path::Path;

use types::{PrEntry, PrSource};

/// Output format for the PR title report.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Terminal,
    Json,
}

/// Run PR title analysis on a local git repository.
///
/// Extracts PR titles from merge commits and checks them.
pub fn run(repo_path: &Path, limit: usize, format: &OutputFormat) -> Result<String> {
    let prs = extract_prs_from_merges(repo_path, limit)?;

    if prs.is_empty() {
        return match format {
            OutputFormat::Terminal => Ok(format!(
                "\n{}\n\n  No merge commits found in {}\n",
                "\u{1f3af} PR Title Roast Report \u{1f3af}",
                repo_path.display()
            )),
            OutputFormat::Json => {
                Ok(r#"{"score":100,"total_prs":0,"issues":[],"stats":{"avg_title_length":0.0,"issues_found":0}}"#.to_string())
            }
        };
    }

    let issues = check_prs(&prs);

    let output = match format {
        OutputFormat::Terminal => format_terminal(&prs, &issues),
        OutputFormat::Json => format_json(&prs, &issues),
    };

    Ok(output)
}

/// Extract PR entries from merge commits in the repository.
fn extract_prs_from_merges(repo_path: &Path, limit: usize) -> Result<Vec<PrEntry>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open repo at {}", repo_path.display()))?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut prs = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Only look at merge commits (2+ parents)
        if commit.parent_count() < 2 {
            continue;
        }

        let message = commit.message().unwrap_or("");
        let title = message.lines().next().unwrap_or("").trim();

        // Skip empty merge commit messages
        if title.is_empty() {
            continue;
        }

        let short_hash = format!("{:.7}", commit.id());
        let author = commit.author().name().map(|s| s.to_string());

        prs.push(PrEntry {
            id: short_hash,
            title: title.to_string(),
            author,
            source: PrSource::Local,
        });

        if prs.len() >= limit {
            break;
        }
    }

    Ok(prs)
}

/// Analyze PRs and return structured data (for CLI integration).
pub fn analyze(repo_path: &Path, limit: usize) -> Result<(Vec<PrEntry>, Vec<types::PrIssue>)> {
    let prs = extract_prs_from_merges(repo_path, limit)?;
    let issues = check_prs(&prs);
    Ok((prs, issues))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_on_nonexistent_path() {
        let path = Path::new("/nonexistent/repo");
        let result = run(path, 10, &OutputFormat::Terminal);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_on_real_repo() {
        // Use the current repo — should have merge commits
        let path = Path::new(".");
        let result = run(path, 5, &OutputFormat::Terminal);
        // Should succeed even if no merge commits found
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_json_format() {
        let path = Path::new(".");
        let result = run(path, 5, &OutputFormat::Json);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed["score"].as_f64().is_some());
    }
}
