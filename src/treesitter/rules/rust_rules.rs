use super::base_rules::{CountRule, MacroRule, MethodCallRule};
use super::complex_rules::{
    AbbreviationAbuseTsRule, ComplexClosureRule, DeepNestingRule, GodFunctionRule,
    HungarianNotationTsRule, LongFunctionRule, MagicNumberRule, PrintlnDebuggingRule,
    SingleLetterTsRule, TerribleNamingRule,
};
use super::remaining_rules::{
    DuplicateImportsRule, FileTooLongRule, MeaninglessRule, TodoCommentRule,
};
use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

/// Register all tree-sitter based Rust rules.
pub fn register_rust_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    // Unwrap abuse (escalating severity)
    engine.add(Box::new(MethodCallRule {
        name: "unwrap-abuse",
        method_name: "unwrap",
        threshold: 0,
        severity_fn: |count| {
            if count > 15 {
                Severity::Nuclear
            } else if count > 8 {
                Severity::Spicy
            } else {
                Severity::Mild
            }
        },
        message_fn: |count| {
            format!(
                "Found {} .unwrap() calls — use proper error handling",
                count
            )
        },
    }));

    // Unnecessary clone
    engine.add(Box::new(MethodCallRule {
        name: "unnecessary-clone",
        method_name: "clone",
        threshold: 24,
        severity_fn: |_| Severity::Spicy,
        message_fn: |count| {
            format!(
                "Found {} .clone() calls — consider using references instead",
                count
            )
        },
    }));

    // Async abuse (>10 async blocks)
    engine.add(Box::new(CountRule {
        name: "async-abuse",
        pattern: "(async_block) @block",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} async blocks — consider consolidating async operations",
                count
            )
        },
    }));

    // Macro abuse
    engine.add(Box::new(CountRule {
        name: "macro-abuse",
        pattern: "(macro_invocation) @m",
        threshold: 20,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} macro invocations — consider reducing macro usage",
                count
            )
        },
    }));

    // Lifetime abuse (non-static lifetimes)
    engine.add(Box::new(CountRule {
        name: "lifetime-abuse",
        pattern: "(lifetime) @life",
        threshold: 20,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} lifetime annotations — consider simplifying lifetime management",
                count
            )
        },
    }));

    // Trait complexity (methods in trait body)
    engine.add(Box::new(CountRule {
        name: "trait-complexity",
        pattern: "(trait_item body: (declaration_list (function_item) @method))",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Trait has {} methods — consider splitting into smaller traits",
                count
            )
        },
    }));

    // Generic abuse (>5 type parameters)
    engine.add(Box::new(CountRule {
        name: "generic-abuse",
        pattern: "(type_parameters (type_parameter) @param)",
        threshold: 5,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} generic parameters — consider simplifying the type signature",
                count
            )
        },
    }));

    // Pattern matching abuse (tuple patterns)
    engine.add(Box::new(CountRule {
        name: "pattern-matching-abuse",
        pattern: "(tuple_pattern) @tp",
        threshold: 15,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} complex tuple patterns — consider using named structs",
                count
            )
        },
    }));

    // Box abuse: count Box::new() calls with text filter
    engine.add(Box::new(BoxAbuseRule));

    // Reference abuse (type references)
    engine.add(Box::new(CountRule {
        name: "reference-abuse",
        pattern: "(reference_type) @rt",
        threshold: 50,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} reference types — consider simplifying ownership",
                count
            )
        },
    }));

    // Slice abuse
    engine.add(Box::new(CountRule {
        name: "slice-abuse",
        pattern: "(slice_type) @st",
        threshold: 29,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} slice types — consider using concrete collection types",
                count
            )
        },
    }));

    // Module complexity (nested mod_items)
    engine.add(Box::new(CountRule {
        name: "module-complexity",
        pattern: "(mod_item body: (declaration_list (mod_item) @nested))",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} nested modules — consider flattening the module structure",
                count
            )
        },
    }));

    // Panic abuse (count panic! invocations)
    engine.add(Box::new(MacroRule {
        name: "panic-abuse",
        macro_name: "panic",
        threshold: 2,
        severity: Severity::Nuclear,
        message_fn: |count| {
            format!(
                "Found {} panic! calls — use proper error handling with Result",
                count
            )
        },
    }));

    // Deep nesting
    engine.add(Box::new(DeepNestingRule));
    // Long function
    engine.add(Box::new(LongFunctionRule));

    // God function
    engine.add(Box::new(GodFunctionRule));

    // Complex closure
    engine.add(Box::new(ComplexClosureRule));
    // Terrible naming
    engine.add(Box::new(TerribleNamingRule));

    // Single letter variable
    engine.add(Box::new(SingleLetterTsRule));

    // Hungarian notation
    engine.add(Box::new(HungarianNotationTsRule));

    // Abbreviation abuse
    engine.add(Box::new(AbbreviationAbuseTsRule));

    // Println debugging
    engine.add(Box::new(PrintlnDebuggingRule));

    // Magic number
    engine.add(Box::new(MagicNumberRule));

    // Meaningless naming
    engine.add(Box::new(MeaninglessRule));

    // TODO/FIXME comments
    engine.add(Box::new(TodoCommentRule));

    // Duplicate imports
    engine.add(Box::new(DuplicateImportsRule));

    // File too long
    engine.add(Box::new(FileTooLongRule));

    // String abuse (String::from, .to_string)
    engine.add(Box::new(MethodCallRule {
        name: "string-abuse",
        method_name: "to_string",
        threshold: 20,
        severity_fn: |_| Severity::Mild,
        message_fn: |count| format!("Found {} .to_string() calls — consider using &str", count),
    }));

    // Vec abuse (vec! macro)
    engine.add(Box::new(CountRule {
        name: "vec-abuse",
        pattern: "(macro_invocation macro: (identifier) @m (#eq? @m \"vec\"))",
        threshold: 15,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("Found {} vec![] calls — consider using arrays", count),
    }));

    // too-many-params: warn if function has > 6 parameters
    engine.add(Box::new(TooManyParamsRule));

    // rust-doc-example: doc comments should contain example code blocks
    engine.add(Box::new(RustDocExampleRule));

    // rust-derive-order: derive attributes should follow standard order
    engine.add(Box::new(RustDeriveOrderRule));

    // rust-error-display: Debug impl without Display impl
    engine.add(Box::new(RustErrorDisplayRule));

    // rust-must-use: missing #[must_use] on Result/Option returning pub fn
    engine.add(Box::new(RustMustUseRule));
}

// ─── Rust: Box::new() abuse — only count actual Box::new() calls ──

struct BoxAbuseRule;

impl TreeSitterRule for BoxAbuseRule {
    fn name(&self) -> &'static str {
        "box-abuse"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = "(call_expression function: (scoped_identifier) @call)";
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let matching: Vec<_> = captures
            .iter()
            .filter_map(|group| group.first())
            .filter(|cap| cap.text == "Box::new")
            .collect();
        let count = matching.len();
        if count > 8 {
            let (line, col) = matching
                .first()
                .map(|cap| {
                    let pos = cap.node.start_position();
                    (pos.row + 1, pos.column + 1)
                })
                .unwrap_or((1, 1));
            vec![CodeIssue {
                file_path: file.path.clone(),
                line,
                column: col,
                rule_name: "box-abuse".to_string(),
                message: format!(
                    "Found {} Box::new() calls — consider using stack allocation",
                    count
                ),
                severity: Severity::Spicy,
            }]
        } else {
            vec![]
        }
    }
}

struct TooManyParamsRule;

impl TreeSitterRule for TooManyParamsRule {
    fn name(&self) -> &'static str {
        "too-many-params"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        use super::base_rules::count_parameters;
        if let Ok(captures) =
            crate::treesitter::query::collect_captures(file, "(function_item) @fn")
        {
            let mut issues = Vec::new();
            for group in &captures {
                if let Some(cap) = group.first() {
                    let count = count_parameters(cap.node);
                    if count > 6 {
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "too-many-params".to_string(),
                            message: format!(
                                "Function has {} parameters — consider splitting",
                                count
                            ),
                            severity: Severity::Mild,
                        });
                    }
                }
            }
            issues
        } else {
            vec![]
        }
    }
}

// ─── Rust: doc comments should contain example code blocks ────────

struct RustDocExampleRule;

impl TreeSitterRule for RustDocExampleRule {
    fn name(&self) -> &'static str {
        "rust-doc-example"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("///") && !line.starts_with("////") {
                let comment_text = line.trim_start_matches("///").trim();
                // Skip if it's a doc attribute, not a doc comment
                if comment_text.is_empty() {
                    i += 1;
                    continue;
                }
                // Collect multi-line doc comment
                let mut doc_lines = vec![comment_text];
                i += 1;
                while i < lines.len() && lines[i].trim().starts_with("///") {
                    doc_lines.push(lines[i].trim().trim_start_matches("///").trim());
                    i += 1;
                }
                let full_doc = doc_lines.join(" ");
                if !full_doc.contains("```") {
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: i - doc_lines.len() + 1,
                        column: 4,
                        rule_name: "rust-doc-example".to_string(),
                        message: "Doc comment should include an example code block (```)"
                            .to_string(),
                        severity: Severity::Mild,
                    });
                }
            } else {
                i += 1;
            }
        }
        issues
    }
}

// ─── Rust: derive order should follow convention ─────────────────

struct RustDeriveOrderRule;

impl TreeSitterRule for RustDeriveOrderRule {
    fn name(&self) -> &'static str {
        "rust-derive-order"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let standard_order = [
            "Debug",
            "Clone",
            "Copy",
            "PartialEq",
            "Eq",
            "PartialOrd",
            "Ord",
            "Hash",
            "Default",
            "Serialize",
            "Deserialize",
        ];
        let mut issues = Vec::new();
        // Simple text-based checking
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("#[derive(") {
                continue;
            }
            let inner = trimmed
                .trim_start_matches("#[derive(")
                .trim_end_matches(")]");
            let derives: Vec<&str> = inner
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if derives.len() < 2 {
                continue;
            }
            // Check if derives follow the standard order
            let mut last_pos = -1i32;
            let order_ok = derives.iter().all(|d| {
                if let Some(pos) = standard_order.iter().position(|s| *s == *d) {
                    let pos_i = pos as i32;
                    if pos_i >= last_pos {
                        last_pos = pos_i;
                        true
                    } else {
                        false
                    }
                } else {
                    // Unknown derive, skip it for ordering
                    true
                }
            });
            if !order_ok {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find("#[derive(").unwrap_or(0) + 1,
                    rule_name: "rust-derive-order".to_string(),
                    message: format!("Derive order should follow: {}", standard_order.join(", ")),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Rust: Error type implements Debug but not Display ────────────

struct RustErrorDisplayRule;

impl TreeSitterRule for RustErrorDisplayRule {
    fn name(&self) -> &'static str {
        "rust-error-display"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        // Find types that have impl Debug for X but no impl Display for X
        let mut debug_types: Vec<String> = Vec::new();
        let mut display_types: Vec<String> = Vec::new();

        for line in file.content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("impl fmt::Debug for ") {
                let type_name = rest.trim_end_matches('{').trim();
                debug_types.push(type_name.to_string());
            } else if let Some(rest) = trimmed.strip_prefix("impl Debug for ") {
                let type_name = rest.trim_end_matches('{').trim();
                debug_types.push(type_name.to_string());
            } else if let Some(rest) = trimmed.strip_prefix("impl fmt::Display for ") {
                let type_name = rest.trim_end_matches('{').trim();
                display_types.push(type_name.to_string());
            } else if let Some(rest) = trimmed.strip_prefix("impl Display for ") {
                let type_name = rest.trim_end_matches('{').trim();
                display_types.push(type_name.to_string());
            }
        }

        for t in &debug_types {
            if !display_types.iter().any(|d| d == t) {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: 1,
                    column: 1,
                    rule_name: "rust-error-display".to_string(),
                    message: format!(
                        "'{}' implements Debug but not Display — error types should implement both",
                        t
                    ),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Rust: missing #[must_use] on Result/Option returning pub fn ──

struct RustMustUseRule;

impl TreeSitterRule for RustMustUseRule {
    fn name(&self) -> &'static str {
        "rust-must-use"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            // Check for pub fn returning Result or Option
            if trimmed.starts_with("pub fn ") && !trimmed.contains("#[must_use]") {
                let has_result = trimmed.contains(" -> Result<") || trimmed.contains(" -> Result<");
                let has_option = trimmed.contains(" -> Option<") || trimmed.contains(" -> Option<");
                if has_result || has_option {
                    // Check previous line doesn't have #[must_use]
                    let prev_line = if line_num > 0 {
                        file.content.lines().nth(line_num - 1).unwrap_or("")
                    } else {
                        ""
                    };
                    if !prev_line.trim().contains("#[must_use]") {
                        let fn_name = trimmed
                            .strip_prefix("pub fn ")
                            .and_then(|s| s.split('(').next())
                            .unwrap_or("<unknown>");
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: line_num + 1,
                            column: 1,
                            rule_name: "rust-must-use".to_string(),
                            message: format!(
                                "pub fn '{}' returns Result/Option but is missing #[must_use]",
                                fn_name
                            ),
                            severity: Severity::Mild,
                        });
                    }
                }
            }
        }
        issues
    }
}

#[cfg(test)]
#[path = "rust_rules_tests.rs"]
mod rust_rules_tests;
