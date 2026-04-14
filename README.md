# vvpn

Stash VPN 环境诊断工具 —— 扫描 macOS 上 [Stash](https://stash.ws) 的日志与配置，自动分析问题并生成可视化 HTML 诊断报告。

## 功能

- 解析 Stash Core 日志，提取错误/警告模式（StashLink 崩溃、NAT 检测失败、代理组无可用节点等）
- 解析 `config.yaml`，统计节点、代理组、分流规则、DNS、端口等配置信息
- 自动诊断常见问题，按严重程度（严重 / 警告 / 提示）排序
- 生成深色主题 HTML 报告，包含 Top 3 问题、错误分布图表、节点地区/线路分布等

## 开发

```bash
# debug 构建 + 运行，报告输出到 ./output/
cargo run -- scan
cargo run -- open
```

## 构建发布

```bash
# 生成 macOS 通用二进制 (x86_64 + aarch64) 到 release/
./build-release.sh

# 或安装到系统
cargo install --path .
```

## 使用

```bash
vvpn scan    # 扫描 Stash 目录，生成诊断报告
vvpn open    # 在浏览器中打开报告
```

| 模式 | 报告输出路径 |
|------|-------------|
| 开发 (`cargo run`) | `./output/vvpn_report.html` |
| 发布 (release 二进制) | `~/Downloads/vvpn_report.html` |

Stash 数据目录：`~/Library/Application Support/Stash/Core`

## 项目结构

```
src/
├── main.rs           # CLI 入口，子命令分发
├── log_parser.rs     # 日志解析（core logs + crash logs）
├── config_parser.rs  # YAML 配置解析（节点、代理组、规则等）
├── diagnosis.rs      # 规则引擎，根据日志和配置生成问题列表
└── report.rs         # HTML 报告渲染（深色主题，响应式布局）
```

## 依赖

- `regex` — 日志行匹配
- `chrono` — 报告时间戳
- `serde` + `serde_yaml` — Stash 配置文件解析

## License

MIT
