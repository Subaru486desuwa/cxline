# cxline

轻量级 [OpenAI Codex CLI](https://github.com/openai/codex) 会话监控工具。

安装后，输入 `codex` 即自动在终端底部显示实时状态栏 —— 模型、Token 用量、轮次、耗时、权限，全部自动刷新。

```
🤖 gpt-5.3-codex │ 📊 11.1k/258.4k ░░░░░░░░░░ 10.3kin (4.1kcached) 836out 622reason │ 🔄 Turn 3 │ ⏱️  2m15s │ 🔒 on-request
```

## 安装

```bash
git clone https://github.com/Subaru486desuwa/cxline.git
cd cxline
./install.sh
```

脚本自动完成：
1. 检查并安装 tmux
2. 编译安装 cxline
3. 配置 tmux 底部状态栏
4. 在 `~/.zshrc`（或 `~/.bashrc`）中注入 `codex` 包装函数

安装完成后，**新开终端，像平时一样输入 `codex` 就行**，底部状态栏自动出现。

## 工作原理

```
用户输入 codex
    ↓
shell 函数自动启动 tmux + codex
    ↓
tmux 底部状态栏每 2 秒调用 cxline show
    ↓
cxline 读取 ~/.codex/sessions/ 下最新的 JSONL 日志
    ↓
解析并渲染：模型 │ Token │ 轮次 │ 耗时 │ 权限
```

## 显示内容

| 模块 | 示例 | 说明 |
|------|------|------|
| `model` | `🤖 gpt-5.3-codex` | 当前模型 |
| `tokens` | `📊 11.1k/258.4k` | Token 使用量 / 上下文窗口 |
| | `10.3kin (4.1kcached) 836out 622reason` | 输入/缓存/输出/推理明细 |
| `turns` | `🔄 Turn 3` | 当前对话轮次 |
| `cost` | `💰 $0.42` | 会话费用（如有） |
| `timer` | `⏱️  2m15s` | 会话耗时 |
| `git` | `🌿 main` | Git 分支 |
| `permission` | `🔒 on-request` | 审批策略 |

## 其他命令

```bash
cxline show                             # 显示最近一次会话摘要
cxline show -s path/to/rollout.jsonl    # 显示指定会话
cxline watch                            # 独立终端实时监控模式
echo '{"model":"o3"}' | cxline          # 管道输入模式（兼容旧版）
```

## 自定义配置

创建 `~/.config/cxline/config.toml`：

```toml
theme = "default"           # default / minimal / powerline
separator = " │ "
modules = ["model", "tokens", "turns", "cost", "timer", "git", "permission"]

[tokens]
show_bar = true
bar_width = 10

[cost]
currency = "CNY"            # USD / CNY
```

## 主题

| 主题 | 说明 |
|------|------|
| `default` | Emoji 图标，适合现代终端 |
| `minimal` | 纯文本无图标，兼容性最好 |
| `powerline` | Nerd Font 图标 + Powerline 分隔符 |

## 卸载

```bash
cargo uninstall cxline
```

然后删除 `~/.tmux.conf` 中 `# cxline-managed` 到 `# cxline-managed-end` 之间的内容，以及 `~/.zshrc` 中 `# cxline-codex-wrapper` 到 `# cxline-codex-wrapper-end` 之间的内容。

## 环境要求

- Rust (cargo)
- tmux
- macOS / Linux / Windows

## License

MIT
