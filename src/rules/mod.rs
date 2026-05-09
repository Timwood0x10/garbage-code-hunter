use std::path::Path;
use syn::File;

use crate::analyzer::CodeIssue;
use crate::context::{FileContext, ProjectConfig};

pub mod advanced_rust;
pub mod code_smells;
pub mod complexity;
pub mod comprehensive_rust;
pub mod duplication;
pub mod file_structure;
pub mod garbage_naming;
pub mod naming;
pub mod rust_patterns;
pub mod rust_specific;
pub mod struct_patterns;
pub mod student_code;

pub trait Rule {
    fn name(&self) -> &'static str;

    /// Original check method (backward compatible)
    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        self.check_with_context(
            file_path,
            syntax_tree,
            content,
            lang,
            is_test_file,
            &FileContext::from_path(file_path),
            &ProjectConfig::default(),
        )
    }

    /// New method: check with context (recommended)
    #[allow(clippy::too_many_arguments)]
    fn check_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
        _context: &FileContext,
        _config: &ProjectConfig,
    ) -> Vec<CodeIssue> {
        self.check(file_path, syntax_tree, content, lang, is_test_file)
    }
}

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
    config: ProjectConfig,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngine {
    pub fn new() -> Self {
        Self::with_config(ProjectConfig::default())
    }

    pub fn with_config(config: ProjectConfig) -> Self {
        let rules: Vec<Box<dyn Rule>> = vec![
            // Add various detection rules
            Box::new(naming::TerribleNamingRule),
            Box::new(naming::SingleLetterVariableRule),
            // Add garbage naming detection rules
            Box::new(garbage_naming::MeaninglessNamingRule),
            Box::new(garbage_naming::HungarianNotationRule),
            Box::new(garbage_naming::AbbreviationAbuseRule),
            // Add student code detection rules
            Box::new(student_code::PrintlnDebuggingRule),
            Box::new(student_code::PanicAbuseRule),
            Box::new(student_code::TodoCommentRule),
            // Add code smell detection rules
            Box::new(code_smells::MagicNumberRule),
            Box::new(code_smells::GodFunctionRule),
            Box::new(code_smells::CommentedCodeRule),
            Box::new(code_smells::DeadCodeRule),
            // Add Rust-specific pattern detection rules
            Box::new(rust_patterns::StringAbuseRule),
            Box::new(rust_patterns::VecAbuseRule),
            Box::new(complexity::DeepNestingRule),
            Box::new(complexity::LongFunctionRule),
            Box::new(duplication::CodeDuplicationRule),
            Box::new(rust_specific::UnwrapAbuseRule),
            Box::new(rust_specific::UnnecessaryCloneRule),
            // Add advanced Rust-specific rules
            Box::new(advanced_rust::ComplexClosureRule),
            Box::new(advanced_rust::LifetimeAbuseRule),
            Box::new(advanced_rust::TraitComplexityRule),
            Box::new(advanced_rust::GenericAbuseRule),
            // Add comprehensive Rust feature rules
            Box::new(comprehensive_rust::ChannelAbuseRule),
            Box::new(comprehensive_rust::AsyncAbuseRule),
            Box::new(comprehensive_rust::DynTraitAbuseRule),
            Box::new(comprehensive_rust::UnsafeAbuseRule),
            Box::new(comprehensive_rust::FFIAbuseRule),
            Box::new(comprehensive_rust::MacroAbuseRule),
            Box::new(comprehensive_rust::ModuleComplexityRule),
            Box::new(comprehensive_rust::PatternMatchingAbuseRule),
            Box::new(struct_patterns::ReferenceAbuseRule),
            Box::new(struct_patterns::BoxAbuseRule),
            Box::new(struct_patterns::SliceAbuseRule),
            // Add file structure rules
            Box::new(file_structure::FileStructureRule),
            Box::new(file_structure::ImportChaosRule),
            Box::new(file_structure::ModuleNestingRule),
        ];

        Self { rules, config }
    }

    /// Use context-aware check method (recommended)
    pub fn check_file_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let context = FileContext::from_path(file_path);
        let mut issues = Vec::new();

        for rule in &self.rules {
            if context.should_skip_rule(rule.name()) {
                continue;
            }

            let rule_issues = rule.check_with_context(
                file_path,
                syntax_tree,
                content,
                lang,
                is_test_file,
                &context,
                &self.config,
            );

            issues.extend(rule_issues);
        }

        issues
    }

    /// Legacy check method (backward compatible)
    pub fn check_file(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        self.check_file_with_context(file_path, syntax_tree, content, lang, is_test_file)
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.name()).collect()
    }
}
