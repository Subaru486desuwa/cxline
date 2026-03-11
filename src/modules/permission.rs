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

        let (icon, color) = match perm.as_str() {
            "full-auto" | "auto" => {
                let ic = match theme.name.as_str() {
                    "minimal" => "",
                    "powerline" => "\u{f09c} ",  // nf-fa-unlock
                    _ => "\u{1f513} ",            // 🔓
                };
                (ic, &theme.colors.error)
            }
            "suggest" | "ask" => {
                let ic = match theme.name.as_str() {
                    "minimal" => "",
                    "powerline" => "\u{f00c} ",  // nf-fa-check
                    _ => "\u{2705} ",             // ✅
                };
                (ic, &theme.colors.accent)
            }
            _ => {
                // on-request and others
                let ic = match theme.name.as_str() {
                    "minimal" => "",
                    "powerline" => "\u{f023} ",  // nf-fa-lock
                    _ => "\u{26a1} ",             // ⚡
                };
                (ic, &theme.colors.warning)
            }
        };

        Some(format!(
            "{}{}",
            icon,
            apply_color(perm, color)
        ))
    }
}
