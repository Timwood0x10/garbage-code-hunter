use crate::treesitter::rule::TreeSitterRuleEngine;

use super::complex_rules::MagicNumberRule;

/// Register rules that apply to all languages with tree-sitter grammar support.
pub fn register_common_rules(engine: &mut TreeSitterRuleEngine) {
    // Magic number — common to all languages
    engine.add(Box::new(MagicNumberRule));
}
