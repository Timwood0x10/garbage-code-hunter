use crate::treesitter::engine::ParsedFile;
use crate::treesitter::TreeSitterEngine;
use std::path::Path;

fn parse_as(filename: &str, code: &str) -> ParsedFile {
    let engine = TreeSitterEngine::new();
    engine
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
