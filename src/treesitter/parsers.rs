use crate::language::Language;
use tree_sitter_c;
use tree_sitter_cpp;
use tree_sitter_go;
use tree_sitter_java;
use tree_sitter_javascript;
use tree_sitter_python;
use tree_sitter_ruby;
use tree_sitter_rust;
use tree_sitter_swift;
use tree_sitter_typescript;
use tree_sitter_zig;

/// Get the tree-sitter grammar for a given language.
/// Returns None if the language has no compiled grammar available.
pub fn get_grammar(lang: Language) -> Option<tree_sitter::Language> {
    match lang {
        Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
        Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
        Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
        Language::Java => Some(tree_sitter_java::LANGUAGE.into()),
        Language::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
        Language::Swift => Some(tree_sitter_swift::LANGUAGE.into()),
        Language::Zig => Some(tree_sitter_zig::LANGUAGE.into()),
        Language::C => Some(tree_sitter_c::LANGUAGE.into()),
        Language::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
        _ => None,
    }
}
