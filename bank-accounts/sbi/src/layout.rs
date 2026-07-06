use crate::pdf_parser::CharItem;

#[derive(Debug, Clone)]
pub struct Line {
    pub chars: Vec<CharItem>,
    pub text: String,
    pub baseline: f64,
}

pub fn group_into_lines(page: &[CharItem], y_tolerance: f64) -> Vec<Line> {
    let mut chars = page.to_vec();
    // Sort by y0 first, then x0
    chars.sort_by(|a, b| {
        a.y0.partial_cmp(&b.y0).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.x0.partial_cmp(&b.x0).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut lines: Vec<Line> = Vec::new();
    let mut current_line_chars: Vec<CharItem> = Vec::new();
    let mut current_baseline = 0.0;

    for ch in chars {
        if current_line_chars.is_empty() {
            current_line_chars.push(ch.clone());
            current_baseline = ch.y0;
        } else {
            if (ch.y0 - current_baseline).abs() <= y_tolerance {
                current_line_chars.push(ch.clone());
                // update baseline to average
                let count = current_line_chars.len() as f64;
                current_baseline = current_baseline * ((count - 1.0) / count) + (ch.y0 / count);
            } else {
                // finish current line
                current_line_chars.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(std::cmp::Ordering::Equal));
                let mut text = String::new();
                let mut last_x1 = current_line_chars.first().map(|c| c.x0).unwrap_or(0.0);
                for c in &current_line_chars {
                    // add space if gap is large enough
                    let gap = c.x0 - last_x1;
                    if gap > (c.y1 - c.y0).abs() * 0.25 { // heuristic space
                        text.push(' ');
                    }
                    text.push_str(&c.text);
                    last_x1 = c.x1;
                }
                lines.push(Line {
                    chars: current_line_chars.clone(),
                    text: text.trim().to_string(),
                    baseline: current_baseline,
                });
                
                current_line_chars.clear();
                current_line_chars.push(ch.clone());
                current_baseline = ch.y0;
            }
        }
    }

    if !current_line_chars.is_empty() {
        current_line_chars.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(std::cmp::Ordering::Equal));
        let mut text = String::new();
        let mut last_x1 = current_line_chars.first().map(|c| c.x0).unwrap_or(0.0);
        for c in &current_line_chars {
            let gap = c.x0 - last_x1;
            if gap > (c.y1 - c.y0).abs() * 0.25 {
                text.push(' ');
            }
            text.push_str(&c.text);
            last_x1 = c.x1;
        }
        lines.push(Line {
            chars: current_line_chars,
            text: text.trim().to_string(),
            baseline: current_baseline,
        });
    }

    lines
}
