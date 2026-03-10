use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::{apply_color, progress_bar};
use crate::theme::Theme;

pub struct TokensModule {
    pub show_bar: bool,
    pub bar_width: usize,
}

impl Default for TokensModule {
    fn default() -> Self {
        Self {
            show_bar: true,
            bar_width: 10,
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

        let mut parts = vec![format!("{}{}", theme.icons.tokens, format_count(used))];

        if let Some(total) = usage.total {
            parts[0] = format!("{}{}/{}", theme.icons.tokens, format_count(used), format_count(total));

            if self.show_bar {
                let ratio = if total > 0 {
                    (used as f64 / total as f64).min(1.0)
                } else {
                    0.0
                };
                let bar_color = if ratio > 0.9 {
                    &theme.colors.error
                } else if ratio > 0.7 {
                    &theme.colors.warning
                } else {
                    &theme.colors.accent
                };
                let bar = progress_bar(ratio, self.bar_width);
                parts.push(apply_color(&bar, bar_color));
            }
        }

        // Append detailed breakdown if available (Codex token categories)
        let mut details = Vec::new();
        if let Some(input) = usage.input {
            let mut s = format!("{}in", format_count(input));
            if let Some(cached) = usage.cached {
                if cached > 0 {
                    s.push_str(&format!(" ({}cached)", format_count(cached)));
                }
            }
            details.push(s);
        }
        if let Some(output) = usage.output {
            details.push(format!("{}out", format_count(output)));
        }
        if let Some(reasoning) = usage.reasoning {
            if reasoning > 0 {
                details.push(format!("{}reason", format_count(reasoning)));
            }
        }
        if !details.is_empty() {
            parts.push(details.join(" "));
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
