use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct ModelModule;

impl Module for ModelModule {
    fn name(&self) -> &str {
        "model"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let model = data.model.as_ref()?;
        Some(format!(
            "{}{}",
            theme.icons.model,
            apply_color(model, &theme.colors.primary)
        ))
    }
}
