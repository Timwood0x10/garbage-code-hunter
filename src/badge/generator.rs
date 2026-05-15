//! SVG badge generator.

/// Badge style.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum BadgeStyle {
    #[default]
    Flat,
    Plastic,
}

/// Get the color hex for a given score.
fn score_color(score: f64) -> &'static str {
    match score as u32 {
        90..=100 => "#4c1",
        70..=89 => "#97CA00",
        50..=69 => "#dfb317",
        30..=49 => "#fe7d37",
        _ => "#e05d44",
    }
}

/// Generate an SVG badge string.
pub fn generate_svg(score: f64, style: &BadgeStyle) -> String {
    let label = "garbage";
    let score_str = format!("{:.0}", score);
    let color = score_color(score);

    let label_width: u32 = 75;
    let score_width: u32 = 85.max(score_str.len() as u32 * 10 + 20);
    let total_width = label_width + score_width;
    let label_center = label_width / 2;
    let score_center = label_width + score_width / 2;

    match style {
        BadgeStyle::Flat => {
            let mut s = String::new();
            s.push_str(&format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"20\">\n",
                total_width
            ));
            s.push_str("  <linearGradient id=\"b\" x2=\"0\" y2=\"100%\">\n");
            s.push_str("    <stop offset=\"0\" stop-color=\"#bbb\" stop-opacity=\".1\"/>\n");
            s.push_str("    <stop offset=\"1\" stop-opacity=\".1\"/>\n");
            s.push_str("  </linearGradient>\n");
            s.push_str(&format!(
                "  <mask id=\"a\"><rect width=\"{}\" height=\"20\" rx=\"3\" fill=\"#fff\"/></mask>\n",
                total_width
            ));
            s.push_str("  <g mask=\"url(#a)\">\n");
            s.push_str(&format!(
                "    <rect width=\"{}\" height=\"20\" fill=\"#555\"/>\n",
                label_width
            ));
            s.push_str(&format!(
                "    <rect x=\"{}\" width=\"{}\" height=\"20\" fill=\"{}\"/>\n",
                label_width, score_width, color
            ));
            s.push_str(&format!(
                "    <rect width=\"{}\" height=\"20\" fill=\"url(#b)\"/>\n",
                total_width
            ));
            s.push_str("  </g>\n");
            s.push_str("  <g fill=\"#fff\" text-anchor=\"middle\" font-family=\"DejaVu Sans,Verdana,Geneva,sans-serif\" font-size=\"11\">\n");
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">{}</text>\n",
                label_center, label
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"14\">{}</text>\n",
                label_center, label
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">{}</text>\n",
                score_center, score_str
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"14\">{}</text>\n",
                score_center, score_str
            ));
            s.push_str("  </g>\n");
            s.push_str("</svg>");
            s
        }
        BadgeStyle::Plastic => {
            let mut s = String::new();
            s.push_str(&format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"20\">\n",
                total_width
            ));
            // Top shine highlight gradient (visible 3D effect)
            s.push_str("  <linearGradient id=\"a\" x2=\"0\" y2=\"100%\">\n");
            s.push_str("    <stop offset=\"0\" stop-color=\"#fff\" stop-opacity=\".35\"/>\n");
            s.push_str("    <stop offset=\"1\" stop-opacity=\".15\"/>\n");
            s.push_str("  </linearGradient>\n");
            // Bottom shadow gradient
            s.push_str("  <linearGradient id=\"b\" x2=\"0\" y2=\"100%\">\n");
            s.push_str("    <stop offset=\"0\" stop-color=\"#000\" stop-opacity=\".12\"/>\n");
            s.push_str("    <stop offset=\"1\" stop-opacity=\".25\"/>\n");
            s.push_str("  </linearGradient>\n");
            s.push_str(&format!(
                "  <mask id=\"m\"><rect width=\"{}\" height=\"20\" rx=\"5\" fill=\"#fff\"/></mask>\n",
                total_width
            ));
            s.push_str("  <g mask=\"url(#m)\">\n");
            // Base colors
            s.push_str(&format!(
                "    <rect width=\"{}\" height=\"20\" fill=\"#555\"/>\n",
                label_width
            ));
            s.push_str(&format!(
                "    <rect x=\"{}\" width=\"{}\" height=\"20\" fill=\"{}\"/>\n",
                label_width, score_width, color
            ));
            // Top shine strip (the signature plastic look)
            s.push_str(&format!(
                "    <rect width=\"{}\" height=\"10\" fill=\"url(#a)\"/>\n",
                total_width
            ));
            // Bottom shadow
            s.push_str(&format!(
                "    <rect y=\"10\" width=\"{}\" height=\"10\" fill=\"url(#b)\"/>\n",
                total_width
            ));
            s.push_str("  </g>\n");
            // Thin white highlight line at very top for extra 3D pop
            s.push_str(&format!(
                "  <rect width=\"{}\" height=\"1\" fill=\"#fff\" fill-opacity=\".3\" rx=\"5\"/>\n",
                total_width
            ));
            s.push_str("  <g fill=\"#fff\" text-anchor=\"middle\" font-family=\"DejaVu Sans,Verdana,Geneva,sans-serif\" font-size=\"11\">\n");
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">{}</text>\n",
                label_center, label
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"14\">{}</text>\n",
                label_center, label
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"15\" fill=\"#010101\" fill-opacity=\".3\">{}</text>\n",
                score_center, score_str
            ));
            s.push_str(&format!(
                "    <text x=\"{}\" y=\"14\">{}</text>\n",
                score_center, score_str
            ));
            s.push_str("  </g>\n");
            s.push_str("</svg>");
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_color_ranges() {
        assert_eq!(score_color(95.0), "#4c1");
        assert_eq!(score_color(80.0), "#97CA00");
        assert_eq!(score_color(60.0), "#dfb317");
        assert_eq!(score_color(40.0), "#fe7d37");
        assert_eq!(score_color(20.0), "#e05d44");
    }

    #[test]
    fn test_generate_flat_contains_svg() {
        let svg = generate_svg(75.0, &BadgeStyle::Flat);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("garbage"));
        assert!(svg.contains("75"));
    }

    #[test]
    fn test_generate_plastic_contains_svg() {
        let svg = generate_svg(90.0, &BadgeStyle::Plastic);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("90"));
    }

    #[test]
    fn test_badge_uses_correct_color() {
        let svg = generate_svg(95.0, &BadgeStyle::Flat);
        assert!(svg.contains("#4c1"));

        let svg = generate_svg(25.0, &BadgeStyle::Flat);
        assert!(svg.contains("#e05d44"));
    }

    #[test]
    fn test_badge_score_formatting() {
        let svg = generate_svg(72.8, &BadgeStyle::Flat);
        assert!(svg.contains("73")); // rounded
    }
}
