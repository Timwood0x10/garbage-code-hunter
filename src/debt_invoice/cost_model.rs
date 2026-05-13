//! Cost model for estimating technical debt in time and money.

use crate::analyzer::CodeIssue;
use std::collections::HashMap;

/// A single line item on the debt invoice.
#[derive(Debug, Clone)]
pub struct InvoiceItem {
    pub category: String,
    pub count: usize,
    pub estimated_hours: f64,
    pub estimated_cost: f64,
    pub pain_description: String,
}

/// The full debt invoice.
#[derive(Debug, Clone)]
pub struct Invoice {
    pub items: Vec<InvoiceItem>,
    pub total_hours: f64,
    pub total_cost: f64,
    pub project_lifespan_months: u32,
    pub weekly_interest_rate: f64,
}

/// Hourly rate for cost estimation (USD).
const HOURLY_RATE: f64 = 75.0;

/// Generate an invoice from code issues.
pub fn generate_invoice(issues: &[CodeIssue], total_lines: usize) -> Invoice {
    let mut categories: HashMap<String, Vec<&CodeIssue>> = HashMap::new();

    for issue in issues {
        let cat = categorize(&issue.rule_name);
        categories.entry(cat.to_string()).or_default().push(issue);
    }

    let mut items = Vec::new();

    for (category, cat_issues) in &categories {
        let count = cat_issues.len();
        let (hours_per, pain) = category_cost(category);
        let estimated_hours = count as f64 * hours_per;
        let estimated_cost = estimated_hours * HOURLY_RATE;

        items.push(InvoiceItem {
            category: category.clone(),
            count,
            estimated_hours,
            estimated_cost,
            pain_description: pain.to_string(),
        });
    }

    // Sort by cost descending
    items.sort_by(|a, b| b.estimated_cost.partial_cmp(&a.estimated_cost).unwrap());

    let total_hours: f64 = items.iter().map(|i| i.estimated_hours).sum();
    let total_cost = total_hours * HOURLY_RATE;

    // Estimate project lifespan based on debt density
    let debt_density = if total_lines > 0 {
        issues.len() as f64 / total_lines as f64 * 1000.0
    } else {
        0.0
    };
    let project_lifespan = estimate_lifespan(debt_density);

    Invoice {
        items,
        total_hours,
        total_cost,
        project_lifespan_months: project_lifespan,
        weekly_interest_rate: 3.0,
    }
}

/// Map a rule name to a cost category.
fn categorize(rule_name: &str) -> &'static str {
    let lower = rule_name.to_lowercase();
    if lower.contains("unwrap") {
        "unwrap() abuse"
    } else if lower.contains("nest") || lower.contains("complex") {
        "Deep nesting / complexity"
    } else if lower.contains("long") || lower.contains("function_length") {
        "Long functions"
    } else if lower.contains("name")
        || lower.contains("single_letter")
        || lower.contains("meaningless")
    {
        "Poor naming"
    } else if lower.contains("magic") {
        "Magic numbers"
    } else if lower.contains("duplicat") || lower.contains("copy") {
        "Code duplication"
    } else if lower.contains("todo") || lower.contains("fixme") || lower.contains("hack") {
        "Legacy comments"
    } else {
        "Other issues"
    }
}

/// Returns (hours_per_issue, pain_description) for a category.
fn category_cost(category: &str) -> (f64, &'static str) {
    match category {
        "unwrap() abuse" => (2.0, "Every unwrap() is a potential panic in production"),
        "Deep nesting / complexity" => (3.0, "Cyclomatic complexity makes code unmaintainable"),
        "Long functions" => (1.5, "Long functions are hard to test and understand"),
        "Poor naming" => (0.5, "Meaningless names slow down every future reader"),
        "Magic numbers" => (0.5, "Magic numbers hide intent and cause bugs"),
        "Code duplication" => (2.0, "Duplicated code multiplies bug-fix cost"),
        "Legacy comments" => (0.3, "TODOs that never get done are lies in the code"),
        _ => (0.5, "General code quality issue"),
    }
}

/// Estimate project lifespan in months based on debt density.
fn estimate_lifespan(density: f64) -> u32 {
    match density as u32 {
        0..=5 => 24,
        6..=15 => 18,
        16..=30 => 12,
        31..=50 => 6,
        51..=80 => 3,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize() {
        assert_eq!(categorize("unwrap_abuse"), "unwrap() abuse");
        assert_eq!(categorize("deep_nesting"), "Deep nesting / complexity");
        assert_eq!(categorize("magic_number"), "Magic numbers");
    }

    #[test]
    fn test_category_cost() {
        let (hours, _) = category_cost("unwrap() abuse");
        assert_eq!(hours, 2.0);
    }

    #[test]
    fn test_estimate_lifespan() {
        assert_eq!(estimate_lifespan(0.0), 24);
        assert_eq!(estimate_lifespan(100.0), 1);
    }
}
