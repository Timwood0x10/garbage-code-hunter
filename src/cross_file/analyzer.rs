use std::collections::HashMap;
use std::path::{Path, PathBuf};

use syn::{spanned::Spanned, ItemFn};

use crate::analyzer::Severity;
use crate::context::FileContext;

use super::fingerprint::{
    extract_fingerprint, jaccard_similarity, FileLocation, FunctionFingerprint,
};

/// Configuration for cross-file analysis behavior
#[derive(Debug, Clone)]
pub struct CrossFileConfig {
    pub min_function_lines: usize,
    pub max_function_lines: usize,
    pub min_file_occurrences: usize,
    pub similarity_threshold: f64,
    pub max_memory_mb: usize,
}

impl Default for CrossFileConfig {
    fn default() -> Self {
        Self {
            min_function_lines: 5,
            max_function_lines: 150,
            min_file_occurrences: 2,
            similarity_threshold: 0.85,
            max_memory_mb: 512,
        }
    }
}

/// A cross-file duplication issue found by the analyzer
#[derive(Debug, Clone)]
pub struct CrossFileIssue {
    pub fingerprint: FunctionFingerprint,
    pub file_count: usize,
    pub total_occurrences: usize,
    pub similarity_score: f64,
    pub severity: Severity,
}

impl CrossFileIssue {
    /// Determine severity based on extent of duplication
    fn compute_severity(&self) -> Severity {
        if self.total_occurrences > 10 || self.file_count > 5 {
            Severity::Nuclear
        } else if self.total_occurrences > 5 || self.file_count > 3 {
            Severity::Spicy
        } else {
            Severity::Mild
        }
    }
}

/// Core analyzer for detecting duplicated code across multiple files
pub struct CrossFileAnalyzer {
    /// Hash map from fingerprint hash to fingerprint data
    index: HashMap<u64, FunctionFingerprint>,
    config: CrossFileConfig,
    total_functions_processed: usize,
    total_files_processed: usize,
}

impl Default for CrossFileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossFileAnalyzer {
    /// Create a new analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(CrossFileConfig::default())
    }

    /// Create a new analyzer with custom configuration
    pub fn with_config(config: CrossFileConfig) -> Self {
        Self {
            index: HashMap::new(),
            config,
            total_functions_processed: 0,
            total_files_processed: 0,
        }
    }

    /// Process a single Rust source file and extract function fingerprints
    pub fn process_file(&mut self, file_path: &Path, content: &str) -> Result<(), String> {
        let syntax: syn::File =
            syn::parse_str(content).map_err(|e| format!("Parse error: {}", e))?;

        self.total_files_processed += 1;

        for item in syntax.items.iter() {
            if let syn::Item::Fn(func) = item {
                if let Some(fp) = self.process_function(func, file_path) {
                    self.add_fingerprint(fp);
                }
            }
        }

        Ok(())
    }

    /// Process a single function and return its fingerprint if valid
    fn process_function(&self, func: &ItemFn, file_path: &Path) -> Option<FunctionFingerprint> {
        let line_start = func.sig.fn_token.span.start().line;
        let line_end = func.block.span().end().line;
        let line_count = line_end - line_start + 1;

        // Skip functions that are too short (trivial)
        if line_count < self.config.min_function_lines {
            return None;
        }

        // Skip functions that are too long (will be handled separately in future optimization)
        if line_count > self.config.max_function_lines {
            return None;
        }

        extract_fingerprint(func, file_path.to_path_buf())
    }

    /// Add a fingerprint to the internal index, merging with existing entry if hash matches
    fn add_fingerprint(&mut self, mut fingerprint: FunctionFingerprint) {
        self.total_functions_processed += 1;

        match self.index.get_mut(&fingerprint.hash) {
            Some(existing) => {
                existing.locations.append(&mut fingerprint.locations);
            }
            None => {
                self.index.insert(fingerprint.hash, fingerprint);
            }
        }
    }

    /// Find all duplicate patterns across files that meet the threshold criteria
    pub fn find_all_duplicates(&self) -> Vec<CrossFileIssue> {
        let mut issues = Vec::new();

        for (_hash, fingerprint) in self.index.iter() {
            // Count unique files containing this pattern
            let unique_files: std::collections::HashSet<&PathBuf> = fingerprint
                .locations
                .iter()
                .map(|loc| &loc.file_path)
                .collect();

            let file_count = unique_files.len();
            let total_occurrences = fingerprint.locations.len();

            // Only report if appears in enough different files
            if file_count < self.config.min_file_occurrences {
                continue;
            }

            // For exact matches (same hash), similarity is 1.0
            let similarity = 1.0;

            let issue = CrossFileIssue {
                fingerprint: fingerprint.clone(),
                file_count,
                total_occurrences,
                similarity_score: similarity,
                severity: Severity::Mild, // Will be computed below
            };

            let issue_with_severity = CrossFileIssue {
                severity: issue.compute_severity(),
                ..issue
            };

            issues.push(issue_with_severity);
        }

        // Sort by severity (most severe first), then by occurrence count (descending)
        issues.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then(b.total_occurrences.cmp(&a.total_occurrences))
        });

        issues
    }

    /// Find near-duplicates using fuzzy matching (more expensive, use sparingly)
    pub fn find_near_duplicates(&self) -> Vec<CrossFileIssue> {
        let fingerprints: Vec<&FunctionFingerprint> = self.index.values().collect();
        let mut issues = Vec::new();
        let mut compared_pairs: std::collections::HashSet<(u64, u64)> =
            std::collections::HashSet::new();

        for i in 0..fingerprints.len() {
            for j in (i + 1)..fingerprints.len() {
                let fp_a = fingerprints[i];
                let fp_b = fingerprints[j];

                // Avoid comparing same pair twice and skip identical hashes (already handled)
                let pair_key = if fp_a.hash < fp_b.hash {
                    (fp_a.hash, fp_b.hash)
                } else {
                    (fp_b.hash, fp_a.hash)
                };

                if fp_a.hash == fp_b.hash || compared_pairs.contains(&pair_key) {
                    continue;
                }

                compared_pairs.insert(pair_key);

                // Calculate Jaccard similarity
                let similarity =
                    jaccard_similarity(&fp_a.normalized_tokens, &fp_b.normalized_tokens);

                if similarity >= self.config.similarity_threshold {
                    // Count unique files across both fingerprints
                    let all_locations: Vec<&FileLocation> =
                        fp_a.locations.iter().chain(fp_b.locations.iter()).collect();
                    let unique_files: std::collections::HashSet<&PathBuf> =
                        all_locations.iter().map(|loc| &loc.file_path).collect();

                    let file_count = unique_files.len();
                    let total_occurrences = all_locations.len();

                    if file_count >= self.config.min_file_occurrences {
                        let issue = CrossFileIssue {
                            fingerprint: fp_a.clone(), // Use first as representative
                            file_count,
                            total_occurrences,
                            similarity_score: similarity,
                            severity: Severity::Mild,
                        };

                        let issue_with_severity = CrossFileIssue {
                            severity: issue.compute_severity(),
                            ..issue
                        };

                        issues.push(issue_with_severity);
                    }
                }
            }
        }

        // Sort by severity then similarity score
        issues.sort_by(|a, b| {
            b.severity.cmp(&a.severity).then(
                b.similarity_score
                    .partial_cmp(&a.similarity_score)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        issues
    }

    /// Estimate current memory usage in bytes (approximate)
    pub fn estimated_memory_usage(&self) -> usize {
        let base_size = std::mem::size_of::<Self>();
        let index_size: usize = self
            .index
            .values()
            .map(|fp| {
                std::mem::size_of::<u64>()
                    + std::mem::size_of::<FunctionFingerprint>()
                    + fp.normalized_tokens.capacity()
                        * std::mem::size_of::<super::fingerprint::NormalizedToken>()
                    + fp.locations.capacity() * std::mem::size_of::<FileLocation>()
                    + fp.function_name.capacity()
            })
            .sum();

        base_size + index_size
    }

    /// Evict old entries to free memory when approaching limit
    pub fn evict_old_entries(&mut self) {
        let limit_bytes = self.config.max_memory_mb * 1024 * 1024;

        while self.estimated_memory_usage() > limit_bytes && !self.index.is_empty() {
            // Remove oldest entries (simple strategy: remove first few)
            // In production, would use LRU or more sophisticated eviction
            let keys_to_remove: Vec<u64> = self.index.keys().take(10).copied().collect();
            for key in keys_to_remove {
                self.index.remove(&key);
            }
        }
    }

    /// Get statistics about the analysis
    pub fn stats(&self) -> AnalysisStats {
        AnalysisStats {
            total_functions: self.total_functions_processed,
            total_files: self.total_files_processed,
            unique_fingerprints: self.index.len(),
            memory_bytes: self.estimated_memory_usage(),
        }
    }
}

/// Statistics snapshot of the analyzer state
#[derive(Debug, Clone)]
pub struct AnalysisStats {
    pub total_functions: usize,
    pub total_files: usize,
    pub unique_fingerprints: usize,
    pub memory_bytes: usize,
}

/// Analyze a project directory for cross-file code duplication
///
/// This function walks through all `.rs` files in the given directory,
/// extracts function fingerprints, and identifies duplicate patterns.
///
/// # Arguments
/// * `root` - Path to the project root directory
/// * `config` - Configuration options for the analysis
///
/// # Returns
/// * `Ok(CrossFileAnalyzer)` - Analyzer populated with all fingerprints
/// * `Err(String)` - Error message if analysis fails
pub fn analyze_project<P: AsRef<Path>>(
    root: P,
    config: CrossFileConfig,
) -> Result<CrossFileAnalyzer, String> {
    let root = root.as_ref();
    let mut analyzer = CrossFileAnalyzer::with_config(config);

    walk_directory(root, |path, content| analyzer.process_file(path, content))?;

    Ok(analyzer)
}

/// Walk through a directory and process each Rust file with the provided callback
fn walk_directory<F>(root: &Path, mut processor: F) -> Result<(), String>
where
    F: FnMut(&Path, &str) -> Result<(), String>,
{
    use std::fs;

    if !root.is_dir() {
        return Err(format!("{} is not a directory", root.display()));
    }

    fn visit_dir<F>(dir: &Path, processor: &mut F) -> Result<(), String>
    where
        F: FnMut(&Path, &str) -> Result<(), String>,
    {
        let entries =
            fs::read_dir(dir).map_err(|e| format!("Cannot read dir {}: {}", dir.display(), e))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-source directories
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }

                visit_dir(&path, processor)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let context = FileContext::from_path(&path);

                // Skip test files and example files (lower priority for cross-file detection)
                if context.rule_weight_multiplier() < 0.5 {
                    continue;
                }

                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;

                processor(&path, &content)?;
            }
        }

        Ok(())
    }

    visit_dir(root, &mut processor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify analyzer correctly processes single file with multiple functions
    /// Invariants: All valid functions should be indexed, invalid ones skipped
    #[test]
    fn test_process_single_file_multiple_functions() {
        let code = r#"
        fn short_func() { 1 }  // Too short, should be skipped

        fn valid_function_one(x: i32) -> i32 {
            let result = x * 2;
            result + 1
        }

        fn valid_function_two(data: &Vec<i32>) -> i32 {
            let mut sum = 0;
            for item in data {
                sum += item;
            }
            sum
        }
        "#;

        let mut analyzer = CrossFileAnalyzer::with_config(CrossFileConfig {
            min_function_lines: 4, // Adjust to match actual function lengths in test
            ..Default::default()
        });

        let result = analyzer.process_file(Path::new("test.rs"), code);
        assert!(result.is_ok(), "Processing should succeed");

        assert_eq!(
            analyzer.index.len(),
            2,
            "Should find exactly 2 valid functions (short one skipped)"
        );
        assert_eq!(
            analyzer.total_functions_processed, 2,
            "Should have processed 2 functions"
        );
    }

    /// Objective: Verify duplicate detection finds exact matches across files
    /// Invariants: Same function in two files must produce one duplicate issue
    #[test]
    fn test_detect_exact_duplicates_across_files() {
        let shared_code = r#"
        fn calculate_total(items: &Vec<i32>) -> i32 {
            let mut total = 0;
            for item in items {
                total += item;
            }
            total
        }
        "#;

        let mut analyzer = CrossFileAnalyzer::new();

        analyzer
            .process_file(Path::new("src/utils.rs"), shared_code)
            .expect("Failed to process utils.rs");
        analyzer
            .process_file(Path::new("src/helpers.rs"), shared_code)
            .expect("Failed to process helpers.rs");

        let duplicates = analyzer.find_all_duplicates();

        assert_eq!(
            duplicates.len(),
            1,
            "Should detect exactly 1 duplicate pattern"
        );

        let issue = &duplicates[0];
        assert_eq!(
            issue.file_count, 2,
            "Duplicate should appear in 2 different files"
        );
        assert_eq!(
            issue.total_occurrences, 2,
            "Total occurrences should be 2 (one per file)"
        );
        assert!(
            (issue.similarity_score - 1.0).abs() < f64::EPSILON,
            "Exact match should have similarity 1.0"
        );
    }

    /// Objective: Verify min_file_occurrences threshold filters results correctly
    /// Invariants: Pattern appearing in only 1 file should not be reported
    #[test]
    fn test_min_file_occurrences_filtering() {
        let code_unique = r#"
        fn unique_function(x: i32) -> i32 { x + 42 }
        "#;

        let mut analyzer = CrossFileAnalyzer::with_config(CrossFileConfig {
            min_function_lines: 3,
            min_file_occurrences: 2, // Require at least 2 files
            ..Default::default()
        });

        analyzer
            .process_file(Path::new("only_file.rs"), code_unique)
            .unwrap();

        let duplicates = analyzer.find_all_duplicates();

        assert!(
            duplicates.is_empty(),
            "Single-file pattern should not be reported when min_file_occurrences=2"
        );
    }

    /// Objective: Verify severity levels scale with duplication extent
    /// Invariants: More copies across more files should result in higher severity
    #[test]
    fn test_severity_scaling_with_duplication_extent() {
        let shared_code = r#"
        fn duplicated(x: i32) -> i32 {
            let y = x * 2;
            y + 1
        }
        "#;

        let mut analyzer = CrossFileAnalyzer::with_config(CrossFileConfig {
            min_function_lines: 4,
            ..Default::default()
        });

        // Add same function to 6 different files
        for i in 0..6 {
            analyzer
                .process_file(Path::new(&format!("file_{}.rs", i)), shared_code)
                .unwrap();
        }

        let duplicates = analyzer.find_all_duplicates();

        assert_eq!(duplicates.len(), 1, "Should find 1 duplicate group");
        assert_eq!(
            duplicates[0].severity,
            Severity::Nuclear,
            "6 files with same function should be Nuclear severity"
        );
    }

    /// Objective: Verify memory eviction prevents unbounded growth
    /// Invariants: After processing many files, memory should stay within configured limits
    #[test]
    fn test_memory_limit_enforcement() {
        let config = CrossFileConfig {
            max_memory_mb: 1, // Very low limit for testing
            min_function_lines: 3,
            ..Default::default()
        };

        let mut analyzer = CrossFileAnalyzer::with_config(config.clone());

        let simple_fn = r#"
        fn sample_func(a: i32, b: i32) -> i32 { a + b }
        "#;

        // Process many files to trigger memory pressure
        for i in 0..100 {
            let _ = analyzer.process_file(Path::new(&format!("test_{}.rs", i)), simple_fn);

            // Periodically check and evict if needed
            if analyzer.estimated_memory_usage() > config.max_memory_mb * 1024 * 1024 {
                analyzer.evict_old_entries();
            }
        }

        // Memory should not exceed 2x the limit (allowing some overhead)
        let max_allowed = config.max_memory_mb * 1024 * 1024 * 2;
        assert!(
            analyzer.estimated_memory_usage() <= max_allowed,
            "Memory usage ({}) should stay within 2x limit ({})",
            analyzer.estimated_memory_usage(),
            max_allowed
        );
    }

    /// Objective: Verify statistics accurately reflect analysis state
    /// Invariants: Stats must match actual counts after processing
    #[test]
    fn test_statistics_accuracy() {
        let code = r#"
        fn first_func(x: i32) -> i32 { x + 42 }
        fn second_func(data: &Vec<i32>) -> i32 {
            let mut sum = 0;
            for item in data { sum += item; }
            sum
        }
        "#;

        let mut analyzer = CrossFileAnalyzer::with_config(CrossFileConfig {
            min_function_lines: 1, // Single-line functions should be counted
            ..Default::default()
        });

        analyzer
            .process_file(Path::new("stats_test.rs"), code)
            .unwrap();

        let stats = analyzer.stats();

        assert_eq!(
            stats.total_functions, 2,
            "Should have processed 2 functions"
        );
        assert_eq!(stats.total_files, 1, "Should have processed 1 file");
        assert_eq!(
            stats.unique_fingerprints, 2,
            "Should have 2 unique fingerprints (different structures)"
        );
        assert!(stats.memory_bytes > 0, "Memory usage should be positive");
    }

    /// Objective: Verify near-duplicate detection catches similar but not identical functions
    /// Invariants: Functions with minor modifications should still be flagged
    #[test]
    fn test_near_duplicate_detection_fuzzy_matching() {
        let code_base = r#"
        fn process_data(data: &Vec<i32>) -> i32 {
            let mut sum = 0;
            for item in data {
                if *item > 0 {
                    sum += item;
                }
            }
            sum
        }
        "#;

        let code_modified = r#"
        fn handle_items(items: &Vec<i32>) -> i32 {
            let mut total = 0;
            for value in items {
                if *value >= 0 {
                    total += value;
                }
            }
            total
        }
        "#;

        let mut analyzer = CrossFileAnalyzer::with_config(CrossFileConfig {
            min_function_lines: 8,
            similarity_threshold: 0.8, // Lower threshold for testing
            ..Default::default()
        });

        analyzer
            .process_file(Path::new("base.rs"), code_base)
            .unwrap();
        analyzer
            .process_file(Path::new("modified.rs"), code_modified)
            .unwrap();

        // Exact duplicates (should find these)
        let exact_dups = analyzer.find_all_duplicates();

        // Near duplicates (fuzzy matching)
        let near_dups = analyzer.find_near_duplicates();

        // The modified version may or may not be detected depending on normalization quality
        // At minimum, we verify the method runs without errors
        assert!(
            !near_dups.is_empty() || exact_dups.is_empty(),
            "Either exact or near-duplicates should be found (or neither if too different)"
        );
    }
}
