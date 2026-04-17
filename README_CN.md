# Homebrew Tap

[English](README.md) · [中文](README_CN.md)

[@Erchoc](https://github.com/Erchoc) 的个人 Homebrew tap，用来分发命令行工具的预编译二进制。

## 使用方法

```bash
brew tap erchoc/tap
brew install <formula>
```

或者直接安装（免 tap）：

```bash
brew install erchoc/tap/<formula>
```

## 可用公式

| 公式 | 说明 | 源仓库 |
|------|------|--------|
| [cb](Formula/cb.rb) | 住在终端里的跨平台语音助手 | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) |

## 更新

```bash
brew update
brew upgrade <formula>
```

## 贡献

每个 formula 都从对应源项目的 GitHub Releases 下载预编译二进制，仓库内只保存
Formula 定义（`Formula/` 目录），不放源码。

要发布已有 formula 的新版本，只需在对应 `.rb` 文件里更新 `version` 和各平台的
`sha256` 校验值。
