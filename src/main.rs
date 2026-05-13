use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

// Use modules from library (lib.rs)
use garbage_code_hunter::{
    analyzer::{CodeAnalyzer, CodeIssue},
    autopsy, ci_bot, commit_roaster,
    common::OutputFormat,
    config::{AppConfig, AppMode},
    context::ProjectConfig,
    danger_zone, debt_invoice, decay, deps_shamer,
    educational::EducationalAdvisor,
    hall_of_shame::HallOfShame,
    last_words,
    llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastProvider},
    personality, personas, pr_title_hunter, radar,
    reporter::Reporter,
    team_roast,
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

    /// Analyze project dependencies and shame bad practices
    #[command(alias = "ds")]
    DepsShamer(DepsShamerArgs),

    /// Scan PR titles from merge commits and roast bad ones
    #[command(alias = "pr")]
    PrTitleHunter(PrTitleHunterArgs),

    /// Run all tools and generate a combined garbage report
    Scan(ScanArgs),

    /// Generate an SVG score badge
    Badge(BadgeArgs),

    /// Show quality score trend over time
    Trend(TrendArgs),

    /// Find legacy TODO/FIXME/HACK comments and their age
    #[command(alias = "lw")]
    LastWords(LastWordsArgs),

    /// Generate a technical debt invoice with cost estimates
    #[command(alias = "debt")]
    DebtInvoice(DebtInvoiceArgs),

    /// Analyze your developer personality based on code patterns
    Personality(PersonalityArgs),

    /// Analyze project quality decay over git history
    Decay(DecayArgs),

    /// Generate a code autopsy report (root cause analysis)
    Autopsy(AutopsyArgs),

    /// Generate a code smell radar chart
    Radar(RadarArgs),

    /// Generate a CI-style PR review comment
    CiBot(CiBotArgs),

    /// Analyze code with a specific roast personality
    Persona(PersonaArgs),

    /// Identify the most dangerous files in the codebase
    #[command(alias = "dz")]
    DangerZone(DangerZoneArgs),

    /// Per-developer team analysis and roasting
    TeamRoast(TeamRoastArgs),
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

    /// Path to .garbage-code-hunter.toml config file (auto-discovered if omitted)
    #[arg(long)]
    project_config: Option<PathBuf>,

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

#[derive(Parser)]
struct DepsShamerArgs {
    /// Path to project directory (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,
}

#[derive(Parser)]
struct PrTitleHunterArgs {
    /// Path to git repository (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum number of merge commits to analyze
    #[arg(short, long, default_value = "50")]
    limit: usize,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Remote repo in owner/repo format (enables GitHub API mode)
    #[arg(short, long)]
    repo: Option<String>,

    /// PR state filter for GitHub API: open, closed, all
    #[arg(long, default_value = "all")]
    state: String,

    /// GitHub token for authentication (or set GITHUB_TOKEN env var)
    #[arg(long)]
    token: Option<String>,

    /// Filter by PR author (GitHub API mode)
    #[arg(long)]
    author: Option<String>,
}

#[derive(Parser)]
struct ScanArgs {
    /// Path to project directory (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Save scan result to history for trend tracking
    #[arg(long)]
    save: bool,

    /// Path to .garbage-code-hunter.toml config file (auto-discovered if omitted)
    #[arg(long)]
    project_config: Option<PathBuf>,
}

#[derive(Parser)]
struct TrendArgs {
    /// Number of recent scans to show
    #[arg(short, long, default_value = "10")]
    last: usize,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,
}

#[derive(Parser)]
struct BadgeArgs {
    /// Output file path
    #[arg(short, long, default_value = "badge.svg")]
    output: PathBuf,

    /// Badge style: flat or plastic
    #[arg(long, default_value = "flat")]
    style: String,

    /// Use a specific score instead of running analysis
    #[arg(long)]
    score: Option<f64>,

    /// Path to project directory (used when --score is not set)
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(Parser)]
struct LastWordsArgs {
    /// Path to scan (file or directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Try to get age of comments via git blame (slower)
    #[arg(long)]
    age: bool,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct DebtInvoiceArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct PersonalityArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct DecayArgs {
    /// Path to git repository
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct AutopsyArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct RadarArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,

    /// Output SVG file path
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Parser)]
struct CiBotArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct PersonaArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Persona: linux-kernel, silicon-valley, japanese-enterprise, rust-fanatic
    #[arg(short, long, default_value = "linux-kernel")]
    persona: String,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct DangerZoneArgs {
    /// Path to project directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
}

#[derive(Parser)]
struct TeamRoastArgs {
    /// Path to git repository
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum number of commits to analyze
    #[arg(short, long, default_value = "100")]
    limit: usize,

    /// Output format (terminal, json)
    #[arg(short = 'f', long, default_value = "terminal")]
    format: String,

    /// Output language (zh-CN, en-US)
    #[arg(short, long, default_value = "en-US")]
    lang: String,
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
        Some(Commands::DepsShamer(args)) => run_deps_shamer(args),
        Some(Commands::PrTitleHunter(args)) => run_pr_title_hunter(args),
        Some(Commands::Scan(args)) => run_scan(args),
        Some(Commands::Badge(args)) => run_badge(args),
        Some(Commands::Trend(args)) => run_trend(args),
        Some(Commands::LastWords(args)) => run_last_words(args),
        Some(Commands::DebtInvoice(args)) => run_debt_invoice(args),
        Some(Commands::Personality(args)) => run_personality(args),
        Some(Commands::Decay(args)) => run_decay(args),
        Some(Commands::Autopsy(args)) => run_autopsy(args),
        Some(Commands::Radar(args)) => run_radar(args),
        Some(Commands::CiBot(args)) => run_ci_bot(args),
        Some(Commands::Persona(args)) => run_persona(args),
        Some(Commands::DangerZone(args)) => run_danger_zone(args),
        Some(Commands::TeamRoast(args)) => run_team_roast(args),
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

fn run_deps_shamer(args: DepsShamerArgs) {
    use deps_shamer::{run, OutputFormat};

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match run(&args.path, &format) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_pr_title_hunter(args: PrTitleHunterArgs) {
    use pr_title_hunter::{run, run_remote, OutputFormat};

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    // Remote mode if --repo is provided
    if let Some(repo) = &args.repo {
        let token = args
            .token
            .clone()
            .or_else(|| std::env::var("GITHUB_TOKEN").ok());
        let config = pr_title_hunter::github::GitHubConfig {
            repo: repo.clone(),
            state: args.state.clone(),
            limit: args.limit,
            token,
            author: args.author.clone(),
        };
        match run_remote(&config, &format) {
            Ok(output) => print!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Local mode (default)
    match run(&args.path, args.limit, &format) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn derive_count_score(count: f64, weight: f64) -> f64 {
    (100.0 - count * weight).clamp(0.0, 100.0)
}

fn derive_radar_score(val: &serde_json::Value) -> f64 {
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
        sum / n
    } else {
        100.0
    }
}

fn derive_danger_zone_score(val: &serde_json::Value) -> f64 {
    match val["files"].as_array() {
        Some(arr) if !arr.is_empty() => {
            let sum: f64 = arr.iter().filter_map(|f| f["risk_score"].as_f64()).sum();
            sum / arr.len() as f64
        }
        _ => 100.0,
    }
}

/// Run all 14 scan tools in parallel using scoped threads.
/// Returns ordered Vec of (tool_name, score, item_count, detail_string).
/// A negative score means the tool was skipped/failed.
fn run_all_tools(
    path: &std::path::Path,
    lang: &str,
    config: &ProjectConfig,
) -> Vec<(&'static str, f64, usize, String)> {
    // We use std::thread::scope so closures can borrow `path`, `lang`, and `config`.
    std::thread::scope(|s| {
        // Spawn all tools; each returns its result tuple.
        let h_code = s.spawn(|| {
            let analyzer = CodeAnalyzer::with_config(&[], lang, config.clone());
            let issues = analyzer.analyze_path(path);
            let path_buf = path.to_path_buf();
            let (_, total_lines) = calculate_metrics(&path_buf, &[]);
            let score = if total_lines > 0 {
                let density = issues.len() as f64 / total_lines as f64 * 1000.0;
                (100.0 - density * 2.0).clamp(0.0, 100.0)
            } else {
                100.0
            };
            (
                "code-hunter",
                score,
                issues.len(),
                format!("{} issues", issues.len()),
            )
        });

        let h_commit = s.spawn(|| {
            match commit_roaster::run(
                path,
                &commit_roaster::analyzer::AnalyzerConfig::default(),
                &OutputFormat::Json,
            ) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let score = v["score"].as_f64().unwrap_or(100.0);
                        let count = v["total_commits"].as_u64().unwrap_or(0) as usize;
                        ("commit-roaster", score, count, format!("{} commits", count))
                    }
                    Err(_) => ("commit-roaster", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("commit-roaster", -1.0, 0, e.to_string()),
            }
        });

        let h_deps = s.spawn(|| match deps_shamer::run(path, &OutputFormat::Json) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let score = v["score"].as_f64().unwrap_or(100.0);
                    let count = v["total_deps"].as_u64().unwrap_or(0) as usize;
                    ("deps-shamer", score, count, format!("{} deps", count))
                }
                Err(_) => ("deps-shamer", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("deps-shamer", -1.0, 0, e.to_string()),
        });

        let h_pr = s.spawn(
            || match pr_title_hunter::run(path, 50, &OutputFormat::Json) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let score = v["score"].as_f64().unwrap_or(100.0);
                        let count = v["total_prs"].as_u64().unwrap_or(0) as usize;
                        ("pr-title-hunter", score, count, format!("{} PRs", count))
                    }
                    Err(_) => ("pr-title-hunter", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("pr-title-hunter", -1.0, 0, e.to_string()),
            },
        );

        let h_lw = s.spawn(
            || match last_words::run(path, &OutputFormat::Json, false, lang) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let total = v["total"].as_u64().unwrap_or(0) as f64;
                        let score = derive_count_score(total, 0.005);
                        (
                            "last-words",
                            score,
                            total as usize,
                            format!("{} stale comments", total as usize),
                        )
                    }
                    Err(_) => ("last-words", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("last-words", -1.0, 0, e.to_string()),
            },
        );

        let h_debt = s.spawn(
            || match debt_invoice::run(path, &OutputFormat::Json, lang) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let hours = v["total_hours"].as_f64().unwrap_or(0.0);
                        let score = derive_count_score(hours, 0.05);
                        (
                            "debt-invoice",
                            score,
                            hours as usize,
                            format!("{:.1} hours", hours),
                        )
                    }
                    Err(_) => ("debt-invoice", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("debt-invoice", -1.0, 0, e.to_string()),
            },
        );

        let h_pers = s.spawn(|| match personality::run(path, &OutputFormat::Json, lang) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let score = v["score"].as_f64().unwrap_or(50.0);
                    let title = v["title"].as_str().unwrap_or("unknown").to_string();
                    ("personality", score, 0, title)
                }
                Err(_) => ("personality", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("personality", -1.0, 0, e.to_string()),
        });

        let h_decay = s.spawn(|| match decay::run(path, &OutputFormat::Json, lang) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let score = v["current_health"].as_f64().unwrap_or(50.0);
                    ("decay", score, 0, format!("health {:.0}", score))
                }
                Err(_) => ("decay", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("decay", -1.0, 0, e.to_string()),
        });

        let h_autopsy = s.spawn(|| match autopsy::run(path, &OutputFormat::Json, lang) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let factors = v["contributing_factors"]
                        .as_array()
                        .map(|a| a.len())
                        .unwrap_or(0) as f64;
                    let score = derive_count_score(factors, 3.0);
                    (
                        "autopsy",
                        score,
                        factors as usize,
                        format!("{} factors", factors as usize),
                    )
                }
                Err(_) => ("autopsy", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("autopsy", -1.0, 0, e.to_string()),
        });

        let h_radar = s.spawn(|| match radar::run(path, &OutputFormat::Json, lang, None) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let score = derive_radar_score(&v);
                    ("radar", score, 6, "6 dimensions".into())
                }
                Err(_) => ("radar", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("radar", -1.0, 0, e.to_string()),
        });

        let h_ci = s.spawn(|| match ci_bot::run(path, &OutputFormat::Json, lang) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let issues = v["total_issues"].as_u64().unwrap_or(0) as f64;
                    let score = derive_count_score(issues, 0.01);
                    (
                        "ci-bot",
                        score,
                        issues as usize,
                        format!("{} issues", issues as usize),
                    )
                }
                Err(_) => ("ci-bot", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("ci-bot", -1.0, 0, e.to_string()),
        });

        let h_persona = s.spawn(|| {
            match personas::run(
                path,
                personas::Persona::LinuxKernel,
                &OutputFormat::Json,
                lang,
            ) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let issues = v["total_issues"].as_u64().unwrap_or(0) as f64;
                        let score = derive_count_score(issues, 0.005);
                        (
                            "persona",
                            score,
                            issues as usize,
                            format!("{} issues", issues as usize),
                        )
                    }
                    Err(_) => ("persona", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("persona", -1.0, 0, e.to_string()),
            }
        });

        let h_danger = s.spawn(|| match danger_zone::run(path, &OutputFormat::Json, lang) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => {
                    let file_count = v["files"].as_array().map(|a| a.len()).unwrap_or(0);
                    let score = derive_danger_zone_score(&v);
                    (
                        "danger-zone",
                        score,
                        file_count,
                        format!("{} risky files", file_count),
                    )
                }
                Err(_) => ("danger-zone", -1.0, 0, "json parse error".into()),
            },
            Err(e) => ("danger-zone", -1.0, 0, e.to_string()),
        });

        let h_team = s.spawn(
            || match team_roast::run(path, &OutputFormat::Json, 20, lang) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let members = v["members"].as_array().map(|a| a.len()).unwrap_or(0) as f64;
                        let score = derive_count_score(members, 2.0);
                        (
                            "team-roast",
                            score,
                            members as usize,
                            format!("{} members", members as usize),
                        )
                    }
                    Err(_) => ("team-roast", -1.0, 0, "json parse error".into()),
                },
                Err(e) => ("team-roast", -1.0, 0, e.to_string()),
            },
        );

        // Join in deterministic order
        vec![
            h_code
                .join()
                .unwrap_or(("code-hunter", -1.0, 0, "thread panicked".into())),
            h_commit
                .join()
                .unwrap_or(("commit-roaster", -1.0, 0, "thread panicked".into())),
            h_deps
                .join()
                .unwrap_or(("deps-shamer", -1.0, 0, "thread panicked".into())),
            h_pr.join()
                .unwrap_or(("pr-title-hunter", -1.0, 0, "thread panicked".into())),
            h_lw.join()
                .unwrap_or(("last-words", -1.0, 0, "thread panicked".into())),
            h_debt
                .join()
                .unwrap_or(("debt-invoice", -1.0, 0, "thread panicked".into())),
            h_pers
                .join()
                .unwrap_or(("personality", -1.0, 0, "thread panicked".into())),
            h_decay
                .join()
                .unwrap_or(("decay", -1.0, 0, "thread panicked".into())),
            h_autopsy
                .join()
                .unwrap_or(("autopsy", -1.0, 0, "thread panicked".into())),
            h_radar
                .join()
                .unwrap_or(("radar", -1.0, 0, "thread panicked".into())),
            h_ci.join()
                .unwrap_or(("ci-bot", -1.0, 0, "thread panicked".into())),
            h_persona
                .join()
                .unwrap_or(("persona", -1.0, 0, "thread panicked".into())),
            h_danger
                .join()
                .unwrap_or(("danger-zone", -1.0, 0, "thread panicked".into())),
            h_team
                .join()
                .unwrap_or(("team-roast", -1.0, 0, "thread panicked".into())),
        ]
    })
}

fn run_scan(args: ScanArgs) {
    use colored::Colorize;

    let is_json = args.format == "json";

    if !is_json {
        println!("\n{}\n", "\u{1f50d} Running Full Garbage Scan...".bold());
        println!("{}", "\u{2501}".repeat(50));
    }

    // Load project config (--project-config flag or auto-discover .garbage-code-hunter.toml)
    let project_config = match &args.project_config {
        Some(path) => ProjectConfig::load_from_file(path).unwrap_or_default(),
        None => ProjectConfig::discover(&args.path),
    };

    // Run all tools in parallel
    let results = run_all_tools(&args.path, &args.lang, &project_config);

    // Print per-tool status (preserves ordering)
    if !is_json {
        for (name, score, _count, detail) in &results {
            if *score >= 0.0 {
                println!("  \u{2705} {}: {:.0}/100 ({})", name, score, detail);
            } else {
                println!("  \u{26a0}\u{fe0f} {}: skipped ({})", name, detail);
            }
        }
    }

    // Build all_scores (filter out skipped tools with score < 0)
    let all_scores: Vec<(&str, f64, usize)> = results
        .iter()
        .filter(|(_, score, _, _)| *score >= 0.0)
        .map(|(name, score, count, _)| (*name, *score, *count))
        .collect();

    // Combined report
    if is_json {
        let output = serde_json::json!({
            "tools": all_scores.iter().map(|(name, score, count)| {
                serde_json::json!({
                    "tool": name,
                    "score": score,
                    "item_count": count,
                })
            }).collect::<Vec<_>>(),
            "overall_score": overall_score(&all_scores),
        });
        if let Ok(json) = serde_json::to_string_pretty(&output) {
            println!("{}", json);
        }
    } else {
        println!("\n{}", "\u{1f4e6} Garbage Report \u{1f4e6}".bold());
        println!("{}", "\u{2501}".repeat(50));
        println!("\n  {}", "Tool Summary".bold());
        println!("  {}", "\u{2500}".repeat(40));

        for (name, score, count) in &all_scores {
            let score_str = if *score >= 80.0 {
                format!("{:.0}", score).green()
            } else if *score >= 60.0 {
                format!("{:.0}", score).yellow()
            } else {
                format!("{:.0}", score).red()
            };
            println!("  {:<20} {:>6}/100  ({} items)", name, score_str, count);
        }

        let overall = overall_score(&all_scores);
        let overall_str = if overall >= 80.0 {
            format!("{:.0}/100", overall).green().bold()
        } else if overall >= 60.0 {
            format!("{:.0}/100", overall).yellow().bold()
        } else {
            format!("{:.0}/100", overall).red().bold()
        };
        println!("\n  \u{1f3c6} Overall Garbage Score: {}", overall_str);
        println!();
    }

    // Save to history if requested
    if args.save {
        use garbage_code_hunter::trend;
        let path_str = args.path.to_string_lossy().to_string();
        let overall = overall_score(&all_scores);
        let tools: Vec<garbage_code_hunter::trend::history::ToolScore> = all_scores
            .iter()
            .map(
                |(name, score, count)| garbage_code_hunter::trend::history::ToolScore {
                    name: name.to_string(),
                    score: *score,
                    item_count: *count,
                },
            )
            .collect();
        match trend::save_scan(&path_str, overall, tools) {
            Ok(record) => {
                if !is_json {
                    println!("  \u{1f4be} Scan saved to history ({})", record.timestamp);
                }
            }
            Err(e) => {
                if !is_json {
                    eprintln!("  \u{26a0}\u{fe0f} Failed to save history: {}", e);
                }
            }
        }
    }
}

fn quick_scan_score(path: &std::path::Path, config: &ProjectConfig) -> f64 {
    // Run 7 key tools in parallel for fast badge scoring
    let results: Vec<(&str, f64, usize)> = std::thread::scope(|s| {
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
            let cfg = commit_roaster::analyzer::AnalyzerConfig::default();
            match commit_roaster::run(path, &cfg, &OutputFormat::Json) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => ("commit-roaster", v["score"].as_f64().unwrap_or(100.0), 0),
                    Err(_) => ("commit-roaster", 100.0, 0),
                },
                Err(_) => ("commit-roaster", 100.0, 0),
            }
        });

        let h_deps = s.spawn(|| match deps_shamer::run(path, &OutputFormat::Json) {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => ("deps-shamer", v["score"].as_f64().unwrap_or(100.0), 0),
                Err(_) => ("deps-shamer", 100.0, 0),
            },
            Err(_) => ("deps-shamer", 100.0, 0),
        });

        let h_pr = s.spawn(
            || match pr_title_hunter::run(path, 50, &OutputFormat::Json) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => ("pr-title-hunter", v["score"].as_f64().unwrap_or(100.0), 0),
                    Err(_) => ("pr-title-hunter", 100.0, 0),
                },
                Err(_) => ("pr-title-hunter", 100.0, 0),
            },
        );

        let h_decay = s.spawn(|| match decay::run(path, &OutputFormat::Json, "en-US") {
            Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(v) => ("decay", v["current_health"].as_f64().unwrap_or(50.0), 0),
                Err(_) => ("decay", 50.0, 0),
            },
            Err(_) => ("decay", 50.0, 0),
        });

        let h_radar = s.spawn(
            || match radar::run(path, &OutputFormat::Json, "en-US", None) {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => ("radar", derive_radar_score(&v), 0),
                    Err(_) => ("radar", 50.0, 0),
                },
                Err(_) => ("radar", 50.0, 0),
            },
        );

        let h_debt = s.spawn(
            || match debt_invoice::run(path, &OutputFormat::Json, "en-US") {
                Ok(json) => match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(v) => {
                        let hours = v["total_hours"].as_f64().unwrap_or(0.0);
                        ("debt-invoice", derive_count_score(hours, 0.5), 0)
                    }
                    Err(_) => ("debt-invoice", 100.0, 0),
                },
                Err(_) => ("debt-invoice", 100.0, 0),
            },
        );

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

fn run_badge(args: BadgeArgs) {
    use garbage_code_hunter::badge;
    use garbage_code_hunter::badge::generator::BadgeStyle;

    let style = match args.style.as_str() {
        "plastic" => BadgeStyle::Plastic,
        _ => BadgeStyle::Flat,
    };

    let score = if let Some(s) = args.score {
        s
    } else {
        let config = ProjectConfig::discover(&args.path);
        quick_scan_score(&args.path, &config)
    };

    match badge::run(score, &args.output, &style) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_trend(args: TrendArgs) {
    use garbage_code_hunter::trend;
    use garbage_code_hunter::trend::OutputFormat;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match trend::run(args.last, &format) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn overall_score(scores: &[(&str, f64, usize)]) -> f64 {
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
            project_config: None,
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

    // Load project config (--project-config flag or auto-discover .garbage-code-hunter.toml)
    let project_config = match &args.project_config {
        Some(path) => ProjectConfig::load_from_file(path).unwrap_or_default(),
        None => ProjectConfig::discover(&args.path),
    };

    let analyzer = CodeAnalyzer::with_config(&args.exclude, &args.lang, project_config);
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

fn run_last_words(args: LastWordsArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::last_words;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match last_words::run(&args.path, &format, args.age, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_debt_invoice(args: DebtInvoiceArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::debt_invoice;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match debt_invoice::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_personality(args: PersonalityArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::personality;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match personality::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_decay(args: DecayArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::decay;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match decay::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_autopsy(args: AutopsyArgs) {
    use garbage_code_hunter::autopsy;
    use garbage_code_hunter::common::OutputFormat;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match autopsy::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_radar(args: RadarArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::radar;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match radar::run(&args.path, &format, &args.lang, args.output.as_deref()) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_ci_bot(args: CiBotArgs) {
    use garbage_code_hunter::ci_bot;
    use garbage_code_hunter::common::OutputFormat;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match ci_bot::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_persona(args: PersonaArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::personas;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    let persona = match personas::Persona::parse_persona(&args.persona) {
        Some(p) => p,
        None => {
            eprintln!("Unknown persona '{}'. Available: linux-kernel, silicon-valley, japanese-enterprise, rust-fanatic", args.persona);
            std::process::exit(1);
        }
    };

    match personas::run(&args.path, persona, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_danger_zone(args: DangerZoneArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::danger_zone;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match danger_zone::run(&args.path, &format, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_team_roast(args: TeamRoastArgs) {
    use garbage_code_hunter::common::OutputFormat;
    use garbage_code_hunter::team_roast;

    let format = match args.format.as_str() {
        "json" => OutputFormat::Json,
        _ => OutputFormat::Terminal,
    };

    match team_roast::run(&args.path, &format, args.limit, &args.lang) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
