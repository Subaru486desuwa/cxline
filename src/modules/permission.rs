use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct PermissionModule;

impl Module for PermissionModule {
    fn name(&self) -> &str {
        "permission"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let perm = data.permission.as_ref()?;

        let color = match perm.as_str() {
            "full-auto" | "auto" => &theme.colors.error,
            "suggest" | "ask" => &theme.colors.accent,
            _ => &theme.colors.warning,
        };

        Some(format!(
            "{}{}",
            theme.icons.permission,
            apply_color(perm, color)
        ))
    }
}
