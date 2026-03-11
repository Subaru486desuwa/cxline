use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub warning: String,
    pub error: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub icons: ThemeIcons,
    pub separator: String,
}

#[derive(Debug, Clone)]
pub struct ThemeIcons {
    pub model: &'static str,
    pub tokens: &'static str,
    pub cost: &'static str,
    pub timer: &'static str,
    pub git: &'static str,
    pub turns: &'static str,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: "cyan".to_string(),
            secondary: "white".to_string(),
            accent: "green".to_string(),
            warning: "yellow".to_string(),
            error: "red".to_string(),
        }
    }
}

const ICONS_DEFAULT: ThemeIcons = ThemeIcons {
    model: "\u{1f916} ",
    tokens: "\u{1f4ca} ",
    cost: "\u{1f4b0} ",
    timer: "\u{23f1}\u{fe0f}  ",
    git: "\u{1f33f} ",
    turns: "\u{1f504} ",
};

const ICONS_MINIMAL: ThemeIcons = ThemeIcons {
    model: "",
    tokens: "",
    cost: "$",
    timer: "",
    git: "",
    turns: "",
};

const ICONS_POWERLINE: ThemeIcons = ThemeIcons {
    model: "\u{f121} ",   // nf-fa-code
    tokens: "\u{f1fe} ",  // nf-fa-area_chart
    cost: "\u{f155} ",    // nf-fa-dollar
    timer: "\u{f017} ",   // nf-fa-clock_o
    git: "\u{e725} ",     // nf-dev-git_branch
    turns: "\u{f01e} ",   // nf-fa-refresh
};

impl Theme {
    pub fn from_name(name: &str, colors: Option<ThemeColors>, separator: Option<String>) -> Self {
        match name {
            "minimal" => Self {
                name: "minimal".to_string(),
                colors: colors.unwrap_or_default(),
                icons: ICONS_MINIMAL,
                separator: separator.unwrap_or_else(|| " | ".to_string()),
            },
            "powerline" => Self {
                name: "powerline".to_string(),
                colors: colors.unwrap_or_default(),
                icons: ICONS_POWERLINE,
                separator: separator.unwrap_or_else(|| " \u{e0b1} ".to_string()),
            },
            _ => Self {
                name: "default".to_string(),
                colors: colors.unwrap_or_default(),
                icons: ICONS_DEFAULT,
                separator: separator.unwrap_or_else(|| " \u{2502} ".to_string()),
            },
        }
    }
}
