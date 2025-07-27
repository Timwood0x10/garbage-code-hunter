use criterion::{criterion_group, criterion_main, Criterion};
use garbage_code_hunter::CodeAnalyzer;
use std::fs;
use std::hint::black_box;
use tempfile::TempDir;

fn create_large_garbage_file() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("large_garbage.rs");

    let mut content = String::new();
    content.push_str("// Large file with lots of garbage code\n");

    // Generate many functions with terrible naming and other issues
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
    let s3 = s2.clone();
    let s4 = s3.clone();
    
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
"#
        ));
    }

    fs::write(&file_path, content).expect("Failed to write large test file");
    (temp_dir, file_path)
}

fn create_clean_file() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
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
    /// Create new user statistics
    pub fn new(user_id: u64) -> Self {
        Self {
            user_id,
            activity_count: 0,
            last_activity: None,
        }
    }
    
    /// Update activity count and timestamp
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
    
    /// Get activity rate per day
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

/// Manage multiple user statistics
pub struct StatisticsManager {
    user_stats: HashMap<u64, UserStatistics>,
}

impl StatisticsManager {
    pub fn new() -> Self {
        Self {
            user_stats: HashMap::new(),
        }
    }
    
    pub fn get_or_create_user_stats(&mut self, user_id: u64) -> &mut UserStatistics {
        self.user_stats
            .entry(user_id)
            .or_insert_with(|| UserStatistics::new(user_id))
    }
    
    pub fn record_user_activity(
        &mut self, 
        user_id: u64, 
        timestamp: chrono::DateTime<chrono::Utc>
    ) -> Result<(), String> {
        let user_stats = self.get_or_create_user_stats(user_id);
        user_stats.record_activity(timestamp)
    }
    
    pub fn get_top_users(&self, limit: usize) -> Vec<(u64, usize)> {
        let mut users: Vec<_> = self.user_stats
            .iter()
            .map(|(id, stats)| (*id, stats.activity_count))
            .collect();
            
        users.sort_by(|a, b| b.1.cmp(&a.1));
        users.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_statistics_creation() {
        let stats = UserStatistics::new(123);
        assert_eq!(stats.user_id, 123);
        assert_eq!(stats.activity_count, 0);
        assert!(stats.last_activity.is_none());
    }
    
    #[test]
    fn test_activity_recording() {
        let mut stats = UserStatistics::new(123);
        let now = chrono::Utc::now();
        
        assert!(stats.record_activity(now).is_ok());
        assert_eq!(stats.activity_count, 1);
        assert_eq!(stats.last_activity, Some(now));
    }
}
"#;

    fs::write(&file_path, content).expect("Failed to write clean test file");
    (temp_dir, file_path)
}

fn bench_analyze_garbage_file(c: &mut Criterion) {
    let (_temp_dir, file_path) = create_large_garbage_file();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    c.bench_function("analyze_large_garbage_file", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_file(black_box(&file_path));
            black_box(issues);
        })
    });
}

fn bench_analyze_clean_file(c: &mut Criterion) {
    let (_temp_dir, file_path) = create_clean_file();
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    c.bench_function("analyze_clean_file", |b| {
        b.iter(|| {
            let issues = analyzer.analyze_file(black_box(&file_path));
            black_box(issues);
        })
    });
}

fn bench_analyzer_creation(c: &mut Criterion) {
    c.bench_function("create_analyzer", |b| {
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

    c.bench_function("create_analyzer_with_exclusions", |b| {
        b.iter(|| {
            let analyzer = CodeAnalyzer::new(black_box(&exclusions), "en-US");
            black_box(analyzer);
        })
    });
}

criterion_group!(
    benches,
    bench_analyze_garbage_file,
    bench_analyze_clean_file,
    bench_analyzer_creation,
    bench_analyzer_with_exclusions
);
criterion_main!(benches);
