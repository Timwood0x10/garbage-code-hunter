use regex::Regex;

use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

use super::base_rules::{
    closure_depth, count_descendants_of_types, count_parameters, find_function_name, BLOCK_KINDS,
    BLOCK_PARENT_TYPES,
};

// ============================================================================
// Deep nesting detection
// ============================================================================

pub(crate) struct DeepNestingRule;

impl TreeSitterRule for DeepNestingRule {
    fn name(&self) -> &'static str {
        "deep-nesting"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn skips_test_files(&self) -> bool {
        false
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let root = file.root_node();
        check_nesting_recursive(file, root, 0, &mut issues);
        issues
    }
}

fn check_nesting_recursive(
    file: &ParsedFile,
    node: tree_sitter::Node,
    depth: usize,
    issues: &mut Vec<CodeIssue>,
) {
    let node_type = node.kind();
    let is_block_parent = BLOCK_PARENT_TYPES.contains(&node_type);
    let is_block_kind = BLOCK_KINDS.contains(&node_type);

    let new_depth = if is_block_kind && depth > 0 {
        // This block is nested inside a control flow structure
        if depth > 5 {
            let messages = [
                "Nesting deeper than Russian dolls, are you writing a maze?",
                "Nesting so deep, trying to dig to the Earth's core?",
                "Code nested like an onion, makes me want to cry",
                "Nesting level exceeded! Consider refactoring",
                "This nesting depth could apply for a Guinness World Record",
            ];
            let severity = if depth > 8 {
                Severity::Nuclear
            } else if depth > 6 {
                Severity::Spicy
            } else {
                Severity::Mild
            };
            let pos = node.start_position();
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: pos.row + 1,
                column: pos.column + 1,
                rule_name: "deep-nesting".to_string(),
                message: format!(
                    "{} (nesting depth: {})",
                    messages[issues.len() % messages.len()],
                    depth
                ),
                severity,
            });
        }
        depth
    } else if is_block_parent {
        depth + 1
    } else {
        depth
    };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        check_nesting_recursive(file, child, new_depth, issues);
    }
}

fn function_query(lang: Language) -> &'static str {
    match lang {
        Language::Rust => "(function_item) @fn",
        Language::Python => "(function_definition) @fn",
        Language::JavaScript => "(function_declaration) @fn",
        Language::C | Language::Cpp => "(function_definition) @fn",
        _ => "(function_item) @fn",
    }
}

// ============================================================================
// Long function detection
// ============================================================================

pub(crate) struct LongFunctionRule;

impl TreeSitterRule for LongFunctionRule {
    fn name(&self) -> &'static str {
        "long-function"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn skips_test_files(&self) -> bool {
        false
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = function_query(file.language);
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let is_test = file.path.to_string_lossy().contains("test");
        let threshold: u32 = if is_test { 150 } else { 80 };
        let content_bytes = file.content.as_bytes();
        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let node = cap.node;
                let start_line = node.start_position().row as u32;
                let end_line = node.end_position().row as u32;
                let line_count = end_line.saturating_sub(start_line) + 1;

                if line_count > threshold {
                    let func_name = find_function_name(node, content_bytes);
                    let messages = [
                        "Function '{}' has {} lines? This isn't a function, it's a novel!",
                        "'{}' function is {} lines long, consider splitting into smaller functions",
                        "Function '{}' is longer than my patience ({} lines), consider refactoring",
                    ];
                    let severity = if line_count > threshold * 2 {
                        Severity::Nuclear
                    } else if line_count > threshold + threshold / 2 {
                        Severity::Spicy
                    } else {
                        Severity::Mild
                    };
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "long-function".to_string(),
                        message: format!(
                            "{} ({} lines)",
                            messages[issues.len() % messages.len()].replace("'{}'", &func_name),
                            line_count
                        ),
                        severity,
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// God function detection
// ============================================================================

pub(crate) struct GodFunctionRule;

impl TreeSitterRule for GodFunctionRule {
    fn name(&self) -> &'static str {
        "god-function"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = function_query(file.language);
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let control_flow_types = [
            "if_expression",
            "for_expression",
            "while_expression",
            "match_expression",
            "loop_expression",
        ];

        let content_bytes = file.content.as_bytes();
        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let node = cap.node;
                let mut score: u32 = 0;

                // Line count
                let start_line = node.start_position().row as u32;
                let end_line = node.end_position().row as u32;
                let line_count = end_line.saturating_sub(start_line) + 1;
                if line_count > 50 {
                    score += (line_count - 50) / 10;
                }

                // Parameter count
                let param_count = count_parameters(node);
                if param_count > 5 {
                    score += (param_count - 5) * 2;
                }

                // Control flow nodes
                score += count_descendants_of_types(node, &control_flow_types);

                if score > 15 {
                    let func_name = find_function_name(node, content_bytes);
                    let messages = [
                        "Function '{}' does more things than I do in a day",
                        "Is '{}' a god function? Wants to control everything",
                        "Function '{}' is as complex as my love life",
                        "Function '{}' needs to be split - too bloated",
                        "Function '{}' violates single responsibility principle",
                    ];
                    let severity = if score > 25 {
                        Severity::Spicy
                    } else {
                        Severity::Mild
                    };
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "god-function".to_string(),
                        message: format!(
                            "{} (score: {})",
                            messages[issues.len() % messages.len()].replace("'{}'", &func_name),
                            score
                        ),
                        severity,
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// Complex closure detection
// ============================================================================

pub(crate) struct ComplexClosureRule;

impl TreeSitterRule for ComplexClosureRule {
    fn name(&self) -> &'static str {
        "complex-closure"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(closure_expression) @closure") {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let closure_node = cap.node;

                // Check nesting depth
                let depth = closure_depth(closure_node);
                if depth > 2 {
                    let messages = [
                        "Closures within closures? Are you writing Russian nesting dolls?",
                        "Nested closures are more complex than my relationships",
                        "These closures are nested like an onion - peel one layer, cry once",
                        "Closures too deeply nested - consider splitting into separate functions",
                    ];
                    let pos = closure_node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "complex-closure".to_string(),
                        message: format!(
                            "{} (depth: {})",
                            messages[issues.len() % messages.len()],
                            depth
                        ),
                        severity: Severity::Spicy,
                    });
                }

                // Check parameter count
                let param_count = closure_node
                    .children(&mut closure_node.walk())
                    .filter(|c| c.kind() == "closure_parameters")
                    .map(|p| p.named_child_count() as u32)
                    .next()
                    .unwrap_or(0);

                if param_count > 5 {
                    let messages = [
                        "This closure has more parameters than my excuses",
                        "Too many parameters for a closure - are you sure this isn't a function?",
                        "So many parameters - consider making this a proper function",
                    ];
                    let pos = closure_node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "complex-closure".to_string(),
                        message: messages[issues.len() % messages.len()].to_string(),
                        severity: Severity::Mild,
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// Terrible naming detection (query + regex)
// ============================================================================

pub(crate) fn variable_name_query(lang: Language) -> &'static str {
    match lang {
        Language::Rust => "(let_declaration pattern: (identifier) @name)",
        Language::Python => "(assignment left: (identifier) @name)",
        Language::JavaScript => "(variable_declarator name: (identifier) @name)",
        _ => "(identifier) @name",
    }
}

pub(crate) struct TerribleNamingRule;

impl TreeSitterRule for TerribleNamingRule {
    fn name(&self) -> &'static str {
        "terrible-naming"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = variable_name_query(file.language);
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let Ok(terrible_re) = Regex::new(
            r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
        ) else {
            tracing::error!("Failed to compile terrible naming regex");
            return vec![];
        };

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text;
                if terrible_re.is_match(&name.to_lowercase()) {
                    let msgs = [
                        format!(
                            "Variable '{}' - more abstract than my programming skills",
                            name
                        ),
                        format!(
                            "Variable '{}' - this name tells me you've given up on naming",
                            name
                        ),
                        format!(
                            "Variable '{}' - trying to make maintainers cry and quit?",
                            name
                        ),
                        format!(
                            "Variable '{}' - congrats on inventing the most meaningless identifier",
                            name
                        ),
                        format!(
                            "Variable '{}' - creativity level of naming a kid 'Child'",
                            name
                        ),
                    ];
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "terrible-naming".to_string(),
                        message: msgs[issues.len() % msgs.len()].clone(),
                        severity: Severity::Spicy,
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// Single letter variable detection (query + exclude list)
// ============================================================================

fn single_letter_query(lang: Language) -> &'static str {
    match lang {
        Language::Rust => {
            r#"(let_declaration pattern: (identifier) @var (#match? @var "^[a-z]$"))"#
        }
        Language::Python => r#"(assignment left: (identifier) @var (#match? @var "^[a-z]$"))"#,
        Language::JavaScript => {
            r#"(variable_declarator name: (identifier) @var (#match? @var "^[a-z]$"))"#
        }
        Language::Go => {
            r#"(short_variable_declaration left: (identifier) @var (#match? @var "^[a-z]$"))"#
        }
        Language::Java => {
            r#"(variable_declarator name: (identifier) @var (#match? @var "^[a-z]$"))"#
        }
        Language::Ruby => r#"(assignment left: (identifier) @var (#match? @var "^[a-z]$"))"#,
        Language::C | Language::Cpp => {
            r#"(init_declarator declarator: (identifier) @var (#match? @var "^[a-z]$"))"#
        }
        _ => r#"(identifier) @var"#,
    }
}

/// Check if a node is a loop counter variable (not a body variable).
/// For for-loops: the loop variable is the first named child.
/// For while/loop/do-while: no loop variable exists.
fn is_loop_counter(node: tree_sitter::Node) -> bool {
    let mut current = node;
    loop {
        let kind = current.kind();
        if matches!(kind, "for_expression" | "for_statement") {
            // The loop variable is the first named child of the for node
            // (Rust: pattern, C: declaration, JS/Go: initializer, Python: left)
            if let Some(first) = current.named_child(0) {
                let ns = node.start_byte();
                let ne = node.end_byte();
                let vs = first.start_byte();
                let ve = first.end_byte();
                if ns >= vs && ne <= ve {
                    return true; // node IS the loop variable
                }
            }
            return false; // not a loop variable (might be in body)
        }
        if matches!(
            kind,
            "while_statement" | "while_expression" | "loop_expression" | "do_statement"
        ) {
            return false; // these loops have no explicit counter variable
        }
        match current.parent() {
            Some(p) => current = p,
            None => return false,
        }
    }
}

/// Check if an identifier is a C++ template parameter.
fn is_template_param(node: tree_sitter::Node) -> bool {
    let mut parent = node.parent();
    // Walk up to find template_parameter_declaration
    while let Some(p) = parent {
        if p.kind() == "template_parameter_declaration" || p.kind() == "type_parameter" {
            return true;
        }
        // If we hit a function/class/struct boundary, stop
        if matches!(
            p.kind(),
            "function_definition"
                | "function_declaration"
                | "class_specifier"
                | "struct_specifier"
                | "translation_unit"
        ) {
            break;
        }
        parent = p.parent();
    }
    false
}

pub(crate) struct SingleLetterTsRule;

impl TreeSitterRule for SingleLetterTsRule {
    fn name(&self) -> &'static str {
        "single-letter-variable"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = single_letter_query(file.language);
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Loop counters are handled by inside_loop() check per-capture

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text;
                // Only flag single-character identifiers
                if name.len() != 1 {
                    continue;
                }

                // Skip identifiers that are loop counters (not body variables)
                // Tree-based detection — no whitelist needed
                if is_loop_counter(cap.node) {
                    continue;
                }

                // Skip C++ template parameters
                if file.language == Language::Cpp && is_template_param(cap.node) {
                    continue;
                }

                let msgs = [
                    format!(
                        "Single-letter variable '{}'? Writing math formulas or torturing readers?",
                        name
                    ),
                    format!(
                        "Variable '{}'? Is this a name or did your keyboard break?",
                        name
                    ),
                    format!(
                        "Using '{}' as a variable name? You need a book on naming",
                        name
                    ),
                    format!(
                        "Single-letter variable '{}': harder to read than hieroglyphics",
                        name
                    ),
                    format!("Variable '{}' has about as much info as a period", name),
                ];
                let pos = cap.node.start_position();
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "single-letter-variable".to_string(),
                    message: msgs[issues.len() % msgs.len()].clone(),
                    severity: Severity::Mild,
                });
            }
        }

        issues
    }
}

// ============================================================================
// Hungarian notation detection (query + regex)
// ============================================================================

pub(crate) struct HungarianNotationTsRule;

impl TreeSitterRule for HungarianNotationTsRule {
    fn name(&self) -> &'static str {
        "hungarian-notation"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(identifier) @id") {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let prefixes = [
            "str", "int", "bool", "float", "double", "char", "arr", "vec", "list", "map", "set",
        ];
        let scope_prefixes = ["g_", "m_", "s_", "p_"];

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text;
                let is_hungarian = scope_prefixes.iter().any(|p| name.starts_with(p))
                    || prefixes.iter().any(|&prefix| {
                        name.starts_with(prefix)
                            && name.len() > prefix.len()
                            && name
                                .chars()
                                .nth(prefix.len())
                                .is_some_and(|c| c.is_uppercase())
                            && !name[prefix.len()..].starts_with("ify")
                            && !name[prefix.len()..].starts_with("nal")
                            && !name[prefix.len()..].starts_with("ean")
                    });

                if is_hungarian {
                    let messages = [
                        format!(
                            "'{}' uses Hungarian notation? This isn't the 1990s anymore",
                            name
                        ),
                        format!(
                            "Seeing '{}' makes me nostalgic for the dark ages of C++",
                            name
                        ),
                        format!(
                            "'{}' - Hungarian notation is as outdated as my haircut",
                            name
                        ),
                        format!(
                            "Hungarian notation '{}'? Rust's type system has got you covered",
                            name
                        ),
                    ];
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "hungarian-notation".to_string(),
                        message: messages[issues.len() % messages.len()].clone(),
                        severity: Severity::Mild,
                    });
                }
            }
        }

        issues
    }
}

// ============================================================================
// Abbreviation abuse detection (query + lookup)
// ============================================================================

pub(crate) struct AbbreviationAbuseTsRule;

impl TreeSitterRule for AbbreviationAbuseTsRule {
    fn name(&self) -> &'static str {
        "abbreviation-abuse"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(identifier) @id") {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let bad_abbrevs: &[(&str, &str)] = &[
            ("mgr", "manager"),
            ("mngr", "manager"),
            ("ctrl", "controller"),
            ("hdlr", "handler"),
            ("usr", "user"),
            ("pwd", "password"),
            ("prefs", "preferences"),
            ("btn", "button"),
            ("lbl", "label"),
            ("pic", "picture"),
            ("tbl", "table"),
            ("col", "column"),
            ("cnt", "count"),
        ];

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text;
                let name_lower = name.to_lowercase();

                for &(abbrev, full) in bad_abbrevs {
                    if name_lower == abbrev || name_lower.starts_with(&format!("{}_", abbrev)) {
                        let messages = [
                            format!("'{}' is too abbreviated, consider '{}'", name, full),
                            format!(
                                "Seeing '{}' makes me feel like I'm decoding, just use '{}'",
                                name, full
                            ),
                            format!(
                                "'{}' saves a few letters but kills readability, use '{}'",
                                name, full
                            ),
                        ];
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "abbreviation-abuse".to_string(),
                            message: messages[issues.len() % messages.len()].clone(),
                            severity: Severity::Mild,
                        });
                        break;
                    }
                }
            }
        }

        issues
    }
}

fn print_debug_query(lang: Language) -> &'static str {
    match lang {
        Language::Rust => r#"(macro_invocation macro: (identifier) @m (#eq? @m "println"))"#,
        Language::Python => r#"(call function: (identifier) @f (#eq? @f "print"))"#,
        Language::JavaScript => {
            r#"(call_expression function: (member_expression object: (identifier) @o property: (property_identifier) @p) (#eq? @o "console") (#eq? @p "log"))"#
        }
        _ => "",
    }
}

pub(crate) struct PrintlnDebuggingRule;

impl TreeSitterRule for PrintlnDebuggingRule {
    fn name(&self) -> &'static str {
        "println-debugging"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = print_debug_query(file.language);
        if query.is_empty() {
            return vec![];
        }
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                let pos = cap.node.start_position();
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "println-debugging".to_string(),
                    message: format!("{} - use proper logging instead", cap.text),
                    severity: Severity::Spicy,
                });
            }
        }
        issues
    }
}

fn number_literal_query(lang: Language) -> &'static str {
    match lang {
        Language::Rust => "(integer_literal) @num",
        Language::Python => "(integer) @num",
        Language::JavaScript | Language::TypeScript => "(number) @num",
        Language::C | Language::Cpp => "(number_literal) @num",
        _ => "",
    }
}

pub(crate) struct MagicNumberRule;

impl TreeSitterRule for MagicNumberRule {
    fn name(&self) -> &'static str {
        "magic-number"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = number_literal_query(file.language);
        if query.is_empty() {
            return vec![];
        }
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        // Named-constant parent kinds: skip literals directly assigned to const/let
        let named_parents: &[&str] = &[
            "const_item",
            "let_declaration",
            "assignment",
            "variable_declarator",
        ];
        // Switch case labels: skip literals that are case values
        let case_label_kinds: &[&str] = &["case", "switch_case", "case_statement"];
        let common: &[&str] = &["0", "1", "-1", "2", "100", "0.0", "1.0", "10", "60", "24"];
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                let text = cap.text.trim();
                // Skip literals assigned to named constants
                let parent = cap.node.parent();
                if parent.is_some_and(|p| named_parents.contains(&p.kind())) {
                    continue;
                }
                // Skip literals that are switch case labels (not magic numbers)
                if parent.is_some_and(|p| case_label_kinds.contains(&p.kind())) {
                    continue;
                }
                if !common.contains(&text) && text.parse::<f64>().is_ok() {
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "magic-number".to_string(),
                        message: format!("Magic number '{}' - consider a named constant", text),
                        severity: Severity::Mild,
                    });
                }
            }
        }
        issues
    }
}
