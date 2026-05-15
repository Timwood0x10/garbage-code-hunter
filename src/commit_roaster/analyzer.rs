//! Git commit analyzer using libgit2.
//!
//! Reads commit history from a git repository and collects
//! commit metadata for rule evaluation.

use anyhow::{Context, Result};
use git2::{Repository, Sort};
use std::path::Path;

/// Parsed commit information extracted from git history.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

/// Configuration for filtering which commits to analyze.
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Maximum number of commits to analyze. None = all.
    pub limit: Option<usize>,
    /// Only commits by this author.
    pub author: Option<String>,
    /// Only commits after this timestamp.
    pub since: Option<i64>,
    /// Only commits before this timestamp.
    pub until: Option<i64>,
    /// Branch name to analyze. None = HEAD.
    pub branch: Option<String>,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            limit: Some(50),
            author: None,
            since: None,
            until: None,
            branch: None,
        }
    }
}

/// Open a git repository and extract commit history.
pub fn analyze_repo(repo_path: &Path, config: &AnalyzerConfig) -> Result<Vec<CommitInfo>> {
    let repo = Repository::open(repo_path)
        .with_context(|| format!("Failed to open git repo at {}", repo_path.display()))?;

    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;

    revwalk
        .set_sorting(Sort::TIME)
        .context("Failed to set sorting")?;

    // Determine starting point
    if let Some(ref branch) = config.branch {
        let refname = format!("refs/heads/{}", branch);
        let reference = repo
            .find_reference(&refname)
            .with_context(|| format!("Branch '{}' not found", branch))?;
        revwalk
            .push(reference.peel_to_commit()?.id())
            .context("Failed to push branch tip")?;
    } else {
        revwalk.push_head().context("Failed to push HEAD")?;
    }

    let mut commits = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result.context("Failed to read commit OID")?;
        let commit = repo.find_commit(oid).context("Failed to find commit")?;

        // Apply author filter
        if let Some(ref author_filter) = config.author {
            let author_name = commit.author().name().unwrap_or("").to_string();
            let author_email = commit.author().email().unwrap_or("").to_string();
            if !author_name.contains(author_filter.as_str())
                && !author_email.contains(author_filter.as_str())
            {
                continue;
            }
        }

        // Apply time filters
        let timestamp = commit.time().seconds();
        if let Some(since) = config.since {
            if timestamp < since {
                continue;
            }
        }
        if let Some(until) = config.until {
            if timestamp > until {
                continue;
            }
        }

        let message = commit.message().unwrap_or("").to_string();

        let hash = oid.to_string();
        let short_hash = oid.to_string()[..7].to_string();

        let author = commit.author().name().unwrap_or("unknown").to_string();

        // Count file changes via diff
        let (files_changed, insertions, deletions) =
            count_changes(&repo, &commit).unwrap_or((0, 0, 0));

        commits.push(CommitInfo {
            hash,
            short_hash,
            author,
            message,
            timestamp,
            files_changed,
            insertions,
            deletions,
        });

        // Apply limit
        if let Some(limit) = config.limit {
            if commits.len() >= limit {
                break;
            }
        }
    }

    Ok(commits)
}

/// Count file changes, insertions, and deletions for a commit.
fn count_changes(repo: &Repository, commit: &git2::Commit) -> Result<(usize, usize, usize)> {
    let tree = commit.tree().context("Failed to get commit tree")?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(
            commit
                .parent(0)
                .context("Failed to get parent commit")?
                .tree()
                .context("Failed to get parent tree")?,
        )
    } else {
        None
    };

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
        .context("Failed to compute diff")?;

    let stats = diff.stats().context("Failed to compute diff stats")?;

    Ok((stats.files_changed(), stats.insertions(), stats.deletions()))
}

/// Truncate commit message to first line only.
pub fn first_line(message: &str) -> &str {
    message.lines().next().unwrap_or("").trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_line_single_line() {
        assert_eq!(first_line("fix: something"), "fix: something");
    }

    #[test]
    fn test_first_line_multi_line() {
        let msg = "fix: something\n\nThis is a longer description\nwith multiple lines.";
        assert_eq!(first_line(msg), "fix: something");
    }

    #[test]
    fn test_first_line_empty() {
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_first_line_whitespace() {
        assert_eq!(first_line("  fix: something  "), "fix: something");
    }

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert_eq!(config.limit, Some(50));
        assert!(config.author.is_none());
        assert!(config.since.is_none());
        assert!(config.until.is_none());
        assert!(config.branch.is_none());
    }
}
