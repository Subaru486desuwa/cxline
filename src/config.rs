use crate::theme::ThemeColors;
use std::path::PathBuf;

// Config is parsed manually from toml::Value, no serde struct needed

#[derive(Debug, Default)]
pub struct ModuleTokensConfig {
    pub show_bar: Option<bool>,
    pub bar_width: Option<usize>,
}

#[derive(Debug, Default)]
pub struct ModuleCostConfig {
    pub currency: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub theme_name: String,
    pub separator: Option<String>,
    pub modules: Vec<String>,
    pub theme_colors: Option<ThemeColors>,
    pub tokens_config: ModuleTokensConfig,
    pub cost_config: ModuleCostConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: "default".to_string(),
            separator: None,
            modules: vec![
                "model".to_string(),
                "tokens".to_string(),
                "cost".to_string(),
                "timer".to_string(),
                "git".to_string(),
                "permission".to_string(),
            ],
            theme_colors: None,
            tokens_config: ModuleTokensConfig::default(),
            cost_config: ModuleCostConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Self::default();
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        Self::parse_toml(&content)
    }

    fn parse_toml(content: &str) -> Self {
        let table: toml::Value = match content.parse() {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };

        let theme_name = table
            .get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let separator = table
            .get("separator")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let modules = table
            .get("modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| Self::default().modules);

        // Theme colors: [theme.colors] table (only if theme is a table, not a string)
        // In TOML, "theme" as a string and "[theme.colors]" table conflict,
        // so we use a separate key: [colors]
        let theme_colors = table.get("colors").and_then(|c| {
            let t = c.as_table()?;
            Some(crate::theme::ThemeColors {
                primary: t.get("primary").and_then(|v| v.as_str()).unwrap_or("cyan").to_string(),
                secondary: t.get("secondary").and_then(|v| v.as_str()).unwrap_or("white").to_string(),
                accent: t.get("accent").and_then(|v| v.as_str()).unwrap_or("green").to_string(),
                warning: t.get("warning").and_then(|v| v.as_str()).unwrap_or("yellow").to_string(),
                error: t.get("error").and_then(|v| v.as_str()).unwrap_or("red").to_string(),
            })
        });

        // Module-specific configs: [modules.tokens], [modules.cost]
        // Note: "modules" as array and "[modules.tokens]" table conflict in TOML,
        // so we use separate top-level keys
        let tokens_config = table.get("tokens").and_then(|t| {
            let t = t.as_table()?;
            Some(ModuleTokensConfig {
                show_bar: t.get("show_bar").and_then(|v| v.as_bool()),
                bar_width: t.get("bar_width").and_then(|v| v.as_integer()).map(|v| v as usize),
            })
        }).unwrap_or_default();

        let cost_config = table.get("cost").and_then(|t| {
            let t = t.as_table()?;
            Some(ModuleCostConfig {
                currency: t.get("currency").and_then(|v| v.as_str()).map(|s| s.to_string()),
            })
        }).unwrap_or_default();

        Config {
            theme_name,
            separator,
            modules,
            theme_colors,
            tokens_config,
            cost_config,
        }
    }

    /// Override config with CLI arguments
    pub fn apply_args(&mut self, theme: Option<String>, modules: Option<String>) {
        if let Some(t) = theme {
            self.theme_name = t;
        }
        if let Some(m) = modules {
            self.modules = m.split(',').map(|s| s.trim().to_string()).collect();
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cxline")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.theme_name, "default");
        assert_eq!(config.modules.len(), 6);
    }

    #[test]
    fn test_parse_simple_toml() {
        let toml = r#"
theme = "minimal"
separator = " | "
modules = ["model", "cost"]
"#;
        let config = Config::parse_toml(toml);
        assert_eq!(config.theme_name, "minimal");
        assert_eq!(config.modules, vec!["model", "cost"]);
    }

    #[test]
    fn test_apply_args() {
        let mut config = Config::default();
        config.apply_args(Some("powerline".to_string()), Some("model,tokens".to_string()));
        assert_eq!(config.theme_name, "powerline");
        assert_eq!(config.modules, vec!["model", "tokens"]);
    }
}
