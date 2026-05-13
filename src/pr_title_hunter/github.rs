//! GitHub API client for fetching PR titles from remote repositories.

use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use super::types::{PrEntry, PrSource};

/// Configuration for GitHub API access.
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    /// Repository in "owner/repo" format.
    pub repo: String,
    /// PR state filter: "open", "closed", or "all".
    pub state: String,
    /// Max PRs to fetch.
    pub limit: usize,
    /// Optional GitHub token for authentication.
    pub token: Option<String>,
    /// Optional author filter (login name).
    pub author: Option<String>,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            repo: String::new(),
            state: "all".to_string(),
            limit: 50,
            token: None,
            author: None,
        }
    }
}

/// GitHub PR response (subset of fields we need).
#[derive(Debug, Deserialize)]
struct GhPullRequest {
    number: u64,
    title: String,
    user: GhUser,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

/// Fetch PRs from GitHub API and convert to PrEntry list.
pub fn fetch_prs(config: &GitHubConfig) -> Result<Vec<PrEntry>> {
    if config.repo.is_empty() || !config.repo.contains('/') {
        bail!(
            "Invalid repo format '{}', expected 'owner/repo'",
            config.repo
        );
    }

    let url = format!(
        "https://api.github.com/repos/{}/pulls?state={}&per_page={}&sort=created&direction=desc",
        config.repo,
        config.state,
        config.limit.min(100),
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("garbage-code-hunter"));

    if let Some(token) = &config.token {
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).context("Invalid GitHub token")?,
        );
    }

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to fetch PRs from {}", config.repo))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        bail!("GitHub API error {}: {}", status, body);
    }

    let prs: Vec<GhPullRequest> = response
        .json()
        .context("Failed to parse GitHub API response")?;

    let entries: Vec<PrEntry> = prs
        .into_iter()
        .filter(|pr| {
            // Filter by author if specified
            if let Some(author_filter) = &config.author {
                pr.user.login.eq_ignore_ascii_case(author_filter)
            } else {
                true
            }
        })
        .map(|pr| PrEntry {
            id: pr.number.to_string(),
            title: pr.title,
            author: Some(pr.user.login),
            source: PrSource::GitHub {
                repo: config.repo.clone(),
            },
        })
        .collect();

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_default() {
        let config = GitHubConfig::default();
        assert_eq!(config.state, "all");
        assert_eq!(config.limit, 50);
        assert!(config.token.is_none());
    }

    #[test]
    fn test_fetch_prs_empty_repo() {
        let config = GitHubConfig {
            repo: String::new(),
            ..Default::default()
        };
        let result = fetch_prs(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_prs_invalid_repo() {
        let config = GitHubConfig {
            repo: "not-a-valid-repo".to_string(),
            ..Default::default()
        };
        let result = fetch_prs(&config);
        assert!(result.is_err());
    }
}
