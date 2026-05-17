use colored::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::analyzer::{CodeIssue, Severity};
use crate::scoring::CodeQualityScore;
use crate::signals::StyleSignal;

use super::autopsy::{AutopsyReport, ProjectPersonality};
use super::Reporter;

const LORE: &[&str] = &[
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

fn random_lore() -> &'static str {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as usize;
    LORE[seed % LORE.len()]
}

impl Reporter {
    pub(super) fn print_header(&self) {
        println!("{}", self.i18n.get("title").bright_red().bold());
        println!("{}", self.i18n.get("preparing").yellow());
        println!();
    }

    pub(super) fn print_personality(
        &self,
        p: &ProjectPersonality,
        score: &CodeQualityScore,
        corruption_pct: f64,
    ) {
        println!("{}", "🧠 Project Personality".bright_magenta().bold());
        println!("{}", "═".repeat(50).bright_black());
        println!("  {}  {}", p.emoji, p.project_type.bright_yellow().bold());
        println!();
        println!(
            "  {}  {}  {}",
            "Score:".bright_white(),
            format!("{:.0}/100", score.total_score).bold(),
            score.quality_level.emoji()
        );
        println!(
            "  {}  {}",
            "Threat Level:".bright_white(),
            p.threat_level.bright_red().bold()
        );
        println!("  {}  {:.0}%", "Corruption:".bright_white(), corruption_pct);
        println!();
        println!("  {}", "Core Traits:".bright_white());
        for t in &p.core_traits {
            println!("    ▸ {}", t.cyan());
        }
        println!();
        println!(
            "  {}  {}",
            "Emotional State:".bright_white(),
            p.emotional_state.yellow()
        );
        println!(
            "  {}  \"{}\"",
            "Philosophy:".bright_white(),
            p.code_philosophy.bright_black().italic()
        );
        println!();
        if !p.lore.is_empty() {
            println!(
                "  {}  \"{}\"",
                "📜 Lore:".bright_white(),
                p.lore[0].bright_black().italic()
            );
            println!();
        }
    }

    pub(super) fn print_autopsy(&self, a: &AutopsyReport) {
        println!("{}", "☠ CODEBASE AUTOPSY".bright_red().bold());
        println!("{}", "═".repeat(50).bright_black());
        println!(
            "  {} \"{}\"",
            "Cause of Death:".bright_white(),
            a.cause_of_death.bright_red()
        );
        println!();
        println!(
            "  {}  {}",
            "Condition:".bright_white(),
            a.patient_condition.bright_yellow()
        );
        println!();
        if !a.corrupted_regions.is_empty() {
            println!("  {}", "☣ High Contamination Zones:".bright_white());
            for (file, pct) in &a.corrupted_regions {
                let bar_len = (*pct as usize / 5).min(20);
                let bar = "#".repeat(bar_len);
                let empty = "·".repeat((20usize).saturating_sub(bar_len));
                println!(
                    "    {} [{}{}] {:.0}%",
                    file.bright_blue(),
                    bar.bright_red(),
                    empty.bright_black(),
                    pct
                );
            }
            println!();
        }
        println!(
            "  {} \"{}\"",
            "Final Words:".bright_white(),
            a.final_words.bright_black().italic()
        );
        println!();
        if !a.spread_chains.is_empty() {
            println!("  {}", "🔄 Mutation Spread Chain:".bright_white());
            for (file, infected_list) in &a.spread_chains {
                println!("    {} infected →", file.bright_blue().bold());
                for (target, count, funcs) in infected_list.iter().take(3) {
                    let func_sample = funcs.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
                    let etc = if funcs.len() > 2 {
                        format!(", +{} more", funcs.len() - 2)
                    } else {
                        String::new()
                    };
                    println!(
                        "      ▸ {} ({} functions: {}{})",
                        target.bright_blue(),
                        count,
                        func_sample,
                        etc
                    );
                }
            }
            println!();
        }
    }

    pub(super) fn print_behavior_distribution(&self, score: &CodeQualityScore) {
        println!("{}", "🧬 Behavior Distribution".bright_magenta().bold());
        println!("{}", "═".repeat(50).bright_black());

        let signals = [
            StyleSignal::Duplication,
            StyleSignal::PanicAddiction,
            StyleSignal::NamingChaos,
            StyleSignal::NestedHell,
            StyleSignal::HotfixCulture,
            StyleSignal::OverEngineering,
            StyleSignal::CodeSmells,
        ];

        let max_score = signals
            .iter()
            .map(|s| score.signal_scores.get(s).copied().unwrap_or(0.0))
            .fold(0.0f64, f64::max)
            .max(1.0);

        let bar_width: usize = 25;
        for signal in &signals {
            let s = score.signal_scores.get(signal).copied().unwrap_or(0.0);
            let pct = (s / max_score * 100.0).min(100.0);
            let filled = (pct / 100.0 * bar_width as f64).round() as usize;
            let empty = bar_width.saturating_sub(filled);
            let bar = "█".repeat(filled) + &"░".repeat(empty);
            let label = format!("{:<18}", signal.display_name());
            println!(
                "  {} {} {:.0}",
                label.bright_white(),
                bar.bright_cyan(),
                pct
            );
        }
        println!();
    }

    pub(super) fn print_boss_file(&self, issues: &[CodeIssue]) {
        if issues.is_empty() {
            return;
        }
        // Find file with most issues
        let mut file_counts: HashMap<String, Vec<&CodeIssue>> = HashMap::new();
        for issue in issues {
            let name = issue.file_path.to_string_lossy().to_string();
            file_counts.entry(name).or_default().push(issue);
        }
        let boss = file_counts.into_iter().max_by_key(|(_, v)| v.len());
        let boss_name = match boss {
            Some((ref name, _)) => name.rsplit('/').next().unwrap_or(name),
            None => return,
        };
        let boss_issues = match boss {
            Some((_, ref v)) => v,
            None => return,
        };

        let count = boss_issues.len();
        let n = boss_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let s = boss_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Spicy))
            .count();
        let m = boss_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Mild))
            .count();

        let threat = if count > 100 {
            "☢ APOCALYPTIC"
        } else if count > 50 {
            "💀 CRITICAL"
        } else if count > 20 {
            "⚠ HIGH"
        } else {
            "⚠ ELEVATED"
        };

        let mut rule_counts: HashMap<&str, usize> = HashMap::new();
        for issue in boss_issues {
            *rule_counts.entry(issue.rule_name.as_str()).or_insert(0) += 1;
        }
        let mut sorted_rules: Vec<_> = rule_counts.into_iter().collect();
        sorted_rules.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        let known_attacks: Vec<String> = sorted_rules
            .iter()
            .take(4)
            .map(|(name, count)| format!("{} (×{})", name.replace('-', " "), count))
            .collect();

        let survival = if n > 10 {
            "none"
        } else if n > 5 {
            "low"
        } else if n > 0 {
            "survivable with casualties"
        } else {
            "high"
        };

        println!("{}", "☠ FINAL BOSS DETECTED".bright_red().bold());
        println!("{}", "═".repeat(50).bright_black());
        println!(
            "  {} {}",
            "File:".bright_white(),
            boss_name.bright_red().bold()
        );
        println!(
            "  {} {}",
            "Threat Level:".bright_white(),
            threat.bright_red().bold()
        );
        println!(
            "  {} {} anomalies (💥{} 🌶️{} 😐{})",
            "Corruption Index:".bright_white(),
            count,
            n,
            s,
            m
        );
        println!();
        println!("  {}", "Known Attacks:".bright_white());
        for attack in &known_attacks {
            println!("    ▸ {}", attack.yellow());
        }
        println!();
        println!(
            "  {} {}",
            "Survival Chance:".bright_white(),
            survival.bright_red()
        );
        println!();
    }

    pub(super) fn print_symptoms(&self, issues: &[CodeIssue]) {
        println!("{}", "🦠 MUTATION ANALYSIS".bright_green().bold());
        println!("{}", "═".repeat(50).bright_black());

        let mut categories: Vec<(&str, Vec<&CodeIssue>, Severity)> = Vec::new();
        let mut dup: Vec<&CodeIssue> = Vec::new();
        let mut naming: Vec<&CodeIssue> = Vec::new();
        let mut complexity: Vec<&CodeIssue> = Vec::new();
        let mut smells: Vec<&CodeIssue> = Vec::new();
        let mut student: Vec<&CodeIssue> = Vec::new();

        for issue in issues {
            let r = issue.rule_name.as_str();
            if r.contains("duplicat") {
                dup.push(issue);
            } else if r.contains("naming")
                || r == "terrible-naming"
                || r == "single-letter-variable"
                || r.contains("meaningless")
                || r == "hungarian-notation"
                || r == "abbreviation-abuse"
                || r == "go-receiver-name"
                || r == "ruby-predicate-method"
                || r == "python-naming"
                || r == "constant-name"
            {
                naming.push(issue);
            } else if r.contains("nesting")
                || r == "god-function"
                || r == "long-function"
                || r.contains("closure")
                || r == "file-too-long"
                || r == "too-many-params"
                || r.contains("complex")
            {
                complexity.push(issue);
            } else if r == "panic-abuse" || r == "println-debugging" || r.contains("todo") {
                student.push(issue);
            } else {
                smells.push(issue);
            }
        }

        let max_severity = |items: &[&CodeIssue]| -> Severity {
            items
                .iter()
                .map(|i| &i.severity)
                .max_by_key(|s| match s {
                    Severity::Nuclear => 3,
                    Severity::Spicy => 2,
                    Severity::Mild => 1,
                })
                .cloned()
                .unwrap_or(Severity::Mild)
        };

        let dup_sev = max_severity(&dup);
        let naming_sev = max_severity(&naming);
        let complexity_sev = max_severity(&complexity);
        let smells_sev = max_severity(&smells);
        let student_sev = max_severity(&student);

        if !dup.is_empty() {
            categories.push(("Duplication Infection", dup, dup_sev));
        }
        if !naming.is_empty() {
            categories.push(("Naming Atrophy", naming, naming_sev));
        }
        if !complexity.is_empty() {
            categories.push(("Complexity Overgrowth", complexity, complexity_sev));
        }
        if !smells.is_empty() {
            categories.push(("Code Smell Syndrome", smells, smells_sev));
        }
        if !student.is_empty() {
            categories.push(("Student Code Fever", student, student_sev));
        }

        for (cat_name, cat_issues, sev) in &categories {
            let icon = match sev {
                Severity::Nuclear => "💥",
                Severity::Spicy => "🌶️",
                Severity::Mild => "😐",
            };
            let spread = if cat_issues.len() > 100 {
                "CRITICAL"
            } else if cat_issues.len() > 50 {
                "HIGH"
            } else if cat_issues.len() > 10 {
                "ELEVATED"
            } else {
                "MODERATE"
            };
            let spread_color = match spread {
                "CRITICAL" => spread.bright_red().bold(),
                "HIGH" => spread.red(),
                "ELEVATED" => spread.yellow(),
                _ => spread.green(),
            };

            println!(
                "{} {}  {}",
                icon,
                cat_name.bright_white().bold(),
                spread_color
            );

            let total = cat_issues.len();
            let n = cat_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Nuclear))
                .count();
            let s = cat_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Spicy))
                .count();
            let m = cat_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Mild))
                .count();
            let detail = if n > 0 {
                format!("💥{} 🌶️{} 😐{}", n, s, m)
            } else if s > 0 {
                format!("🌶️{} 😐{}", s, m)
            } else {
                format!("😐{}", m)
            };
            println!(
                "    {} {} anomalies ({})",
                "Anomalies:".bright_white(),
                total,
                detail
            );

            let mut file_counts: HashMap<String, usize> = HashMap::new();
            for issue in cat_issues {
                let name = issue
                    .file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                *file_counts.entry(name).or_insert(0) += 1;
            }
            let mut sorted: Vec<_> = file_counts.into_iter().collect();
            sorted.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
            if !sorted.is_empty() {
                println!("    {}", "Mutation Sources:".bright_white());
                for (file, count) in sorted.iter().take(3) {
                    let bar = "#".repeat((*count / 10).min(15));
                    println!(
                        "      {} [{}] ({} anomalies)",
                        file.bright_blue(),
                        bar.bright_red(),
                        count
                    );
                }
            }

            if let Some(first) = cat_issues.first() {
                let preview = if first.message.len() > 60 {
                    format!("{}...", &first.message[..57])
                } else {
                    first.message.clone()
                };
                println!(
                    "    {} \"{}\"",
                    "Specimen:".bright_white(),
                    preview.bright_black().italic()
                );
            }
            println!();
        }
    }

    pub(super) fn print_final_summary(&self, score: &CodeQualityScore, file_count: usize) {
        println!("{}", "📊 FINAL VERDICT".bright_cyan().bold());
        println!("{}", "═".repeat(50).bright_black());

        let emoji = score.quality_level.emoji();
        let level = score.quality_level.description("en-US");
        let n = score.severity_distribution.nuclear;
        let s = score.severity_distribution.spicy;
        let m = score.severity_distribution.mild;
        let total = n + s + m;

        println!(
            "  {} {:.0}/100 — {}  |  📁 {}  |  🔍 {:.1}/1k",
            emoji, score.total_score, level, file_count, score.issue_density
        );
        println!(
            "  {} {} anomalies (💥{} 🌶️{} 😐{})",
            "Corruption Index:".bright_white(),
            total,
            n,
            s,
            m
        );
        println!();

        // Punchline
        let punchline = if n > 0 {
            "Mutation Density: extreme — aggressive intervention required"
        } else if total > 50 {
            "Mutation Density: elevated — quarantine recommended"
        } else {
            "Mutation Density: low — patient is stable"
        };
        println!("  {}", punchline.bright_green().bold());

        // Random lore
        println!(
            "  {}  \"{}\"",
            "📜".bright_white(),
            random_lore().bright_black().italic()
        );
        println!();
    }

    #[expect(dead_code)]
    fn rule_display_name(&self, rule_name: &str) -> String {
        let name = match (self.i18n.lang.as_str(), rule_name) {
            ("zh-CN", r) => match r {
                "panic-abuse" => "panic滥用",
                "god-function" => "上帝函数",
                "magic-number" => "魔法数字",
                "todo-comment" => "TODO注释",
                "println-debugging" => "println调试",
                "code-duplication" => "代码重复",
                "cross-file-duplication" => "跨文件重复",
                "deep-nesting" => "深层嵌套",
                "long-function" => "过长函数",
                "file-too-long" => "过长文件",
                "single-letter-variable" => "单字母变量",
                "terrible-naming" => "糟糕命名",
                "meaningless-naming" => "无意义命名",
                "hungarian-notation" => "匈牙利命名",
                "abbreviation-abuse" => "过度缩写",
                "unwrap-abuse" => "unwrap滥用",
                "box-abuse" => "Box滥用",
                "duplicate-imports" => "重复导入",
                "bare-except" => "裸except",
                "any-type" => "any类型",
                _ => return rule_name.replace('-', " "),
            },
            (_, name) => return name.replace('-', " "),
        };
        name.to_string()
    }
}
