use clap::Parser;
use tracing_subscriber::EnvFilter;

use garbage_code_hunter::{
    analyzer::{CodeAnalyzer, CodeIssue},
    autopsy, ci_bot, commit_roaster,
    common::OutputFormat,
    config::{AppConfig, AppMode},
    context::ProjectConfig,
    danger_zone, debt_invoice, decay, deps_shamer,
    detectors::{
        CodeSmellsDetector, DuplicationDetector, HotfixCultureDetector, LegacyCodeDetector,
        LineCountSmellDetector, NamingChaosDetector, NestedHellDetector, OverEngineeringDetector,
        PanicAddictionDetector, TodoMountainDetector,
    },
    educational::EducationalAdvisor,
    hall_of_shame::HallOfShame,
    last_words,
    llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastProvider},
    personality, personas, pr_title_hunter, radar,
    reporter::Reporter,
    team_roast,
};

mod args;
mod helpers;

use args::*;
use helpers::*;

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

    let analyzer = CodeAnalyzer::with_config(&args.exclude, &args.lang, project_config)
        .with_detectors(vec![
            Box::new(PanicAddictionDetector::new()),
            Box::new(NamingChaosDetector::new()),
            Box::new(NestedHellDetector::new()),
            Box::new(HotfixCultureDetector::new()),
            Box::new(OverEngineeringDetector::new()),
            Box::new(CodeSmellsDetector::new()),
            Box::new(DuplicationDetector::new()),
            Box::new(LegacyCodeDetector::new()),
            Box::new(TodoMountainDetector::new()),
            Box::new(LineCountSmellDetector::new()),
        ]);
    let full = analyzer.analyze_full(&args.path);

    let findings = full.findings;
    let file_count = full.file_count;
    let total_lines = full.total_lines;

    let issues: Vec<CodeIssue> = findings.iter().map(|f| f.to_code_issue()).collect();
    let spread = analyzer.infection_spread();
    let style_ir_files: Vec<AnalyzeJsonFile> = full
        .style_ir_files
        .into_iter()
        .map(|f| AnalyzeJsonFile {
            file_path: f.file_path,
            style_ir_summary: f.summary,
        })
        .collect();
    let style_ir_summary = summarize_style_ir_files(&style_ir_files);

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

    let direct_scores = analyzer.direct_signal_scores();
    let reporter = Reporter::new(
        args.harsh,
        args.verbose,
        args.top,
        args.issues,
        args.summary,
        args.brief,
        args.markdown,
        &args.lang,
        roast_provider,
    )
    .with_direct_scores(direct_scores)
    .with_style_ir_summary(style_ir_summary.clone())
    .with_friend_feedback(true);

    // Handle JSON output format
    if args.format == "json" {
        output_json(&issues, &style_ir_files, style_ir_summary.as_ref());
        return;
    }

    // Handle GitHub Actions output format
    if args.format == "github-actions" || args.format == "github" {
        output_github_actions(&issues);
        return;
    }

    if args.educational || args.hall_of_shame || args.suggestions {
        reporter.report_with_spread(issues.clone(), file_count, total_lines, &spread);

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
        reporter.report_with_spread(issues, file_count, total_lines, &spread);
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
