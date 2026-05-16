use crate::treesitter::engine::ParsedFile;
use crate::treesitter::TreeSitterEngine;
use std::path::Path;
use std::sync::OnceLock;

/// Shared engine reused across all tests — saves ~85s by loading grammars once
fn shared_engine() -> &'static TreeSitterEngine {
    static ENGINE: OnceLock<TreeSitterEngine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let engine = TreeSitterEngine::new();
        // Pre-warm all grammars so test timings are consistent
        engine.ensure_parser(crate::language::Language::Rust);
        engine.ensure_parser(crate::language::Language::Go);
        engine.ensure_parser(crate::language::Language::Python);
        engine.ensure_parser(crate::language::Language::JavaScript);
        engine.ensure_parser(crate::language::Language::TypeScript);
        engine.ensure_parser(crate::language::Language::Java);
        engine.ensure_parser(crate::language::Language::Ruby);
        engine.ensure_parser(crate::language::Language::C);
        engine.ensure_parser(crate::language::Language::Cpp);
        engine
    })
}

fn parse_as(filename: &str, code: &str) -> ParsedFile {
    shared_engine()
        .parse_file(Path::new(filename), code)
        .expect("Should parse")
}

pub fn parse_rust(code: &str) -> ParsedFile {
    parse_as("main.rs", code)
}

pub fn parse_rust_as(filename: &str, code: &str) -> ParsedFile {
    parse_as(filename, code)
}

pub fn parse_python(code: &str) -> ParsedFile {
    parse_as("test.py", code)
}

pub fn parse_python_as(filename: &str, code: &str) -> ParsedFile {
    parse_as(filename, code)
}

pub fn parse_go(code: &str) -> ParsedFile {
    parse_as("test.go", code)
}

pub fn parse_java(code: &str) -> ParsedFile {
    parse_as("Test.java", code)
}

pub fn parse_ruby(code: &str) -> ParsedFile {
    parse_as("test.rb", code)
}

pub fn parse_ts(code: &str) -> ParsedFile {
    parse_as("test.ts", code)
}

pub fn parse_c(code: &str) -> ParsedFile {
    parse_as("test.c", code)
}

pub fn parse_cpp(code: &str) -> ParsedFile {
    parse_as("test.cpp", code)
}
