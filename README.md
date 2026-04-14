# vvpn

Stash VPN 环境诊断工具 — 扫描 macOS 上 [Stash](https://stash.ws) 的日志与配置，自动分析问题并生成可视化 HTML 诊断报告。

## 安装

### Homebrew（推荐）

```bash
brew install erchoc/vvpn/vvpn
```

### 手动安装

从 [Releases](https://github.com/Erchoc/homebrew-vvpn/releases) 下载 macOS 通用二进制，放到 `$PATH` 中即可。

## 使用

```bash
vvpn scan                  # 扫描全部日志并生成报告
vvpn scan --today          # 仅分析今天的日志
vvpn scan --week           # 仅分析最近 7 天
vvpn scan --month          # 仅分析最近 30 天
vvpn open                  # 在浏览器中打开诊断报告
vvpn cd                    # 进入 Stash 数据目录
vvpn clean                 # 清除 Stash 日志
```

| 模式 | 报告输出路径 |
|------|-------------|
| 开发 (`cargo run`) | `./output/vvpn_report.html` |
| 发布 (release 二进制) | `~/Downloads/vvpn_report.html` |

## 功能

- 解析 Stash Core 日志，提取错误/警告模式（StashLink 崩溃、NAT 检测失败、代理组无可用节点等）
- 解析 `config.yaml`，统计节点、代理组、分流规则、DNS、端口等配置信息
- 自动诊断常见问题，按严重程度（严重 / 警告 / 提示）排序
- 生成深色主题 HTML 报告，包含 Top 问题、错误分布图表、节点地区/线路分布等

## 开发

```bash
cargo run -- scan          # debug 模式，报告输出到 ./output/
cargo run -- open          # 打开报告
./build-release.sh         # 构建 macOS 通用二进制到 release/
```

## 项目结构

```
src/
├── main.rs           # CLI 入口，子命令分发
├── log_parser.rs     # 日志解析（core logs + crash logs）
├── config_parser.rs  # YAML 配置解析（节点、代理组、规则等）
├── diagnosis.rs      # 规则引擎，根据日志和配置生成问题列表
└── report.rs         # HTML 报告渲染（深色主题，响应式布局）
```

## License

MIT
