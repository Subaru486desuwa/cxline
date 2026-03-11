# Changelog

## 2026-03-11

### Fixed

- **Token 百分比口径修正**: 优先使用 `last_token_usage`（当前轮上下文占用）计算百分比，而非 `total_token_usage`（会话累计），后者与 `model_context_window` 不是同一口径。旧日志无 `last_token_usage` 时自动 fallback 到 `total_token_usage`
- **Token 计数修正**: 新增 `total_tokens` 字段解析，优先使用；fallback 计算改为 `input + output`（之前误加 `reasoning`，而 `reasoning ⊂ output` 会导致重复计数）
- **百分比精度**: 从整数 (`4%`) 改为 < 10% 时保留一位小数 (`4.3%`)，避免早期使用量显示为 `0%`

### Added

- **cwd 模块**: 新增工作目录显示模块
- **tokens detail 配置**: 新增 `show_detail` 选项，可选显示 in/out/cache/reason 明细
- **时间戳解析**: 支持 ISO 8601 时间戳，用于计算会话经过时间

### Changed

- **permission 图标**: `on-request` 策略图标从锁 (🔒) 改为闪电 (⚡)
- **默认配置**: `show_bar` 默认关闭，模块列表新增 `cwd`
