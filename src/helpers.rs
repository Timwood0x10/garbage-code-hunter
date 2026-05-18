use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use garbage_code_hunter::analyzer::{CodeAnalyzer, CodeIssue, Severity};
use garbage_code_hunter::common::OutputFormat;
use garbage_code_hunter::context::ProjectConfig;
use garbage_code_hunter::language::SUPPORTED_EXTENSIONS;
use garbage_code_hunter::style_ir::{StyleIr, StyleIrSummary};
use garbage_code_hunter::treesitter::engine::TreeSitterEngine;

pub fn derive_count_score(count: f64, weight: f64) -> f64 {
    (100.0 - count * weight).clamp(0.0, 100.0)
}

pub fn derive_radar_score(val: &serde_json::Value) -> f64 {
    let dims = [
        "complexity",
        "duplication",
        "naming",
        "panic_risk",
        "dependency_hell",
        "legacy_smell",
    ];
    let sum: f64 = dims.iter().filter_map(|d| val[*d].as_f64()).sum();
    let n = dims.iter().filter(|d| val[**d].is_number()).count() as f64;
    if n > 0.0 {
        (sum / n).clamp(0.0, 100.0)
    } else {
        100.0
    }
}

pub fn derive_danger_zone_score(val: &serde_json::Value) -> f64 {
    match val["files"].as_array() {
        Some(arr) if !arr.is_empty() => {
            let sum: f64 = arr.iter().filter_map(|f| f["risk_score"].as_f64()).sum();
            (sum / arr.len() as f64).clamp(0.0, 100.0)
        }
        _ => 100.0,
    }
}

pub fn quick_scan_score(path: &Path, config: &ProjectConfig) -> f64 {
    let results: Vec<(&str, f64, usize)> =
        std::thread::scope(|s| {
            let h_code = s.spawn(|| {
                let analyzer = CodeAnalyzer::with_config(&[], "en-US", config.clone());
                let issues = analyzer.analyze_path(path);
                let path_buf = path.to_path_buf();
                let (_, total_lines) = calculate_metrics(&path_buf, &[]);
                let score = if total_lines > 0 {
                    let density = issues.len() as f64 / total_lines as f64 * 1000.0;
                    (100.0 - density * 2.0).clamp(0.0, 100.0)
                } else {
                    100.0
                };
                ("code-hunter", score, issues.len())
            });

            let h_commit = s.spawn(|| {
                let cfg = garbage_code_hunter::commit_roaster::analyzer::AnalyzerConfig::default();
                match garbage_code_hunter::commit_roaster::run(path, &cfg, &OutputFormat::Json) {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => ("commit-roaster", v["score"].as_f64().unwrap_or(100.0), 0),
                        Err(_) => ("commit-roaster", 100.0, 0),
                    },
                    Err(_) => ("commit-roaster", 100.0, 0),
                }
            });

            let h_deps = s.spawn(|| {
                match garbage_code_hunter::deps_shamer::run(path, &OutputFormat::Json) {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => ("deps-shamer", v["score"].as_f64().unwrap_or(100.0), 0),
                        Err(_) => ("deps-shamer", 100.0, 0),
                    },
                    Err(_) => ("deps-shamer", 100.0, 0),
                }
            });

            let h_pr = s.spawn(|| {
                match garbage_code_hunter::pr_title_hunter::run(path, 50, &OutputFormat::Json) {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => ("pr-title-hunter", v["score"].as_f64().unwrap_or(100.0), 0),
                        Err(_) => ("pr-title-hunter", 100.0, 0),
                    },
                    Err(_) => ("pr-title-hunter", 100.0, 0),
                }
            });

            let h_decay = s.spawn(|| {
                match garbage_code_hunter::decay::run(path, &OutputFormat::Json, "en-US") {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => ("decay", v["current_health"].as_f64().unwrap_or(50.0), 0),
                        Err(_) => ("decay", 50.0, 0),
                    },
                    Err(_) => ("decay", 50.0, 0),
                }
            });

            let h_radar = s.spawn(|| {
                match garbage_code_hunter::radar::run(path, &OutputFormat::Json, "en-US", None) {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => ("radar", derive_radar_score(&v), 0),
                        Err(_) => ("radar", 50.0, 0),
                    },
                    Err(_) => ("radar", 50.0, 0),
                }
            });

            let h_debt = s.spawn(|| {
                match garbage_code_hunter::debt_invoice::run(path, &OutputFormat::Json, "en-US") {
                    Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                        Ok(v) => {
                            let hours = v["total_hours"].as_f64().unwrap_or(0.0);
                            ("debt-invoice", derive_count_score(hours, 0.5), 0)
                        }
                        Err(_) => ("debt-invoice", 100.0, 0),
                    },
                    Err(_) => ("debt-invoice", 100.0, 0),
                }
            });

            vec![
                h_code.join().unwrap_or(("code-hunter", 100.0, 0)),
                h_commit.join().unwrap_or(("commit-roaster", 100.0, 0)),
                h_deps.join().unwrap_or(("deps-shamer", 100.0, 0)),
                h_pr.join().unwrap_or(("pr-title-hunter", 100.0, 0)),
                h_decay.join().unwrap_or(("decay", 50.0, 0)),
                h_radar.join().unwrap_or(("radar", 50.0, 0)),
                h_debt.join().unwrap_or(("debt-invoice", 100.0, 0)),
            ]
        });

    overall_score(&results)
}

pub fn overall_score(scores: &[(&str, f64, usize)]) -> f64 {
    if scores.is_empty() {
        return 100.0;
    }

    let weights: std::collections::HashMap<&str, f64> = [
        ("code-hunter", 0.20),
        ("commit-roaster", 0.10),
        ("deps-shamer", 0.05),
        ("pr-title-hunter", 0.05),
        ("last-words", 0.05),
        ("debt-invoice", 0.10),
        ("personality", 0.05),
        ("decay", 0.10),
        ("autopsy", 0.05),
        ("radar", 0.10),
        ("ci-bot", 0.05),
        ("persona", 0.03),
        ("danger-zone", 0.05),
        ("team-roast", 0.02),
    ]
    .into_iter()
    .collect();

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for (name, score, _) in scores {
        let w = weights.get(name).copied().unwrap_or(0.05);
        weighted_sum += score * w;
        total_weight += w;
    }
    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        100.0
    }
}

pub fn parse_date_to_timestamp(date_str: &str) -> Option<i64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    let days_since_epoch = days_from_ymd(year, month, day);
    Some(days_since_epoch * 86400)
}

pub fn days_from_ymd(year: i32, month: u32, day: u32) -> i64 {
    let mut y = year as i64;
    let mut m = month as i64;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn calculate_metrics(path: &PathBuf, exclude_patterns: &[String]) -> (usize, usize) {
    let mut file_count = 0;
    let mut total_lines = 0;

    let default_excludes = [
        "target",
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        "build",
        "dist",
        "out",
        "__pycache__",
        ".DS_Store",
        ".venv",
        "venv",
        "vendor",
    ];
    let all_patterns: Vec<String> = default_excludes
        .iter()
        .map(|s| s.to_string())
        .chain(exclude_patterns.iter().cloned())
        .collect();

    let exclude_regexes: Vec<regex::Regex> = all_patterns
        .iter()
        .filter_map(|pattern| {
            let regex_pattern = pattern
                .replace(".", r"\.")
                .replace("*", ".*")
                .replace("?", ".");
            regex::Regex::new(&regex_pattern).ok()
        })
        .collect();

    let should_exclude = |path: &Path| -> bool {
        let path_str = path.to_string_lossy();
        exclude_regexes
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    };

    if path.is_file() {
        if !should_exclude(path) {
            if let Some(ext) = path.extension() {
                if SUPPORTED_EXTENSIONS.contains(&ext.to_str().unwrap_or("")) {
                    file_count = 1;
                    if let Ok(content) = fs::read_to_string(path) {
                        total_lines = content.lines().count();
                    }
                }
            }
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !should_exclude(e.path()))
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
            })
        {
            file_count += 1;
            if let Ok(content) = fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
            }
        }
    }

    (file_count, total_lines)
}

pub fn group_issues_by_file(
    issues: &[CodeIssue],
) -> std::collections::HashMap<std::path::PathBuf, Vec<CodeIssue>> {
    let mut grouped = std::collections::HashMap::new();
    for issue in issues {
        grouped
            .entry(issue.file_path.clone())
            .or_insert_with(Vec::new)
            .push(issue.clone());
    }
    grouped
}

pub fn count_file_lines(file_path: &Path) -> usize {
    fs::read_to_string(file_path)
        .map(|content| content.lines().count())
        .unwrap_or(0)
}

#[derive(serde::Serialize)]
struct AnalyzeJsonIssue {
    file_path: String,
    line: usize,
    column: usize,
    rule_name: String,
    message: String,
    severity: String,
}

#[derive(serde::Serialize)]
struct AnalyzeJsonSignal {
    signal: String,
    file_path: String,
    severity: String,
    violation_count: usize,
}

#[derive(Clone, serde::Serialize)]
pub struct AnalyzeJsonFile {
    pub file_path: String,
    pub style_ir_summary: StyleIrSummary,
}

#[derive(serde::Serialize)]
struct AnalyzeJsonSummary {
    file_count: usize,
    issue_count: usize,
    signal_count: usize,
    total_score: f64,
    style_ir_summary: Option<StyleIrSummary>,
}

#[derive(serde::Serialize)]
struct AnalyzeJsonReport {
    schema_version: &'static str,
    issues: Vec<AnalyzeJsonIssue>,
    signals: Vec<AnalyzeJsonSignal>,
    files: Vec<AnalyzeJsonFile>,
    summary: AnalyzeJsonSummary,
    style_ir_summary: Option<StyleIrSummary>,
}

fn compute_json_score(issues: &[AnalyzeJsonIssue], total_lines: usize) -> f64 {
    let (nuclear, spicy, mild) =
        issues
            .iter()
            .fold((0, 0, 0), |(n, s, m), i| match i.severity.as_str() {
                "Nuclear" => (n + 1, s, m),
                "Spicy" => (n, s + 1, m),
                _ => (n, s, m + 1),
            });
    let k_lines = (total_lines as f64 / 1000.0).max(0.001);
    let n_score = ((nuclear as f64 + 1.0).log2() * 8.0).min(40.0);
    let density = (spicy as f64 * 1.5 + mild as f64) / k_lines;
    let d_score = ((density + 1.0).log2() * 6.0).min(60.0);
    ((n_score + d_score) * 100.0).round() / 100.0
}

pub fn output_json(
    issues: &[CodeIssue],
    files: &[AnalyzeJsonFile],
    style_ir_summary: Option<&StyleIrSummary>,
) {
    // Separate signal findings (line=0) from per-line rule issues
    let mut json_issues = Vec::new();
    let mut json_signals = Vec::new();
    for issue in issues {
        let json = AnalyzeJsonIssue {
            file_path: issue.file_path.to_string_lossy().to_string(),
            line: issue.line,
            column: issue.column,
            rule_name: issue.rule_name.clone(),
            message: issue.message.clone(),
            severity: format!("{:?}", issue.severity),
        };
        if issue.line == 0 {
            // Extract violation count from message for signal display
            let count = issue
                .message
                .split_whitespace()
                .next()
                .and_then(|w| w.parse::<usize>().ok())
                .unwrap_or(0);
            json_signals.push(AnalyzeJsonSignal {
                signal: issue.rule_name.clone(),
                file_path: issue.file_path.to_string_lossy().to_string(),
                severity: format!("{:?}", issue.severity),
                violation_count: count,
            });
        } else {
            json_issues.push(json);
        }
    }

    let total_lines = style_ir_summary.map(|s| s.line_count).unwrap_or(1);
    let total_score = compute_json_score(&json_issues, total_lines);

    let summary = AnalyzeJsonSummary {
        file_count: files.len(),
        issue_count: json_issues.len(),
        signal_count: json_signals.len(),
        total_score,
        style_ir_summary: style_ir_summary.cloned(),
    };

    let json_report = AnalyzeJsonReport {
        schema_version: "1.0",
        issues: json_issues,
        signals: json_signals,
        files: files.to_vec(),
        summary,
        style_ir_summary: style_ir_summary.cloned(),
    };

    if let Ok(json_output) = serde_json::to_string_pretty(&json_report) {
        println!("{}", json_output);
    } else {
        eprintln!("Error: Failed to serialize analyze report to JSON");
        std::process::exit(1);
    }
}

pub fn collect_style_ir_files(path: &PathBuf, exclude_patterns: &[String]) -> Vec<AnalyzeJsonFile> {
    let supported_files = collect_supported_files(path, exclude_patterns);
    let mut files = Vec::new();

    let tree_sitter_engine = TreeSitterEngine::new();
    for file_path in supported_files {
        let content = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        if let Some(parsed) = tree_sitter_engine.parse_file(&file_path, &content) {
            if let Some(style_ir) = StyleIr::from_parsed(&parsed) {
                files.push(AnalyzeJsonFile {
                    file_path: file_path.to_string_lossy().to_string(),
                    style_ir_summary: style_ir.summary(),
                });
            }
        }
    }

    files
}

pub fn summarize_style_ir_files(files: &[AnalyzeJsonFile]) -> Option<StyleIrSummary> {
    if files.is_empty() {
        return None;
    }

    let language = if files
        .iter()
        .all(|file| file.style_ir_summary.language == files[0].style_ir_summary.language)
    {
        files[0].style_ir_summary.language.clone()
    } else {
        "Mixed".to_string()
    };

    let mut line_count = 0usize;
    let mut function_count = 0usize;
    let mut god_function_count = 0usize;
    let mut panic_call_count = 0usize;
    let mut naming_violation_count = 0usize;
    let mut deeply_nested_block_count = 0usize;
    let mut debug_call_count = 0usize;
    let mut excessive_param_count = 0usize;
    let mut unsafe_block_count = 0usize;
    let mut magic_number_count = 0usize;
    let mut commented_out_lines = 0usize;
    let mut todo_count = 0usize;
    let mut goroutine_spawn_count = 0usize;
    let mut defer_in_loop_count = 0usize;
    let mut go_convention_count = 0usize;
    let thresholds = files[0].style_ir_summary.thresholds;

    for file in files {
        let summary = &file.style_ir_summary;
        line_count += summary.line_count;
        function_count += summary.function_count;
        god_function_count += summary.god_function_count;
        panic_call_count += summary.panic_call_count;
        naming_violation_count += summary.naming_violation_count;
        deeply_nested_block_count += summary.deeply_nested_block_count;
        debug_call_count += summary.debug_call_count;
        excessive_param_count += summary.excessive_param_count;
        unsafe_block_count += summary.unsafe_block_count;
        magic_number_count += summary.magic_number_count;
        commented_out_lines += summary.commented_out_lines;
        todo_count += summary.todo_count;
        goroutine_spawn_count += summary.goroutine_spawn_count;
        defer_in_loop_count += summary.defer_in_loop_count;
        go_convention_count += summary.go_convention_count;
    }
    Some(StyleIrSummary {
        language,
        line_count,
        function_count,
        god_function_count,
        panic_call_count,
        naming_violation_count,
        deeply_nested_block_count,
        debug_call_count,
        excessive_param_count,
        unsafe_block_count,
        magic_number_count,
        commented_out_lines,
        todo_count,
        goroutine_spawn_count,
        defer_in_loop_count,
        go_convention_count,
        over_engineering_count: god_function_count + excessive_param_count,
        code_smell_count: unsafe_block_count * 2 + magic_number_count,
        is_clean_signal_baseline: panic_call_count == 0
            && naming_violation_count == 0
            && deeply_nested_block_count == 0
            && debug_call_count == 0
            && excessive_param_count == 0
            && unsafe_block_count == 0
            && magic_number_count == 0
            && commented_out_lines == 0
            && todo_count == 0
            && goroutine_spawn_count == 0
            && defer_in_loop_count == 0
            && go_convention_count == 0,
        thresholds,
    })
}

pub fn collect_supported_files(path: &PathBuf, exclude_patterns: &[String]) -> Vec<PathBuf> {
    let default_excludes = [
        "target",
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        "build",
        "dist",
        "out",
        "__pycache__",
        ".DS_Store",
        ".venv",
        "venv",
        "vendor",
    ];
    let all_patterns: Vec<String> = default_excludes
        .iter()
        .map(|s| s.to_string())
        .chain(exclude_patterns.iter().cloned())
        .collect();

    let exclude_regexes: Vec<regex::Regex> = all_patterns
        .iter()
        .filter_map(|pattern| {
            let regex_pattern = pattern
                .replace('.', r"\.")
                .replace('*', ".*")
                .replace('?', ".");
            regex::Regex::new(&regex_pattern).ok()
        })
        .collect();

    let should_exclude = |path: &Path| -> bool {
        let path_str = path.to_string_lossy();
        exclude_regexes
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    };

    let mut files = Vec::new();
    if path.is_file() {
        if !should_exclude(path)
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
        {
            files.push(path.clone());
        }
        return files;
    }

    if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| !should_exclude(entry.path()))
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
            })
        {
            files.push(entry.path().to_path_buf());
        }
    }

    files
}

/// Output GitHub Actions workflow commands as annotations.
pub fn output_github_actions(issues: &[CodeIssue]) {
    for issue in issues {
        let sev = match issue.severity {
            Severity::Nuclear => "error",
            Severity::Spicy => "warning",
            Severity::Mild => "notice",
        };
        let file = issue
            .file_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| issue.file_path.to_string_lossy().to_string());
        println!(
            "::{} file={},line={},title={}::{}",
            sev, file, issue.line, issue.rule_name, issue.message
        );
    }
}
