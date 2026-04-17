# homebrew-tap

Distribution repo for pre-built CLI binaries. Ships each tool through **two
equivalent channels**: Homebrew (`brew install erchoc/tap/<tool>`) and npm
(`npm install -g @erchoc/<tool>`).

## Structure

```
homebrew-tap/
├── Formula/                     ← Homebrew formulae (one .rb per tool)
├── npm/
│   ├── <tool>/                  ← per-tool npm package (launcher + manifest)
│   └── scripts/build-npm-package.mjs
├── templates/npm/tool-template/ ← starter for self-publishing projects
├── docs/npm-convention.md       ← full @<org>/<tool> packaging spec
└── .github/workflows/
    ├── test.yml                 ← brew style/audit/install + npm build dry-run
    └── publish-npm.yml          ← tag-triggered npm publish
```

## Adding a new formula

Each formula downloads a pre-built binary from the source project's GitHub
Releases. No source code lives here — only Formula definitions.

## Adding a new npm package

Follow `docs/npm-convention.md`. Copy `templates/npm/tool-template/` to
`npm/<tool>/`, fill in placeholders, make sure a matching `Formula/<tool>.rb`
exists. The build script uses the formula as source of truth for version and
per-platform sha256.

## Updating a tool

Update `version` and each `sha256` in `Formula/<tool>.rb`. Then:

- **Homebrew side** — nothing else to do; users get it on `brew upgrade`.
- **npm side** — push tag `npm-<tool>-v<version>`; the `publish-npm`
  workflow builds + verifies + publishes.

## 当前进度

- ✅ Homebrew 分发通道（`Formula/cb.rb`，macOS universal + linux x64/arm64）
- ✅ npm 分发通道（`@<org>/<tool>` 单包内嵌二进制规范 + `@erchoc/cb` 参考实现）
- ✅ 打包命名/结构规范文档（`docs/npm-convention.md`）
- ✅ 其他项目自发布的模板（`templates/npm/tool-template/`）
- ✅ CI：`publish-npm.yml` 按 tag 触发、`test.yml` 新增 npm 构建干跑
- ⏳ 首次发布前置（需大哥亲手）：npmjs.com 注册 `@erchoc` 组织 + 仓库
  secret `NPM_TOKEN`
