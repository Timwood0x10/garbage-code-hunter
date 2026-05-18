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

    pub(super) fn is_zh(&self) -> bool {
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
        // Skip signal-level findings (line=0) — only actionable per-line issues
        let real: Vec<&CodeIssue> = issues.iter().filter(|i| i.line > 0).collect();
        if real.is_empty() {
            return;
        }
        // Find file with most issues
        let mut file_counts: HashMap<String, Vec<&CodeIssue>> = HashMap::new();
        for issue in real {
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

        for issue in issues.iter().filter(|i| i.line > 0) {
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
                let preview: String = first.message.chars().take(57).collect();
                let preview = if preview.len() < first.message.len() {
                    format!("{}...", preview)
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

    pub(super) fn print_style_ir_summary(&self) {
        let Some(ref summary) = self.style_ir_summary else {
            return;
        };

        let title = if self.is_zh() {
            "📐 风格IR 摘要"
        } else {
            "📐 Style IR Summary"
        };
        println!("{}", title.bright_cyan().bold());
        println!("{}", "═".repeat(50).bright_black());

        println!(
            "  Language: {}  |  Lines: {}  |  Fns: {}",
            summary.language.bright_white(),
            summary.line_count,
            summary.function_count,
        );

        if summary.is_clean_signal_baseline {
            let clean_msg = if self.is_zh() {
                "✅ 信号基线干净 —— 无显著风格问题"
            } else {
                "✅ Clean signal baseline — no significant style issues"
            };
            println!("  {}", clean_msg.bright_green());
        } else {
            println!(
                "  {} {}  {} {}  {} {}  {} {}",
                "🔥".bright_red(),
                summary.panic_call_count,
                "🏷️".bright_white(),
                summary.naming_violation_count,
                "🏗️".bright_blue(),
                summary.god_function_count,
                "📦".bright_yellow(),
                summary.deeply_nested_block_count,
            );
            println!(
                "  {} {}  {} {}  {} {}  {} {}",
                "🐛".bright_green(),
                summary.debug_call_count,
                "⚠️".bright_yellow(),
                summary.excessive_param_count,
                "🛡️".bright_blue(),
                summary.unsafe_block_count,
                "🔢".bright_cyan(),
                summary.magic_number_count,
            );
            println!(
                "  Over-engineering: {}  |  Code smells: {}",
                summary.over_engineering_count, summary.code_smell_count,
            );
        }
        println!();
    }
}

#[cfg(test)]
#[path = "display_tests.rs"]
mod display_tests;
