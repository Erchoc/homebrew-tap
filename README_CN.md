# Homebrew Tap · npm 分发

[English](README.md) · [中文](README_CN.md)

[@Erchoc](https://github.com/Erchoc) 命令行工具的双通道分发中心。每个工具都通过两条等价通道分发：

```bash
brew install erchoc/tap/<tool>    # Homebrew
npm  install -g @erchoc/<tool>    # npm
```

两条通道交付的是同一份来自工具 GitHub Release 的预编译二进制。

## 本仓库提供什么

- **Homebrew 公式**（`Formula/` 下，每个工具一份 `.rb`）。
- **可复用 GitHub Actions workflow**
  [`.github/workflows/publish-npm-reusable.yml`](.github/workflows/publish-npm-reusable.yml)——每个工具仓库调用它来发布自己的
  `@erchoc/<tool>`。**本仓库不再集中维护任何工具的 npm 打包**。
- **共享构建脚本**
  [`npm/scripts/build-npm-package-from-release.mjs`](npm/scripts/build-npm-package-from-release.mjs)（被
  reusable workflow 调用）。
- **打包规范** [`docs/npm-convention.md`](docs/npm-convention.md)。
- **起点模板** [`templates/npm/tool-template/`](templates/npm/tool-template/)。

## 使用 Homebrew 安装

```bash
brew tap erchoc/tap
brew install <formula>
```

或免 tap 直接安装：

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

## 仓库结构

```
homebrew-tap/
├── Formula/                                # Homebrew 公式
├── npm/
│   └── scripts/
│       └── build-npm-package-from-release.mjs
├── templates/npm/tool-template/            # 工具仓库的起点模板
│   ├── bin/__TOOL__.js
│   ├── package.template.json
│   ├── .github/workflows/publish-npm.yml
│   └── .gitignore
├── docs/npm-convention.md                  # @<org>/<tool> 打包规范
└── .github/workflows/
    ├── test.yml                            # brew style/audit/install
    └── publish-npm-reusable.yml            # 被工具仓库调用
```

## 新增一个工具（发布流程）

每个新加入 `@erchoc` 家族的 CLI 都要接通两条通道：

### 1. Homebrew 侧（本仓库）

加 `Formula/<tool>.rb`，指向工具 GitHub Release 的资产。每次发版更新
`version` 与各平台 `sha256`。

### 2. npm 侧（**工具自己的源仓库**）

把 [`templates/npm/tool-template/`](templates/npm/tool-template/) 拷到工具
仓库里当 `npm/`，按 [`docs/npm-convention.md`](docs/npm-convention.md) 落地。
工具仓库的 `publish-npm.yml` 调用本仓库的 reusable workflow——共享 workflow
升级一次，所有工具一起受益。

每个工具仓库的前置条件：
- `NPM_TOKEN` secret（granular Automation token，对 `@erchoc` scope 有写权限）。
  推荐用 GitHub **组织级** secret，一次配置所有 Erchoc 仓库共享。
- release workflow 产出的资产文件名匹配约定的平台关键词（见规范 §7）。

## 安装后自检

```bash
# Homebrew
which cb                          # → $(brew --prefix)/bin/cb
cb --version

# npm
npm ls -g --depth=0 @erchoc/cb    # 确认 scoped 包已装
which cb                          # → <npm-prefix>/bin/cb
cb --version

# 两条通道交付的是同一份上游二进制。
```
