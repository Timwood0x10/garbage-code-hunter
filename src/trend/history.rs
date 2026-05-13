//! History record storage and retrieval.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single scan history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: String,
    pub project_path: String,
    pub overall_score: f64,
    pub tools: Vec<ToolScore>,
}

/// Score for a single tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolScore {
    pub name: String,
    pub score: f64,
    pub item_count: usize,
}

/// Get the history directory path (~/.garbage-code-hunter/history/).
fn history_dir() -> Result<PathBuf> {
    let home = dirs_home().context("Could not determine home directory")?;
    let dir = home.join(".garbage-code-hunter").join("history");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Save a history record to disk.
pub fn save(record: &HistoryRecord) -> Result<PathBuf> {
    let dir = history_dir()?;
    let filename = format!("{}.json", record.timestamp.replace(':', "-"));
    let path = dir.join(&filename);
    let json = serde_json::to_string_pretty(record)?;
    fs::write(&path, json)?;
    Ok(path)
}

/// Load all history records, sorted by timestamp ascending.
pub fn load_all() -> Result<Vec<HistoryRecord>> {
    let dir = match history_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]),
    };

    let mut records = Vec::new();
    if !dir.exists() {
        return Ok(records);
    }

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(record) = serde_json::from_str::<HistoryRecord>(&content) {
                    records.push(record);
                }
            }
        }
    }

    records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(records)
}

/// Load the last N records.
pub fn load_last(n: usize) -> Result<Vec<HistoryRecord>> {
    let all = load_all()?;
    let start = all.len().saturating_sub(n);
    Ok(all[start..].to_vec())
}

/// Load records since a given date (YYYY-MM-DD).
pub fn load_since(date: &str) -> Result<Vec<HistoryRecord>> {
    let all = load_all()?;
    Ok(all
        .into_iter()
        .filter(|r| r.timestamp.as_str() >= date)
        .collect())
}

/// Generate a timestamp string for now.
pub fn now_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple epoch -> datetime conversion
    epoch_to_datetime(secs)
}

fn epoch_to_datetime(secs: u64) -> String {
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Days since 1970-01-01 to Y-M-D
    let (y, m, d) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

fn days_to_ymd(days: u64) -> (i64, u32, u32) {
    // Simplified civil calendar from days since epoch
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era as i64 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_to_datetime() {
        // 2024-01-15T14:30:00Z = 1705328400
        let dt = epoch_to_datetime(1705328400);
        assert!(dt.starts_with("2024-01-15"));
    }

    #[test]
    fn test_now_timestamp() {
        let ts = now_timestamp();
        assert!(ts.contains("T"));
        assert!(ts.ends_with("Z"));
    }

    #[test]
    fn test_history_record_serde() {
        let record = HistoryRecord {
            timestamp: "2024-01-15T14:30:00Z".to_string(),
            project_path: "/tmp/test".to_string(),
            overall_score: 72.5,
            tools: vec![
                ToolScore {
                    name: "code-hunter".to_string(),
                    score: 65.0,
                    item_count: 45,
                },
                ToolScore {
                    name: "commit-roaster".to_string(),
                    score: 80.0,
                    item_count: 12,
                },
            ],
        };
        let json = serde_json::to_string(&record).unwrap();
        let parsed: HistoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.overall_score, 72.5);
        assert_eq!(parsed.tools.len(), 2);
    }
}
