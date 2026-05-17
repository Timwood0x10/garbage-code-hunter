use crate::analyzer::CodeIssue;
use crate::scoring::CodeQualityScore;
use crate::signals::{StyleProfile, StyleSignal};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub type SpreadTarget = (String, usize, Vec<String>);
pub type SpreadChain = (String, Vec<SpreadTarget>);

pub struct ProjectPersonality {
    pub project_type: &'static str,
    pub emoji: &'static str,
    pub core_traits: Vec<&'static str>,
    pub emotional_state: &'static str,
    pub code_philosophy: &'static str,
    pub threat_level: &'static str,
    pub lore: Vec<&'static str>,
}

pub struct AutopsyReport {
    pub cause_of_death: &'static str,
    pub patient_condition: &'static str,
    pub corrupted_regions: Vec<(String, f64)>,
    pub final_words: &'static str,
    pub spread_chains: Vec<SpreadChain>,
}

const ALL_LORE: &[&str] = &[
    "nobody remembers why this function exists",
    "this file has more authors than tests",
    "touching this module may awaken ancient bugs",
    "the original developer has left the company",
    "three developers entered main.rs, only one returned",
    "this comment predates the git history",
    "the architecture decision was made at 3am",
    "this code has survived three rewrites",
    "legacy code — proceed with caution and incense",
    "the spec was 'make it work' and it shows",
];

fn pick_lore() -> Vec<&'static str> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as usize;
    vec![ALL_LORE[seed % ALL_LORE.len()]]
}

pub fn analyze(
    issues: &[CodeIssue],
    score: &CodeQualityScore,
    _file_count: usize,
    _spread: &HashMap<String, Vec<SpreadTarget>>,
) -> (ProjectPersonality, AutopsyReport) {
    let profile = StyleProfile::from_signal_scores(score.signal_scores.clone());
    let personality_type = profile.infer_personality_type();

    let threat_level = if score.total_score >= 80.0 {
        "☢ CRITICAL"
    } else if score.total_score >= 60.0 {
        "⚠ HIGH"
    } else if score.total_score >= 40.0 {
        "⚠ ELEVATED"
    } else {
        "🟢 MODERATE"
    };

    let total_f = issues.len().max(1) as f64;
    let files_per_category = categorize_files(issues, total_f);

    let mut spread_list: Vec<_> = _spread
        .iter()
        .map(|(file, targets)| (file.clone(), targets.clone()))
        .collect();
    spread_list.sort_by_key(|(_, targets)| std::cmp::Reverse(targets.len()));
    spread_list.truncate(5);

    let dup = profile.score(StyleSignal::Duplication);
    let panic = profile.score(StyleSignal::PanicAddiction);
    let nested = profile.score(StyleSignal::NestedHell);

    let (personality, autopsy) = match personality_type {
        "The Copy-Paste Artist" => (
            ProjectPersonality {
                project_type: "The Copy-Paste Artist",
                emoji: "📋",
                core_traits: vec![
                    "Ctrl+C, Ctrl+V is your IDE's most used shortcut",
                    "Why abstract when you can duplicate",
                    "Same bug in 5 places = 5x debugging fun",
                    "DRY stands for 'Don't Repeat... wait, too late'",
                ],
                emotional_state: "numb from repetitive work",
                code_philosophy: "it worked once, it'll work again",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "uncontrolled duplication metastasis",
                patient_condition: if dup >= 20.0 {
                    "terminal — palliative care recommended"
                } else {
                    "critical but treatable"
                },
                corrupted_regions: files_per_category
                    .get("duplication")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "just one more hotfix",
            },
        ),
        "The YOLO Engineer" => (
            ProjectPersonality {
                project_type: "The YOLO Engineer",
                emoji: "🤘",
                core_traits: vec![
                    "Believes every Result is Ok",
                    "unwrap() used more often than error handling",
                    "Production incidents are just 'surprise features'",
                    "Has never met a None they couldn't ignore",
                ],
                emotional_state: "denial",
                code_philosophy: "it won't crash in production",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "panic-driven development",
                patient_condition: if panic >= 15.0 {
                    "critical — needs immediate Result therapy"
                } else {
                    "stable but concerning"
                },
                corrupted_regions: files_per_category
                    .get("unwrap-abuse")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "i'll add error handling later",
            },
        ),
        "The Trait Wizard" => (
            ProjectPersonality {
                project_type: "The Trait Wizard",
                emoji: "🧙",
                core_traits: vec![
                    "Loves building pyramids of doom",
                    "Each function is a journey through layers of abstraction",
                    "Indentation is a competitive sport",
                    "Thinks 'extract method' is for amateurs",
                ],
                emotional_state: "proud of the complexity",
                code_philosophy: "if it's not nested, it's not sophisticated",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "complexity collapse — nesting level exceeded event horizon",
                patient_condition: if nested >= 15.0 {
                    "severe — god functions threatening readability"
                } else {
                    "moderate — some functions need splitting"
                },
                corrupted_regions: files_per_category
                    .get("complexity")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "i can still understand it",
            },
        ),
        "The Legacy Necromancer" => (
            ProjectPersonality {
                project_type: "The Legacy Necromancer",
                emoji: "☢",
                core_traits: vec![
                    "Why use many word when few letter do trick",
                    "Variables named like chess coordinates",
                    "'data', 'temp', 'val' — the holy trinity of naming",
                    "Has never heard of domain-driven design",
                ],
                emotional_state: "confident in their abbreviations",
                code_philosophy: "comments are for people who can't read code",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "semantic starvation — variable names lost all meaning",
                patient_condition: "chronic but manageable",
                corrupted_regions: files_per_category
                    .get("naming")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "temp2 should work",
            },
        ),
        "The Hotfix Mercenary" => (
            ProjectPersonality {
                project_type: "The Hotfix Mercenary",
                emoji: "🔥",
                core_traits: vec![
                    "More TODOs than actual features",
                    "'TODO: fix this later' — later never came",
                    "Commits with 'temp', 'test', 'asdf' messages",
                    "WIP is a lifestyle, not a branch name",
                ],
                emotional_state: "overwhelmed but optimistic",
                code_philosophy: "future me will deal with it",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "TODO accumulation — promises exceeded delivery capacity",
                patient_condition: "stable with high technical debt interest",
                corrupted_regions: files_per_category.get("todo").cloned().unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "i'll fix it in the next sprint",
            },
        ),
        "The Startup Survivor" => (
            ProjectPersonality {
                project_type: "The Startup Survivor",
                emoji: "🚀",
                core_traits: vec![
                    "Shipped fast, now paying the interest",
                    "Copy-paste driven development",
                    "Production incidents are 'learning experiences'",
                    "Technical debt is just 'future velocity'",
                ],
                emotional_state: "battle-scarred but still shipping",
                code_philosophy: "move fast and definitely break things",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "growth-at-all-costs syndrome",
                patient_condition: "critical — needs stabilization sprint",
                corrupted_regions: files_per_category
                    .get("duplication")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "we'll fix it after the next funding round",
            },
        ),
        "The Academic Wizard" => (
            ProjectPersonality {
                project_type: "The Academic Wizard",
                emoji: "🧙",
                core_traits: vec![
                    "Loves building pyramids of abstraction",
                    "Each function is a thesis in disguise",
                    "Naming conventions from another dimension",
                    "If it's not complex, it's not 'elegant'",
                ],
                emotional_state: "intellectually satisfied, practically lost",
                code_philosophy: "the theory is beautiful, the practice is secondary",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "abstraction overdose — complexity exceeded comprehension",
                patient_condition: "chronic — needs simplification therapy",
                corrupted_regions: files_per_category
                    .get("complexity")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "it's actually quite simple once you understand the theory",
            },
        ),
        _ => (
            ProjectPersonality {
                project_type: "The Enterprise Bureaucrat",
                emoji: "🏢",
                core_traits: vec![
                    "A balanced mix of code smells",
                    "Not great at anything, not terrible at anything",
                    "Jack of all trades, master of technical debt",
                ],
                emotional_state: "professional detachment",
                code_philosophy: "it compiles, it ships",
                threat_level,
                lore: pick_lore(),
            },
            AutopsyReport {
                cause_of_death: "death by a thousand paper cuts",
                patient_condition: "fair — routine maintenance recommended",
                corrupted_regions: files_per_category
                    .get("duplication")
                    .cloned()
                    .unwrap_or_default(),
                spread_chains: spread_list.clone(),
                final_words: "we'll refactor next quarter",
            },
        ),
    };

    (personality, autopsy)
}

fn categorize_files(
    issues: &[CodeIssue],
    total_f: f64,
) -> HashMap<&'static str, Vec<(String, f64)>> {
    let mut dups: HashMap<String, usize> = HashMap::new();
    let mut naming: HashMap<String, usize> = HashMap::new();
    let mut complexity: HashMap<String, usize> = HashMap::new();
    let mut unwrap: HashMap<String, usize> = HashMap::new();
    let mut todo_files: HashMap<String, usize> = HashMap::new();

    for issue in issues {
        let name = issue.file_path.to_string_lossy().to_string();
        let r = issue.rule_name.as_str();
        if r.contains("duplicat") {
            *dups.entry(name.clone()).or_insert(0) += 1;
        } else if r.contains("naming")
            || r == "terrible-naming"
            || r == "single-letter-variable"
            || r.contains("meaningless")
            || r == "hungarian-notation"
            || r == "abbreviation-abuse"
        {
            *naming.entry(name.clone()).or_insert(0) += 1;
        } else if r.contains("nesting")
            || r == "god-function"
            || r == "long-function"
            || r.contains("closure")
            || r == "file-too-long"
            || r == "too-many-params"
            || r.contains("complex")
        {
            *complexity.entry(name.clone()).or_insert(0) += 1;
        }
        if r == "unwrap-abuse" {
            *unwrap.entry(name.clone()).or_insert(0) += 1;
        }
        if r.contains("todo") {
            *todo_files.entry(name.clone()).or_insert(0) += 1;
        }
    }

    let mut result = HashMap::new();
    let top_pct = |m: &HashMap<String, usize>| -> Vec<(String, f64)> {
        let mut v: Vec<_> = m
            .iter()
            .map(|(k, v)| {
                let pct = (*v as f64 / total_f.max(1.0)) * 100.0;
                (k.clone(), pct)
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(3);
        v
    };
    result.insert("duplication", top_pct(&dups));
    result.insert("naming", top_pct(&naming));
    result.insert("complexity", top_pct(&complexity));
    result.insert("unwrap-abuse", top_pct(&unwrap));
    result.insert("todo", top_pct(&todo_files));
    result
}
