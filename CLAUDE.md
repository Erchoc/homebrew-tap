# vvpn - Stash VPN 诊断工具

## 项目概述

Rust CLI 工具，扫描 macOS 上 Stash VPN 客户端的日志和配置，自动诊断问题并生成 HTML 报告。

## 架构

单二进制 CLI，4 个模块流水线：

```
main.rs (CLI 入口)
  → log_parser.rs    解析 Stash/Core/logs/*.log 和 crashes/*.log
  → config_parser.rs 解析 Stash/Core/config.yaml (serde_yaml)
  → diagnosis.rs     规则引擎：LogReport + ConfigReport → Vec<Issue>
  → report.rs        HTML 模板渲染（内联 CSS，无外部依赖）
```

数据流向：`scan_logs() + parse_config() → diagnose() → render() → HTML 文件`

## 关键路径

- Stash 数据目录：`~/Library/Application Support/Stash/Core`
- 报告输出：`~/Downloads/vvpn_report.html`

## 编码规范

- Rust 2024 edition
- 中文用户界面（CLI 输出、诊断描述、HTML 报告全部中文）
- 错误模式用 `normalize_error()` 统一归类，新增错误类型需同时更新 `log_parser.rs` 和 `diagnosis.rs`
- HTML 报告使用内联 CSS（`report.rs` 底部 `const CSS`），深色主题，响应式布局
- 所有用户输入通过 `html_escape()` 转义后才能插入 HTML

## 构建与运行

```bash
cargo run -- scan          # debug 模式，报告输出到 ./output/
cargo run -- open          # 打开报告
./build-release.sh         # 构建 macOS 通用二进制到 release/
```

- debug 模式 (`cfg!(debug_assertions)`)：报告输出到项目下 `output/` 目录
- release 模式：报告输出到 `~/Downloads/vvpn_report.html`

## 添加新诊断规则的步骤

1. 在 `log_parser.rs` 的 `normalize_error()` 中添加新的错误模式匹配
2. 在 `diagnosis.rs` 的 `diagnose()` 中添加对应的 Issue 生成逻辑
3. Issue 必须包含：severity、title、description、solution、count
4. severity 排序：Critical > Warning > Info，同级按 count 降序
