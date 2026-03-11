use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct CwdModule;

impl Module for CwdModule {
    fn name(&self) -> &str {
        "cwd"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let cwd = data.cwd.as_ref()?;

        // Shorten: replace $HOME with ~, then show last 2 components
        let display = shorten_path(cwd);

        let icon = match theme.name.as_str() {
            "minimal" => "",
            "powerline" => "\u{f07b} ", // nf-fa-folder
            _ => "\u{1f4c2} ",          // 📂
        };

        Some(format!(
            "{}{}",
            icon,
            apply_color(&display, &theme.colors.secondary)
        ))
    }
}

fn shorten_path(path: &str) -> String {
    let shortened = if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path.starts_with(home_str.as_ref()) {
            format!("~{}", &path[home_str.len()..])
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };

    // Show last 2 path components for brevity
    let parts: Vec<&str> = shortened.rsplitn(3, '/').collect();
    if parts.len() >= 3 {
        format!(".../{}/{}", parts[1], parts[0])
    } else {
        shortened
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_path_short() {
        assert_eq!(shorten_path("/foo"), "/foo");
    }

    #[test]
    fn test_shorten_path_long() {
        assert_eq!(shorten_path("/a/b/c/d"), ".../c/d");
    }
}
