use std::fs;
use std::io::{self, BufRead, Seek};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{EventKind, RecursiveMode, Watcher};

use crate::config;
use crate::formatter;
use crate::modules;
use crate::parser::{self, CodexSession};
use crate::style;
use crate::theme;

/// Find the most recent rollout JSONL file under ~/.codex/sessions/
pub fn find_latest_rollout() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let sessions_dir = home.join(".codex").join("sessions");
    if !sessions_dir.exists() {
        return None;
    }
    find_newest_jsonl(&sessions_dir)
}

/// Recursively find the newest .jsonl file in a directory tree
fn find_newest_jsonl(dir: &std::path::Path) -> Option<PathBuf> {
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

    fn walk(dir: &std::path::Path, newest: &mut Option<(PathBuf, std::time::SystemTime)>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, newest);
            } else if path.extension().is_some_and(|e| e == "jsonl") {
                if let Ok(meta) = path.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
                            *newest = Some((path, modified));
                        }
                    }
                }
            }
        }
    }

    walk(dir, &mut newest);
    newest.map(|(p, _)| p)
}

/// Watch a JSONL file and render status on each change
pub fn watch_session(path: Option<PathBuf>, config: config::Config, title_mode: bool) -> io::Result<()> {
    let file_path = match path {
        Some(p) => p,
        None => find_latest_rollout().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No Codex session files found in ~/.codex/sessions/",
            )
        })?,
    };

    eprintln!(
        "Watching: {}",
        file_path.display()
    );

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

    let mut session = CodexSession::new();

    // Read existing content first
    let mut file = fs::File::open(&file_path)?;
    let mut reader = io::BufReader::new(&file);
    let mut line_buf = String::new();
    while reader.read_line(&mut line_buf)? > 0 {
        if let Some(event) = parser::parse_codex_line(line_buf.trim()) {
            session.apply_event(&event);
        }
        line_buf.clear();
    }

    // Render initial state
    render_status(&session, &mods, &theme, title_mode);

    // Remember current file position
    let mut pos = file.stream_position()?;

    // Set up file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                let _ = tx.send(());
            }
        }
    })
    .map_err(io::Error::other)?;

    // Watch the parent directory (some editors write temp files then rename)
    let watch_dir = file_path.parent().unwrap_or(std::path::Path::new("."));
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .map_err(io::Error::other)?;

    // Event loop
    loop {
        // Wait for file change notification (with timeout for periodic refresh)
        let _ = rx.recv_timeout(Duration::from_secs(2));

        // Re-open file to handle rotation/truncation
        file = match fs::File::open(&file_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if file_len < pos {
            // File was truncated, re-read from start
            pos = 0;
            session = CodexSession::new();
        }

        file.seek(std::io::SeekFrom::Start(pos))?;
        reader = io::BufReader::new(&file);

        line_buf.clear();
        while reader.read_line(&mut line_buf)? > 0 {
            let trimmed = line_buf.trim();
            if !trimmed.is_empty() {
                if let Some(event) = parser::parse_codex_line(trimmed) {
                    session.apply_event(&event);
                }
            }
            line_buf.clear();
        }

        pos = file_len;

        // Always re-render: timer updates even when no new events arrive
        render_status(&session, &mods, &theme, title_mode);
    }
}

/// Show summary of the most recent (or specified) session
pub fn show_session(path: Option<PathBuf>, config: config::Config) -> io::Result<()> {
    let file_path = match path {
        Some(p) => p,
        None => find_latest_rollout().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No Codex session files found in ~/.codex/sessions/",
            )
        })?,
    };

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

    let mut session = CodexSession::new();

    let content = fs::read_to_string(&file_path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if let Some(event) = parser::parse_codex_line(trimmed) {
                session.apply_event(&event);
            }
        }
    }

    let data = session.to_session_data();
    // Use large width to avoid truncation (tmux manages its own width)
    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or(200);
    let output = formatter::format_statusline(&mods, &data, &theme, width);

    if !output.is_empty() {
        println!("{}", output);
    }

    // Only print session path in interactive use (not when called from tmux status bar)
    if std::env::var("CXLINE_QUIET").is_err() {
        eprintln!("Session: {}", file_path.display());
    }

    Ok(())
}

fn render_status(
    session: &CodexSession,
    mods: &[Box<dyn modules::Module>],
    theme: &theme::Theme,
    title_mode: bool,
) {
    let data = session.to_session_data();
    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or(80);
    let output = formatter::format_statusline(mods, &data, theme, width);

    if title_mode {
        // Write to terminal title bar using OSC escape sequence
        let plain = style::strip_ansi(&output);
        if !plain.is_empty() {
            print!("\x1b]0;{}\x07", plain);
        }
    } else {
        // Clear line and redraw
        eprint!("\x1b[2K\r");
        if !output.is_empty() {
            print!("{}", output);
        }
    }
    // Flush stdout
    use std::io::Write;
    let _ = io::stdout().flush();
}
