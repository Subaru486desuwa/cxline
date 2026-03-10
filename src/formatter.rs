use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::{truncate_ansi, visible_len};
use crate::theme::Theme;

/// Render all modules and join them into a single status line
pub fn format_statusline(
    modules: &[Box<dyn Module>],
    data: &SessionData,
    theme: &Theme,
    terminal_width: u16,
) -> String {
    let segments: Vec<String> = modules
        .iter()
        .filter_map(|m| m.render(data, theme))
        .collect();

    if segments.is_empty() {
        return String::new();
    }

    let line = segments.join(&theme.separator);

    // Truncate if exceeds terminal width
    let width = terminal_width as usize;
    if width > 0 && visible_len(&line) > width {
        truncate_ansi(&line, width.saturating_sub(1))
    } else {
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model::ModelModule;
    use crate::theme::Theme;

    #[test]
    fn test_format_empty_data() {
        let modules: Vec<Box<dyn Module>> = vec![Box::new(ModelModule)];
        let data = SessionData::default();
        let theme = Theme::from_name("default", None, None);
        let result = format_statusline(&modules, &data, &theme, 80);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_with_model() {
        let modules: Vec<Box<dyn Module>> = vec![Box::new(ModelModule)];
        let data = SessionData {
            model: Some("o3-mini".to_string()),
            ..Default::default()
        };
        let theme = Theme::from_name("default", None, None);
        let result = format_statusline(&modules, &data, &theme, 80);
        assert!(result.contains("o3-mini"));
    }
}
