use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct GitModule;

impl Module for GitModule {
    fn name(&self) -> &str {
        "git"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        // Try JSON-provided branch first
        if let Some(branch) = &data.git_branch {
            let dirty_marker = if data.git_dirty.unwrap_or(false) {
                "*"
            } else {
                ""
            };
            return Some(format!(
                "{}{}",
                theme.icons.git,
                apply_color(&format!("{}{}", branch, dirty_marker), &theme.colors.accent)
            ));
        }

        // Fallback: detect from current working directory
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            return None;
        }

        Some(format!(
            "{}{}",
            theme.icons.git,
            apply_color(&branch, &theme.colors.accent)
        ))
    }
}
