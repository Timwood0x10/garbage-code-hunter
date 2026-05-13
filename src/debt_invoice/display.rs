//! Display debt invoice in terminal or JSON.

use super::cost_model::Invoice;
use crate::common::i18n_ext::t;
use colored::Colorize;

/// Format invoice for terminal display.
pub fn format_terminal(invoice: &Invoice, lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        t(
            lang,
            "\u{1f4b8} 技术债账单",
            "\u{1f4b8} Technical Debt Invoice"
        )
        .bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    // Header
    out.push_str(&format!(
        "  {:<30} {:>6} {:>10} {:>12}\n",
        t(lang, "类别", "Category").bold(),
        t(lang, "数量", "Count").bold(),
        t(lang, "工时", "Hours").bold(),
        t(lang, "成本 (USD)", "Cost (USD)").bold()
    ));
    out.push_str(&format!("  {}\n", "\u{2500}".repeat(62)));

    // Line items
    for item in &invoice.items {
        let cost_str = format!("${:.0}", item.estimated_cost);
        out.push_str(&format!(
            "  {:<30} {:>6} {:>10.1} {:>12}\n",
            item.category, item.count, item.estimated_hours, cost_str
        ));
        out.push_str(&format!("    {}\n", item.pain_description.dimmed()));
    }

    out.push_str(&format!("  {}\n", "\u{2500}".repeat(62)));

    // Totals
    out.push_str(&format!(
        "  {:<30} {:>6} {:>10.1} {:>12}\n",
        "TOTAL".bold(),
        invoice.items.iter().map(|i| i.count).sum::<usize>(),
        invoice.total_hours,
        format!("${:.0}", invoice.total_cost).bold()
    ));

    out.push('\n');

    // Interest calculation
    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f4c8} 利息", "\u{1f4c8} Interest").bold()
    ));
    out.push_str(&format!(
        "  {}: {:.0}%\n",
        t(lang, "周利率", "Weekly interest rate"),
        invoice.weekly_interest_rate
    ));
    let monthly_cost = invoice.total_cost * (1.0 + invoice.weekly_interest_rate / 100.0).powf(4.0)
        - invoice.total_cost;
    out.push_str(&format!(
        "  {}: ${:.0}\n",
        t(lang, "月利息", "Monthly interest"),
        monthly_cost
    ));
    let yearly_cost = invoice.total_cost * (1.0 + invoice.weekly_interest_rate / 100.0).powf(52.0)
        - invoice.total_cost;
    out.push_str(&format!(
        "  {}: ${:.0}\n",
        t(lang, "年利息", "Yearly interest"),
        yearly_cost
    ));

    out.push('\n');

    // Project lifespan
    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f52e} 项目寿命", "\u{1f52e} Project Lifespan").bold()
    ));
    out.push_str(&format!(
        "  {}\n",
        t(
            lang,
            "代码库变得无法维护的预估时间：",
            "Estimated time before codebase becomes unmaintainable:"
        )
    ));
    let lifespan_str = if invoice.project_lifespan_months >= 24 {
        format!(
            "{} {}",
            invoice.project_lifespan_months,
            t(lang, "个月（暂时安全）", "months (you're fine, for now)")
        )
        .green()
        .to_string()
    } else if invoice.project_lifespan_months >= 12 {
        format!(
            "{} {}",
            invoice.project_lifespan_months,
            t(
                lang,
                "个月（该重构了）",
                "months (better start refactoring)"
            )
        )
        .yellow()
        .to_string()
    } else {
        format!(
            "{} {}",
            invoice.project_lifespan_months,
            t(
                lang,
                "个月（红色警报 \u{1f6a8}）",
                "months (CODE RED \u{1f6a8})"
            )
        )
        .red()
        .bold()
        .to_string()
    };
    out.push_str(&format!("  {}\n", lifespan_str));

    out
}

/// Format invoice as JSON.
pub fn format_json(invoice: &Invoice) -> String {
    serde_json::json!({
        "items": invoice.items.iter().map(|item| {
            serde_json::json!({
                "category": item.category,
                "count": item.count,
                "estimated_hours": item.estimated_hours,
                "estimated_cost": item.estimated_cost,
                "pain_description": item.pain_description,
            })
        }).collect::<Vec<_>>(),
        "total_hours": invoice.total_hours,
        "total_cost": invoice.total_cost,
        "project_lifespan_months": invoice.project_lifespan_months,
        "weekly_interest_rate": invoice.weekly_interest_rate,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debt_invoice::cost_model::InvoiceItem;

    fn make_invoice() -> Invoice {
        Invoice {
            items: vec![InvoiceItem {
                category: "unwrap() abuse".to_string(),
                count: 10,
                estimated_hours: 20.0,
                estimated_cost: 1500.0,
                pain_description: "test".to_string(),
            }],
            total_hours: 20.0,
            total_cost: 1500.0,
            project_lifespan_months: 12,
            weekly_interest_rate: 3.0,
        }
    }

    #[test]
    fn test_format_terminal() {
        let invoice = make_invoice();
        let out = format_terminal(&invoice, "en-US");
        assert!(out.contains("Technical Debt Invoice"));
        assert!(out.contains("unwrap() abuse"));
        assert!(out.contains("$1500"));
    }

    #[test]
    fn test_format_json() {
        let invoice = make_invoice();
        let json = format_json(&invoice);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total_cost"], 1500.0);
    }

    #[test]
    fn test_format_terminal_chinese() {
        let invoice = make_invoice();
        let out = format_terminal(&invoice, "zh-CN");
        assert!(out.contains("技术债账单"));
        assert!(out.contains("类别"));
        assert!(out.contains("成本"));
        assert!(out.contains("利息"));
        assert!(out.contains("项目寿命"));
    }

    #[test]
    fn test_format_terminal_zero_items() {
        let invoice = Invoice {
            items: vec![],
            total_hours: 0.0,
            total_cost: 0.0,
            project_lifespan_months: 24,
            weekly_interest_rate: 0.0,
        };
        let out = format_terminal(&invoice, "en-US");
        assert!(out.contains("Technical Debt Invoice"));
        assert!(out.contains("TOTAL"));
    }

    #[test]
    fn test_format_terminal_critical_lifespan() {
        let invoice = Invoice {
            items: vec![],
            total_hours: 0.0,
            total_cost: 0.0,
            project_lifespan_months: 6,
            weekly_interest_rate: 0.0,
        };
        let out = format_terminal(&invoice, "en-US");
        assert!(out.contains("CODE RED"));
    }

    #[test]
    fn test_format_terminal_warning_lifespan() {
        let invoice = Invoice {
            items: vec![],
            total_hours: 0.0,
            total_cost: 0.0,
            project_lifespan_months: 18,
            weekly_interest_rate: 0.0,
        };
        let out = format_terminal(&invoice, "en-US");
        assert!(out.contains("better start refactoring"));
    }
}
