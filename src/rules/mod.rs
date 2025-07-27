use std::path::Path;
use syn::File;

use crate::analyzer::CodeIssue;

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
pub mod student_code;

pub trait Rule {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue>;
}

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngine {
    pub fn new() -> Self {
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
            Box::new(rust_patterns::IteratorAbuseRule),
            Box::new(rust_patterns::MatchAbuseRule),
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
            Box::new(comprehensive_rust::ReferenceAbuseRule),
            Box::new(comprehensive_rust::BoxAbuseRule),
            Box::new(comprehensive_rust::SliceAbuseRule),

            // Add file structure rules
            Box::new(file_structure::FileStructureRule),
            Box::new(file_structure::ImportChaosRule),
            Box::new(file_structure::ModuleNestingRule),
        ];

        Self { rules }
    }

    pub fn check_file(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            issues.extend(rule.check(file_path, syntax_tree, content, lang));
        }

        issues
    }
}
