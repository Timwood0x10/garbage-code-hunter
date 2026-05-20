use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use garbage_code_hunter::CodeAnalyzer;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ================================================================
// Fixture generators — self-contained, no external downloads
// ================================================================

fn create_large_garbage_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let file_path = temp_dir.path().join("large_garbage.rs");

    let mut content = String::new();
    content.push_str("// Large file with lots of garbage code\n");
    for i in 0..100 {
        content.push_str(&format!(
            r#"
fn function_{i}() {{
    let data = "hello";
    let temp = 42;
    let info = vec![1, 2, 3];
    let obj = String::new();
    let a = 10;
    let b = 20;
    let c = a + b;

    let result = Some(42);
    let value = result.unwrap();
    let another = Some("test").unwrap();

    let s1 = String::from("hello");
    let s2 = s1.clone();

    if true {{
        if true {{
            if true {{
                if true {{
                    if true {{
                        if true {{
                            println!("Deep nesting in function {i}");
                        }}
                    }}
                }}
            }}
        }}
    }}

    println!("{{}} {{}} {{}} {{}}", value, another.len(), s4.len(), c);
}}
"#,
        ));
    }

    fs::write(&file_path, content).expect("write");
    (temp_dir, file_path)
}

fn create_clean_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let file_path = temp_dir.path().join("clean_code.rs");

    let content = r#"
use std::collections::HashMap;

/// Calculate user statistics based on activity data
pub struct UserStatistics {
    user_id: u64,
    activity_count: usize,
    last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserStatistics {
    pub fn new(user_id: u64) -> Self {
        Self { user_id, activity_count: 0, last_activity: None }
    }

    pub fn record_activity(&mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), String> {
        if let Some(last) = self.last_activity {
            if timestamp < last {
                return Err("Cannot record activity in the past".to_string());
            }
        }
        self.activity_count += 1;
        self.last_activity = Some(timestamp);
        Ok(())
    }

    pub fn calculate_daily_rate(&self) -> Option<f64> {
        let last_activity = self.last_activity?;
        let days_since_start = chrono::Utc::now()
            .signed_duration_since(last_activity)
            .num_days() as f64;
        if days_since_start > 0.0 {
            Some(self.activity_count as f64 / days_since_start)
        } else {
            None
        }
    }
}

pub struct StatisticsManager {
    user_stats: HashMap<u64, UserStatistics>,
}

impl StatisticsManager {
    pub fn new() -> Self { Self { user_stats: HashMap::new() } }

    pub fn get_or_create_user_stats(&mut self, user_id: u64) -> &mut UserStatistics {
        self.user_stats.entry(user_id).or_insert_with(|| UserStatistics::new(user_id))
    }

    pub fn record_user_activity(&mut self, user_id: u64, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), String> {
        self.get_or_create_user_stats(user_id).record_activity(timestamp)
    }

    pub fn get_top_users(&self, limit: usize) -> Vec<(u64, usize)> {
        let mut users: Vec<_> = self.user_stats.iter().map(|(id, stats)| (*id, stats.activity_count)).collect();
        users.sort_by(|a, b| b.1.cmp(&a.1));
        users.into_iter().take(limit).collect()
    }
}
"#;

    fs::write(&file_path, content).expect("write");
    (temp_dir, file_path)
}

fn create_python_garbage_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let file_path = temp_dir.path().join("bad_code.py");
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            r#"
def function_{i}(a, b, c, d, e):
    data = "hello"
    temp = 42
    info = [1, 2, 3]
    obj = dict()
    if a > 0:
        if b > 0:
            if c > 0:
                if d > 0:
                    if e > 0:
                        print("deep in {i}")
    result = a + b + c + d + e
    print("result is", result)
    return result
"#,
        ));
    }
    fs::write(&file_path, content).expect("write");
    (temp_dir, file_path)
}

fn create_js_garbage_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let file_path = temp_dir.path().join("bad_code.js");
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            r#"
function function_{i}(a, b, c, d, e) {{
    let data = "hello";
    let temp = 42;
    let info = [1, 2, 3];
    let obj = {{}};
    if (a > 0) {{
        if (b > 0) {{
            if (c > 0) {{
                if (d > 0) {{
                    if (e > 0) {{
                        console.log("deep in {i}");
                    }}
                }}
            }}
        }}
    }}
    let result = a + b + c + d + e;
    console.log("result", result);
    return result;
}}
"#,
        ));
    }
    fs::write(&file_path, content).expect("write");
    (temp_dir, file_path)
}

fn create_go_garbage_file() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let file_path = temp_dir.path().join("bad_code.go");
    let mut content = String::new();
    content.push_str("package main\n\n");
    for i in 0..50 {
        content.push_str(&format!(
            r#"
func function_{i}(a int, b int, c int, d int, e int) int {{
    data := "hello"
    temp := 42
    info := []int{{1, 2, 3}}
    obj := make(map[string]int)
    if a > 0 {{
        if b > 0 {{
            if c > 0 {{
                if d > 0 {{
                    if e > 0 {{
                        fmt.Println("deep in {i}")
                    }}
                }}
            }}
        }}
    }}
    result := a + b + c + d + e
    fmt.Println("result", result)
    return result
}}
"#,
        ));
    }
    fs::write(&file_path, content).expect("write");
    (temp_dir, file_path)
}

fn create_multi_file_project(file_count: usize, garbage: bool) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    for i in 0..file_count {
        let file_path = src_dir.join(format!("module_{i}.rs"));
        let content = if garbage {
            format!(
                r#"
// Module {i}
use std::collections::HashMap;

pub fn process_{i}(data: &str, temp: i32) -> Result<String, String> {{
    let info = format!("processing {{}}: {{}}", data, temp);
    let thing = HashMap::new();
    let mut manager = Vec::new();
    manager.push(info.clone());
    if temp > 0 {{
        if temp > 10 {{
            if temp > 100 {{
                if temp > 1000 {{
                    return Ok(manager.join(","));
                }}
            }}
        }}
    }}
    Err("failed".to_string())
}}

fn helper_{i}_a() {{ let _ = Some(42).unwrap(); }}
fn helper_{i}_b() {{ let _ = foo().unwrap().bar().unwrap(); }}
fn helper_{i}_c() {{ let _ = (0..100).collect::<Vec<_>>(); }}
"#,
            )
        } else {
            format!(
                r#"
/// Module {i} — provides processing utilities
use std::collections::HashMap;

/// Process input data with the given configuration.
pub fn process_{i}(input: &str, config: i32) -> Result<String, String> {{
    let mut buffer = Vec::new();
    buffer.push(format!("processing {{}}: {{}}", input, config));
    if config > 0 {{
        Ok(buffer.join(","))
    }} else {{
        Err("invalid config".to_string())
    }}
}}

/// Validate that the given value is within acceptable range.
fn validate_{i}(value: i32) -> bool {{
    value > 0 && value < 10000
}}
"#,
            )
        };
        fs::write(&file_path, content).expect("write module file");
    }

    (temp_dir, src_dir)
}

fn create_cross_file_project() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("tempdir");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    // Create multiple files with near-identical code to stress cross-file dup detection
    let common_body = r#"
pub fn calculate(a: i32, b: i32) -> i32 {
    let result = a + b;
    if result > 100 {
        if result > 200 {
            if result > 300 {
                println!("Large result: {}", result);
            }
        }
    }
    // Unnecessary clone chain
    let s1 = String::from("data");
    let s2 = s1.clone();
    let s3 = s2.clone();
    let s4 = s3.clone();
    let s5 = s4.clone();
    let data = Some(result);
    let value = data.unwrap();
    println!("Final: {} {} {} {} {}", s5, value, a, b, result);
    value
}
"#;

    for i in 0..5 {
        let file_path = src_dir.join(format!("processor_{i}.rs"));
        let content = format!(
            "pub mod processor_{i} {{\n{}\n}}",
            common_body.replace("println!", "log::info!")
        );
        fs::write(&file_path, content).expect("write");
    }

    (temp_dir, src_dir)
}

// ================================================================
// Existing benchmarks (improved)
// ================================================================

fn bench_analyzer_creation(c: &mut Criterion) {
    c.bench_function("create_analyzer/default", |b| {
        b.iter(|| {
            let analyzer = CodeAnalyzer::new(black_box(&[]), "en-US");
            black_box(analyzer);
        })
    });
}

fn bench_analyzer_with_exclusions(c: &mut Criterion) {
    let exclusions = vec![
        "target/*".to_string(),
        "test_*".to_string(),
        "tmp_*".to_string(),
        "*.tmp".to_string(),
        "build/*".to_string(),
    ];

    c.bench_function("create_analyzer/with_exclusions", |b| {
        b.iter(|| {
            let analyzer = CodeAnalyzer::new(black_box(&exclusions), "en-US");
            black_box(analyzer);
        })
    });
}

fn bench_analyze_clean_file(c: &mut Criterion) {
    let (_temp_dir, file_path) = create_clean_file();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    c.bench_function("analyze_file/clean_rust", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_file(black_box(&file_path));
            black_box(issues);
        })
    });
}

// ================================================================
// New: Micro-benchmarks — parsing only
// ================================================================

fn bench_parse_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_file");

    // Rust parsing
    let (td1, rust_path) = create_large_garbage_file();
    group.bench_with_input(
        BenchmarkId::new("rust_large", "100_functions"),
        &rust_path,
        |b, path| {
            let analyzer = CodeAnalyzer::new(&[], "en-US");
            b.iter(|| {
                let _ = analyzer.analyze_file(black_box(path));
            });
        },
    );
    drop(td1);

    // Python parsing
    let (td2, py_path) = create_python_garbage_file();
    group.bench_with_input(
        BenchmarkId::new("python", "50_functions"),
        &py_path,
        |b, path| {
            b.iter(|| {
                let analyzer = CodeAnalyzer::new(&[], "en-US");
                let _ = analyzer.analyze_file(black_box(path));
            });
        },
    );
    drop(td2);

    // JavaScript parsing
    let (td3, js_path) = create_js_garbage_file();
    group.bench_with_input(
        BenchmarkId::new("javascript", "50_functions"),
        &js_path,
        |b, path| {
            b.iter(|| {
                let analyzer = CodeAnalyzer::new(&[], "en-US");
                let _ = analyzer.analyze_file(black_box(path));
            });
        },
    );
    drop(td3);

    // Go parsing
    let (td4, go_path) = create_go_garbage_file();
    group.bench_with_input(
        BenchmarkId::new("go", "50_functions"),
        &go_path,
        |b, path| {
            b.iter(|| {
                let analyzer = CodeAnalyzer::new(&[], "en-US");
                let _ = analyzer.analyze_file(black_box(path));
            });
        },
    );
    drop(td4);

    // Clean Rust file
    let (td5, clean_path) = create_clean_file();
    group.bench_with_input(
        BenchmarkId::new("rust_clean", "well_written"),
        &clean_path,
        |b, path| {
            b.iter(|| {
                let analyzer = CodeAnalyzer::new(&[], "en-US");
                let _ = analyzer.analyze_file(black_box(path));
            });
        },
    );
    drop(td5);

    group.finish();
}

// ================================================================
// New: Multi-language mixed project benchmark
// ================================================================

fn bench_multi_language_mixed_project(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("tempdir");
    let (td, large_rs) = create_large_garbage_file();
    let (td_py, py_file) = create_python_garbage_file();
    let (td_js, js_file) = create_js_garbage_file();
    let (td_go, go_file) = create_go_garbage_file();

    // Copy all files into temp_dir
    for (src_name, src_path) in [
        ("large.rs", &large_rs),
        ("bad.py", &py_file),
        ("bad.js", &js_file),
        ("bad.go", &go_file),
    ] {
        let dst = temp_dir.path().join(src_name);
        fs::copy(src_path, &dst).expect("copy");
    }
    drop(td);
    drop(td_py);
    drop(td_js);
    drop(td_go);

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    c.bench_function("analyze_path/mixed_4_languages", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_path(black_box(temp_dir.path()));
            black_box(issues);
        })
    });
}

// ================================================================
// New: Pipeline comparison benchmarks
// ================================================================

fn bench_pipeline_comparison(c: &mut Criterion) {
    let (_temp_dir, file_path) = create_large_garbage_file();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    // Smoke test: verify analysis finds issues
    let smoke_issues = analyzer.analyze_file(&file_path);
    let _ = black_box(!smoke_issues.is_empty());

    c.bench_function("analyze_file/single_large_file", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_file(black_box(&file_path));
            black_box(issues);
        })
    });

    c.bench_function("analyze_to_findings/single_large_file", |b| {
        b.iter(|| {
            let findings = analyzer.analyze_to_findings(black_box(&file_path));
            black_box(findings);
        })
    });

    // analyze_path on a directory
    let (_td2, dir_path) = create_multi_file_project(10, true);
    let analyzer2 = CodeAnalyzer::new(&[], "en-US");

    // Smoke test: verify dir analysis finds issues
    let smoke_path = dir_path.parent().unwrap();
    let smoke_dir_issues = analyzer2.analyze_path(smoke_path);
    let _ = black_box(!smoke_dir_issues.is_empty());

    c.bench_function("analyze_path/dir_10_garbage_files", |b| {
        b.iter(|| {
            let issues = analyzer2.analyze_path(black_box(smoke_path));
            black_box(issues);
        })
    });
}

fn bench_analyze_full(c: &mut Criterion) {
    let (_temp_dir, file_path) = create_large_garbage_file();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    // Smoke test: verify analyze_full finds issues
    let smoke = analyzer.analyze_full(&file_path);
    let _ = black_box(!smoke.findings.is_empty());

    c.bench_function("analyze_full/single_large_file", |b| {
        b.iter(|| {
            let result = analyzer.analyze_full(black_box(&file_path));
            black_box(result);
        })
    });
}

// ================================================================
// New: Scalability — parameterized file count
// ================================================================

fn bench_scalability_by_file_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability/file_count");

    for &file_count in &[1, 5, 20, 50] {
        let (temp_dir, _) = create_multi_file_project(file_count, true);
        let analyzer = CodeAnalyzer::new(&[], "en-US");
        // The path is the parent of src/
        let path = temp_dir.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &path,
            |b, proj_path| {
                b.iter(|| {
                    let issues = analyzer.analyze_path(black_box(proj_path));
                    black_box(issues);
                })
            },
        );
    }

    group.finish();
}

fn bench_scalability_garbage_vs_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability/garbage_vs_clean");

    for &(file_count, garbage, label) in &[
        (10, true, "10_garbage"),
        (10, false, "10_clean"),
        (50, true, "50_garbage"),
        (50, false, "50_clean"),
    ] {
        let (temp_dir, _) = create_multi_file_project(file_count, garbage);
        let analyzer = CodeAnalyzer::new(&[], "en-US");
        let path = temp_dir.path().to_path_buf();

        group.bench_with_input(
            BenchmarkId::new(label, file_count),
            &path,
            |b, proj_path| {
                b.iter(|| {
                    let issues = analyzer.analyze_path(black_box(proj_path));
                    black_box(issues);
                })
            },
        );
    }

    group.finish();
}

// ================================================================
// New: Cross-file duplication benchmark
// ================================================================

fn bench_cross_file_dup_detection(c: &mut Criterion) {
    let (_temp_dir, project_dir) = create_cross_file_project();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    c.bench_function("analyze_path/cross_file_5_similar", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_path(black_box(&project_dir));
            black_box(issues);
        })
    });
}

// ================================================================
// New: Real-world example file benchmarks
// ================================================================

fn bench_example_files(c: &mut Criterion) {
    // Try to load real example files from the project
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let func_example = project_root.join("example/func.rs");
    let ultimate_example = project_root.join("example/ultimate_garbage_code_example.rs");

    if func_example.exists() {
        c.bench_function("analyze_file/example_func_rs", |b| {
            let analyzer = CodeAnalyzer::new(&[], "en-US");
            b.iter(|| {
                let issues = analyzer.analyze_file(black_box(&func_example));
                black_box(issues);
            })
        });
    }

    if ultimate_example.exists() {
        c.bench_function("analyze_file/example_ultimate_garbage", |b| {
            let analyzer = CodeAnalyzer::new(&[], "en-US");
            b.iter(|| {
                let issues = analyzer.analyze_file(black_box(&ultimate_example));
                black_box(issues);
            })
        });
    }

    // Analyze the entire example/ directory as a project
    let example_dir = project_root.join("example");
    if example_dir.exists() {
        c.bench_function("analyze_path/example_directory", |b| {
            let analyzer = CodeAnalyzer::new(&[], "en-US");
            b.iter(|| {
                let issues = analyzer.analyze_path(black_box(&example_dir));
                black_box(issues);
            })
        });
    }
}

// ================================================================
// New: Clean project vs garbage project benchmark
// ================================================================

fn bench_clean_vs_garbage_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("project/clean_vs_garbage");

    // Garbage project
    let (td_g, _) = create_multi_file_project(20, true);
    let analyzer_g = CodeAnalyzer::new(&[], "en-US");
    let path_g = td_g.path().to_path_buf();

    group.bench_with_input(BenchmarkId::new("garbage", 20), &path_g, |b, path| {
        b.iter(|| {
            let issues = analyzer_g.analyze_path(black_box(path));
            black_box(issues);
        })
    });

    // Clean project
    let (td_c, _) = create_multi_file_project(20, false);
    let analyzer_c = CodeAnalyzer::new(&[], "en-US");
    let path_c = td_c.path().to_path_buf();

    group.bench_with_input(BenchmarkId::new("clean", 20), &path_c, |b, path| {
        b.iter(|| {
            let issues = analyzer_c.analyze_path(black_box(path));
            black_box(issues);
        })
    });

    drop(td_g);
    drop(td_c);

    group.finish();
}

// ================================================================
// Criterion setup
// ================================================================

criterion_group!(
    benches,
    // Creation
    bench_analyzer_creation,
    bench_analyzer_with_exclusions,
    // Single-file analysis
    bench_analyze_clean_file,
    // Real-world examples
    bench_example_files,
    // Parsing / multi-language
    bench_parse_files,
    bench_multi_language_mixed_project,
    // Pipeline comparison
    bench_pipeline_comparison,
    bench_analyze_full,
    // Scalability
    bench_scalability_by_file_count,
    bench_scalability_garbage_vs_clean,
    bench_clean_vs_garbage_project,
    // Cross-file detection
    bench_cross_file_dup_detection,
);
criterion_main!(benches);
