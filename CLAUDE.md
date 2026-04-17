# homebrew-tap

Dual-channel distribution **hub** for pre-built CLI binaries. Ships each tool
through Homebrew (`brew install erchoc/tap/<tool>`) and npm
(`npm install -g @erchoc/<tool>`).

This repo is NOT where individual tools' npm packages are built — those live
in each tool's own source repo. Here we host the Homebrew formulae, the
reusable GitHub Actions workflow that every tool repo calls, the shared
build script, and the packaging spec + template.

## Structure

```
homebrew-tap/
├── Formula/                                     ← Homebrew formulae (one .rb per tool)
├── npm/scripts/build-npm-package-from-release.mjs   ← shared build script
├── templates/npm/tool-template/                 ← starter for tool repos
├── docs/npm-convention.md                       ← @<org>/<tool> packaging spec
└── .github/workflows/
    ├── test.yml                                 ← brew style/audit/install
    └── publish-npm-reusable.yml                 ← called by tool repos via workflow_call
```

## Adding a tool to this family

1. **Homebrew side** — add `Formula/<tool>.rb` here. Bump `version` +
   per-platform `sha256` on each release.
2. **npm side** — copy `templates/npm/tool-template/` into the tool's **own**
   source repo as `npm/`. Follow `docs/npm-convention.md`. The tool repo's
   `publish-npm.yml` calls the reusable workflow from this repo.
3. **Secrets** — each tool repo needs `NPM_TOKEN` (granular Automation token
   with write access to `@erchoc`). Prefer a GitHub org-level secret.

## Updating a tool

- **Homebrew**: bump `version` and `sha256` in `Formula/<tool>.rb`.
- **npm**: nothing here — the tool repo's `publish-npm.yml` fires on its own
  `release: published` event.

## 当前进度

- ✅ Homebrew 分发通道（`Formula/cb.rb`）
- ✅ npm 分发通道（规范 + reusable workflow + 模板）
- ✅ `cb` 的 npm 打包搬到 `Erchoc/chatbot` 作为去中心化范例
- ✅ 其他项目按 `templates/npm/tool-template/` 复制即可接入
- ⏳ 自动化：tool repo release → homebrew-tap formula 自动 PR（暂人工）
