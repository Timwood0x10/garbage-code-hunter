use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "garbage-code-hunter")]
#[command(about = "A humorous Rust code quality detector that roasts your garbage code \u{1f525}")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Analyze(AnalyzeArgs),
    #[command(alias = "cr")]
    CommitRoaster(CommitRoasterArgs),
    #[command(alias = "ds")]
    DepsShamer(DepsShamerArgs),
    #[command(alias = "pr")]
    PrTitleHunter(PrTitleHunterArgs),
    Scan(ScanArgs),
    Badge(BadgeArgs),
    Trend(TrendArgs),
    #[command(alias = "lw")]
    LastWords(LastWordsArgs),
    #[command(alias = "debt")]
    DebtInvoice(DebtInvoiceArgs),
    Personality(PersonalityArgs),
    Decay(DecayArgs),
    Autopsy(AutopsyArgs),
    Radar(RadarArgs),
    CiBot(CiBotArgs),
    Persona(PersonaArgs),
    #[command(alias = "dz")]
    DangerZone(DangerZoneArgs),
    TeamRoast(TeamRoastArgs),
}

#[derive(Parser)]
pub struct AnalyzeArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub harsh: bool,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short = 'i', long, default_value = "5")]
    pub issues: usize,
    #[arg(short = 's', long)]
    pub summary: bool,
    #[arg(short, long)]
    pub brief: bool,
    #[arg(short, long)]
    pub markdown: bool,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
    #[arg(short, long)]
    pub exclude: Vec<String>,
    #[arg(long)]
    pub educational: bool,
    #[arg(long)]
    pub project_config: Option<PathBuf>,
    #[arg(long)]
    pub hall_of_shame: bool,
    #[arg(long)]
    pub suggestions: bool,
    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub llm: bool,
    #[arg(long, default_value = "ollama")]
    pub llm_provider: String,
    #[arg(long)]
    pub llm_endpoint: Option<String>,
    #[arg(long)]
    pub llm_model: Option<String>,
    #[arg(long)]
    pub llm_api_key: Option<String>,
    #[arg(long, default_value = "30")]
    pub llm_timeout: u64,
    #[arg(long)]
    pub config: Option<PathBuf>,
}

impl Default for AnalyzeArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            harsh: false,
            verbose: false,
            issues: 5,
            summary: false,
            brief: false,
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

#[derive(Parser)]
pub struct CommitRoasterArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short, long, default_value = "50")]
    pub limit: usize,
    #[arg(short, long)]
    pub author: Option<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub until: Option<String>,
    #[arg(short, long)]
    pub branch: Option<String>,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
}

#[derive(Parser)]
pub struct DepsShamerArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
}

#[derive(Parser)]
pub struct PrTitleHunterArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short, long, default_value = "50")]
    pub limit: usize,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long)]
    pub repo: Option<String>,
    #[arg(long, default_value = "all")]
    pub state: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub author: Option<String>,
}

#[derive(Parser)]
pub struct ScanArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(long)]
    pub save: bool,
    #[arg(long)]
    pub project_config: Option<PathBuf>,
}

#[derive(Parser)]
pub struct TrendArgs {
    #[arg(short, long, default_value = "10")]
    pub last: usize,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
}

#[derive(Parser)]
pub struct BadgeArgs {
    #[arg(short, long, default_value = "badge.svg")]
    pub output: PathBuf,
    #[arg(long, default_value = "flat")]
    pub style: String,
    #[arg(long)]
    pub score: Option<f64>,
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct LastWordsArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(long)]
    pub age: bool,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct DebtInvoiceArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct PersonalityArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct DecayArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct AutopsyArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct RadarArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Parser)]
pub struct CiBotArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct PersonaArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short, long, default_value = "linux-kernel")]
    pub persona: String,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct DangerZoneArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short, long, default_value = "en-US")]
    pub lang: String,
}

#[derive(Parser)]
pub struct TeamRoastArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    #[arg(short, long, default_value = "100")]
    pub limit: usize,
    #[arg(short = 'f', long, default_value = "terminal")]
    pub format: String,
    #[arg(short = 'L', long, default_value = "en-US")]
    pub lang: String,
}
