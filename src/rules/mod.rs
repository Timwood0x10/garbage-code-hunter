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

impl RuleEngine {
    pub fn new() -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();

        // Add various detection rules
        rules.push(Box::new(naming::TerribleNamingRule));
        rules.push(Box::new(naming::SingleLetterVariableRule));
        
        // Add garbage naming detection rules
        rules.push(Box::new(garbage_naming::MeaninglessNamingRule));
        rules.push(Box::new(garbage_naming::HungarianNotationRule));
        rules.push(Box::new(garbage_naming::AbbreviationAbuseRule));
        
        // Add student code detection rules
        rules.push(Box::new(student_code::PrintlnDebuggingRule));
        rules.push(Box::new(student_code::PanicAbuseRule));
        rules.push(Box::new(student_code::TodoCommentRule));
        
        // Add code smell detection rules
        rules.push(Box::new(code_smells::MagicNumberRule));
        rules.push(Box::new(code_smells::GodFunctionRule));
        rules.push(Box::new(code_smells::CommentedCodeRule));
        rules.push(Box::new(code_smells::DeadCodeRule));
        
        // Add Rust-specific pattern detection rules
        rules.push(Box::new(rust_patterns::StringAbuseRule));
        rules.push(Box::new(rust_patterns::VecAbuseRule));
        rules.push(Box::new(rust_patterns::IteratorAbuseRule));
        rules.push(Box::new(rust_patterns::MatchAbuseRule));
        rules.push(Box::new(complexity::DeepNestingRule));
        rules.push(Box::new(complexity::LongFunctionRule));
        rules.push(Box::new(duplication::CodeDuplicationRule));
        rules.push(Box::new(rust_specific::UnwrapAbuseRule));
        rules.push(Box::new(rust_specific::UnnecessaryCloneRule));

        // Add advanced Rust-specific rules
        rules.push(Box::new(advanced_rust::ComplexClosureRule));
        rules.push(Box::new(advanced_rust::LifetimeAbuseRule));
        rules.push(Box::new(advanced_rust::TraitComplexityRule));
        rules.push(Box::new(advanced_rust::GenericAbuseRule));

        // Add comprehensive Rust feature rules
        rules.push(Box::new(comprehensive_rust::ChannelAbuseRule));
        rules.push(Box::new(comprehensive_rust::AsyncAbuseRule));
        rules.push(Box::new(comprehensive_rust::DynTraitAbuseRule));
        rules.push(Box::new(comprehensive_rust::UnsafeAbuseRule));
        rules.push(Box::new(comprehensive_rust::FFIAbuseRule));
        rules.push(Box::new(comprehensive_rust::MacroAbuseRule));
        rules.push(Box::new(comprehensive_rust::ModuleComplexityRule));
        rules.push(Box::new(comprehensive_rust::PatternMatchingAbuseRule));
        rules.push(Box::new(comprehensive_rust::ReferenceAbuseRule));
        rules.push(Box::new(comprehensive_rust::BoxAbuseRule));
        rules.push(Box::new(comprehensive_rust::SliceAbuseRule));

        // Add file structure rules
        rules.push(Box::new(file_structure::FileStructureRule));
        rules.push(Box::new(file_structure::ImportChaosRule));
        rules.push(Box::new(file_structure::ModuleNestingRule));

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
