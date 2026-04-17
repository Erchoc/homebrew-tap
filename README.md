# Homebrew Tap

[English](README.md) · [中文](README_CN.md)

Personal Homebrew tap for CLI tools by [@Erchoc](https://github.com/Erchoc).

## Usage

```bash
brew tap erchoc/tap
brew install <formula>
```

Or install directly without tapping first:

```bash
brew install erchoc/tap/<formula>
```

## Available Formulae

| Formula | Description | Source |
|---------|-------------|--------|
| [cb](Formula/cb.rb) | Cross-platform voice assistant for the terminal | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) |

## Updating

```bash
brew update
brew upgrade <formula>
```

## Contributing

Each formula downloads a pre-built binary from the source project's GitHub Releases.
No source code lives in this repo — only Formula definitions under `Formula/`.

To publish a new release of an existing formula, bump `version` and update the
`sha256` checksums in the `.rb` file.
