use colored::Colorize;

/// Apply a named color to text
pub fn apply_color(text: &str, color: &str) -> String {
    match color {
        "red" => text.red().to_string(),
        "green" => text.green().to_string(),
        "yellow" => text.yellow().to_string(),
        "blue" => text.blue().to_string(),
        "magenta" => text.magenta().to_string(),
        "cyan" => text.cyan().to_string(),
        "white" => text.white().to_string(),
        "bright_red" => text.bright_red().to_string(),
        "bright_green" => text.bright_green().to_string(),
        "bright_yellow" => text.bright_yellow().to_string(),
        "bright_blue" => text.bright_blue().to_string(),
        "bright_cyan" => text.bright_cyan().to_string(),
        "bright_magenta" => text.bright_magenta().to_string(),
        "bright_white" => text.bright_white().to_string(),
        _ => text.to_string(),
    }
}

/// Render a progress bar: [████░░░░░░] style
pub fn progress_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Strip all ANSI escape sequences, returning plain visible text.
/// Used by title mode since terminal title bars don't render ANSI colors.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            result.push(c);
        }
    }
    result
}

/// Strip ANSI escape codes to get the visible character count
pub fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            // CJK and emoji can be wider, but for terminal purposes
            // we count each Unicode scalar as 1 (terminal handles width)
            len += 1;
        }
    }
    len
}

/// Truncate a string with ANSI codes to fit within max visible width
pub fn truncate_ansi(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut visible = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if in_escape {
            result.push(c);
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
            result.push(c);
        } else {
            if visible >= max_width {
                break;
            }
            result.push(c);
            visible += 1;
        }
    }

    // Reset colors after truncation
    if visible >= max_width {
        result.push_str("\x1b[0m");
    }
    result
}
