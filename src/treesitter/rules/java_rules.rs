use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_java_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(EmptyCatchRule));
    engine.add(Box::new(ConstantNameRule));
    engine.add(Box::new(JavaJavadocMissingRule));
    engine.add(Box::new(JavaTryResourceRule));
    engine.add(Box::new(JavaStringConcatRule));
    engine.add(Box::new(JavaWildcardImportRule));
}

/// Empty catch block: `catch (Exception e) {}`
/// Swallowing exceptions silently is the #1 source of "it works on my machine"
/// bugs that take 3 days to debug in production.
struct EmptyCatchRule;

impl TreeSitterRule for EmptyCatchRule {
    fn name(&self) -> &'static str {
        "empty-catch"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let root = file.root_node();
        find_empty_catch(file, root, &mut issues);
        issues
    }
}

fn find_empty_catch(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    if node.kind() == "catch_clause" {
        // Check if the body block is empty
        if let Some(body) = node.child_by_field_name("body") {
            let named_count = body.named_child_count();
            if named_count == 0 {
                let pos = node.start_position();
                let msgs = [
                    "Empty catch block — exceptions don't fix themselves by being ignored",
                    "Swallowing exceptions silently? Bold strategy, let's see how it plays out.",
                    "Empty catch block. Future you will spend 3 days debugging this.",
                    "Catch block is empty. At least log it, you monster.",
                    "Empty catch block detected — the debugging equivalent of 'la la la I can't hear you'",
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "empty-catch".to_string(),
                    message: msgs[issues.len() % msgs.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_empty_catch(file, child, issues);
    }
}

// ─── Java constant naming: static final fields should be UPPER_SNAKE_CASE ──

struct ConstantNameRule;

impl TreeSitterRule for ConstantNameRule {
    fn name(&self) -> &'static str {
        "constant-name"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        // Simple text-based check: find lines with "static final" or "final static"
        // followed by a type and name, check if name is UPPER_SNAKE_CASE
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.contains("static final") && !trimmed.contains("final static") {
                continue;
            }
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            // Find constant name: word right before '=' or ';'
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let name = parts
                .iter()
                .position(|p| *p == "=" || p.ends_with('=') || p.ends_with(';'))
                .and_then(|idx| {
                    if idx > 0 {
                        parts
                            .get(idx - 1)
                            .map(|s| s.trim_end_matches('=').trim_end_matches(';'))
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            if !name.is_empty()
                && name != name.to_uppercase()
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find(name).unwrap_or(0) + 1,
                    rule_name: "constant-name".to_string(),
                    message: format!("'{}' should be UPPER_SNAKE_CASE for constants", name),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Java: public methods should have Javadoc ─────────────────────

struct JavaJavadocMissingRule;

impl TreeSitterRule for JavaJavadocMissingRule {
    fn name(&self) -> &'static str {
        "java-javadoc-missing"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("public ") || trimmed.starts_with("protected ") {
                // Check if this is a method declaration
                if trimmed.contains("(")
                    && (trimmed.contains(")")
                        || lines.get(i + 1).is_some_and(|l| l.trim().contains(")")))
                {
                    // Check if previous non-blank line has Javadoc
                    let mut j = i as i32 - 1;
                    while j >= 0 && lines[j as usize].trim().is_empty() {
                        j -= 1;
                    }
                    if j >= 0 {
                        let prev = lines[j as usize].trim();
                        if prev.starts_with("/**") || prev.ends_with("*/") {
                            i += 1;
                            continue;
                        }
                    }
                    // Also check the line itself for multi-line annotation
                    if trimmed.starts_with("@Override") || trimmed.starts_with("@Suppress") {
                        i += 1;
                        continue;
                    }
                    let pos = lines[i].find(trimmed).unwrap_or(0);
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: i + 1,
                        column: pos + 1,
                        rule_name: "java-javadoc-missing".to_string(),
                        message: "Public/protected method is missing Javadoc comment".to_string(),
                        severity: Severity::Mild,
                    });
                }
            }
            i += 1;
        }
        issues
    }
}

// ─── Java: try-finally with .close() should use try-with-resources ─

struct JavaTryResourceRule;

impl TreeSitterRule for JavaTryResourceRule {
    fn name(&self) -> &'static str {
        "java-try-resource"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = file.content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("finally") {
                // Check a window of 3 lines after "finally" for .close()
                for k in 1..=3 {
                    if let Some(next) = lines.get(line_num + k) {
                        if next.trim().contains(".close()") {
                            issues.push(CodeIssue {
                                file_path: file.path.clone(),
                                line: line_num + 1,
                                column: trimmed.find("finally").unwrap_or(0) + 1,
                                rule_name: "java-try-resource".to_string(),
                                message:
                                    "Use try-with-resources instead of try-finally with .close()"
                                        .to_string(),
                                severity: Severity::Mild,
                            });
                            break;
                        }
                    }
                }
            }
        }
        issues
    }
}

// ─── Java: string concatenation in loops ──────────────────────────

struct JavaStringConcatRule;

impl TreeSitterRule for JavaStringConcatRule {
    fn name(&self) -> &'static str {
        "java-string-concat"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let content = &file.content;
        // Scan for for/while loop patterns followed by += within the same block
        let in_loop = content.contains("for ") || content.contains("while ");
        if !in_loop {
            return issues;
        }
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(" += ") && !trimmed.starts_with("//") && !trimmed.starts_with("/*")
            {
                // Check if there's a for/while loop within the last 10 lines
                let lines: Vec<&str> = file.content.lines().collect();
                let start = line_num.saturating_sub(10);
                for k in (start..line_num).rev() {
                    let prev = lines[k].trim();
                    if prev.starts_with("for ") || prev.starts_with("while ") {
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: line_num + 1,
                            column: trimmed.find("+=").unwrap_or(0) + 1,
                            rule_name: "java-string-concat".to_string(),
                            message: "Use StringBuilder instead of string concatenation in loop"
                                .to_string(),
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

// ─── Java: wildcard import (Google Java Style §3.3.1) ──────────────

struct JavaWildcardImportRule;

impl TreeSitterRule for JavaWildcardImportRule {
    fn name(&self) -> &'static str {
        "java-wildcard-import"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") && trimmed.ends_with(".*;") {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: 1,
                    rule_name: "java-wildcard-import".to_string(),
                    message:
                        "Avoid wildcard imports — use explicit imports (Google Java Style §3.3.1)"
                            .to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_java;
    use super::*;

    #[test]
    fn test_empty_catch_detected() {
        let file = parse_java(
            r#"
class Test {
    void foo() {
        try {
            risky();
        } catch (Exception e) {
        }
    }
}
"#,
        );
        let rule = EmptyCatchRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect empty catch");
    }

    #[test]
    fn test_non_empty_catch_not_detected() {
        let file = parse_java(
            r#"
class Test {
    void foo() {
        try {
            risky();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
"#,
        );
        let rule = EmptyCatchRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Non-empty catch should not trigger");
    }

    #[test]
    fn test_constant_name_uppercase_ok() {
        let file = parse_java(
            r#"
public class Test {
    public static final int MAX_SIZE = 100;
}
"#,
        );
        let rule = ConstantNameRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "UPPER_CASE constant should be OK");
    }

    #[test]
    fn test_javadoc_missing_detected() {
        let file = parse_java(
            r#"
public class Test {
    public void foo() {}
}
"#,
        );
        let rule = JavaJavadocMissingRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Missing Javadoc should be flagged");
        assert_eq!(issues[0].rule_name, "java-javadoc-missing");
    }

    #[test]
    fn test_javadoc_present_ok() {
        let file = parse_java(
            r#"
public class Test {
    /** Does something */
    public void foo() {}
}
"#,
        );
        let rule = JavaJavadocMissingRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "With Javadoc should be OK");
    }

    #[test]
    fn test_try_resource_detected() {
        let file = parse_java(
            r#"
try {
    // do stuff
} finally {
    resource.close();
}
"#,
        );
        let rule = JavaTryResourceRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "try-finally close should be flagged");
        assert_eq!(issues[0].rule_name, "java-try-resource");
    }

    #[test]
    fn test_try_resource_twr_ok() {
        let file = parse_java(
            r#"
try (var r = new Resource()) {
    // do stuff
}
"#,
        );
        let rule = JavaTryResourceRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "try-with-resources should not be flagged"
        );
    }

    #[test]
    fn test_string_concat_in_loop_detected() {
        let file = parse_java(
            r#"
for (int i = 0; i < 10; i++) {
    result += items[i];
}
"#,
        );
        let rule = JavaStringConcatRule;
        let issues = rule.check(&file);
        assert!(
            !issues.is_empty(),
            "String concat in loop should be flagged"
        );
        assert_eq!(issues[0].rule_name, "java-string-concat");
    }

    #[test]
    fn test_string_concat_outside_loop_ok() {
        let file = parse_java(
            r#"
String result = a + b;
// no string concat inside this loop
for (int i = 0; i < 10; i++) {
    processItem(i);
}
"#,
        );
        let rule = JavaStringConcatRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Outside loop + no += inside loop should not be flagged"
        );
    }

    #[test]
    fn test_constant_name_lowercase_flagged() {
        let file = parse_java(
            r#"
public class Test {
    public static final int maxSize = 100;
}
"#,
        );
        let rule = ConstantNameRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "lowercase constant should be flagged");
    }

    #[test]
    fn test_wildcard_import_detected() {
        let file = parse_java(
            r#"
import java.util.*;
import java.io.BufferedReader;
"#,
        );
        let rule = JavaWildcardImportRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "Wildcard import should be flagged");
        assert_eq!(issues[0].rule_name, "java-wildcard-import");
    }

    #[test]
    fn test_explicit_import_ok() {
        let file = parse_java(
            r#"
import java.util.List;
import java.util.ArrayList;
import java.io.BufferedReader;
"#,
        );
        let rule = JavaWildcardImportRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Explicit imports should not be flagged");
    }
}
