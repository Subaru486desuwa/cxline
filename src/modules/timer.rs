use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct TimerModule;

fn format_duration(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

impl Module for TimerModule {
    fn name(&self) -> &str {
        "timer"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let elapsed = data.elapsed_seconds?;
        Some(format!(
            "{}{}",
            theme.icons.timer,
            apply_color(&format_duration(elapsed), &theme.colors.secondary)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(5.0), "5s");
        assert_eq!(format_duration(65.0), "1m5s");
        assert_eq!(format_duration(323.0), "5m23s");
        assert_eq!(format_duration(3661.0), "1h1m1s");
    }
}
