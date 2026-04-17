# Homebrew Tap · npm channel

[English](README.md) · [中文](README_CN.md)

Dual-channel distribution hub for [@Erchoc](https://github.com/Erchoc)'s CLI
tools. Every tool ships through two equivalent channels:

```bash
brew install erchoc/tap/<tool>    # Homebrew
npm  install -g @erchoc/<tool>    # npm
```

Both deliver the exact same pre-built binary from the tool's own GitHub
Release.

## What this repo provides

- **Homebrew formulae** under `Formula/` — one `.rb` per tool.
- **Reusable GitHub Actions workflow** at
  [`.github/workflows/publish-npm-reusable.yml`](.github/workflows/publish-npm-reusable.yml) —
  every tool repo calls this to publish its own `@erchoc/<tool>` to npm. No
  per-tool npm packaging lives here.
- **Shared build script** at
  [`npm/scripts/build-npm-package-from-release.mjs`](npm/scripts/build-npm-package-from-release.mjs)
  (invoked by the reusable workflow).
- **Packaging spec** at [`docs/npm-convention.md`](docs/npm-convention.md).
- **Starter template** at
  [`templates/npm/tool-template/`](templates/npm/tool-template/).

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

## Repo layout

```
homebrew-tap/
├── Formula/                                # one .rb per tool (Homebrew side)
├── npm/
│   └── scripts/
│       └── build-npm-package-from-release.mjs
├── templates/npm/tool-template/            # starter kit for tool repos
│   ├── bin/__TOOL__.js
│   ├── package.template.json
│   ├── .github/workflows/publish-npm.yml
│   └── .gitignore
├── docs/npm-convention.md                  # @<org>/<tool> packaging spec
└── .github/workflows/
    ├── test.yml                            # brew style/audit/install
    └── publish-npm-reusable.yml            # called by tool repos
```

## Adding a new tool (publishing flow)

For each new CLI tool in the `@erchoc` family, wire up both channels:

### 1. Homebrew side (this repo)

Add `Formula/<tool>.rb` pointing at the tool's GitHub Release assets. Bump
`version` + per-platform `sha256` on each release.

### 2. npm side (the tool's own source repo)

Copy [`templates/npm/tool-template/`](templates/npm/tool-template/) into
the tool's repo as `npm/`. Follow
[`docs/npm-convention.md`](docs/npm-convention.md). The tool repo's
`publish-npm.yml` calls this repo's reusable workflow — one-line upgrade for
every tool whenever the shared workflow improves.

Prereqs per tool repo:
- `NPM_TOKEN` secret (granular Automation token, write access to `@erchoc`
  scope). An **organisation** secret shared across all Erchoc repos is
  recommended.
- Release workflow uploads assets whose filenames match the recognised
  platform patterns (see spec §7).

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
```
