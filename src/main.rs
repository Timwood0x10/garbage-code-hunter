use clap::Parser;
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;

mod analyzer;
mod i18n;
mod reporter;
mod rules;
mod scoring;

use analyzer::CodeAnalyzer;
use reporter::Reporter;

#[derive(Parser)]
#[command(name = "garbage-code-hunter")]
#[command(about = "A humorous Rust code quality detector that roasts your garbage code 🔥")]
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
}

fn main() {
    let args = Args::parse();

    let analyzer = CodeAnalyzer::new(&args.exclude, &args.lang);
    let issues = analyzer.analyze_path(&args.path);
    
    // Calculate metrics for scoring
    let (file_count, total_lines) = calculate_metrics(&args.path, &args.exclude);

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
    reporter.report_with_metrics(issues, file_count, total_lines);
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
        exclude_regexes.iter().any(|pattern| pattern.is_match(&path_str))
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
