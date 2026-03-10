mod config;
mod formatter;
mod modules;
mod parser;
mod style;
mod theme;
mod watcher;

use std::io::{self, BufRead};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut theme_arg = None;
    let mut modules_arg = None;
    let mut subcommand = None;
    let mut session_path = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "watch" | "show" | "setup" if subcommand.is_none() => {
                subcommand = Some(args[i].clone());
            }
            "--session" | "-s" => {
                if i + 1 < args.len() {
                    session_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--theme" | "-t" => {
                if i + 1 < args.len() {
                    theme_arg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--modules" | "-m" => {
                if i + 1 < args.len() {
                    modules_arg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-V" => {
                println!("cxline {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let mut config = config::Config::load();
    config.apply_args(theme_arg, modules_arg);

    match subcommand.as_deref() {
        Some("watch") => {
            if let Err(e) = watcher::watch_session(session_path, config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some("show") => {
            if let Err(e) = watcher::show_session(session_path, config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some("setup") => {
            run_setup();
        }
        _ => {
            // Legacy stdin mode
            run_stdin_mode(config);
        }
    }
}

fn run_stdin_mode(config: config::Config) {
    let theme = theme::Theme::from_name(
        &config.theme_name,
        config.theme_colors,
        config.separator,
    );
    let mods = modules::create_modules_from_config(
        &config.modules,
        config.tokens_config.show_bar.unwrap_or(true),
        config.tokens_config.bar_width.unwrap_or(10),
        config.cost_config.currency.as_deref().unwrap_or("USD"),
    );
    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or(80);

    let stdin = io::stdin();
    let mut input = String::new();

    if stdin.lock().read_line(&mut input).unwrap_or(0) == 0 {
        return;
    }

    let data = parser::parse_input(input.trim());
    let output = formatter::format_statusline(&mods, &data, &theme, width);

    if !output.is_empty() {
        println!("{}", output);
    }
}

fn run_setup() {
    let cxline_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "cxline".to_string());

    let home = dirs::home_dir().expect("Cannot determine home directory");
    let mut changes = Vec::new();

    // --- 1. Write ~/.tmux.conf ---
    let tmux_conf = home.join(".tmux.conf");
    let cxline_marker = "# cxline-managed";
    let status_right = format!(
        "#(CXLINE_QUIET=1 {} show 2>/dev/null)",
        cxline_path
    );

    let tmux_block = format!(
        r#"{marker}
set -g status on
set -g status-style 'bg=default,fg=white'
set -g status-left ''
set -g status-right-length 120
set -g status-right '{status_right}'
set -g status-interval 2
set -g status-position bottom
{marker}-end"#,
        marker = cxline_marker,
        status_right = status_right,
    );

    if tmux_conf.exists() {
        let content = std::fs::read_to_string(&tmux_conf).unwrap_or_default();
        if content.contains(cxline_marker) {
            // Replace existing cxline block
            let mut new_content = String::new();
            let mut skipping = false;
            for line in content.lines() {
                if line.trim() == cxline_marker {
                    skipping = true;
                    continue;
                }
                if line.trim() == format!("{}-end", cxline_marker) {
                    skipping = false;
                    continue;
                }
                if !skipping {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            new_content.push_str(&tmux_block);
            new_content.push('\n');
            std::fs::write(&tmux_conf, new_content).expect("Failed to write ~/.tmux.conf");
            changes.push("~/.tmux.conf (updated cxline block)");
        } else {
            // Append
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(&tmux_block);
            content.push('\n');
            std::fs::write(&tmux_conf, content).expect("Failed to write ~/.tmux.conf");
            changes.push("~/.tmux.conf (appended cxline config)");
        }
    } else {
        std::fs::write(&tmux_conf, format!("{}\n", tmux_block))
            .expect("Failed to create ~/.tmux.conf");
        changes.push("~/.tmux.conf (created)");
    }

    // --- 2. Add alias to shell rc ---
    let alias_line = r#"alias cx='tmux new-session -A -s codex'"#;
    let alias_marker = "# cxline: codex+tmux shortcut";
    let alias_block = format!("{}\n{}", alias_marker, alias_line);

    // Detect shell rc file
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    if rc_file.exists() {
        let content = std::fs::read_to_string(&rc_file).unwrap_or_default();
        if !content.contains(alias_line) {
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(&alias_block);
            content.push('\n');
            std::fs::write(&rc_file, content)
                .unwrap_or_else(|_| panic!("Failed to write {}", rc_file.display()));
            changes.push(if shell.contains("zsh") {
                "~/.zshrc (added cx alias)"
            } else {
                "~/.bashrc (added cx alias)"
            });
        }
    } else {
        std::fs::write(&rc_file, format!("{}\n", alias_block))
            .unwrap_or_else(|_| panic!("Failed to create {}", rc_file.display()));
        changes.push(if shell.contains("zsh") {
            "~/.zshrc (created with cx alias)"
        } else {
            "~/.bashrc (created with cx alias)"
        });
    }

    // --- 3. If inside tmux, apply immediately ---
    if std::env::var("TMUX").is_ok() {
        let _ = std::process::Command::new("tmux")
            .args(["source-file", &tmux_conf.display().to_string()])
            .output();
        changes.push("tmux config reloaded (live)");
    }

    // --- Output ---
    println!("cxline setup complete!");
    println!();
    for c in &changes {
        println!("  [+] {}", c);
    }
    println!();
    println!("Usage:");
    println!("  cx        - Start tmux + cxline status bar at bottom");
    println!("  codex     - Run Codex as usual, status bar auto-updates");
    println!();
    if std::env::var("TMUX").is_err() {
        println!("Run `source {}` or open a new terminal to activate.", rc_file.display());
    }
}

fn print_help() {
    println!(
        r#"cxline - A lightweight status bar & session monitor for OpenAI Codex CLI

USAGE:
    cxline [COMMAND] [OPTIONS]

COMMANDS:
    setup       One-click setup: tmux.conf + shell alias (recommended!)
    watch       Watch the latest Codex session in real-time
    show        Show summary of the latest Codex session
    (none)      Legacy stdin mode: pipe JSON for one-shot rendering

OPTIONS:
    -t, --theme <THEME>        Theme: default, minimal, powerline
    -m, --modules <MODULES>    Comma-separated module list
    -s, --session <PATH>       Path to a specific .jsonl session file
    -h, --help                 Show this help
    -V, --version              Show version

MODULES:
    model, tokens, cost, timer, git, permission, turns

EXAMPLES:
    cxline setup                          # One-click install
    cx                                    # Start tmux with cxline
    cxline watch                          # Watch latest Codex session
    cxline show                           # Show latest session summary
    echo '{{"model":"o3-mini"}}' | cxline   # Legacy stdin mode"#
    );
}
