# Homebrew Tap

[English](#english) · [中文](#中文)

---

## English

Personal Homebrew tap for CLI tools by [@Erchoc](https://github.com/Erchoc).

### Usage

```bash
brew tap erchoc/tap
brew install <formula>
```

Or install directly without tapping first:

```bash
brew install erchoc/tap/<formula>
```

### Available Formulae

| Formula | Description | Source |
|---------|-------------|--------|
| [cb](Formula/cb.rb) | Cross-platform voice assistant for the terminal | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) |

### Updating

```bash
brew update
brew upgrade <formula>
```

### Contributing

Each formula downloads a pre-built binary from the source project's GitHub Releases.
No source code lives in this repo — only Formula definitions under `Formula/`.

To publish a new release of an existing formula, bump `version` and update the
`sha256` checksums in the `.rb` file.

---

## 中文

[@Erchoc](https://github.com/Erchoc) 的个人 Homebrew tap，用来分发命令行工具的预编译二进制。

### 使用方法

```bash
brew tap erchoc/tap
brew install <formula>
```

或者直接安装（免 tap）：

```bash
brew install erchoc/tap/<formula>
```

### 可用公式

| 公式 | 说明 | 源仓库 |
|------|------|--------|
| [cb](Formula/cb.rb) | 住在终端里的跨平台语音助手 | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) |

### 更新

```bash
brew update
brew upgrade <formula>
```

### 贡献

每个 formula 都从对应源项目的 GitHub Releases 下载预编译二进制，仓库内只保存
Formula 定义（`Formula/` 目录），不放源码。

要发布已有 formula 的新版本，只需在对应 `.rb` 文件里更新 `version` 和各平台的
`sha256` 校验值。
