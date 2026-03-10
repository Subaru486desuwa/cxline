pub mod model;
pub mod tokens;
pub mod cost;
pub mod timer;
pub mod git;
pub mod permission;
pub mod turns;

use crate::parser::SessionData;
use crate::theme::Theme;

/// Trait for pluggable status bar modules
pub trait Module {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String>;
}

/// Create a module by name
pub fn create_module(name: &str) -> Option<Box<dyn Module>> {
    match name {
        "model" => Some(Box::new(model::ModelModule)),
        "tokens" => Some(Box::new(tokens::TokensModule::default())),
        "cost" => Some(Box::new(cost::CostModule::default())),
        "timer" => Some(Box::new(timer::TimerModule)),
        "git" => Some(Box::new(git::GitModule)),
        "permission" => Some(Box::new(permission::PermissionModule)),
        "turns" => Some(Box::new(turns::TurnsModule)),
        _ => None,
    }
}

/// Create modules from config list, with optional per-module config
pub fn create_modules_from_config(
    names: &[String],
    tokens_show_bar: bool,
    tokens_bar_width: usize,
    cost_currency: &str,
) -> Vec<Box<dyn Module>> {
    names
        .iter()
        .filter_map(|name| {
            match name.as_str() {
                "tokens" => Some(Box::new(tokens::TokensModule {
                    show_bar: tokens_show_bar,
                    bar_width: tokens_bar_width,
                }) as Box<dyn Module>),
                "cost" => Some(Box::new(cost::CostModule {
                    currency: cost_currency.to_string(),
                }) as Box<dyn Module>),
                other => create_module(other),
            }
        })
        .collect()
}
