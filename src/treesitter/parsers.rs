use crate::language::Language;

/// Get the tree-sitter grammar for a given language.
/// Returns None if the language has no compiled grammar available.
pub fn get_grammar(lang: Language) -> Option<tree_sitter::Language> {
    match lang {
        Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
        Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
        Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
        Language::Java => Some(tree_sitter_java::LANGUAGE.into()),
        Language::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
        Language::C => Some(tree_sitter_c::LANGUAGE.into()),
        Language::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
        _ => None,
    }
}
