# homebrew-tap — 个人 Rust CLI 工具集

## 项目概述

个人 Homebrew Tap 仓库，使用 `packages/<name>/` 管理各工具。所有工具均为 Rust CLI，macOS 平台。

安装方式：`brew install erchoc/tap/<工具名>`

## 仓库结构

```
homebrew-tap/
├── Formula/<name>.rb              ← Homebrew formula（release 自动更新）
├── packages/<name>/               ← 各工具源码
│   ├── Cargo.toml
│   ├── src/
│   └── build-release.sh
└── .github/workflows/
    ├── ci.yml                     ← push 时 check/test/clippy
    └── release.yml                ← tag 推送时构建 + 发布 + 更新 Formula
```

## 工具清单

### vvpn — Stash VPN 诊断工具

路径：`packages/vvpn/`

扫描 macOS 上 Stash VPN 客户端的日志和配置，自动诊断问题并生成 HTML 报告。

架构（4 个模块流水线）：

```
main.rs (CLI 入口)
  → log_parser.rs    解析 Stash/Core/logs/*.log 和 crashes/*.log
  → config_parser.rs 解析 Stash/Core/config.yaml (serde_yaml)
  → diagnosis.rs     规则引擎：LogReport + ConfigReport → Vec<Issue>
  → report.rs        HTML 模板渲染（内联 CSS，无外部依赖）
```

关键路径：
- Stash 数据目录：`~/Library/Application Support/Stash/Core`
- 报告输出：`~/Downloads/vvpn_report.html`（debug 模式输出到 `packages/vvpn/output/`）

添加新诊断规则：
1. 在 `log_parser.rs` 的 `normalize_error()` 中添加新的错误模式匹配
2. 在 `diagnosis.rs` 的 `diagnose()` 中添加对应的 Issue 生成逻辑
3. Issue 必须包含：severity、title、description、solution、count
4. severity 排序：Critical > Warning > Info，同级按 count 降序

## 编码规范

- Rust 2024 edition
- 中文用户界面（CLI 输出、诊断描述、HTML 报告全部中文）
- 每个工具独立 Cargo.toml，不使用 workspace
- HTML 报告使用内联 CSS，深色主题，响应式布局
- 所有用户输入通过 `html_escape()` 转义后才能插入 HTML

## 构建与运行

```bash
cd packages/vvpn
cargo run -- scan          # debug 模式
cargo test                 # 运行测试
./build-release.sh         # 构建 macOS 通用二进制
```

## 发版

```bash
# 1. 修改 packages/<name>/Cargo.toml 中的 version
# 2. 打 tag 并推送
git tag v0.1.0             # vvpn 用 v* 格式，未来其他工具用 <name>-v* 格式
git push origin v0.1.0     # 触发 CI 构建 + GitHub Release + 自动更新 Formula
```
