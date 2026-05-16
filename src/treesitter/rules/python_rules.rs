use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_python_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(BareExceptRule));
    engine.add(Box::new(WildcardImportRule));
    engine.add(Box::new(PythonNamingRule));
    engine.add(Box::new(ComparedToBoolRule));
    engine.add(Box::new(NotIsNoneRule));
    engine.add(Box::new(PythonTypeIgnoreRule));
    engine.add(Box::new(PythonFStringRule));
    engine.add(Box::new(PythonMagicMethodRule));
    engine.add(Box::new(PythonImportOrderRule));
}

/// bare except: `except:` without specifying exception type.
/// Every language has `except SomeError:`, but bare `except:` catches
/// KeyboardInterrupt, SystemExit, GeneratorExit — things you almost
/// never want to swallow silently.
struct BareExceptRule;

impl TreeSitterRule for BareExceptRule {
    fn name(&self) -> &'static str {
        "bare-except"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        // Walk AST: except_clause nodes without a 'value' child are bare except
        let mut issues = Vec::new();
        let root = file.root_node();
        find_bare_except(file, root, &mut issues);
        issues
    }
}

fn find_bare_except(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    if node.kind() == "except_clause" {
        // bare except has no 'value' field child
        if node.child_by_field_name("value").is_none() {
            let pos = node.start_position();
            let msgs = [
                "Bare `except:` — catching KeyboardInterrupt and SystemExit too? Bold move.",
                "Bare `except:` is the programming equivalent of '¯\\_(ツ)_/¯'",
                "Bare `except:` — when you don't care what breaks, as long as it stops complaining",
                "Using bare `except:`? Even `except Exception:` would be less reckless.",
            ];
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: pos.row + 1,
                column: pos.column + 1,
                rule_name: "bare-except".to_string(),
                message: msgs[issues.len() % msgs.len()].to_string(),
                severity: Severity::Spicy,
            });
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_bare_except(file, child, issues);
    }
}

/// wildcard import: `from module import *`
/// Pollutes the namespace, makes it impossible to track where names come from.
struct WildcardImportRule;

impl TreeSitterRule for WildcardImportRule {
    fn name(&self) -> &'static str {
        "wildcard-import"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(wildcard_import) @wi") {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        // Libraries where `import *` is idiomatic
        let acceptable_modules = [
            "manim",
            "numpy",
            "matplotlib",
            "pytest",
            "tensorflow",
            "torch",
            "tkinter",
            "PyQt5",
            "PySide6",
            "gi.repository",
        ];
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                let line = cap.node.start_position().row;
                // Check the source line for acceptable module imports
                if let Some(source_line) = file.content.lines().nth(line) {
                    let is_acceptable = acceptable_modules
                        .iter()
                        .any(|m| source_line.contains(&format!("from {} import *", m)));
                    if is_acceptable {
                        continue;
                    }
                }
                let pos = cap.node.start_position();
                let msgs = [
                    "`import *` — namespace pollution speedrun any%",
                    "Wildcard import? Good luck finding where that name came from.",
                    "`from x import *` — the 'I give up on explicit imports' special",
                    "Wildcard import detected. Your IDE is crying.",
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "wildcard-import".to_string(),
                    message: msgs[issues.len() % msgs.len()].to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Python naming convention: snake_case functions, PascalCase classes ──

struct PythonNamingRule;

impl TreeSitterRule for PythonNamingRule {
    fn name(&self) -> &'static str {
        "python-naming"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        // Check function definitions: not snake_case
        if let Ok(captures) = collect_captures(file, "(function_definition name: (identifier) @fn)")
        {
            for group in &captures {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.chars().any(|c| c.is_uppercase()) && !name.starts_with("__") {
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "python-naming".to_string(),
                            message: format!(
                                "'{}' should be snake_case (e.g. '{}')",
                                name,
                                to_snake_case(name)
                            ),
                            severity: Severity::Mild,
                        });
                    }
                }
            }
        }
        // Check class definitions: not PascalCase
        if let Ok(captures) = collect_captures(file, "(class_definition name: (identifier) @cls)") {
            for group in &captures {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.chars().next().is_some_and(|c| c.is_lowercase()) {
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "python-naming".to_string(),
                            message: format!(
                                "'{}' should be PascalCase (e.g. '{}')",
                                name,
                                to_pascal_case(name)
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

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut upper = true;
    for c in name.chars() {
        if c == '_' {
            upper = true;
            continue;
        }
        if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

// ─── Compared to bool: `if x == True` → `if x` ─────────────────────

struct ComparedToBoolRule;

impl TreeSitterRule for ComparedToBoolRule {
    fn name(&self) -> &'static str {
        "compared-to-bool"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if (trimmed.contains("== True") || trimmed.contains("== False"))
                && !trimmed.contains("is True")
                && !trimmed.contains("is False")
            {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find("==").unwrap_or(0) + 1,
                    rule_name: "compared-to-bool".to_string(),
                    message: "Comparing to True/False via '==' is redundant — use 'if x:' instead"
                        .to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Not is None: `if x == None` → `if x is None` ─────────────────

struct NotIsNoneRule;

impl TreeSitterRule for NotIsNoneRule {
    fn name(&self) -> &'static str {
        "not-is-none"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains("== None") && !trimmed.contains("is None") {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find("==").unwrap_or(0) + 1,
                    rule_name: "not-is-none".to_string(),
                    message: "Use 'is None' instead of '== None'".to_string(),
                    severity: Severity::Mild,
                });
            }
            if trimmed.contains("!= None") && !trimmed.contains("is not None") {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find("!=").unwrap_or(0) + 1,
                    rule_name: "not-is-none".to_string(),
                    message: "Use 'is not None' instead of '!= None'".to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Python type: ignore — detect # type: ignore comments ────────

struct PythonTypeIgnoreRule;

impl TreeSitterRule for PythonTypeIgnoreRule {
    fn name(&self) -> &'static str {
        "python-type-ignore"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            if line.trim().contains("# type: ignore") {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: line.find("# type: ignore").unwrap_or(0) + 1,
                    rule_name: "python-type-ignore".to_string(),
                    message: "Found '# type: ignore' — fix the type issue instead of hiding it"
                        .to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Python f-string — prefer f-strings over .format() and % ───────

struct PythonFStringRule;

impl TreeSitterRule for PythonFStringRule {
    fn name(&self) -> &'static str {
        "python-fstring"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains(".format(") && !trimmed.contains("f-string") {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find(".format(").unwrap_or(0) + 1,
                    rule_name: "python-fstring".to_string(),
                    message: "Use f-string instead of .format()".to_string(),
                    severity: Severity::Mild,
                });
            } else if trimmed.matches('%').count() >= 2
                && !trimmed.contains("'%")
                && !trimmed.contains("\"%")
                && (trimmed.contains("%s") || trimmed.contains("%d") || trimmed.contains("%r"))
            {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: trimmed.find('%').unwrap_or(0) + 1,
                    rule_name: "python-fstring".to_string(),
                    message: "Use f-string instead of % formatting".to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

// ─── Python magic method — non-standard __dunder__ methods ────────

struct PythonMagicMethodRule;

impl TreeSitterRule for PythonMagicMethodRule {
    fn name(&self) -> &'static str {
        "python-magic-method"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let standard_dunders = [
            "__init__",
            "__new__",
            "__del__",
            "__repr__",
            "__str__",
            "__bytes__",
            "__format__",
            "__lt__",
            "__le__",
            "__eq__",
            "__ne__",
            "__gt__",
            "__ge__",
            "__hash__",
            "__bool__",
            "__getattr__",
            "__getattribute__",
            "__setattr__",
            "__delattr__",
            "__call__",
            "__len__",
            "__getitem__",
            "__setitem__",
            "__delitem__",
            "__iter__",
            "__next__",
            "__reversed__",
            "__contains__",
            "__enter__",
            "__exit__",
            "__aenter__",
            "__aexit__",
            "__await__",
            "__aiter__",
            "__anext__",
            "__add__",
            "__sub__",
            "__mul__",
            "__truediv__",
            "__floordiv__",
            "__mod__",
            "__divmod__",
            "__pow__",
            "__lshift__",
            "__rshift__",
            "__and__",
            "__xor__",
            "__or__",
            "__radd__",
            "__rsub__",
            "__rmul__",
            "__rtruediv__",
            "__rfloordiv__",
            "__rmod__",
            "__rdivmod__",
            "__rpow__",
            "__rlshift__",
            "__rrshift__",
            "__rand__",
            "__rxor__",
            "__ror__",
            "__iadd__",
            "__isub__",
            "__imul__",
            "__itruediv__",
            "__ifloordiv__",
            "__imod__",
            "__ipow__",
            "__ilshift__",
            "__irshift__",
            "__iand__",
            "__ixor__",
            "__ior__",
            "__neg__",
            "__pos__",
            "__abs__",
            "__invert__",
            "__complex__",
            "__int__",
            "__float__",
            "__round__",
            "__index__",
            "__copy__",
            "__deepcopy__",
            "__sizeof__",
            "__reduce__",
            "__reduce_ex__",
            "__getnewargs__",
            "__getstate__",
            "__setstate__",
            "__dir__",
            "__class__",
            "__subclasshook__",
            "__init_subclass__",
            "__instancecheck__",
            "__subclasscheck__",
            "__fspath__",
            "__prepare__",
            "__slots__",
        ];
        let mut issues = Vec::new();
        if let Ok(captures) = collect_captures(file, "(function_definition name: (identifier) @fn)")
        {
            for group in &captures {
                if let Some(cap) = group.first() {
                    let name = &cap.text;
                    if name.starts_with("__")
                        && name.ends_with("__")
                        && !standard_dunders.contains(name)
                    {
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "python-magic-method".to_string(),
                            message: format!(
                                "'{}' is not a standard dunder method — avoid custom magic methods",
                                name
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

// ─── Python: import order (PEP 8) — stdlib before third-party ───────

/// Python stdlib module names (common subset for detection)
const PYTHON_STDLIB_MODULES: &[&str] = &[
    "os",
    "sys",
    "re",
    "json",
    "math",
    "datetime",
    "time",
    "collections",
    "functools",
    "itertools",
    "typing",
    "pathlib",
    "io",
    "abc",
    "copy",
    "enum",
    "dataclasses",
    "logging",
    "unittest",
    "argparse",
    "subprocess",
    "threading",
    "multiprocessing",
    "socket",
    "http",
    "urllib",
    "email",
    "html",
    "xml",
    "csv",
    "hashlib",
    "hmac",
    "secrets",
    "base64",
    "struct",
    "pickle",
    "shelve",
    "sqlite3",
    "gzip",
    "zipfile",
    "tarfile",
    "shutil",
    "tempfile",
    "glob",
    "fnmatch",
    "contextlib",
    "textwrap",
    "string",
    "operator",
    "bisect",
    "heapq",
    "array",
    "weakref",
    "types",
    "pprint",
    "warnings",
    "traceback",
    "inspect",
    "importlib",
    "pkgutil",
    "pdb",
    "profile",
    "timeit",
    "dis",
    "ast",
    "token",
    "tokenize",
    "keyword",
    "platform",
    "ctypes",
    "concurrent",
    "asyncio",
    "signal",
    "mmap",
    "codecs",
    "locale",
    "gettext",
    "unicodedata",
    "difflib",
    "textwrap",
];

struct PythonImportOrderRule;

impl TreeSitterRule for PythonImportOrderRule {
    fn name(&self) -> &'static str {
        "python-import-order"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let mut seen_third_party = false;
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments, blanks, and non-import lines
            if trimmed.is_empty() || trimmed.starts_with("#") {
                continue;
            }
            if !trimmed.starts_with("import ") && !trimmed.starts_with("from ") {
                // Reset state when we hit non-import code
                if !trimmed.is_empty() {
                    seen_third_party = false;
                }
                continue;
            }
            // Extract module name
            let module = if trimmed.starts_with("from ") {
                trimmed
                    .strip_prefix("from ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
            } else {
                trimmed
                    .strip_prefix("import ")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
            };
            // Skip relative imports (they're always "local")
            if module.starts_with('.') {
                continue;
            }
            let top_module = module.split('.').next().unwrap_or(module);
            let is_stdlib = PYTHON_STDLIB_MODULES.contains(&top_module);
            if !is_stdlib {
                seen_third_party = true;
            } else if seen_third_party {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: 1,
                    rule_name: "python-import-order".to_string(),
                    message: format!(
                        "stdlib import '{}' should come before third-party imports (PEP 8)",
                        module
                    ),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_python;
    use super::*;

    #[test]
    fn test_bare_except_detected() {
        let file = parse_python(
            r#"
try:
    risky()
except:
    pass
"#,
        );
        let rule = BareExceptRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect bare except");
    }

    #[test]
    fn test_typed_except_not_detected() {
        let file = parse_python(
            r#"
try:
    risky()
except ValueError:
    pass
"#,
        );
        let rule = BareExceptRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Typed except should not trigger");
    }

    #[test]
    fn test_wildcard_import_detected() {
        let file = parse_python(
            r#"
from os import *
from sys import *
"#,
        );
        let rule = WildcardImportRule;
        let issues = rule.check(&file);
        assert!(issues.len() >= 2, "Should detect both wildcard imports");
    }

    #[test]
    fn test_normal_import_not_detected() {
        let file = parse_python(
            r#"
from os import path
import sys
"#,
        );
        let rule = WildcardImportRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Normal imports should not trigger");
    }

    // ── Python naming tests ─────────────────────────────────

    #[test]
    fn test_python_naming_camelcase_function() {
        let file = parse_python("def badFunction(): pass\n");
        let rule = PythonNamingRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "camelCase function should be flagged");
    }

    #[test]
    fn test_python_naming_snakecase_function_ok() {
        let file = parse_python("def good_function(): pass\n");
        let rule = PythonNamingRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "snake_case function should not be flagged"
        );
    }

    #[test]
    fn test_python_naming_snakecase_class() {
        let file = parse_python("class bad_class: pass\n");
        let rule = PythonNamingRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "snake_case class should be flagged");
    }

    #[test]
    fn test_python_naming_pascalcase_class_ok() {
        let file = parse_python("class GoodClass: pass\n");
        let rule = PythonNamingRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "PascalCase class should not be flagged");
    }

    // ── Compared-to-bool tests ──────────────────────────────

    #[test]
    fn test_compared_to_bool_detected() {
        let file = parse_python("if x == True: pass\nif y == False: pass\n");
        let rule = ComparedToBoolRule;
        let issues = rule.check(&file);
        assert_eq!(
            issues.len(),
            2,
            "Both == True and == False should be flagged"
        );
    }

    #[test]
    fn test_compared_to_bool_is_ok() {
        let file = parse_python("if x is True: pass\nif x: pass\n");
        let rule = ComparedToBoolRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "'is True' and bare 'if x' should not be flagged"
        );
    }

    // ── Not-is-none tests ───────────────────────────────────

    #[test]
    fn test_not_is_none_eq_detected() {
        let file = parse_python("if x == None: pass\n");
        let rule = NotIsNoneRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "x == None should be flagged");
    }

    #[test]
    fn test_python_type_ignore_detected() {
        let file = parse_python("x = get_value()  # type: ignore\n");
        let rule = PythonTypeIgnoreRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "# type: ignore should be flagged");
        assert_eq!(issues[0].rule_name, "python-type-ignore");
    }

    #[test]
    fn test_python_type_ignore_clean_ok() {
        let file = parse_python("x = get_value()\n");
        let rule = PythonTypeIgnoreRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Clean code should not be flagged");
    }

    #[test]
    fn test_python_fstring_format_detected() {
        let file = parse_python("msg = \"hello {}\".format(name)\n");
        let rule = PythonFStringRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, ".format() should be flagged");
        assert_eq!(issues[0].rule_name, "python-fstring");
    }

    #[test]
    fn test_python_fstring_percent_detected() {
        let file = parse_python("msg = \"hello %s\" % name\n");
        let rule = PythonFStringRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "% formatting should be flagged");
    }

    #[test]
    fn test_python_fstring_ok() {
        let file = parse_python("msg = f\"hello {name}\"\n");
        let rule = PythonFStringRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "f-string should not be flagged");
    }

    #[test]
    fn test_python_magic_method_detected() {
        let file = parse_python("class Foo:\n    def __custom__(self): pass\n");
        let rule = PythonMagicMethodRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "custom dunder should be flagged");
        assert_eq!(issues[0].rule_name, "python-magic-method");
    }

    #[test]
    fn test_python_magic_method_standard_ok() {
        let file =
            parse_python("class Foo:\n    def __init__(self): pass\n    def __str__(self): pass\n");
        let rule = PythonMagicMethodRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Standard dunder methods should not be flagged"
        );
    }

    #[test]
    fn test_not_is_none_is_ok() {
        let file = parse_python("if x is None: pass\nif x is not None: pass\n");
        let rule = NotIsNoneRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "'is None' and 'is not None' should not be flagged"
        );
    }

    #[test]
    fn test_import_order_ok() {
        let file = parse_python("import os\nimport sys\nimport requests\n");
        let rule = PythonImportOrderRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "stdlib before third-party should not be flagged"
        );
    }

    #[test]
    fn test_import_order_wrong() {
        let file = parse_python("import requests\nimport os\nimport sys\n");
        let rule = PythonImportOrderRule;
        let issues = rule.check(&file);
        assert_eq!(
            issues.len(),
            2,
            "stdlib after third-party should be flagged"
        );
        assert_eq!(issues[0].rule_name, "python-import-order");
    }

    #[test]
    fn test_import_order_relative_ok() {
        let file = parse_python("import requests\nfrom . import utils\nfrom .helpers import foo\n");
        let rule = PythonImportOrderRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Relative imports after third-party should not be flagged"
        );
    }
}
