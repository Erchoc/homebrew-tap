# Homebrew Tap · npm channel

[English](README.md) · [中文](README_CN.md)

Personal distribution repo for CLI tools by [@Erchoc](https://github.com/Erchoc).
Every tool ships through two equivalent channels: Homebrew and npm. Both
deliver the same pre-built binary from the upstream project's GitHub Release.

## Install via Homebrew

```bash
brew tap erchoc/tap
brew install <formula>
```

Or without tapping first:

```bash
brew install erchoc/tap/<formula>
```

## Install via npm

```bash
npm install -g @erchoc/<tool>
```

> The leading `@` is required. `npm install -g erchoc/<tool>` (without `@`)
> is npm's GitHub shorthand and will try to clone `github.com/erchoc/<tool>`
> instead — that is **not** the same thing.

No `postinstall` download, no network at install time, works under
`--ignore-scripts`. Supports macOS (arm64 + x64) and Linux (x64 + arm64).

## Available tools

| Tool | Description | Source | Homebrew | npm |
|------|-------------|--------|----------|-----|
| [cb](Formula/cb.rb) | Cross-platform voice assistant for the terminal | [Erchoc/chatbot](https://github.com/Erchoc/chatbot) | `brew install erchoc/tap/cb` | `npm install -g @erchoc/cb` |

## Updating

```bash
brew update && brew upgrade <formula>     # Homebrew
npm update -g @erchoc/<tool>              # npm
```

## Repo layout

```
homebrew-tap/
├── Formula/                     # one .rb per tool (Homebrew side)
├── npm/
│   ├── <tool>/                  # one subdir per tool (npm side)
│   │   ├── bin/<tool>.js        # Node launcher shim
│   │   ├── package.template.json
│   │   ├── README.md
│   │   └── LICENSE
│   └── scripts/
│       └── build-npm-package.mjs  # reads Formula, downloads + verifies binaries
├── templates/npm/tool-template/ # starter kit for self-publishing projects
├── docs/npm-convention.md       # full spec for @<org>/<tool> packaging
└── .github/workflows/
    ├── test.yml                 # brew style/audit/install + npm build dry-run
    └── publish-npm.yml          # tag-triggered npm publish
```

## Releasing a new version

The Homebrew formula is the source of truth for `version` and each
per-platform `sha256`. Releasing both channels is therefore:

1. Bump `version` and each `sha256` in `Formula/<tool>.rb`, commit.
2. Tag `npm-<tool>-v<version>` and push — the `publish-npm` workflow
   downloads the matching binaries, verifies their sha256 against the
   formula, and publishes `@erchoc/<tool>` to npmjs.org.

## Verifying an install

```bash
# Homebrew
which cb                          # → $(brew --prefix)/bin/cb
cb --version

# npm
npm ls -g --depth=0 @erchoc/cb    # confirm the scoped package is installed
which cb                          # → <npm-prefix>/bin/cb
cb --version

# Both channels deliver the same binary from the upstream GitHub Release.
# Version strings are identical by construction.
```

## Self-publishing from your own repo

You don't need to be hosted here to use the npm packaging convention. Copy
`templates/npm/tool-template/` into your own repo and follow
[docs/npm-convention.md](docs/npm-convention.md). Works with any source of
truth for version + binary hashes (Homebrew formula, GitHub Release API,
static manifest, etc.).
