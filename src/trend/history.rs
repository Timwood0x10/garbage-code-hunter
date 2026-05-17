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

    // ── days_to_ymd ────────────────────────────────────────────────

    /// Objective: Verify days_to_ymd returns the correct date for the Unix epoch.
    /// Invariants: 0 days since epoch → 1970-01-01.
    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!(
            (y, m, d),
            (1970, 1, 1),
            "epoch = 1970-01-01, got {y}-{m}-{d}"
        );
    }

    /// Objective: Verify days_to_ymd correctly computes dates around leap-year boundary.
    /// Invariants: 2024 is a leap year; 2024-03-01 is 60 days after 2024-01-01 (31 Jan + 29 Feb).
    #[test]
    fn test_days_to_ymd_leap_year() {
        let (y, m, d) = days_to_ymd(19723);
        assert_eq!(
            (y, m, d),
            (2024, 1, 1),
            "19723 days = 2024-01-01, got {y}-{m}-{d}"
        );

        let (y2, m2, d2) = days_to_ymd(19783);
        assert_eq!(
            (y2, m2, d2),
            (2024, 3, 1),
            "19783 days = 2024-03-01, got {y2}-{m2}-{d2}"
        );
    }

    /// Objective: Verify days_to_ymd handles a non-leap year correctly.
    /// Invariants: 2023-03-01 is 60 days after 2023-01-01 (non-leap Feb has 28).
    #[test]
    fn test_days_to_ymd_non_leap() {
        let (y, m, d) = days_to_ymd(19358); // 2023-01-01
        assert_eq!(
            (y, m, d),
            (2023, 1, 1),
            "19358 days = 2023-01-01, got {y}-{m}-{d}"
        );

        let (y2, m2, d2) = days_to_ymd(19358 + 59); // 2023-03-01
        assert_eq!(
            (y2, m2, d2),
            (2023, 3, 1),
            "19358+59 = 2023-03-01, got {y2}-{m2}-{d2}"
        );
    }

    /// Objective: Verify boundary at year 2000, a leap year.
    #[test]
    fn test_days_to_ymd_year_2000() {
        let (y, m, d) = days_to_ymd(10957); // 2000-01-01
        assert_eq!(
            (y, m, d),
            (2000, 1, 1),
            "10957 days = 2000-01-01, got {y}-{m}-{d}"
        );
    }

    /// Objective: Verify days_to_ymd for 2000-01-01 (leap-year century) and 2001-01-01.
    #[test]
    fn test_days_to_ymd_century_boundary() {
        // 30 years from epoch (1970-2000) = 10957 days including 7 leap years
        let (y, m, d) = days_to_ymd(10957);
        assert_eq!(
            (y, m, d),
            (2000, 1, 1),
            "10957 days = 2000-01-01, got {y}-{m}-{d}"
        );

        // 2000 is leap (366 days), so 10957 + 366 = 11323 → 2001-01-01
        let (y2, m2, d2) = days_to_ymd(11323);
        assert_eq!(
            (y2, m2, d2),
            (2001, 1, 1),
            "11323 days = 2001-01-01, got {y2}-{m2}-{d2}"
        );
    }

    // ── epoch_to_datetime ──────────────────────────────────────────

    /// Objective: Verify epoch_to_datetime for a known value (2024-01-15T14:20:00Z).
    #[test]
    fn test_epoch_to_datetime_known() {
        let dt = epoch_to_datetime(1705328400);
        assert_eq!(
            dt, "2024-01-15T14:20:00Z",
            "1705328400 → 2024-01-15T14:20:00Z, got {dt}"
        );
    }

    /// Objective: Verify epoch_to_datetime for the Unix epoch itself (0 seconds).
    /// Invariants: 0 seconds since epoch → 1970-01-01T00:00:00Z.
    #[test]
    fn test_epoch_to_datetime_zero() {
        let dt = epoch_to_datetime(0);
        assert_eq!(dt, "1970-01-01T00:00:00Z", "0 seconds → epoch, got {dt}");
    }

    /// Objective: Verify epoch_to_datetime for a leap-year date (2024-02-29 12:00:00Z).
    #[test]
    fn test_epoch_to_datetime_leap_day() {
        // 2024-02-29T12:00:00Z = 1709208000
        let dt = epoch_to_datetime(1709208000);
        assert!(dt.starts_with("2024-02-29T12:"), "leap day noon, got {dt}");
    }

    // ── load_last / load_since ─────────────────────────────────────

    // ── Serialization ──────────────────────────────────────────────

    /// Objective: Verify HistoryRecord round-trips through JSON without data loss.
    /// Invariants: All fields are preserved after serde round-trip.
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
        assert_eq!(parsed.overall_score, 72.5, "score after round-trip");
        assert_eq!(parsed.tools.len(), 2, "tool count after round-trip");
        assert_eq!(
            parsed.timestamp, "2024-01-15T14:30:00Z",
            "timestamp preserved"
        );
        assert_eq!(parsed.project_path, "/tmp/test", "project path preserved");
    }

    /// Objective: Verify HistoryRecord with empty tools vector round-trips correctly.
    #[test]
    fn test_history_record_empty_tools() {
        let record = HistoryRecord {
            timestamp: "2024-06-01T00:00:00Z".to_string(),
            project_path: "/tmp/empty".to_string(),
            overall_score: 0.0,
            tools: vec![],
        };
        let json = serde_json::to_string(&record).unwrap();
        let parsed: HistoryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tools.len(), 0, "empty tools preserved");
        assert_eq!(parsed.overall_score, 0.0, "score 0.0 preserved");
    }

    // ── now_timestamp ──────────────────────────────────────────────

    /// Objective: Verify now_timestamp produces a valid ISO-like timestamp.
    /// Invariants: Contains 'T' separator and ends with 'Z'.
    #[test]
    fn test_now_timestamp_format() {
        let ts = now_timestamp();
        assert!(ts.contains('T'), "timestamp must contain T separator: {ts}");
        assert!(ts.ends_with('Z'), "timestamp must end with Z: {ts}");
        assert_eq!(
            ts.len(),
            20,
            "expected 20-char ISO timestamp, got len {}: {ts}",
            ts.len()
        );
    }
}
