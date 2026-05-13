use std::path::Path;
use syn::{visit::Visit, ExprClosure, File, GenericParam, ItemImpl, ItemTrait, Lifetime};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// Detects closures that are too complex and should be extracted to named functions.
pub struct ComplexClosureRule;

impl Rule for ComplexClosureRule {
    fn name(&self) -> &'static str {
        "complex-closure"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = ClosureVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// Detects excessive or unnecessary lifetime annotations.
pub struct LifetimeAbuseRule;

impl Rule for LifetimeAbuseRule {
    fn name(&self) -> &'static str {
        "lifetime-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = LifetimeVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// Detects overly complex trait definitions with too many methods or bounds.
pub struct TraitComplexityRule;

impl Rule for TraitComplexityRule {
    fn name(&self) -> &'static str {
        "trait-complexity"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = TraitVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// Detects excessive generic type parameters that hurt readability.
pub struct GenericAbuseRule;

impl Rule for GenericAbuseRule {
    fn name(&self) -> &'static str {
        "generic-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = GenericVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct ClosureVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    closure_depth: usize,
    lang: String,
}

impl ClosureVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            closure_depth: 0,
            lang: lang.to_string(),
        }
    }

    fn check_closure_complexity(&mut self, closure: &ExprClosure) {
        // Check for nested closures
        if self.closure_depth > 2 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "闭包套闭包？你这是在写俄罗斯套娃还是在考验读者的智商？",
                    "嵌套闭包比我的人际关系还复杂",
                    "这闭包嵌套得像洋葱一样，剥一层哭一次",
                    "闭包嵌套过深，建议拆分成独立函数",
                ]
            } else {
                vec![
                    "Closures within closures? Are you writing Russian nesting dolls or testing our IQ?",
                    "Nested closures are more complex than my relationships",
                    "These closures are nested like an onion - peel one layer, cry once",
                    "Closures too deeply nested - consider splitting into separate functions",
                ]
            };

            let (line, column) = get_position(closure);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "complex-closure".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        // Check for complex closure parameters
        if closure.inputs.len() > 5 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "这个闭包的参数比我的借口还多",
                    "闭包参数过多，你确定不是在写函数？",
                    "这么多参数的闭包，建议改成正经函数",
                ]
            } else {
                vec![
                    "This closure has more parameters than my excuses",
                    "Too many parameters for a closure - are you sure this isn't a function?",
                    "So many parameters - consider making this a proper function",
                ]
            };

            let (line, column) = get_position(closure);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "complex-closure".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }
    }
}

impl<'ast> Visit<'ast> for ClosureVisitor {
    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        self.closure_depth += 1;
        self.check_closure_complexity(closure);
        syn::visit::visit_expr_closure(self, closure);
        self.closure_depth -= 1;
    }
}

struct LifetimeVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lifetime_count: usize,
    lang: String,
}

impl LifetimeVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lifetime_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for LifetimeVisitor {
    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        // Skip 'static lifetime
        if lifetime.ident == "static" {
            return;
        }

        self.lifetime_count += 1;

        // Check for excessive lifetime usage, cap to 1 issue per file
        // Increased threshold: 20+ to avoid false positives in trait implementations
        // (each syn::Visit impl requires 'ast lifetime)
        if self.lifetime_count > 20 && self.issues.is_empty() {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "生命周期标注比我的生命还复杂",
                    "这么多生命周期，你是在写哲学论文吗？",
                    "生命周期滥用，建议重新设计数据结构",
                    "生命周期多到让人怀疑人生",
                ]
            } else {
                vec![
                    "Lifetime annotations are more complex than my life",
                    "So many lifetimes - are you writing a philosophy paper?",
                    "Lifetime abuse - consider redesigning your data structure",
                    "Too many lifetimes - making people question their existence",
                ]
            };

            let (line, column) = get_position(lifetime);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "lifetime-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        syn::visit::visit_lifetime(self, lifetime);
    }
}

struct TraitVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl TraitVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn check_trait_complexity(&mut self, trait_item: &ItemTrait) {
        // Check for traits with too many methods
        if trait_item.items.len() > 10 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "这个 trait 的方法比我的借口还多",
                    "trait 方法过多，违反了单一职责原则",
                    "这个 trait 比瑞士军刀还要全能",
                    "trait 臃肿，建议拆分成多个小 trait",
                ]
            } else {
                vec![
                    "This trait has more methods than my excuses",
                    "Too many trait methods - violates single responsibility principle",
                    "This trait is more versatile than a Swiss Army knife",
                    "Bloated trait - consider splitting into smaller traits",
                ]
            };

            let (line, column) = get_position(trait_item);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "trait-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        // Check for traits with too many generic parameters
        if trait_item.generics.params.len() > 3 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "泛型参数比我的密码还复杂",
                    "这么多泛型，你是在写数学公式吗？",
                    "泛型滥用，建议简化设计",
                ]
            } else {
                vec![
                    "Generic parameters are more complex than my password",
                    "So many generics - are you writing a math formula?",
                    "Generic abuse - simplify your design",
                ]
            };

            let (line, column) = get_position(trait_item);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "trait-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }
    }
}

impl<'ast> Visit<'ast> for TraitVisitor {
    fn visit_item_trait(&mut self, trait_item: &'ast ItemTrait) {
        self.check_trait_complexity(trait_item);
        syn::visit::visit_item_trait(self, trait_item);
    }
}

struct GenericVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl GenericVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn check_generic_abuse(&mut self, generics: &syn::Generics) {
        if generics.params.len() > 5 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "泛型参数比我的购物清单还长",
                    "这么多泛型，编译器都要哭了",
                    "泛型滥用，建议重新设计架构",
                    "泛型多到让人怀疑这还是 Rust 吗",
                ]
            } else {
                vec![
                    "Generic parameters are longer than my shopping list",
                    "So many generics - even the compiler is crying",
                    "Generic abuse - consider redesigning your architecture",
                    "So many generics - making people question if this is still Rust",
                ]
            };

            let (line, column) = get_position(generics);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "generic-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        // Check for single-letter generic names (except T, U, V)
        for param in &generics.params {
            if let GenericParam::Type(type_param) = param {
                let name = type_param.ident.to_string();
                if name.len() == 1 && !matches!(name.as_str(), "T" | "U" | "V" | "E" | "K") {
                    let messages = if self.lang == "zh-CN" {
                        vec![
                            format!("泛型参数 '{name}' 的命名创意约等于零"),
                            format!("泛型 '{name}' 的名字比我的耐心还短"),
                            format!("用 '{name}' 做泛型名？建议用更有意义的名字"),
                        ]
                    } else {
                        vec![
                            format!("Generic parameter '{name}' has approximately zero creativity"),
                            format!("Generic '{name}' is shorter than my patience"),
                            format!("Using '{name}' as a generic name? Consider something more meaningful"),
                        ]
                    };

                    let (line, column) = get_position(type_param);
                    self.issues.push(CodeIssue {
                        file_path: self.file_path.clone(),
                        line,
                        column,
                        rule_name: "generic-abuse".to_string(),
                        message: messages[self.issues.len() % messages.len()].clone(),
                        severity: Severity::Mild,
                    });
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for GenericVisitor {
    fn visit_item_fn(&mut self, func: &'ast syn::ItemFn) {
        self.check_generic_abuse(&func.sig.generics);
        syn::visit::visit_item_fn(self, func);
    }

    fn visit_item_struct(&mut self, struct_item: &'ast syn::ItemStruct) {
        self.check_generic_abuse(&struct_item.generics);
        syn::visit::visit_item_struct(self, struct_item);
    }

    fn visit_item_enum(&mut self, enum_item: &'ast syn::ItemEnum) {
        self.check_generic_abuse(&enum_item.generics);
        syn::visit::visit_item_enum(self, enum_item);
    }

    fn visit_item_impl(&mut self, impl_item: &'ast ItemImpl) {
        self.check_generic_abuse(&impl_item.generics);
        syn::visit::visit_item_impl(self, impl_item);
    }
}
