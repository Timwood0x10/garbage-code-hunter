use std::path::Path;

/// File context type - used to adjust rule sensitivity
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileContext {
    /// Business code (default) - normal detection intensity
    #[default]
    Business,
    /// Example/demo code - 70% sensitivity reduction
    Example,
    /// Test code - 80% sensitivity reduction
    Test,
    /// Performance benchmark code - 60% sensitivity reduction
    Benchmark,
    /// Documentation code - 90% sensitivity reduction
    Documentation,
    /// Config files (non-Rust) - skip most rules
    Config,
}

impl FileContext {
    /// Infer context type from file path
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();

        if Self::is_test_file(&path_str) {
            FileContext::Test
        } else if Self::is_example_file(&path_str) {
            FileContext::Example
        } else if Self::is_benchmark_file(&path_str) {
            FileContext::Benchmark
        } else if Self::is_documentation_file(&path_str) {
            FileContext::Documentation
        } else {
            FileContext::Business
        }
    }

    /// Returns the rule weight multiplier for this context (0.0 = skip completely, 1.0 = normal)
    pub fn rule_weight_multiplier(&self) -> f64 {
        match self {
            FileContext::Business => 1.0,
            FileContext::Example => 0.3,
            FileContext::Test => 0.2,
            FileContext::Benchmark => 0.4,
            FileContext::Documentation => 0.1,
            FileContext::Config => 0.0,
        }
    }

    /// Determine whether a specific rule should be skipped
    pub fn should_skip_rule(&self, rule_name: &str) -> bool {
        let multiplier = self.rule_weight_multiplier();

        if multiplier == 0.0 {
            return true;
        }

        match self {
            FileContext::Test => matches!(
                rule_name,
                "unwrap-abuse"
                    | "panic-abuse"
                    | "todo-comment"
                    | "terrible-naming"
                    | "single-letter-variable"
            ),
            FileContext::Example => matches!(
                rule_name,
                "terrible-naming"
                    | "meaningless-naming"
                    | "hungarian-notation"
                    | "abbreviation-abuse"
            ),
            _ => false,
        }
    }

    fn is_test_file(path_str: &str) -> bool {
        path_str.contains("/tests/")
            || path_str.ends_with("_test.rs")
            || path_str.contains("test_")
            || path_str.contains(".test.")
    }

    fn is_example_file(path_str: &str) -> bool {
        // Check for standard example/demo directories first
        if path_str.contains("/examples/")
            || path_str.contains("/demo/")
            || path_str.contains("/sample/")
        {
            return true;
        }

        // Check for example-like file names (but not in src/ main code)
        let file_name = path_str.rsplit('/').next().unwrap_or(path_str);

        // Message/example files (but exclude src/)
        if path_str.contains("/messages/") {
            return !path_str.contains("/src/");
        }

        // Only match if filename explicitly contains these patterns
        (file_name.contains("example")
            || file_name.contains("demo")
            || file_name.contains("sample"))
            && !path_str.contains("/src/") // Exclude main source code
    }

    fn is_benchmark_file(path_str: &str) -> bool {
        path_str.contains("/benches/")
            || path_str.contains("bench")
            || path_str.ends_with("_bench.rs")
    }

    fn is_documentation_file(path_str: &str) -> bool {
        path_str.contains("/docs/") || path_str.starts_with("doc/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_context() {
        let ctx = FileContext::from_path(Path::new("src/lib.rs"));
        assert_eq!(ctx, FileContext::Business);
        assert!((ctx.rule_weight_multiplier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_main_file_is_business() {
        let ctx = FileContext::from_path(Path::new("src/main.rs"));
        assert_eq!(ctx, FileContext::Business);
    }

    #[test]
    fn test_example_context() {
        let cases = vec![
            ("examples/demo.rs",),
            ("src/bin/advanced_demo.rs",),
            ("src/messages/english.rs",),
            ("src/messages/chinese.rs",),
        ];

        for (path,) in cases {
            let ctx = FileContext::from_path(Path::new(path));
            assert_eq!(ctx, FileContext::Example, "Failed for {}", path);
        }
    }

    #[test]
    fn test_test_context() {
        let cases = vec![
            ("tests/integration_test.rs",),
            ("src/my_module_test.rs",),
            ("src/test_helpers.rs",),
        ];

        for (path,) in cases {
            let ctx = FileContext::from_path(Path::new(path));
            assert_eq!(ctx, FileContext::Test, "Failed for {}", path);
        }
    }

    #[test]
    fn test_benchmark_context() {
        let ctx = FileContext::from_path(Path::new("benches/my_bench.rs"));
        assert_eq!(ctx, FileContext::Benchmark);
    }

    #[test]
    fn test_should_skip_rules_in_test() {
        let test_ctx = FileContext::Test;

        assert!(test_ctx.should_skip_rule("panic-abuse"));
        assert!(test_ctx.should_skip_rule("unwrap-abuse"));
        assert!(!test_ctx.should_skip_rule("magic-number"));
    }

    #[test]
    fn test_should_skip_rules_in_example() {
        let example_ctx = FileContext::Example;

        assert!(example_ctx.should_skip_rule("terrible-naming"));
        assert!(example_ctx.should_skip_rule("meaningless-naming"));
        assert!(!example_ctx.should_skip_rule("panic-abuse"));
    }

    #[test]
    fn test_business_does_not_skip() {
        let business_ctx = FileContext::Business;

        assert!(!business_ctx.should_skip_rule("panic-abuse"));
        assert!(!business_ctx.should_skip_rule("terrible-naming"));
        assert!(!business_ctx.should_skip_rule("magic-number"));
    }

    #[test]
    fn test_weight_multipliers() {
        assert_eq!(FileContext::Business.rule_weight_multiplier(), 1.0);
        assert_eq!(FileContext::Example.rule_weight_multiplier(), 0.3);
        assert_eq!(FileContext::Test.rule_weight_multiplier(), 0.2);
        assert_eq!(FileContext::Benchmark.rule_weight_multiplier(), 0.4);
        assert_eq!(FileContext::Documentation.rule_weight_multiplier(), 0.1);
        assert_eq!(FileContext::Config.rule_weight_multiplier(), 0.0);
    }
}
