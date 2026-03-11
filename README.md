# cxline

[中文文档](README_CN.md)

A lightweight, blazing-fast session monitor for [OpenAI Codex CLI](https://github.com/openai/codex).

After install, just type `codex` as usual — a real-time status bar auto-appears at the bottom of your terminal.

```
🤖 gpt-5.3-codex │ 📊 11.1k/258.4k 4.3% │ 🔄 Turn 3 │ ⏱️  2m15s │ 📂 ~/project │ ⚡ on-request
```

## Platform Support

| Platform | Status display | Requires |
|----------|---------------|----------|
| **macOS** | tmux bottom status bar | tmux |
| **Linux** | tmux bottom status bar | tmux |
| **Windows** | Terminal title bar | PowerShell |

## Install

### macOS / Linux

```bash
git clone https://github.com/Subaru486desuwa/cxline.git
cd cxline
./install.sh
```

`install.sh` does everything:
1. Checks/installs tmux
2. `cargo install` builds the binary
3. Configures `~/.tmux.conf` for the status bar
4. Wraps `codex` in your shell rc (`~/.zshrc` or `~/.bashrc`) so tmux auto-launches

**Open a new terminal, type `codex` — done.**

### Windows

```powershell
git clone https://github.com/Subaru486desuwa/cxline.git
cd cxline
powershell -ExecutionPolicy Bypass -File install.ps1
```

`install.ps1` does everything:
1. `cargo install` builds the binary
2. Writes a `codex` wrapper into your PowerShell profile

**Restart PowerShell, type `codex` — status appears in the terminal title bar.**

> Windows doesn't have tmux, so cxline uses the terminal title bar to display status instead. You can also use this mode manually: `cxline watch --title`

## How it works

**macOS / Linux (tmux mode):**

```
You type: codex
    ↓
Shell wrapper launches tmux + codex
    ↓
tmux status bar calls "cxline show" every 2s
    ↓
cxline reads ~/.codex/sessions/*.jsonl
    ↓
Renders: model │ tokens │ turns │ timer │ cwd │ permission
```

**Windows (title mode):**

```
You type: codex
    ↓
PowerShell wrapper starts "cxline watch --title" in background
    ↓
cxline watches ~/.codex/sessions/*.jsonl in real-time
    ↓
Writes status to terminal title bar via OSC escape
    ↓
When codex exits, background process is cleaned up
```

## What it shows

| Module | Example | Description |
|--------|---------|-------------|
| `model` | `🤖 gpt-5.3-codex` | Current model name |
| `tokens` | `📊 11.1k/258.4k 4.3%` | Token usage / context window + percentage |
| `turns` | `🔄 Turn 3` | Current turn number |
| `cost` | `💰 $0.42` | Session cost (if available) |
| `timer` | `⏱️  2m15s` | Session elapsed time |
| `cwd` | `📂 ~/project` | Current working directory |
| `git` | `🌿 main` | Git branch |
| `permission` | `⚡ on-request` | Approval policy |

## Commands

```bash
cxline setup                            # One-click setup (auto-detects platform)
cxline show                             # One-shot summary of latest session
cxline show -s path/to/rollout.jsonl    # Summary of specific session
cxline watch                            # Live watch mode (stdout, for tmux)
cxline watch --title                    # Live watch mode (terminal title bar, for Windows)
echo '{"model":"o3"}' | cxline          # Legacy stdin pipe mode
```

## Configuration

`~/.config/cxline/config.toml`:

```toml
theme = "default"           # default / minimal / powerline
separator = " │ "
modules = ["model", "tokens", "turns", "cost", "timer", "cwd", "git", "permission"]

[tokens]
show_bar = false            # optional progress bar
show_detail = false         # optional in/out/cache/reason breakdown
# bar_width = 10

[cost]
currency = "USD"            # USD / CNY
```

## Themes

- **default** — Emoji icons, modern terminals
- **minimal** — Plain text, no icons
- **powerline** — Nerd Font icons + Powerline separators

## Uninstall

```bash
cargo uninstall cxline
```

Then remove the injected config blocks:
- **macOS/Linux**: Delete `# cxline-managed` block in `~/.tmux.conf` and `# cxline-codex-wrapper` block in `~/.zshrc` or `~/.bashrc`
- **Windows**: Delete `# cxline-codex-wrapper` block in your PowerShell profile (`$PROFILE`)

## Requirements

- [Rust](https://rustup.rs/) (cargo)
- tmux (macOS / Linux only)
- macOS / Linux / Windows

## License

MIT
