use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

mod analyzer;
mod educational;
mod hall_of_shame;
mod i18n;
mod reporter;
mod rules;
mod scoring;
mod utils;

use analyzer::CodeAnalyzer;
use educational::EducationalAdvisor;
use hall_of_shame::HallOfShame;
use reporter::Reporter;

#[derive(Parser)]
#[command(name = "garbage-code-hunter")]
#[command(about = "A humorous Rust code quality detector that roasts your garbage code 🔥")]
#[command(version)]
struct Args {
    /// Path to analyze (file or directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show only the worst offenders
    #[arg(long)]
    harsh: bool,

    /// Be extra mean in the output (deprecated, use --lang for language control)
    #[arg(long)]
    savage: bool,

    /// Show detailed analysis report
    #[arg(short, long)]
    verbose: bool,

    /// Show top N files with most issues (default: 5)
    #[arg(short = 't', long, default_value = "5")]
    top: usize,

    /// Show N issues per file (default: 5)
    #[arg(short = 'i', long, default_value = "5")]
    issues: usize,

    /// Only show summary conclusion, skip details
    #[arg(short = 's', long)]
    summary: bool,

    /// Output Markdown format report for AI tools
    #[arg(short, long)]
    markdown: bool,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,

    /// Exclude file/directory patterns (can be used multiple times)
    #[arg(short, long)]
    exclude: Vec<String>,

    /// Show educational advice for each issue type
    #[arg(long)]
    educational: bool,

    /// Show hall of shame (worst files and patterns)
    #[arg(long)]
    hall_of_shame: bool,

    /// Show improvement suggestions based on analysis
    #[arg(long)]
    suggestions: bool,
}

fn main() {
    let args = Args::parse();

    let analyzer = CodeAnalyzer::new(&args.exclude, &args.lang);
    let issues = analyzer.analyze_path(&args.path);

    // Calculate metrics for scoring
    let (file_count, total_lines) = calculate_metrics(&args.path, &args.exclude);

    // Initialize educational advisor if needed
    let educational_advisor = if args.educational {
        Some(EducationalAdvisor::new(&args.lang))
    } else {
        None
    };

    // Initialize hall of shame if needed
    let mut hall_of_shame = if args.hall_of_shame || args.suggestions {
        Some(HallOfShame::new())
    } else {
        None
    };

    // Populate hall of shame with analysis results
    if let Some(ref mut shame) = hall_of_shame {
        let issues_by_file = group_issues_by_file(&issues);
        for (file_path, file_issues) in issues_by_file {
            let file_lines = count_file_lines(&file_path);
            shame.add_file_analysis(file_path, &file_issues, file_lines);
        }
    }

    let reporter = Reporter::new(
        args.harsh,
        args.savage,
        args.verbose,
        args.top,
        args.issues,
        args.summary,
        args.markdown,
        &args.lang,
    );
    
    if args.educational || args.hall_of_shame || args.suggestions {
        reporter.report_with_enhanced_features(
            issues, 
            file_count, 
            total_lines,
            educational_advisor.as_ref(),
            hall_of_shame.as_ref(),
            args.suggestions,
        );
    } else {
        reporter.report_with_metrics(issues, file_count, total_lines);
    }
}

fn calculate_metrics(path: &PathBuf, exclude_patterns: &[String]) -> (usize, usize) {
    let mut file_count = 0;
    let mut total_lines = 0;

    // Convert exclude patterns to regex patterns
    let exclude_regexes: Vec<regex::Regex> = exclude_patterns
        .iter()
        .filter_map(|pattern| {
            let regex_pattern = pattern
                .replace(".", r"\.")
                .replace("*", ".*")
                .replace("?", ".");
            regex::Regex::new(&regex_pattern).ok()
        })
        .collect();

    let should_exclude = |path: &std::path::Path| -> bool {
        let path_str = path.to_string_lossy();
        exclude_regexes
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    };

    if path.is_file() {
        if !should_exclude(path) {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
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
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            file_count += 1;
            if let Ok(content) = fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
            }
        }
    }

    (file_count, total_lines)
}

fn group_issues_by_file(issues: &[analyzer::CodeIssue]) -> std::collections::HashMap<std::path::PathBuf, Vec<analyzer::CodeIssue>> {
    let mut grouped = std::collections::HashMap::new();
    for issue in issues {
        grouped.entry(issue.file_path.clone()).or_insert_with(Vec::new).push(issue.clone());
    }
    grouped
}

fn count_file_lines(file_path: &std::path::Path) -> usize {
    std::fs::read_to_string(file_path)
        .map(|content| content.lines().count())
        .unwrap_or(0)
}
