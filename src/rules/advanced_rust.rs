use std::path::Path;
use syn::{visit::Visit, ExprClosure, File, GenericParam, ItemImpl, ItemTrait, Lifetime};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ClosureVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = LifetimeVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = TraitVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = GenericVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct ClosureVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    closure_depth: usize,
}

impl ClosureVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            closure_depth: 0,
        }
    }

    fn check_closure_complexity(&mut self, closure: &ExprClosure) {
        // Check for nested closures
        if self.closure_depth > 2 {
            let messages = vec![
                "闭包套闭包？你这是在写俄罗斯套娃还是在考验读者的智商？",
                "嵌套闭包比我的人际关系还复杂",
                "这闭包嵌套得像洋葱一样，剥一层哭一次",
                "闭包嵌套过深，建议拆分成独立函数",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // Simplified handling
                column: 1,
                rule_name: "complex-closure".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        // Check for complex closure parameters
        if closure.inputs.len() > 5 {
            let messages = vec![
                "这个闭包的参数比我的借口还多",
                "闭包参数过多，你确定不是在写函数？",
                "这么多参数的闭包，建议改成正经函数",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "complex-closure".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
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
}

impl LifetimeVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lifetime_count: 0,
        }
    }
}

impl<'ast> Visit<'ast> for LifetimeVisitor {
    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        self.lifetime_count += 1;

        // Check for excessive lifetime usage
        if self.lifetime_count > 5 {
            let messages = vec![
                "生命周期标注比我的生命还复杂",
                "这么多生命周期，你是在写哲学论文吗？",
                "生命周期滥用，建议重新设计数据结构",
                "生命周期多到让人怀疑人生",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "lifetime-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        syn::visit::visit_lifetime(self, lifetime);
    }
}

struct TraitVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
}

impl TraitVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
        }
    }

    fn check_trait_complexity(&mut self, trait_item: &ItemTrait) {
        // Check for traits with too many methods
        if trait_item.items.len() > 10 {
            let messages = vec![
                "这个 trait 的方法比我的借口还多",
                "trait 方法过多，违反了单一职责原则",
                "这个 trait 比瑞士军刀还要全能",
                "trait 臃肿，建议拆分成多个小 trait",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "trait-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        // Check for traits with too many generic parameters
        if trait_item.generics.params.len() > 3 {
            let messages = vec![
                "泛型参数比我的密码还复杂",
                "这么多泛型，你是在写数学公式吗？",
                "泛型滥用，建议简化设计",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "trait-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
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
}

impl GenericVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
        }
    }

    fn check_generic_abuse(&mut self, generics: &syn::Generics) {
        if generics.params.len() > 5 {
            let messages = vec![
                "泛型参数比我的购物清单还长",
                "这么多泛型，编译器都要哭了",
                "泛型滥用，建议重新设计架构",
                "泛型多到让人怀疑这还是 Rust 吗",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "generic-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        // Check for single-letter generic names (except T, U, V)
        for param in &generics.params {
            if let GenericParam::Type(type_param) = param {
                let name = type_param.ident.to_string();
                if name.len() == 1 && !matches!(name.as_str(), "T" | "U" | "V" | "E" | "K") {
                    let messages = vec![
                        format!("泛型参数 '{}' 的命名创意约等于零", name),
                        format!("泛型 '{}' 的名字比我的耐心还短", name),
                        format!("用 '{}' 做泛型名？建议用更有意义的名字", name),
                    ];

                    self.issues.push(CodeIssue {
                        file_path: self.file_path.clone(),
                        line: 1,
                        column: 1,
                        rule_name: "generic-abuse".to_string(),
                        message: messages[self.issues.len() % messages.len()].clone(),
                        severity: Severity::Mild,
                        roast_level: RoastLevel::Gentle,
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
