use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

// Use modules from library (lib.rs)
use garbage_code_hunter::{
    analyzer::{CodeAnalyzer, CodeIssue},
    commit_roaster,
    config::{AppConfig, AppMode},
    educational::EducationalAdvisor,
    hall_of_shame::HallOfShame,
    llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastProvider},
    reporter::Reporter,
};

#[derive(Parser)]
#[command(name = "garbage-code-hunter")]
#[command(about = "A humorous Rust code quality detector that roasts your garbage code \u{1f525}")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze Rust code quality (default behavior)
    Analyze(AnalyzeArgs),

    /// Scan git history and roast bad commit messages
    #[command(alias = "cr")]
    CommitRoaster(CommitRoasterArgs),
}

#[derive(Parser)]
struct AnalyzeArgs {
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

#[derive(Parser)]
struct CommitRoasterArgs {
    /// Path to git repository (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum number of commits to analyze
    #[arg(short, long, default_value = "50")]
    limit: usize,

    /// Filter by author name or email
    #[arg(short, long)]
    author: Option<String>,

    /// Only commits after this date (YYYY-MM-DD)
    #[arg(long)]
    since: Option<String>,

    /// Only commits before this date (YYYY-MM-DD)
    #[arg(long)]
    until: Option<String>,

    /// Branch to analyze
    #[arg(short, long)]
    branch: Option<String>,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,
}

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::CommitRoaster(args)) => run_commit_roaster(args),
        Some(Commands::Analyze(args)) => run_analyze(args),
        None => run_analyze(AnalyzeArgs::default()),
    }
}

fn run_commit_roaster(args: CommitRoasterArgs) {
    use commit_roaster::{run, OutputFormat};

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    // Parse date strings to timestamps
    let since = args.since.and_then(|s| parse_date_to_timestamp(&s));
    let until = args.until.and_then(|s| parse_date_to_timestamp(&s));

    let config = commit_roaster::analyzer::AnalyzerConfig {
        limit: Some(args.limit),
        author: args.author,
        since,
        until,
        branch: args.branch,
    };

    match run(&args.path, &config, &format) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Parse YYYY-MM-DD to Unix timestamp.
fn parse_date_to_timestamp(date_str: &str) -> Option<i64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    // Simple date-to-timestamp calculation
    let days_since_epoch = days_from_ymd(year, month, day);
    Some(days_since_epoch * 86400)
}

fn days_from_ymd(year: i32, month: u32, day: u32) -> i64 {
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

impl Default for AnalyzeArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            harsh: false,
            savage: false,
            verbose: false,
            top: 5,
            issues: 5,
            summary: false,
            markdown: false,
            lang: "en-US".to_string(),
            exclude: Vec::new(),
            educational: false,
            hall_of_shame: false,
            suggestions: false,
            format: "text".to_string(),
            llm: false,
            llm_provider: "ollama".to_string(),
            llm_endpoint: None,
            llm_model: None,
            llm_api_key: None,
            llm_timeout: 30,
            config: None,
        }
    }
}

fn run_analyze(args: AnalyzeArgs) {
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
        Some(args.llm_timeout),
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
        reporter.report_with_metrics(issues.clone(), file_count, total_lines);

        if args.educational {
            if let Some(advisor) = educational_advisor.as_ref() {
                println!("\n\u{1f393} Educational Advice:");
                println!("{}", "\u{2500}".repeat(50));
                for issue in &issues {
                    if let Some(advice) = advisor.get_advice(&issue.rule_name) {
                        println!("\n\u{1f4da} {}: {}", issue.rule_name, advice.why_bad);
                        println!("\u{1f4a1} How to fix: {}", advice.how_to_fix);
                        if let Some(tip) = &advice.best_practice_tip {
                            println!("\u{2728} Tip: {}", tip);
                        }
                    }
                }
            }
        }

        if args.hall_of_shame {
            if let Some(hall) = hall_of_shame.as_ref() {
                let stats = hall.generate_shame_report();
                println!("\n\u{1f3c6} Hall of Shame:");
                println!("{}", "\u{2500}".repeat(50));
                println!(
                    "\u{1f4ca} Total files analyzed: {}",
                    stats.total_files_analyzed
                );
                println!(
                    "\u{1f5d1}\u{fe0f} Total issues found: {}",
                    stats.total_issues
                );
                println!(
                    "\u{1f4c8} Garbage density: {:.2} issues per 1000 lines",
                    stats.garbage_density
                );

                println!("\n\u{1f525} Worst Files:");
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
            println!("\n\u{1f3af} Improvement Suggestions:");
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

fn count_file_lines(file_path: &std::path::Path) -> usize {
    std::fs::read_to_string(file_path)
        .map(|content| content.lines().count())
        .unwrap_or(0)
}

fn output_json(issues: &[CodeIssue]) {
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
