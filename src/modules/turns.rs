use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct TurnsModule;

impl Module for TurnsModule {
    fn name(&self) -> &str {
        "turns"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let turn = data.turn?;
        Some(format!(
            "{}{}",
            theme.icons.turns,
            apply_color(&format!("Turn {}", turn), &theme.colors.secondary)
        ))
    }
}
