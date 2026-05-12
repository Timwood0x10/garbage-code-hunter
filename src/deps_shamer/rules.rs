//! Dependency analysis rules.
//!
//! Each rule checks for a specific anti-pattern in dependency management.

use super::types::{DepFile, DepIssue, DepSeverity, DepSource, Ecosystem};

/// A rule that checks dependency files for issues.
pub trait DepRule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue>;
}

/// Too many dependencies.
pub struct TooManyDepsRule {
    pub threshold: usize,
}

impl DepRule for TooManyDepsRule {
    fn id(&self) -> &str {
        "too-many-deps"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        let total = dep_file.dependencies.len();
        if total > self.threshold {
            vec![DepIssue {
                rule_id: self.id().to_string(),
                severity: DepSeverity::Medium,
                message: format!(
                    "{} dependencies? Are you building a supermarket or a software project?",
                    total
                ),
                dep_name: None,
            }]
        } else {
            vec![]
        }
    }
}

/// Git-based dependencies.
pub struct GitDepsRule;

impl DepRule for GitDepsRule {
    fn id(&self) -> &str {
        "git-deps"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        dep_file
            .dependencies
            .iter()
            .filter_map(|dep| {
                if let DepSource::Git { url } = &dep.source {
                    Some(DepIssue {
                        rule_id: self.id().to_string(),
                        severity: DepSeverity::Medium,
                        message: format!(
                            "Directly referencing git repo '{}' — don't trust package managers?",
                            url
                        ),
                        dep_name: Some(dep.name.clone()),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Wildcard or star version.
pub struct WildcardVersionRule;

impl DepRule for WildcardVersionRule {
    fn id(&self) -> &str {
        "wildcard-version"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        dep_file
            .dependencies
            .iter()
            .filter_map(|dep| {
                let v = dep.version.trim();
                if v == "*" || v == ">=0" || v == "" {
                    Some(DepIssue {
                        rule_id: self.id().to_string(),
                        severity: DepSeverity::High,
                        message: format!(
                            "Version '{}' for '{}' — enjoy your daily breaking changes?",
                            v, dep.name
                        ),
                        dep_name: Some(dep.name.clone()),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Pre-release / alpha / beta version in non-dev dependency.
pub struct PreReleaseRule;

impl DepRule for PreReleaseRule {
    fn id(&self) -> &str {
        "pre-release"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        dep_file
            .dependencies
            .iter()
            .filter(|dep| !dep.is_dev)
            .filter_map(|dep| {
                let v = dep.version.to_lowercase();
                if v.contains("alpha")
                    || v.contains("beta")
                    || v.contains("rc")
                    || v.contains("pre")
                    || v.contains("snapshot")
                {
                    Some(DepIssue {
                        rule_id: self.id().to_string(),
                        severity: DepSeverity::Medium,
                        message: format!(
                            "Production dependency '{}' uses pre-release version '{}' — brave!",
                            dep.name, dep.version
                        ),
                        dep_name: Some(dep.name.clone()),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Deprecated or unmaintained dependency patterns.
pub struct DeprecatedDepRule {
    pub ecosystem: Ecosystem,
}

impl DepRule for DeprecatedDepRule {
    fn id(&self) -> &str {
        "deprecated-dep"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        let deprecated = match &self.ecosystem {
            Ecosystem::Rust => vec![
                "failure",
                "iron",
                "nickel",
                "rustc-serialize",
                "quickcheck",
                "tempdir",
                "toml_query",
            ],
            Ecosystem::Node => vec![
                "request",
                "bower",
                "node-uuid",
                "nomnom",
                "optimist",
                "colors",
                "left-pad",
            ],
            _ => vec![],
        };

        dep_file
            .dependencies
            .iter()
            .filter_map(|dep| {
                if deprecated.contains(&dep.name.as_str()) {
                    Some(DepIssue {
                        rule_id: self.id().to_string(),
                        severity: DepSeverity::High,
                        message: format!(
                            "'{}' is deprecated — are you an archaeologist?",
                            dep.name
                        ),
                        dep_name: Some(dep.name.clone()),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Duplicate dependencies (same name appearing multiple times).
pub struct DuplicatedDepRule;

impl DepRule for DuplicatedDepRule {
    fn id(&self) -> &str {
        "duplicated-dep"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();

        for dep in &dep_file.dependencies {
            if !seen.insert(&dep.name) {
                duplicates.push(DepIssue {
                    rule_id: self.id().to_string(),
                    severity: DepSeverity::Medium,
                    message: format!(
                        "'{}' appears more than once — Ctrl+C and Ctrl+V are working overtime?",
                        dep.name
                    ),
                    dep_name: Some(dep.name.clone()),
                });
            }
        }

        duplicates
    }
}

/// Too many dev dependencies.
pub struct TooManyDevDepsRule {
    pub threshold: usize,
}

impl DepRule for TooManyDevDepsRule {
    fn id(&self) -> &str {
        "too-many-dev-deps"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        let dev_count = dep_file.dependencies.iter().filter(|d| d.is_dev).count();
        if dev_count > self.threshold {
            vec![DepIssue {
                rule_id: self.id().to_string(),
                severity: DepSeverity::Low,
                message: format!(
                    "{} dev dependencies — your test setup is heavier than the app itself?",
                    dev_count
                ),
                dep_name: None,
            }]
        } else {
            vec![]
        }
    }
}

/// High ratio of optional dependencies.
pub struct TooManyOptionalRule {
    pub ratio_threshold: f64,
}

impl DepRule for TooManyOptionalRule {
    fn id(&self) -> &str {
        "too-many-optional"
    }

    fn check(&self, dep_file: &DepFile) -> Vec<DepIssue> {
        let total = dep_file.dependencies.len();
        if total == 0 {
            return vec![];
        }
        let optional_count = dep_file
            .dependencies
            .iter()
            .filter(|d| d.is_optional)
            .count();
        let ratio = optional_count as f64 / total as f64;
        if ratio > self.ratio_threshold {
            vec![DepIssue {
                rule_id: self.id().to_string(),
                severity: DepSeverity::Low,
                message: format!(
                    "{}% of dependencies are optional ({}/{}) — are you sure what you actually need?",
                    (ratio * 100.0) as usize,
                    optional_count,
                    total
                ),
                dep_name: None,
            }]
        } else {
            vec![]
        }
    }
}

/// Get all default rules for a given ecosystem.
pub fn default_rules(ecosystem: &Ecosystem) -> Vec<Box<dyn DepRule>> {
    vec![
        Box::new(TooManyDepsRule { threshold: 50 }),
        Box::new(GitDepsRule),
        Box::new(WildcardVersionRule),
        Box::new(PreReleaseRule),
        Box::new(DeprecatedDepRule {
            ecosystem: ecosystem.clone(),
        }),
        Box::new(DuplicatedDepRule),
        Box::new(TooManyDevDepsRule { threshold: 20 }),
        Box::new(TooManyOptionalRule {
            ratio_threshold: 0.5,
        }),
    ]
}

/// Apply all rules to a dependency file and return issues.
pub fn check_dep_file(dep_file: &DepFile) -> Vec<DepIssue> {
    let rules = default_rules(&dep_file.ecosystem);
    rules.iter().flat_map(|rule| rule.check(dep_file)).collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::Dependency;
    use super::*;

    fn make_dep(name: &str, version: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            source: DepSource::Registry,
            is_dev: false,
            is_optional: false,
        }
    }

    fn make_dep_file(deps: Vec<Dependency>) -> DepFile {
        DepFile {
            path: "test.toml".to_string(),
            ecosystem: Ecosystem::Rust,
            dependencies: deps,
        }
    }

    #[test]
    fn test_too_many_deps_triggers() {
        let rule = TooManyDepsRule { threshold: 5 };
        let deps: Vec<Dependency> = (0..10)
            .map(|i| make_dep(&format!("dep{}", i), "1.0"))
            .collect();
        let dep_file = make_dep_file(deps);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("10"));
    }

    #[test]
    fn test_too_many_deps_no_trigger() {
        let rule = TooManyDepsRule { threshold: 50 };
        let deps = vec![make_dep("serde", "1.0")];
        let dep_file = make_dep_file(deps);
        let issues = rule.check(&dep_file);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_git_deps_detected() {
        let rule = GitDepsRule;
        let dep_file = make_dep_file(vec![
            make_dep("serde", "1.0"),
            Dependency {
                name: "my-lib".to_string(),
                version: "main".to_string(),
                source: DepSource::Git {
                    url: "https://github.com/foo/bar".to_string(),
                },
                is_dev: false,
                is_optional: false,
            },
        ]);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].dep_name.as_deref() == Some("my-lib"));
    }

    #[test]
    fn test_wildcard_version_detected() {
        let rule = WildcardVersionRule;
        let dep_file = make_dep_file(vec![
            make_dep("ok-dep", "1.0"),
            make_dep("bad-dep", "*"),
            make_dep("also-bad", ">=0"),
        ]);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_pre_release_detected() {
        let rule = PreReleaseRule;
        let dep_file = make_dep_file(vec![
            make_dep("stable", "1.0"),
            make_dep("beta-pkg", "2.0.0-beta.1"),
            make_dep("alpha-pkg", "1.0.0-alpha"),
        ]);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_pre_release_ignores_dev_deps() {
        let rule = PreReleaseRule;
        let mut dep = make_dep("dev-beta", "1.0.0-beta");
        dep.is_dev = true;
        let dep_file = make_dep_file(vec![dep]);
        let issues = rule.check(&dep_file);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_deprecated_dep_rust() {
        let rule = DeprecatedDepRule {
            ecosystem: Ecosystem::Rust,
        };
        let dep_file = make_dep_file(vec![make_dep("serde", "1.0"), make_dep("failure", "0.1")]);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].dep_name.as_deref() == Some("failure"));
    }

    #[test]
    fn test_duplicated_dep_detected() {
        let rule = DuplicatedDepRule;
        let dep_file = make_dep_file(vec![make_dep("serde", "1.0"), make_dep("serde", "1.1")]);
        let issues = rule.check(&dep_file);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_check_dep_file_integration() {
        let deps = vec![
            make_dep("serde", "1.0"),
            make_dep("tokio", "*"),
            make_dep("failure", "0.1"),
        ];
        let dep_file = make_dep_file(deps);
        let issues = check_dep_file(&dep_file);
        // Should catch wildcard (tokio) and deprecated (failure)
        assert!(issues.len() >= 2);
        assert!(issues.iter().any(|i| i.rule_id == "wildcard-version"));
        assert!(issues.iter().any(|i| i.rule_id == "deprecated-dep"));
    }
}
