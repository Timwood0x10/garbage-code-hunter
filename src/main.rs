use clap::Parser;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

mod analyzer;
mod config;
mod educational;
mod hall_of_shame;
mod i18n;
mod llm;
mod reporter;
mod rules;
mod scoring;
mod utils;

use analyzer::CodeAnalyzer;
use config::{AppConfig, AppMode};
use educational::EducationalAdvisor;
use hall_of_shame::HallOfShame;
use llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastProvider};
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

    /// Output format (text, json)
    #[arg(short = 'f', long, default_value = "text")]
    format: String,

    /// Enable LLM-powered roast generation
    #[arg(long)]
    llm: bool,

    /// LLM provider type: ollama or openai-compatible
    #[arg(long, default_value = "ollama")]
    llm_provider: String,

    /// LLM API endpoint URL
    #[arg(long)]
    llm_endpoint: Option<String>,

    /// LLM model name
    #[arg(long)]
    llm_model: Option<String>,

    /// LLM API key (optional, for OpenAI-compatible providers)
    #[arg(long)]
    llm_api_key: Option<String>,

    /// LLM request timeout in seconds
    #[arg(long, default_value = "30")]
    llm_timeout: u64,

    /// Path to configuration file (default: ./config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Load config file and merge with CLI arguments
    let mut app_config = AppConfig::from_file(args.config.as_deref()).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {e}");
        AppConfig {
            mode: AppMode::Local,
        }
    });
    app_config.merge_cli(
        args.llm,
        &args.llm_provider,
        args.llm_endpoint.as_deref(),
        args.llm_model.as_deref(),
        args.llm_api_key.as_deref(),
        args.llm_timeout,
    );

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

    // Construct roast provider based on active mode
    let roast_provider: Box<dyn RoastProvider> = match &app_config.mode {
        AppMode::Local => Box::new(LocalRoastProvider),
        AppMode::Llm(llm_cfg) => {
            let config = LlmConfig::from_args(
                &llm_cfg.provider,
                Some(&llm_cfg.endpoint),
                Some(&llm_cfg.model),
                llm_cfg.api_key.as_deref(),
                llm_cfg.timeout_secs,
            );
            Box::new(LlmRoastProvider::new(config))
        }
    };

    let reporter = Reporter::new(
        args.harsh,
        args.savage,
        args.verbose,
        args.top,
        args.issues,
        args.summary,
        args.markdown,
        &args.lang,
        roast_provider,
    );

    // Handle JSON output format
    if args.format == "json" {
        output_json(&issues);
        return;
    }

    if args.educational || args.hall_of_shame || args.suggestions {
        // Enhanced reporting with educational features
        reporter.report_with_metrics(issues.clone(), file_count, total_lines);

        if args.educational {
            if let Some(advisor) = educational_advisor.as_ref() {
                println!("\n🎓 Educational Advice:");
                println!("{}", "─".repeat(50));
                for issue in &issues {
                    if let Some(advice) = advisor.get_advice(&issue.rule_name) {
                        println!("\n📚 {}: {}", issue.rule_name, advice.why_bad);
                        println!("💡 How to fix: {}", advice.how_to_fix);
                        if let Some(tip) = &advice.best_practice_tip {
                            println!("✨ Tip: {}", tip);
                        }
                    }
                }
            }
        }

        if args.hall_of_shame {
            if let Some(hall) = hall_of_shame.as_ref() {
                let stats = hall.generate_shame_report();
                println!("\n🏆 Hall of Shame:");
                println!("{}", "─".repeat(50));
                println!("📊 Total files analyzed: {}", stats.total_files_analyzed);
                println!("🗑️ Total issues found: {}", stats.total_issues);
                println!(
                    "📈 Garbage density: {:.2} issues per 1000 lines",
                    stats.garbage_density
                );

                println!("\n🔥 Worst Files:");
                for (i, entry) in stats.hall_of_shame.iter().take(5).enumerate() {
                    println!(
                        "  {}. {} - {} issues (score: {:.1})",
                        i + 1,
                        entry
                            .file_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy(),
                        entry.total_issues,
                        entry.shame_score
                    );
                }
            }
        }

        if args.suggestions {
            println!("\n🎯 Improvement Suggestions:");
            println!("- Focus on renaming meaningless variables");
            println!("- Reduce function complexity and nesting");
            println!("- Replace unwrap() with proper error handling");
        }
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
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            file_count += 1;
            if let Ok(content) = fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
            }
        }
    }

    (file_count, total_lines)
}

fn group_issues_by_file(
    issues: &[analyzer::CodeIssue],
) -> std::collections::HashMap<std::path::PathBuf, Vec<analyzer::CodeIssue>> {
    let mut grouped = std::collections::HashMap::new();
    for issue in issues {
        grouped
            .entry(issue.file_path.clone())
            .or_insert_with(Vec::new)
            .push(issue.clone());
    }
    grouped
}

fn count_file_lines(file_path: &std::path::Path) -> usize {
    std::fs::read_to_string(file_path)
        .map(|content| content.lines().count())
        .unwrap_or(0)
}

fn output_json(issues: &[analyzer::CodeIssue]) {
    use serde_json;

    let json_issues: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            serde_json::json!({
                "file_path": issue.file_path.to_string_lossy(),
                "line": issue.line,
                "column": issue.column,
                "rule_name": issue.rule_name,
                "message": issue.message,
                "severity": format!("{:?}", issue.severity)
            })
        })
        .collect();

    if let Ok(json_output) = serde_json::to_string_pretty(&json_issues) {
        println!("{}", json_output);
    } else {
        eprintln!("Error: Failed to serialize issues to JSON");
        std::process::exit(1);
    }
}
