use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::{apply_color, progress_bar};
use crate::theme::Theme;

pub struct TokensModule {
    pub show_bar: bool,
    pub bar_width: usize,
    pub show_detail: bool,
}

impl Default for TokensModule {
    fn default() -> Self {
        Self {
            show_bar: false,
            bar_width: 10,
            show_detail: false,
        }
    }
}

fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

impl Module for TokensModule {
    fn name(&self) -> &str {
        "tokens"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let usage = data.token_usage.as_ref()?;
        let used = usage.used?;

        let (ratio, pct_str, pct_color) = if let Some(total) = usage.total {
            let r = if total > 0 { (used as f64 / total as f64).min(1.0) } else { 0.0 };
            let pct = r * 100.0;
            let pct_str = if pct < 10.0 {
                format!("{:.1}%", pct)
            } else {
                format!("{:.0}%", pct)
            };
            let color = if r > 0.9 {
                &theme.colors.error
            } else if r > 0.7 {
                &theme.colors.warning
            } else {
                &theme.colors.accent
            };
            (Some(r), Some(pct_str), Some(color))
        } else {
            (None, None, None)
        };

        // Base: icon used/total
        let base = if let Some(total) = usage.total {
            format!("{}{}/{}", theme.icons.tokens, format_count(used), format_count(total))
        } else {
            format!("{}{}", theme.icons.tokens, format_count(used))
        };

        let mut parts = vec![base];

        // Bar mode (optional, off by default)
        if self.show_bar {
            if let (Some(r), Some(color)) = (ratio, pct_color) {
                let bar = progress_bar(r, self.bar_width);
                parts.push(apply_color(&bar, color));
            }
        }

        // Percentage
        if let (Some(pct), Some(color)) = (&pct_str, pct_color) {
            parts.push(apply_color(pct, color));
        }

        // Detail breakdown (optional, off by default)
        if self.show_detail {
            let mut details = Vec::new();
            if let Some(input) = usage.input {
                details.push(format!("in:{}", format_count(input)));
            }
            if let Some(output) = usage.output {
                details.push(format!("out:{}", format_count(output)));
            }
            if let Some(cached) = usage.cached {
                if cached > 0 {
                    details.push(format!("cache:{}", format_count(cached)));
                }
            }
            if let Some(reasoning) = usage.reasoning {
                if reasoning > 0 {
                    details.push(format!("reason:{}", format_count(reasoning)));
                }
            }
            if !details.is_empty() {
                parts.push(details.join(" "));
            }
        }

        Some(apply_color(&parts.join(" "), &theme.colors.secondary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(500), "500");
        assert_eq!(format_count(1500), "1.5k");
        assert_eq!(format_count(12500), "12.5k");
        assert_eq!(format_count(1_500_000), "1.5M");
    }
}
