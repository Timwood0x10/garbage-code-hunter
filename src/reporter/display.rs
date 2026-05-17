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

    fn is_zh(&self) -> bool {
        self.i18n.lang == "zh-CN"
    }

    pub(super) fn print_personality(
        &self,
        p: &ProjectPersonality,
        score: &CodeQualityScore,
        corruption_pct: f64,
    ) {
        let title = if self.is_zh() {
            "🧠 项目人格"
        } else {
            "🧠 Project Personality"
        };
        let score_label = if self.is_zh() { "评分:" } else { "Score:" };
        let threat_label = if self.is_zh() {
            "威胁等级:"
        } else {
            "Threat Level:"
        };
        let corruption_label = if self.is_zh() {
            "腐化度:"
        } else {
            "Corruption:"
        };
        let traits_label = if self.is_zh() {
            "核心特征:"
        } else {
            "Core Traits:"
        };
        let emotion_label = if self.is_zh() {
            "情绪状态:"
        } else {
            "Emotional State:"
        };
        let philosophy_label = if self.is_zh() {
            "代码哲学:"
        } else {
            "Philosophy:"
        };
        let lore_label = if self.is_zh() {
            "📜 传说:"
        } else {
            "📜 Lore:"
        };

        let project_type = self.translate_personality_type(p.project_type);
        let threat = self.translate_threat(p.threat_level);

        println!("{}", title.bright_magenta().bold());
        println!("{}", "═".repeat(50).bright_black());
        println!("  {}  {}", p.emoji, project_type.bright_yellow().bold());
        println!();
        println!(
            "  {}  {}  {}",
            score_label.bright_white(),
            format!("{:.0}/100", score.total_score).bold(),
            score.quality_level.emoji()
        );
        println!(
            "  {}  {}",
            threat_label.bright_white(),
            threat.bright_red().bold()
        );
        println!(
            "  {}  {:.0}%",
            corruption_label.bright_white(),
            corruption_pct
        );
        println!();
        println!("  {}", traits_label.bright_white());
        for t in &p.core_traits {
            let trait_zh = self.translate_trait(t);
            println!("    ▸ {}", trait_zh.cyan());
        }
        println!();
        // Randomly show 1-2 of emotion/philosophy/lore for dynamic feel
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as usize;
        let mut extras: Vec<(&str, String)> = Vec::new();
        extras.push((
            emotion_label,
            self.translate_emotion(p.emotional_state).to_string(),
        ));
        extras.push((
            philosophy_label,
            format!("\"{}\"", self.translate_philosophy(p.code_philosophy)),
        ));
        if !p.lore.is_empty() {
            extras.push((lore_label, format!("\"{}\"", p.lore[0])));
        }
        // Pick 1-2 items based on seed
        let count = if extras.len() >= 3 {
            1 + (seed % 2)
        } else {
            extras.len()
        };
        let start = seed % extras.len();
        for i in 0..count {
            let idx = (start + i) % extras.len();
            let (label, value) = &extras[idx];
            println!("  {}  {}", label.bright_white(), value.yellow());
        }
        println!();
    }

    pub(super) fn print_autopsy(&self, a: &AutopsyReport) {
        let title = if self.is_zh() {
            "☠ 代码库尸检报告"
        } else {
            "☠ CODEBASE AUTOPSY"
        };
        let cod_label = if self.is_zh() {
            "死因:"
        } else {
            "Cause of Death:"
        };
        let cond_label = if self.is_zh() {
            "状况:"
        } else {
            "Condition:"
        };
        let contamination_label = if self.is_zh() {
            "☣ 高污染区域:"
        } else {
            "☣ High Contamination Zones:"
        };
        let final_words_label = if self.is_zh() {
            "遗言:"
        } else {
            "Final Words:"
        };
        let propagation_label = if self.is_zh() {
            "🧬 突变传播:"
        } else {
            "🧬 Mutation Propagation:"
        };
        let origin_label = if self.is_zh() {
            "☣ 感染源"
        } else {
            "☣ Infection Origin"
        };
        let cluster_label = if self.is_zh() {
            "⚠ 复制集群"
        } else {
            "⚠ Replication Cluster"
        };

        println!("{}", title.bright_red().bold());
        println!("{}", "═".repeat(50).bright_black());
        let cod = self.translate_cause_of_death(a.cause_of_death);
        println!("  {} \"{}\"", cod_label.bright_white(), cod.bright_red());
        println!();
        let cond = self.translate_condition(a.patient_condition);
        println!("  {}  {}", cond_label.bright_white(), cond.bright_yellow());
        println!();
        if !a.corrupted_regions.is_empty() {
            println!("  {}", contamination_label.bright_white());
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
        let fw = self.translate_final_words(a.final_words);
        println!(
            "  {} \"{}\"",
            final_words_label.bright_white(),
            fw.bright_black().italic()
        );
        println!();
        if !a.spread_chains.is_empty() {
            println!("  {}", propagation_label.bright_white());
            for (i, (file, infected_list)) in a.spread_chains.iter().enumerate() {
                let label = if i == 0 { origin_label } else { cluster_label };
                println!(
                    "    {}: {} →",
                    label.bright_yellow(),
                    file.bright_blue().bold()
                );
                for (target, count, funcs) in infected_list.iter().take(3) {
                    let func_sample = funcs.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
                    let etc = if funcs.len() > 2 {
                        if self.is_zh() {
                            format!("，另有{}个", funcs.len() - 2)
                        } else {
                            format!(", +{} more", funcs.len() - 2)
                        }
                    } else {
                        String::new()
                    };
                    let fn_label = if self.is_zh() {
                        "个函数"
                    } else {
                        "functions"
                    };
                    println!(
                        "      ▸ {} ({} {}: {}{})",
                        target.bright_blue(),
                        count,
                        fn_label,
                        func_sample,
                        etc
                    );
                }
            }
            println!();
        }
    }

    pub(super) fn print_behavior_distribution(&self, score: &CodeQualityScore) {
        let title = if self.is_zh() {
            "🧬 行为分布"
        } else {
            "🧬 Behavior Distribution"
        };
        println!("{}", title.bright_magenta().bold());
        println!("{}", "═".repeat(50).bright_black());

        let mut signals: Vec<StyleSignal> = vec![
            StyleSignal::Duplication,
            StyleSignal::PanicAddiction,
            StyleSignal::NamingChaos,
            StyleSignal::NestedHell,
            StyleSignal::HotfixCulture,
            StyleSignal::OverEngineering,
            StyleSignal::CodeSmells,
        ];
        signals.sort_by(|a, b| {
            let sa = score.signal_scores.get(a).copied().unwrap_or(0.0);
            let sb = score.signal_scores.get(b).copied().unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

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
            let signal_name = if self.is_zh() {
                signal.display_name_zh()
            } else {
                signal.display_name().to_string()
            };
            let label = format!("{:<18}", signal_name);
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

        let threat = if self.is_zh() {
            if count > 100 {
                "☢ 末日级"
            } else if count > 50 {
                "💀 危险级"
            } else if count > 20 {
                "⚠ 高危"
            } else {
                "⚠ 升高"
            }
        } else {
            if count > 100 {
                "☢ APOCALYPTIC"
            } else if count > 50 {
                "💀 CRITICAL"
            } else if count > 20 {
                "⚠ HIGH"
            } else {
                "⚠ ELEVATED"
            }
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

        let survival = if self.is_zh() {
            if n > 10 {
                "无"
            } else if n > 5 {
                "低"
            } else if n > 0 {
                "可存活但有伤亡"
            } else {
                "高"
            }
        } else {
            if n > 10 {
                "none"
            } else if n > 5 {
                "low"
            } else if n > 0 {
                "survivable with casualties"
            } else {
                "high"
            }
        };

        let boss_title = if self.is_zh() {
            "☠ 最终BOSS"
        } else {
            "☠ FINAL BOSS DETECTED"
        };
        let file_label = if self.is_zh() { "文件:" } else { "File:" };
        let threat_label = if self.is_zh() {
            "威胁等级:"
        } else {
            "Threat Level:"
        };
        let corruption_label = if self.is_zh() {
            "腐化指数:"
        } else {
            "Corruption Index:"
        };
        let attacks_label = if self.is_zh() {
            "已知攻击:"
        } else {
            "Known Attacks:"
        };
        let survival_label = if self.is_zh() {
            "存活几率:"
        } else {
            "Survival Chance:"
        };

        println!("{}", boss_title.bright_red().bold());
        println!("{}", "═".repeat(50).bright_black());
        println!(
            "  {} {}",
            file_label.bright_white(),
            boss_name.bright_red().bold()
        );
        println!(
            "  {} {}",
            threat_label.bright_white(),
            threat.bright_red().bold()
        );
        println!(
            "  {} {} (💥{} 🌶️{} 😐{})",
            corruption_label.bright_white(),
            count,
            n,
            s,
            m
        );
        println!();
        println!("  {}", attacks_label.bright_white());
        for attack in &known_attacks {
            println!("    ▸ {}", attack.yellow());
        }
        println!();
        println!(
            "  {} {}",
            survival_label.bright_white(),
            survival.bright_red()
        );
        println!();
    }

    pub(super) fn print_symptoms(&self, issues: &[CodeIssue]) {
        let title = if self.is_zh() {
            "🦠 突变分析"
        } else {
            "🦠 MUTATION ANALYSIS"
        };
        println!("{}", title.bright_green().bold());
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
            let name = if self.is_zh() {
                "重复感染"
            } else {
                "Duplication Infection"
            };
            categories.push((name, dup, dup_sev));
        }
        if !naming.is_empty() {
            let name = if self.is_zh() {
                "命名萎缩"
            } else {
                "Naming Atrophy"
            };
            categories.push((name, naming, naming_sev));
        }
        if !complexity.is_empty() {
            let name = if self.is_zh() {
                "复杂度过度增长"
            } else {
                "Complexity Overgrowth"
            };
            categories.push((name, complexity, complexity_sev));
        }
        if !smells.is_empty() {
            let name = if self.is_zh() {
                "代码异味综合症"
            } else {
                "Code Smell Syndrome"
            };
            categories.push((name, smells, smells_sev));
        }
        if !student.is_empty() {
            let name = if self.is_zh() {
                "学生代码热"
            } else {
                "Student Code Fever"
            };
            categories.push((name, student, student_sev));
        }

        for (cat_name, cat_issues, sev) in &categories {
            let icon = match sev {
                Severity::Nuclear => "💥",
                Severity::Spicy => "🌶️",
                Severity::Mild => "😐",
            };
            let spread = if cat_issues.len() > 100 {
                if self.is_zh() {
                    "危急"
                } else {
                    "CRITICAL"
                }
            } else if cat_issues.len() > 50 {
                if self.is_zh() {
                    "高危"
                } else {
                    "HIGH"
                }
            } else if cat_issues.len() > 10 {
                if self.is_zh() {
                    "升高"
                } else {
                    "ELEVATED"
                }
            } else {
                if self.is_zh() {
                    "中等"
                } else {
                    "MODERATE"
                }
            };
            let spread_color = if cat_issues.len() > 100 {
                spread.bright_red().bold()
            } else if cat_issues.len() > 50 {
                spread.red()
            } else if cat_issues.len() > 10 {
                spread.yellow()
            } else {
                spread.green()
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
            let anomalies_label = if self.is_zh() {
                "异常数:"
            } else {
                "Anomalies:"
            };
            let unit = if self.is_zh() {
                "个异常"
            } else {
                "anomalies"
            };
            println!(
                "    {} {} {} ({})",
                anomalies_label.bright_white(),
                total,
                unit,
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
                let sources_label = if self.is_zh() {
                    "突变源:"
                } else {
                    "Mutation Sources:"
                };
                println!("    {}", sources_label.bright_white());
                for (file, count) in sorted.iter().take(3) {
                    let bar = "#".repeat((*count / 10).min(15));
                    let unit = if self.is_zh() {
                        "个异常"
                    } else {
                        "anomalies"
                    };
                    println!(
                        "      {} [{}] ({} {})",
                        file.bright_blue(),
                        bar.bright_red(),
                        count,
                        unit
                    );
                }
            }

            if let Some(first) = cat_issues.first() {
                let preview = if first.message.len() > 60 {
                    format!("{}...", &first.message[..57])
                } else {
                    first.message.clone()
                };
                let specimen_label = if self.is_zh() { "样本:" } else { "Specimen:" };
                println!(
                    "    {} \"{}\"",
                    specimen_label.bright_white(),
                    preview.bright_black().italic()
                );
            }
            println!();
        }
    }

    pub(super) fn print_final_summary(
        &self,
        score: &CodeQualityScore,
        file_count: usize,
        personality_type: Option<&str>,
    ) {
        let title = if self.is_zh() {
            "📊 最终判决"
        } else {
            "📊 FINAL VERDICT"
        };
        let corruption_label = if self.is_zh() {
            "腐化指数:"
        } else {
            "Corruption Index:"
        };

        println!("{}", title.bright_cyan().bold());
        println!("{}", "═".repeat(50).bright_black());

        let emoji = score.quality_level.emoji();
        let level = score.quality_level.description(&self.i18n.lang);
        let n = score.severity_distribution.nuclear;
        let s = score.severity_distribution.spicy;
        let m = score.severity_distribution.mild;
        let total = n + s + m;

        println!(
            "  {} {:.0}/100 — {}  |  📁 {}  |  🔍 {:.1}/1k",
            emoji, score.total_score, level, file_count, score.issue_density
        );
        println!(
            "  {} {} (💥{} 🌶️{} 😐{})",
            corruption_label.bright_white(),
            total,
            n,
            s,
            m
        );
        println!();

        // Personality diagnosis — closes the Personality → Verdict loop
        if let Some(ptype) = personality_type {
            let ptype_zh = self.translate_personality_type(ptype);
            let dom_label = if self.is_zh() {
                "主导人格:"
            } else {
                "Dominant Personality:"
            };
            let diag_label = if self.is_zh() {
                "诊断:"
            } else {
                "Diagnosis:"
            };
            let diagnosis = self.diagnose_personality(ptype, score.total_score);
            println!(
                "  {} {}",
                dom_label.bright_white(),
                ptype_zh.bright_magenta().bold()
            );
            println!(
                "  {} {}",
                diag_label.bright_white(),
                diagnosis.bright_yellow()
            );
            println!();
        }

        // Punchline
        let punchline = if self.is_zh() {
            if n > 0 {
                "突变密度: 极端——需要紧急干预"
            } else if total > 50 {
                "突变密度: 升高——建议隔离"
            } else {
                "突变密度: 低——病人状况稳定"
            }
        } else {
            if n > 0 {
                "Mutation Density: extreme — aggressive intervention required"
            } else if total > 50 {
                "Mutation Density: elevated — quarantine recommended"
            } else {
                "Mutation Density: low — patient is stable"
            }
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

    fn diagnose_personality(&self, ptype: &str, score: f64) -> String {
        let severity = if score >= 80.0 {
            if self.is_zh() {
                "严重"
            } else {
                "severe"
            }
        } else if score >= 60.0 {
            if self.is_zh() {
                "中度"
            } else {
                "moderate"
            }
        } else {
            if self.is_zh() {
                "轻度"
            } else {
                "mild"
            }
        };
        let cause = match ptype {
            "The Copy-Paste Artist" => {
                if self.is_zh() {
                    "重复代码驱动开发综合症"
                } else {
                    "duplication-driven development syndrome"
                }
            }
            "The YOLO Engineer" => {
                if self.is_zh() {
                    "恐慌驱动开发综合症"
                } else {
                    "panic-driven development syndrome"
                }
            }
            "The Trait Wizard" => {
                if self.is_zh() {
                    "复杂度膨胀综合症"
                } else {
                    "complexity inflation syndrome"
                }
            }
            "The Legacy Necromancer" => {
                if self.is_zh() {
                    "语义退化综合症"
                } else {
                    "semantic decay syndrome"
                }
            }
            "The Hotfix Mercenary" => {
                if self.is_zh() {
                    "技术债累积综合症"
                } else {
                    "technical debt accumulation syndrome"
                }
            }
            "The Startup Survivor" => {
                if self.is_zh() {
                    "增长代价综合症"
                } else {
                    "growth-at-all-costs syndrome"
                }
            }
            "The Academic Wizard" => {
                if self.is_zh() {
                    "抽象过度综合症"
                } else {
                    "abstraction overdose syndrome"
                }
            }
            _ => {
                if self.is_zh() {
                    "代码异味综合症"
                } else {
                    "code smell syndrome"
                }
            }
        };
        format!("{severity} {cause}")
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

    fn translate_personality_type<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "The Copy-Paste Artist" => "复制粘贴艺术家",
            "The YOLO Engineer" => "YOLO工程师",
            "The Trait Wizard" => "特质巫师",
            "The Legacy Necromancer" => "遗留代码亡灵法师",
            "The Hotfix Mercenary" => "热修复雇佣兵",
            "The Startup Survivor" => "创业幸存者",
            "The Academic Wizard" => "学术巫师",
            "The Enterprise Bureaucrat" => "企业官僚",
            _ => en,
        }
    }

    fn translate_threat<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "☢ CRITICAL" => "☢ 危险",
            "⚠ HIGH" => "⚠ 高危",
            "⚠ ELEVATED" => "⚠ 升高",
            "🟢 MODERATE" => "🟢 中等",
            _ => en,
        }
    }

    fn translate_trait<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "Ctrl+C, Ctrl+V is your IDE's most used shortcut" => {
                "Ctrl+C、Ctrl+V是你IDE最常用的快捷键"
            }
            "Why abstract when you can duplicate" => "能复制何必抽象",
            "Same bug in 5 places = 5x debugging fun" => "同一个bug出现在5个地方 = 5倍调试乐趣",
            "DRY stands for 'Don't Repeat... wait, too late'" => "DRY代表'不要重复……等等，太晚了'",
            "Believes every Result is Ok" => "坚信每个Result都是Ok",
            "unwrap() used more often than error handling" => "unwrap()比错误处理用得还多",
            "Production incidents are just 'surprise features'" => "生产事故不过是'惊喜功能'",
            "Has never met a None they couldn't ignore" => "从未遇到过不能忽略的None",
            "Loves building pyramids of doom" => "热爱建造末日金字塔",
            "Each function is a journey through layers of abstraction" => {
                "每个函数都是穿越抽象层的旅程"
            }
            "Indentation is a competitive sport" => "缩进是一项竞技运动",
            "Thinks 'extract method' is for amateurs" => "认为'提取方法'是业余爱好者的做法",
            "Why use many word when few letter do trick" => "能用短名何必用长名",
            "Variables named like chess coordinates" => "变量命名像棋盘坐标",
            "'data', 'temp', 'val' — the holy trinity of naming" => {
                "'data'、'temp'、'val' — 命名三圣"
            }
            "Has never heard of domain-driven design" => "从未听说过领域驱动设计",
            "More TODOs than actual features" => "TODO比实际功能还多",
            "'TODO: fix this later' — later never came" => "'TODO: 以后修复' — 以后从未到来",
            "Commits with 'temp', 'test', 'asdf' messages" => "提交信息是'temp'、'test'、'asdf'",
            "WIP is a lifestyle, not a branch name" => "WIP是生活方式，不是分支名",
            "Shipped fast, now paying the interest" => "快速上线，现在在还债",
            "Copy-paste driven development" => "复制粘贴驱动开发",
            "Production incidents are 'learning experiences'" => "生产事故是'学习机会'",
            "Technical debt is just 'future velocity'" => "技术债只是'未来速度'",
            "Loves building pyramids of abstraction" => "热爱建造抽象金字塔",
            "Each function is a thesis in disguise" => "每个函数都是伪装的论文",
            "Naming conventions from another dimension" => "来自异次元的命名规范",
            "If it's not complex, it's not 'elegant'" => "不够复杂就不够'优雅'",
            "A balanced mix of code smells" => "代码异味的平衡组合",
            "Not great at anything, not terrible at anything" => "什么都不精通，什么都不算差",
            "Jack of all trades, master of technical debt" => "样样通，技术债专精",
            _ => en,
        }
    }

    fn translate_emotion<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "numb from repetitive work" => "重复劳动导致麻木",
            "denial" => "否认",
            "proud of the complexity" => "为复杂度自豪",
            "confident in their abbreviations" => "对自己的缩写充满信心",
            "overwhelmed but optimistic" => "不堪重负但保持乐观",
            "battle-scarred but still shipping" => "伤痕累累但仍在交付",
            "intellectually satisfied, practically lost" => "理论上满足，实践中迷失",
            "professional detachment" => "职业性超脱",
            _ => en,
        }
    }

    fn translate_philosophy<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "it worked once, it'll work again" => "成功过一次，就会再成功",
            "it won't crash in production" => "生产环境不会崩溃的",
            "if it's not nested, it's not sophisticated" => "不够嵌套就不够精妙",
            "comments are for people who can't read code" => "注释是给读不懂代码的人看的",
            "future me will deal with it" => "未来的我会处理的",
            "move fast and definitely break things" => "快速行动，必然出错",
            "the theory is beautiful, the practice is secondary" => "理论很美，实践其次",
            "it compiles, it ships" => "能编译就能上线",
            _ => en,
        }
    }

    fn translate_cause_of_death<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "uncontrolled duplication metastasis" => "失控的重复代码转移",
            "panic-driven development" => "恐慌驱动开发",
            "complexity collapse — nesting level exceeded event horizon" => {
                "复杂度坍塌 — 嵌套层级超过事件视界"
            }
            "semantic starvation — variable names lost all meaning" => {
                "语义饥饿 — 变量名完全丧失含义"
            }
            "TODO accumulation — promises exceeded delivery capacity" => {
                "TODO堆积 — 承诺超出交付能力"
            }
            "growth-at-all-costs syndrome" => "不惜一切代价增长综合症",
            "abstraction overdose — complexity exceeded comprehension" => {
                "抽象过度 — 复杂度超出理解范围"
            }
            "death by a thousand paper cuts" => "千刀万剐",
            _ => en,
        }
    }

    fn translate_condition<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "terminal — palliative care recommended" => "晚期 — 建议姑息治疗",
            "critical but treatable" => "危重但可治疗",
            "critical — needs immediate Result therapy" => "危重 — 需要立即Result治疗",
            "stable but concerning" => "稳定但令人担忧",
            "severe — god functions threatening readability" => "严重 — 上帝函数威胁可读性",
            "moderate — some functions need splitting" => "中度 — 部分函数需要拆分",
            "chronic but manageable" => "慢性但可控",
            "stable with high technical debt interest" => "稳定但技术债利息很高",
            "critical — needs stabilization sprint" => "危重 — 需要稳定化冲刺",
            "chronic — needs simplification therapy" => "慢性 — 需要简化治疗",
            "fair — routine maintenance recommended" => "尚可 — 建议常规维护",
            _ => en,
        }
    }

    fn translate_final_words<'a>(&self, en: &'a str) -> &'a str {
        if !self.is_zh() {
            return en;
        }
        match en {
            "just one more hotfix" => "再修一个热修复就好",
            "i'll add error handling later" => "以后再加错误处理",
            "i can still understand it" => "我还是能看懂的",
            "temp2 should work" => "temp2应该能用",
            "i'll fix it in the next sprint" => "下个冲刺再修",
            "we'll fix it after the next funding round" => "下一轮融资后就修",
            "it's actually quite simple once you understand the theory" => "理解了理论其实很简单",
            "we'll refactor next quarter" => "下个季度重构",
            _ => en,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::{QualityLevel, SeverityDistribution};

    fn en_reporter() -> Reporter {
        Reporter::new(
            false,
            false,
            false,
            10,
            5,
            false,
            false,
            false,
            "en",
            Box::new(crate::llm::LocalRoastProvider),
        )
    }

    fn zh_reporter() -> Reporter {
        Reporter::new(
            false,
            false,
            false,
            10,
            5,
            false,
            false,
            false,
            "zh-CN",
            Box::new(crate::llm::LocalRoastProvider),
        )
    }

    fn dummy_score(total: f64) -> CodeQualityScore {
        CodeQualityScore {
            total_score: total,
            n_score: 0.0,
            d_score: 0.0,
            category_scores: std::collections::HashMap::new(),
            signal_scores: std::collections::HashMap::new(),
            file_count: 0,
            total_lines: 0,
            issue_density: 0.0,
            severity_distribution: SeverityDistribution {
                nuclear: 0,
                spicy: 0,
                mild: 0,
            },
            quality_level: QualityLevel::from_score(total),
        }
    }

    // ── is_zh ─────────────────────────────────────────────────────

    #[test]
    fn test_is_zh_english() {
        let r = en_reporter();
        assert!(!r.is_zh(), "EN reporter => is_zh = false");
    }

    #[test]
    fn test_is_zh_chinese() {
        let r = zh_reporter();
        assert!(r.is_zh(), "ZH reporter => is_zh = true");
    }

    // ── diagnose_personality — all types × score bands ─────────────

    #[test]
    fn test_diagnose_copy_paste_artist_severe() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Copy-Paste Artist", 85.0);
        assert!(d.contains("severe"), "score 85 => 'severe', got {d}");
        assert!(
            d.contains("duplication"),
            "Copy-Paste => duplication syndrome"
        );
    }

    #[test]
    fn test_diagnose_copy_paste_artist_moderate() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Copy-Paste Artist", 65.0);
        assert!(d.contains("moderate"), "score 65 => 'moderate', got {d}");
    }

    #[test]
    fn test_diagnose_copy_paste_artist_mild() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Copy-Paste Artist", 45.0);
        assert!(d.contains("mild"), "score 45 => 'mild', got {d}");
    }

    #[test]
    fn test_diagnose_yolo_engineer() {
        let r = en_reporter();
        let d = r.diagnose_personality("The YOLO Engineer", 70.0);
        assert!(d.contains("panic"), "YOLO => panic syndrome");
    }

    #[test]
    fn test_diagnose_trait_wizard() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Trait Wizard", 70.0);
        assert!(
            d.contains("complexity"),
            "Trait Wizard => complexity syndrome"
        );
    }

    #[test]
    fn test_diagnose_legacy_necromancer() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Legacy Necromancer", 70.0);
        assert!(
            d.contains("semantic decay"),
            "Legacy Necromancer => semantic decay"
        );
    }

    #[test]
    fn test_diagnose_hotfix_mercenary() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Hotfix Mercenary", 70.0);
        assert!(d.contains("debt"), "Hotfix Mercenary => debt syndrome");
    }

    #[test]
    fn test_diagnose_startup_survivor() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Startup Survivor", 70.0);
        assert!(d.contains("growth"), "Startup Survivor => growth syndrome");
    }

    #[test]
    fn test_diagnose_academic_wizard() {
        let r = en_reporter();
        let d = r.diagnose_personality("The Academic Wizard", 70.0);
        assert!(
            d.contains("abstraction"),
            "Academic Wizard => abstraction syndrome"
        );
    }

    #[test]
    fn test_diagnose_unknown_fallback() {
        let r = en_reporter();
        let d = r.diagnose_personality("Unknown Type", 70.0);
        assert!(
            d.contains("smell"),
            "unknown => code smell syndrome, got {d}"
        );
    }

    #[test]
    fn test_diagnose_chinese_output() {
        let r = zh_reporter();
        let d_en = en_reporter().diagnose_personality("The Copy-Paste Artist", 85.0);
        let d_zh = r.diagnose_personality("The Copy-Paste Artist", 85.0);
        assert_ne!(d_en, d_zh, "ZH diagnosis should differ from EN");
        assert!(
            d_zh.contains("严重") || d_zh.contains("中度") || d_zh.contains("轻度"),
            "ZH diagnosis should contain Chinese severity, got {d_zh}"
        );
    }

    // ── translate_personality_type ────────────────────────────────

    #[test]
    fn test_translate_personality_type_english() {
        let r = en_reporter();
        assert_eq!(
            r.translate_personality_type("The Copy-Paste Artist"),
            "The Copy-Paste Artist"
        );
        assert_eq!(
            r.translate_personality_type("The YOLO Engineer"),
            "The YOLO Engineer"
        );
    }

    #[test]
    fn test_translate_personality_type_chinese() {
        let r = zh_reporter();
        assert_eq!(
            r.translate_personality_type("The Copy-Paste Artist"),
            "复制粘贴艺术家"
        );
        assert_eq!(
            r.translate_personality_type("The YOLO Engineer"),
            "YOLO工程师"
        );
        assert_eq!(
            r.translate_personality_type("The Enterprise Bureaucrat"),
            "企业官僚"
        );
        assert_eq!(r.translate_personality_type("Unknown"), "Unknown");
    }

    // ── translate_threat ──────────────────────────────────────────

    #[test]
    fn test_translate_threat_english() {
        let r = en_reporter();
        assert_eq!(r.translate_threat("⚠ HIGH"), "⚠ HIGH");
        assert_eq!(r.translate_threat("☢ CRITICAL"), "☢ CRITICAL");
    }

    #[test]
    fn test_translate_threat_chinese() {
        let r = zh_reporter();
        assert_eq!(r.translate_threat("⚠ HIGH"), "⚠ 高危");
        assert_eq!(r.translate_threat("☢ CRITICAL"), "☢ 危险");
        assert_eq!(r.translate_threat("🟢 MODERATE"), "🟢 中等");
    }

    // ── translate_trait ───────────────────────────────────────────

    #[test]
    fn test_translate_trait_english_passthrough() {
        let r = en_reporter();
        assert_eq!(
            r.translate_trait("Ctrl+C, Ctrl+V is your IDE's most used shortcut"),
            "Ctrl+C, Ctrl+V is your IDE's most used shortcut"
        );
    }

    #[test]
    fn test_translate_trait_chinese_known() {
        let r = zh_reporter();
        assert_eq!(
            r.translate_trait("Ctrl+C, Ctrl+V is your IDE's most used shortcut"),
            "Ctrl+C、Ctrl+V是你IDE最常用的快捷键"
        );
        assert_eq!(
            r.translate_trait("Why abstract when you can duplicate"),
            "能复制何必抽象"
        );
        assert_eq!(
            r.translate_trait("Jack of all trades, master of technical debt"),
            "样样通，技术债专精"
        );
    }

    #[test]
    fn test_translate_trait_unknown_fallback() {
        let r = zh_reporter();
        assert_eq!(
            r.translate_trait("some unknown trait"),
            "some unknown trait"
        );
    }

    // ── translate_emotion ─────────────────────────────────────────

    #[test]
    fn test_translate_emotion_all() {
        let r = zh_reporter();
        let pairs = [
            ("numb from repetitive work", "重复劳动导致麻木"),
            ("denial", "否认"),
            ("proud of the complexity", "为复杂度自豪"),
            ("confident in their abbreviations", "对自己的缩写充满信心"),
            ("overwhelmed but optimistic", "不堪重负但保持乐观"),
            ("battle-scarred but still shipping", "伤痕累累但仍在交付"),
            (
                "intellectually satisfied, practically lost",
                "理论上满足，实践中迷失",
            ),
            ("professional detachment", "职业性超脱"),
        ];
        for (en, zh) in &pairs {
            assert_eq!(r.translate_emotion(en), *zh, "emotion '{en}'");
        }
    }

    #[test]
    fn test_translate_emotion_unknown() {
        let r = zh_reporter();
        assert_eq!(r.translate_emotion("unknown emotion"), "unknown emotion");
    }

    // ── translate_philosophy ──────────────────────────────────────

    #[test]
    fn test_translate_philosophy_all() {
        let r = zh_reporter();
        let pairs = [
            ("it worked once, it'll work again", "成功过一次，就会再成功"),
            ("it won't crash in production", "生产环境不会崩溃的"),
            (
                "if it's not nested, it's not sophisticated",
                "不够嵌套就不够精妙",
            ),
            (
                "comments are for people who can't read code",
                "注释是给读不懂代码的人看的",
            ),
            ("future me will deal with it", "未来的我会处理的"),
            (
                "move fast and definitely break things",
                "快速行动，必然出错",
            ),
            (
                "the theory is beautiful, the practice is secondary",
                "理论很美，实践其次",
            ),
            ("it compiles, it ships", "能编译就能上线"),
        ];
        for (en, zh) in &pairs {
            assert_eq!(r.translate_philosophy(en), *zh, "philosophy '{en}'");
        }
    }

    // ── translate_cause_of_death ──────────────────────────────────

    #[test]
    fn test_translate_cause_of_death_all() {
        let r = zh_reporter();
        let pairs = [
            ("uncontrolled duplication metastasis", "失控的重复代码转移"),
            ("panic-driven development", "恐慌驱动开发"),
            (
                "complexity collapse — nesting level exceeded event horizon",
                "复杂度坍塌 — 嵌套层级超过事件视界",
            ),
            (
                "semantic starvation — variable names lost all meaning",
                "语义饥饿 — 变量名完全丧失含义",
            ),
            (
                "TODO accumulation — promises exceeded delivery capacity",
                "TODO堆积 — 承诺超出交付能力",
            ),
            ("growth-at-all-costs syndrome", "不惜一切代价增长综合症"),
            (
                "abstraction overdose — complexity exceeded comprehension",
                "抽象过度 — 复杂度超出理解范围",
            ),
            ("death by a thousand paper cuts", "千刀万剐"),
        ];
        for (en, zh) in &pairs {
            assert_eq!(r.translate_cause_of_death(en), *zh, "cause '{en}'");
        }
    }

    #[test]
    fn test_translate_cause_of_death_unknown() {
        let r = zh_reporter();
        assert_eq!(r.translate_cause_of_death("unknown cause"), "unknown cause");
    }

    // ── translate_condition ───────────────────────────────────────

    #[test]
    fn test_translate_condition_all() {
        let r = zh_reporter();
        let pairs = [
            (
                "terminal — palliative care recommended",
                "晚期 — 建议姑息治疗",
            ),
            ("critical but treatable", "危重但可治疗"),
            (
                "critical — needs immediate Result therapy",
                "危重 — 需要立即Result治疗",
            ),
            ("stable but concerning", "稳定但令人担忧"),
            (
                "severe — god functions threatening readability",
                "严重 — 上帝函数威胁可读性",
            ),
            (
                "moderate — some functions need splitting",
                "中度 — 部分函数需要拆分",
            ),
            ("chronic but manageable", "慢性但可控"),
            (
                "stable with high technical debt interest",
                "稳定但技术债利息很高",
            ),
            (
                "critical — needs stabilization sprint",
                "危重 — 需要稳定化冲刺",
            ),
            (
                "chronic — needs simplification therapy",
                "慢性 — 需要简化治疗",
            ),
            (
                "fair — routine maintenance recommended",
                "尚可 — 建议常规维护",
            ),
        ];
        for (en, zh) in &pairs {
            assert_eq!(r.translate_condition(en), *zh, "condition '{en}'");
        }
    }

    // ── translate_final_words ─────────────────────────────────────

    #[test]
    fn test_translate_final_words_all() {
        let r = zh_reporter();
        let pairs = [
            ("just one more hotfix", "再修一个热修复就好"),
            ("i'll add error handling later", "以后再加错误处理"),
            ("i can still understand it", "我还是能看懂的"),
            ("temp2 should work", "temp2应该能用"),
            ("i'll fix it in the next sprint", "下个冲刺再修"),
            (
                "we'll fix it after the next funding round",
                "下一轮融资后就修",
            ),
            (
                "it's actually quite simple once you understand the theory",
                "理解了理论其实很简单",
            ),
            ("we'll refactor next quarter", "下个季度重构"),
        ];
        for (en, zh) in &pairs {
            assert_eq!(r.translate_final_words(en), *zh, "final words '{en}'");
        }
    }

    // ── print_behavior_distribution ───────────────────────────────

    /// Objective: Smoke test — print_behavior_distribution does not panic with empty scores.
    #[test]
    fn test_print_behavior_distribution_empty_scores() {
        let r = en_reporter();
        let score = dummy_score(50.0);
        r.print_behavior_distribution(&score);
    }

    /// Objective: Smoke test — print_behavior_distribution does not panic with realistic scores.
    #[test]
    fn test_print_behavior_distribution_with_scores() {
        let r = en_reporter();
        let mut score = dummy_score(50.0);
        score.signal_scores.insert(StyleSignal::Duplication, 25.0);
        score.signal_scores.insert(StyleSignal::NamingChaos, 10.0);
        score.signal_scores.insert(StyleSignal::PanicAddiction, 5.0);
        r.print_behavior_distribution(&score);
    }

    // ── final_summary ─────────────────────────────────────────────

    /// Objective: Smoke test — print_final_summary does not panic.
    #[test]
    fn test_print_final_summary_no_panic() {
        let r = en_reporter();
        let score = dummy_score(75.0);
        r.print_final_summary(&score, 10, Some("The Copy-Paste Artist"));
    }

    #[test]
    fn test_print_final_summary_none_personality() {
        let r = en_reporter();
        let score = dummy_score(30.0);
        r.print_final_summary(&score, 0, None);
    }
}
