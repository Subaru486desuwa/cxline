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
    let mut title_mode = false;
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
            "--title" => {
                title_mode = true;
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
            if let Err(e) = watcher::watch_session(session_path, config, title_mode) {
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
        config.tokens_config.show_bar.unwrap_or(false),
        config.tokens_config.bar_width.unwrap_or(10),
        config.tokens_config.show_detail.unwrap_or(false),
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
    if cfg!(windows) {
        run_setup_windows();
    } else {
        run_setup_unix();
    }
}

#[cfg(windows)]
fn run_setup_windows() {
    let cxline_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "cxline".to_string());

    let home = dirs::home_dir().expect("Cannot determine home directory");
    let mut changes: Vec<String> = Vec::new();

    // --- Write PowerShell profile ---
    // PowerShell profile path: ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1
    let ps_dir = home.join("Documents").join("PowerShell");
    let profile_path = ps_dir.join("Microsoft.PowerShell_profile.ps1");

    let func_marker = "# cxline-codex-wrapper";
    let func_marker_end = "# cxline-codex-wrapper-end";
    let func_block = format!(
        r#"{marker}
function codex {{
    $cxlineArgs = @("watch", "--title")
    $cxlineProc = Start-Process -FilePath "{cxline}" -ArgumentList $cxlineArgs -NoNewWindow -PassThru
    try {{
        & codex.exe @args
    }} finally {{
        if (!$cxlineProc.HasExited) {{
            Stop-Process -Id $cxlineProc.Id -Force -ErrorAction SilentlyContinue
        }}
        # Clear terminal title
        Write-Host "`e]0;`a" -NoNewline
    }}
}}
{marker_end}"#,
        marker = func_marker,
        marker_end = func_marker_end,
        cxline = cxline_path,
    );

    if !ps_dir.exists() {
        std::fs::create_dir_all(&ps_dir).expect("Failed to create PowerShell profile directory");
    }

    if profile_path.exists() {
        let content = std::fs::read_to_string(&profile_path).unwrap_or_default();
        if content.contains(func_marker) {
            let mut new_content = String::new();
            let mut skipping = false;
            for line in content.lines() {
                if line.trim() == func_marker {
                    skipping = true;
                    continue;
                }
                if line.trim() == func_marker_end {
                    skipping = false;
                    continue;
                }
                if !skipping {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            new_content.push_str(&func_block);
            new_content.push('\n');
            std::fs::write(&profile_path, new_content)
                .expect("Failed to write PowerShell profile");
            changes.push("PowerShell profile (updated codex wrapper)".into());
        } else {
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(&func_block);
            content.push('\n');
            std::fs::write(&profile_path, content)
                .expect("Failed to write PowerShell profile");
            changes.push("PowerShell profile (added codex wrapper)".into());
        }
    } else {
        std::fs::write(&profile_path, format!("{}\n", func_block))
            .expect("Failed to create PowerShell profile");
        changes.push("PowerShell profile (created with codex wrapper)".into());
    }

    // --- Output ---
    println!("cxline setup complete! (Windows)");
    println!();
    for c in &changes {
        println!("  [+] {}", c);
    }
    println!();
    println!("  Profile: {}", profile_path.display());
    println!();
    println!("Usage: restart PowerShell, then type 'codex' - status appears in title bar.");
}

#[cfg(not(windows))]
fn run_setup_windows() {
    eprintln!("Windows setup is only available on Windows.");
    std::process::exit(1);
}

fn run_setup_unix() {
    let cxline_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "cxline".to_string());

    let home = dirs::home_dir().expect("Cannot determine home directory");
    let mut changes: Vec<String> = Vec::new();

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
set -g status-left-length 120
set -g status-left '{status_right}'
set -g window-status-format ''
set -g window-status-current-format ''
set -g status-right ''
set -g status-right-length 0
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
            changes.push("~/.tmux.conf (updated cxline block)".into());
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
            changes.push("~/.tmux.conf (appended cxline config)".into());
        }
    } else {
        std::fs::write(&tmux_conf, format!("{}\n", tmux_block))
            .expect("Failed to create ~/.tmux.conf");
        changes.push("~/.tmux.conf (created)".into());
    }

    // --- 2. Add codex wrapper function to shell rc ---
    let func_marker = "# cxline-codex-wrapper";
    let func_marker_end = "# cxline-codex-wrapper-end";
    // Shell function: if not in tmux, launch tmux and run codex inside;
    // if already in tmux, just run codex directly.
    let func_block = format!(
        r#"{marker}
codex() {{
  if [ -z "$TMUX" ]; then
    tmux new-session -A -s codex "command codex $*"
  else
    command codex "$@"
  fi
}}
{marker_end}"#,
        marker = func_marker,
        marker_end = func_marker_end,
    );

    // Detect shell rc file
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc_file = if shell.contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    let rc_name = if shell.contains("zsh") { "~/.zshrc" } else { "~/.bashrc" };

    if rc_file.exists() {
        let content = std::fs::read_to_string(&rc_file).unwrap_or_default();
        if content.contains(func_marker) {
            // Replace existing block
            let mut new_content = String::new();
            let mut skipping = false;
            for line in content.lines() {
                if line.trim() == func_marker {
                    skipping = true;
                    continue;
                }
                if line.trim() == func_marker_end {
                    skipping = false;
                    continue;
                }
                if !skipping {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            new_content.push_str(&func_block);
            new_content.push('\n');
            std::fs::write(&rc_file, new_content)
                .unwrap_or_else(|_| panic!("Failed to write {}", rc_file.display()));
            changes.push(format!("{} (updated codex wrapper)", rc_name));
        } else {
            // Append
            let mut content = content;
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push('\n');
            content.push_str(&func_block);
            content.push('\n');
            std::fs::write(&rc_file, content)
                .unwrap_or_else(|_| panic!("Failed to write {}", rc_file.display()));
            changes.push(format!("{} (added codex wrapper)", rc_name));
        }
    } else {
        std::fs::write(&rc_file, format!("{}\n", func_block))
            .unwrap_or_else(|_| panic!("Failed to create {}", rc_file.display()));
        changes.push(format!("{} (created with codex wrapper)", rc_name));
    }

    // --- 3. Remove old cx alias if present ---
    if rc_file.exists() {
        let content = std::fs::read_to_string(&rc_file).unwrap_or_default();
        if content.contains("alias cx='tmux new-session") {
            let new_content: String = content
                .lines()
                .filter(|l| !l.contains("alias cx='tmux new-session") && l.trim() != "# cxline: codex+tmux shortcut")
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(&rc_file, format!("{}\n", new_content))
                .unwrap_or_else(|_| panic!("Failed to write {}", rc_file.display()));
            changes.push(format!("{} (removed old cx alias)", rc_name));
        }
    }

    // --- 4. If inside tmux, apply immediately ---
    if std::env::var("TMUX").is_ok() {
        let _ = std::process::Command::new("tmux")
            .args(["source-file", &tmux_conf.display().to_string()])
            .output();
        changes.push("tmux config reloaded (live)".to_string());
    }

    // --- Output ---
    println!("cxline setup complete!");
    println!();
    for c in &changes {
        println!("  [+] {}", c);
    }
    println!();
    println!("Usage: just type 'codex' as usual, status bar auto-appears at bottom.");
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
        --title                Output status to terminal title bar (for Windows)
    -h, --help                 Show this help
    -V, --version              Show version

MODULES:
    model, tokens, cost, timer, cwd, git, permission, turns

EXAMPLES:
    cxline setup                          # One-click setup, then just use 'codex'
    cxline show                           # Show latest session summary
    echo '{{"model":"o3-mini"}}' | cxline   # Legacy stdin mode"#
    );
}
