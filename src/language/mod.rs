use std::path::Path;

/// Programming languages supported by the analyzer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    C,
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Ruby,
    Swift,
    Zig,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Language::Rust,
            "c" | "h" => Language::C,
            "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" | "C" | "CPP" => Language::Cpp,
            "py" => Language::Python,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Language::TypeScript,
            "go" => Language::Go,
            "java" => Language::Java,
            "rb" => Language::Ruby,
            "swift" => Language::Swift,
            "zig" => Language::Zig,
            _ => Language::Unknown,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(Language::Unknown)
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cxx", "cc", "hpp", "hh", "hxx", "C", "CPP"],
            Language::Python => &["py"],
            Language::JavaScript => &["js", "mjs", "cjs"],
            Language::TypeScript => &["ts", "tsx", "mts", "cts"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::Ruby => &["rb"],
            Language::Swift => &["swift"],
            Language::Zig => &["zig"],
            Language::Unknown => &[],
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Zig => "Zig",
            Language::Unknown => "Unknown",
        }
    }

    pub fn line_comment(&self) -> &'static str {
        match self {
            Language::Rust
            | Language::C
            | Language::Cpp
            | Language::Go
            | Language::Java
            | Language::JavaScript
            | Language::TypeScript
            | Language::Ruby
            | Language::Swift
            | Language::Zig => "//",
            Language::Python => "#",
            Language::Unknown => "//",
        }
    }

    pub fn has_tree_sitter_grammar(&self) -> bool {
        matches!(
            self,
            Language::Rust
                | Language::C
                | Language::Cpp
                | Language::Python
                | Language::JavaScript
                | Language::TypeScript
                | Language::Go
                | Language::Java
                | Language::Ruby
                | Language::Swift
                | Language::Zig
        )
    }
}

/// All languages with a compiled tree-sitter grammar available.
pub const LANGUAGES_WITH_GRAMMAR: &[Language] = &[
    Language::Rust,
    Language::Python,
    Language::JavaScript,
    Language::TypeScript,
    Language::Go,
    Language::Java,
    Language::Ruby,
    Language::Swift,
    Language::Zig,
    Language::C,
    Language::Cpp,
];

/// All supported source file extensions for discovery.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs", "c", "h", "cpp", "cxx", "cc", "hpp", "hh", "hxx", "C", "CPP", "py", "js", "mjs", "cjs",
    "ts", "tsx", "mts", "cts", "go", "java", "rb", "swift", "zig",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_extension_rust() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
    }

    #[test]
    fn test_from_extension_python() {
        assert_eq!(Language::from_extension("py"), Language::Python);
    }

    #[test]
    fn test_from_extension_unknown() {
        assert_eq!(Language::from_extension("xyz"), Language::Unknown);
    }

    #[test]
    fn test_extensions_rust() {
        assert!(Language::Rust.extensions().contains(&"rs"));
    }

    #[test]
    fn test_display_names() {
        assert_eq!(Language::Rust.display_name(), "Rust");
        assert_eq!(Language::Python.display_name(), "Python");
        assert_eq!(Language::Unknown.display_name(), "Unknown");
    }

    #[test]
    fn test_line_comments() {
        assert_eq!(Language::Rust.line_comment(), "//");
        assert_eq!(Language::Python.line_comment(), "#");
    }

    #[test]
    fn test_tree_sitter_grammar_available() {
        assert!(Language::Rust.has_tree_sitter_grammar());
        assert!(Language::Python.has_tree_sitter_grammar());
        assert!(!Language::Unknown.has_tree_sitter_grammar());
    }
}
