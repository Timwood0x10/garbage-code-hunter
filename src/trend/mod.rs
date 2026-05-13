//! History trend module.
//!
//! Tracks scan results over time and displays quality trends.

pub mod display;
pub mod history;

use anyhow::Result;
use history::{HistoryRecord, ToolScore};

pub use crate::common::OutputFormat;

/// Run trend display (load history and show chart).
pub fn run(last_n: usize, format: &OutputFormat) -> Result<String> {
    let records = history::load_last(last_n)?;

    let output = match format {
        OutputFormat::Terminal => display::format_terminal(&records, last_n),
        OutputFormat::Json => display::format_json(&records),
    };

    Ok(output)
}

/// Save a scan result to history.
pub fn save_scan(
    project_path: &str,
    overall_score: f64,
    tools: Vec<ToolScore>,
) -> Result<history::HistoryRecord> {
    let record = HistoryRecord {
        timestamp: history::now_timestamp(),
        project_path: project_path.to_string(),
        overall_score,
        tools,
    };
    history::save(&record)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_empty_history() {
        // This test relies on no history existing, which is fine for CI
        let result = run(10, &OutputFormat::Terminal);
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_and_load() {
        let tools = vec![
            ToolScore {
                name: "code-hunter".to_string(),
                score: 75.0,
                item_count: 20,
            },
            ToolScore {
                name: "commit-roaster".to_string(),
                score: 85.0,
                item_count: 50,
            },
        ];
        let record = save_scan("/tmp/test-project", 80.0, tools).unwrap();
        assert_eq!(record.overall_score, 80.0);
        assert_eq!(record.tools.len(), 2);

        // Verify it was saved
        let records = history::load_all().unwrap();
        assert!(records
            .iter()
            .any(|r| r.project_path == "/tmp/test-project"));
    }
}
