# cxline

A lightweight, blazing-fast session monitor for [OpenAI Codex CLI](https://github.com/openai/codex).

Reads Codex session logs (`~/.codex/sessions/`) and displays a real-time status bar in your tmux bottom bar — model, tokens, turns, timer, permission, all auto-updating.

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
4. Adds `cx` alias to your `~/.zshrc` (or `~/.bashrc`)

## Usage

```bash
cx        # start tmux with cxline status bar
codex     # use Codex as usual, bottom bar auto-updates
```

That's it. The tmux bottom bar refreshes every 2 seconds with the latest Codex session data.

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
cxline show                             # one-shot summary of latest session
cxline show -s path/to/rollout.jsonl    # summary of specific session
cxline watch                            # live watch mode (standalone terminal)
echo '{"model":"o3"}' | cxline          # legacy stdin pipe mode
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

```bash
cxline show --theme minimal      # plain text, no icons
cxline show --theme powerline    # nerd font icons
```

## Requirements

- Rust (cargo)
- tmux
- macOS / Linux / Windows (cross-platform)

## License

MIT
