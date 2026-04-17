# npm Embedded-Binary Convention (v1)

CLI tools in the `@erchoc` namespace are published to npm as a mirror of
their Homebrew distribution: `npm install -g @erchoc/<tool>` puts the same
binary on PATH as `brew install erchoc/tap/<tool>`.

This document specifies the packaging convention. Each tool's **own source
repo** publishes its own `@erchoc/<tool>` — this spec exists so every tool's
packaging looks the same. Homebrew-tap provides a shared reusable GitHub
Actions workflow; tool repos add a ~10-line caller workflow and follow the
layout below.

## Guarantees

A package that follows this spec guarantees:

- `npm install -g @<org>/<tool>` places a `<tool>` executable on PATH.
- No `postinstall` script, no network access at install time.
- Works under `npm install --ignore-scripts`.
- Same binary bytes as the matching upstream GitHub Release asset.
- macOS (arm64 + x64), Linux (x64 + arm64). Windows returns a friendly error.

## 1. Package naming

- npm name: `@<org>/<tool>` (scoped).
- `<tool>` MUST match the Homebrew formula name (`@erchoc/cb` ↔
  `erchoc/tap/cb`).
- Version: MUST equal the upstream Git release tag with any leading `v`
  stripped (`v0.1.0-beta` → `0.1.0-beta`).

## 2. Package layout (inside the tool's source repo)

Place the package skeleton at whichever path fits the repo — `npm/` at root
for single-package repos, `packages/npm/` for pnpm/yarn monorepos. The
caller workflow's `package_dir` input pins the exact location.

```
<tool-repo>/
└── <package_dir>/              # e.g. npm/ or packages/npm/
    ├── package.template.json
    ├── README.md
    ├── LICENSE
    └── bin/
        ├── <tool>.js            # launcher shim (Node, shebang)
        ├── <tool>-darwin        # macOS universal; or split -darwin-arm64 / -darwin-x64
        ├── <tool>-linux-x64
        └── <tool>-linux-arm64
```

Only the first four items (`package.template.json`, `README.md`, `LICENSE`,
`bin/<tool>.js`) are committed. The native binaries under `bin/` are
downloaded into this directory by CI at publish time and never committed —
add them to `.gitignore`.

If you place it inside a pnpm/yarn workspace, **exclude the directory** from
the workspace members — it's a publish-only wrapper whose `package.json` is
generated at release time. For pnpm:

```yaml
# pnpm-workspace.yaml
packages:
  - 'packages/*'
  - '!packages/npm'
```

Rules:

- Launcher is always `bin/<tool>.js`. `package.json`'s `bin` map points to it.
- Native binaries go alongside under `bin/`, named by Node's
  `process.platform`-`process.arch`: `darwin`, `linux`; `x64`, `arm64`.
- If upstream ships a macOS universal binary, name it `<tool>-darwin` (the
  launcher maps both arm64 and x64 to it). Otherwise split into
  `<tool>-darwin-arm64` and `<tool>-darwin-x64`.
- Every native binary MUST be chmod 0755 before `npm publish`. The shared
  build script does this.

## 3. Launcher shim contract

`bin/<tool>.js` MUST follow this contract:

```js
#!/usr/bin/env node
const { spawnSync } = require("node:child_process");
const { join } = require("node:path");

const map = {
  "darwin-arm64": "<tool>-darwin",
  "darwin-x64":   "<tool>-darwin",
  "linux-x64":    "<tool>-linux-x64",
  "linux-arm64":  "<tool>-linux-arm64",
};
const key = `${process.platform}-${process.arch}`;
const name = map[key];
if (!name) {
  console.error(
    `<tool>: unsupported platform ${key}. Supported: ${Object.keys(map).join(", ")}`
  );
  process.exit(1);
}
const bin = join(__dirname, name);
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) { console.error(r.error.message); process.exit(1); }
process.exit(r.status ?? 1);
```

Contract details:

- Use the `node:` prefix for built-ins (safe against name collisions).
- `stdio: "inherit"` is mandatory — keeps TTY, colours, and stdin working.
- Exit with the child's status; fall back to `1` on signal death.
- Unknown platform MUST print a clear diagnostic and exit non-zero.

## 4. package.json template

```jsonc
{
  "name": "@<org>/<tool>",
  "version": "__VERSION__",
  "description": "<same one-liner as the Homebrew formula's desc>",
  "homepage": "<upstream repo URL>",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/<org>/<source-repo>.git"
  },
  "license": "<SPDX identifier>",
  "bin": { "<tool>": "bin/<tool>.js" },
  "files": ["bin/", "README.md", "LICENSE"],
  "os": ["darwin", "linux"],
  "cpu": ["x64", "arm64"],
  "engines": { "node": ">=16" },
  "publishConfig": { "access": "public" }
}
```

Keep `"version": "__VERSION__"` as a literal — CI substitutes it from the
release tag at publish time.

Hard requirements:

- `bin` declares exactly one command, pointing at the launcher `.js`.
- `files` whitelists only the runtime payload. No tests, no sources.
- No `scripts.postinstall`, no `scripts.preinstall` — install MUST be offline.
- `os` / `cpu` MUST match the set of platforms actually bundled.

## 5. Caller workflow (in the tool's source repo)

Add `.github/workflows/publish-npm.yml`:

```yaml
name: publish npm
on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      release_tag:
        description: "Git tag of the release to publish from"
        required: true
        type: string
      dry_run:
        description: "If true, run npm publish --dry-run only"
        required: false
        type: boolean
        default: true

jobs:
  npm:
    uses: Erchoc/homebrew-tap/.github/workflows/publish-npm-reusable.yml@master
    with:
      tool:        <tool>
      release_tag: ${{ github.event.release.tag_name || inputs.release_tag }}
      package_dir: npm
      dry_run:     ${{ github.event_name == 'workflow_dispatch' && inputs.dry_run }}
    secrets:
      NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
```

The reusable workflow:

1. Checks out your repo and `homebrew-tap` (for shared scripts).
2. Downloads every asset from the release via `gh release download`.
3. Maps asset filenames to `<tool>-<platform>-<arch>` in-package names by
   filename convention (`*macos-universal*` → `<tool>-darwin`, etc.).
4. chmod 0755 on every binary.
5. Substitutes `__VERSION__` in your `package.template.json`.
6. Guards against re-publishing a version that's already on the npm registry.
7. Runs `npm publish --access public` (or `--dry-run`).

## 6. Prerequisites (per tool repo)

- `@<org>` organisation exists on npmjs.com.
- `NPM_TOKEN` repo secret set (granular Automation token with write access
  to `@<org>` scope). Consider using a GitHub **organisation** secret so all
  tool repos share one.
- The tool's release workflow uploads binaries named so the convention
  matches (see §7).

## 7. Release asset filename patterns the reusable workflow recognises

| Pattern in asset name | Maps to in-package binary |
|---|---|
| `*macos-universal*`, `*darwin-universal*` | `<tool>-darwin` |
| `*macos-arm64*`, `*darwin-arm64*`        | `<tool>-darwin-arm64` |
| `*macos-x86_64*`, `*macos-x64*`, `*darwin-x64*`, `*macos-intel*` | `<tool>-darwin-x64` |
| `*linux-x86_64*`, `*linux-x64*`, `*linux-amd64*` | `<tool>-linux-x64` |
| `*linux-arm64*`, `*linux-aarch64*` | `<tool>-linux-arm64` |

Anything ending in `.sha256`, `.txt`, `.md`, `.zip`, `.tar.gz`, `.tgz` is
skipped automatically.

## 8. Keeping the brew formula in sync

The Homebrew formula under `Formula/<tool>.rb` in `homebrew-tap` is a
separate artefact owned by the tap. After your tool's release publishes, bump
`version` + per-platform `sha256` in the formula (PR or direct commit). The
npm channel does not depend on this step — it downloads from the release
directly.

(Future improvement: `repository_dispatch` from your release workflow to
homebrew-tap to automate the formula bump.)

## 9. Caveats

1. **Tarball size.** Every install downloads all bundled binaries. Monitor
   tarball size; once it exceeds ~30 MB seriously consider migrating to
   split per-platform subpackages with `optionalDependencies` (a future v2
   of this spec). npm's hard tarball limit is 100 MB.
2. **macOS Gatekeeper.** Unsigned / un-notarised binaries delivered via npm
   may trigger "cannot be opened because the developer cannot be verified"
   on first run. The root fix is to sign + notarise upstream; as a
   workaround, users can run
   `xattr -d com.apple.quarantine "$(which <tool>)"`.
3. **Executable bit.** npm preserves file modes from the tarball but only if
   they're already set. The reusable workflow chmod's 0755 before `npm pack`.
4. **Unpublish policy.** npm restricts `unpublish` after 72 hours. Use the
   `dry_run: true` path on `workflow_dispatch` to validate before releasing.
5. **Scope squatting.** The `@<org>` scope must be claimed on npmjs.com
   before the first publish.
