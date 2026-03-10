# cxline

[中文文档](README_CN.md)

A lightweight, blazing-fast session monitor for [OpenAI Codex CLI](https://github.com/openai/codex).

After install, just type `codex` as usual — a real-time status bar auto-appears at the bottom of your terminal.

```
🤖 gpt-5.3-codex │ 📊 11.1k/258.4k ░░░░░░░░░░ 10.3kin (4.1kcached) 836out 622reason │ 🔄 Turn 3 │ ⏱️  2m15s │ 🔒 on-request
```

## Install

```bash
git clone https://github.com/Subaru486desuwa/cxline.git
cd cxline
./install.sh
```

`install.sh` does everything:
1. Checks/installs tmux
2. `cargo install` builds the binary
3. Configures `~/.tmux.conf` for the status bar
4. Wraps `codex` in your shell rc so tmux auto-launches

**Open a new terminal, type `codex` — done.**

## What it shows

| Module | Example | Description |
|--------|---------|-------------|
| `model` | `🤖 gpt-5.3-codex` | Current model name |
| `tokens` | `📊 11.1k/258.4k` | Token usage + context progress bar |
| | `10.3kin (4.1kcached) 836out 622reason` | Detailed breakdown |
| `turns` | `🔄 Turn 3` | Current turn number |
| `cost` | `💰 $0.42` | Session cost (if available) |
| `timer` | `⏱️  2m15s` | Session elapsed time |
| `git` | `🌿 main` | Git branch |
| `permission` | `🔒 on-request` | Approval policy |

## Other commands

```bash
cxline show                             # One-shot summary of latest session
cxline show -s path/to/rollout.jsonl    # Summary of specific session
cxline watch                            # Live watch mode (standalone terminal)
echo '{"model":"o3"}' | cxline          # Legacy stdin pipe mode
```

## Configuration

`~/.config/cxline/config.toml`:

```toml
theme = "default"           # default / minimal / powerline
separator = " │ "
modules = ["model", "tokens", "turns", "cost", "timer", "git", "permission"]

[tokens]
show_bar = true
bar_width = 10

[cost]
currency = "USD"            # USD / CNY
```

## Themes

- **default** — Emoji icons, modern terminals
- **minimal** — Plain text, no icons
- **powerline** — Nerd Font icons + Powerline separators

## Requirements

- Rust (cargo)
- tmux
- macOS / Linux / Windows

## License

MIT
