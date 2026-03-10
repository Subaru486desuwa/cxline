use crate::modules::Module;
use crate::parser::SessionData;
use crate::style::apply_color;
use crate::theme::Theme;

pub struct CostModule {
    pub currency: String,
}

impl Default for CostModule {
    fn default() -> Self {
        Self {
            currency: "USD".to_string(),
        }
    }
}

impl Module for CostModule {
    fn name(&self) -> &str {
        "cost"
    }

    fn render(&self, data: &SessionData, theme: &Theme) -> Option<String> {
        let cost = data.cost.as_ref()?;
        let total = cost.total?;

        let symbol = match self.currency.as_str() {
            "CNY" => "\u{00a5}",
            _ => "$",
        };

        let formatted = if total < 0.01 {
            format!("{}{:.4}", symbol, total)
        } else {
            format!("{}{:.2}", symbol, total)
        };

        Some(format!(
            "{}{}",
            theme.icons.cost,
            apply_color(&formatted, &theme.colors.accent)
        ))
    }
}
