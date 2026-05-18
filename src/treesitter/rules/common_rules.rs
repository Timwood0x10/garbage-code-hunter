/// Register rules that apply to all languages with tree-sitter grammar support.
pub fn register_common_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    use super::complex_rules::{
        AbbreviationAbuseTsRule, DeepNestingRule, HungarianNotationTsRule, MagicNumberRule,
        PrintlnDebuggingRule, SingleLetterTsRule, TerribleNamingRule,
    };

    engine.add(Box::new(TerribleNamingRule));
    engine.add(Box::new(SingleLetterTsRule));
    engine.add(Box::new(HungarianNotationTsRule));
    engine.add(Box::new(AbbreviationAbuseTsRule));
    engine.add(Box::new(DeepNestingRule));
    engine.add(Box::new(PrintlnDebuggingRule));
    engine.add(Box::new(MagicNumberRule));
}
