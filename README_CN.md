# Homebrew Tap · npm 分发

[English](README.md) · [中文](README_CN.md)

[@Erchoc](https://github.com/Erchoc) 的个人命令行工具分发仓库。每个工具都通过
Homebrew 与 npm 两条等价通道分发，两条通道交付同一份来自上游 GitHub
Releases 的预编译二进制。

## 使用 Homebrew 安装

```bash
brew tap erchoc/tap
brew install <formula>
```

或者免 tap 直接安装：

```bash
brew install erchoc/tap/<formula>
```

## 使用 npm 安装

```bash
npm install -g @erchoc/<tool>
```

> **`@` 前缀必须加**。`npm install -g erchoc/<tool>`（没有 `@`）是 npm 的
> GitHub shorthand，会尝试 clone `github.com/erchoc/<tool>`——和这里的 npm
> 包完全是两回事。

安装过程**不执行 postinstall、不联网下载**，兼容 `--ignore-scripts`。支持
macOS（arm64 + x64）与 Linux（x64 + arm64）。

## 可用工具

| 工具 | 说明 | 源仓库 | Homebrew | npm |
|------|------|--------|----------|-----|
| [cb](Formula/cb.rb) | 住在终端里的跨平台语音助手 | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) | `brew install erchoc/tap/cb` | `npm install -g @erchoc/cb` |

## 更新

```bash
brew update && brew upgrade <formula>     # Homebrew
npm update -g @erchoc/<tool>              # npm
```

## 仓库结构

```
homebrew-tap/
├── Formula/                     # Homebrew 公式（每个工具一份 .rb）
├── npm/
│   ├── <tool>/                  # npm 侧每工具一个子目录
│   │   ├── bin/<tool>.js        # Node 启动垫片
│   │   ├── package.template.json
│   │   ├── README.md
│   │   └── LICENSE
│   └── scripts/
│       └── build-npm-package.mjs  # 读取 Formula、下载并校验二进制
├── templates/npm/tool-template/ # 其他项目自发布时的起点模板
├── docs/npm-convention.md       # @<org>/<tool> 打包规范全文
└── .github/workflows/
    ├── test.yml                 # brew style/audit/install + npm 构建干跑
    └── publish-npm.yml          # 打 tag 触发 npm 发布
```

## 发新版

Homebrew 公式是 `version` 与各平台 `sha256` 的唯一事实来源。发布流程：

1. 修改 `Formula/<tool>.rb` 里的 `version` 与各 `sha256`，提交。
2. 打 tag `npm-<tool>-v<version>` 并 push——`publish-npm` workflow 会下载
   对应二进制、比对 sha256 与 formula 一致，然后发布 `@erchoc/<tool>`。

## 安装后自检

```bash
# Homebrew
which cb                          # → $(brew --prefix)/bin/cb
cb --version

# npm
npm ls -g --depth=0 @erchoc/cb    # 确认 scoped 包已装
which cb                          # → <npm-prefix>/bin/cb
cb --version

# 两条通道交付的是同一份上游二进制，版本号按设计逐字符一致。
```

## 在自己的仓库自发布

本仓库不是必经路径。把 `templates/npm/tool-template/` 拷到自己仓库，按
[docs/npm-convention.md](docs/npm-convention.md) 落地即可。事实来源不一定是
Homebrew 公式——GitHub Release API、静态 manifest 都行，只要能提供版本号和
每份二进制的 sha256。
