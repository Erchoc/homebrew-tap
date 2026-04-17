# npm Embedded-Binary Convention (v1)

This repo distributes CLI tools through two channels: Homebrew (`brew install
erchoc/tap/<tool>`) and npm (`npm install -g @erchoc/<tool>`). Both channels
hand users the same pre-built binary that ships in the upstream project's
GitHub Release.

This document specifies the npm packaging convention. Any project — not just
those whose formulae live here — can follow it to publish a mirror npm package
with identical user-facing behaviour.

## Guarantees

A package that follows this spec guarantees:

- `npm install -g @<org>/<tool>` places a `<tool>` executable on PATH.
- No `postinstall` script, no network access at install time.
- Works under `npm install --ignore-scripts`.
- Same binary bytes as the matching Homebrew formula (sha256-verified).
- macOS (arm64 + x64), Linux (x64 + arm64). Windows returns a friendly error.

## 1. Package naming

- npm name: `@<org>/<tool>` (scoped).
- `<tool>` MUST match the Homebrew formula name (`@erchoc/cb` ↔
  `erchoc/tap/cb`).
- Version: MUST equal the upstream release version character-for-character
  (drop any `v` prefix). The same string appears in `Formula/<tool>.rb`'s
  `version` field and in `@<org>/<tool>`'s `package.json` `version`.

## 2. Package layout

```
<package-root>/
├── package.json
├── README.md
├── LICENSE
└── bin/
    ├── <tool>.js            # launcher shim (Node, shebang)
    ├── <tool>-darwin        # macOS universal; or split -darwin-arm64 / -darwin-x64
    ├── <tool>-linux-x64
    └── <tool>-linux-arm64
```

Rules:

- Launcher is always `bin/<tool>.js`. `package.json`'s `bin` map points to it.
- Native binaries go alongside under `bin/` with names keyed on Node's
  `process.platform`-`process.arch`: `darwin`, `linux`; `x64`, `arm64`.
- If upstream ships a macOS universal binary, name it `<tool>-darwin` (the
  launcher maps both arm64 and x64 to it). Otherwise split into
  `<tool>-darwin-arm64` and `<tool>-darwin-x64`.
- Every native binary MUST be chmod 0755 before `npm publish`. npm preserves
  the tar mode bits, but only if they are set at pack time.

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
  "version": "0.0.0",
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

Hard requirements:

- `bin` declares exactly one command, pointing at the launcher `.js`.
- `files` whitelists only the runtime payload. No tests, no sources.
- No `scripts.postinstall`, no `scripts.preinstall` — install MUST be offline.
- `os` / `cpu` MUST match the set of platforms actually bundled.

## 5. Binary provenance and verification

- Binaries MUST be downloaded from the upstream GitHub Release that the
  Homebrew formula references (same URL pattern, same tag).
- The sha256 of each downloaded binary MUST match the corresponding `sha256`
  line in `Formula/<tool>.rb`. The publish tooling fails closed on any
  mismatch.

## 6. Release tagging and automation

- Git tag format: `npm-<tool>-v<version>` (e.g. `npm-cb-v0.1.0-beta`). The
  prefix scopes the tag so it never collides with upstream source tags (which
  conventionally use `v<version>`).
- CI picks up `npm-<tool>-v*` tags, parses `<tool>`, builds the package, and
  runs `npm publish --access public` with a repo secret token.
- `workflow_dispatch` is also supported for ad-hoc runs and dry-runs.

## 7. Self-publishing from another repo

Any project can follow this spec without involving `homebrew-tap`:

1. Copy `templates/npm/tool-template/` into your repo.
2. Replace `__TOOL__` placeholders with your tool name, fill in
   `description`, `homepage`, `repository`, `license`.
3. Replace `__FORMULA_PATH__` with wherever your `version` and per-platform
   `sha256` values live. The template's build script reads a Homebrew-style
   formula; if your source of truth is different (JSON manifest, GitHub
   Release API, etc.), adapt the parser — everything downstream is the same.
4. Configure `NPM_TOKEN` as a repo secret.
5. Push a `npm-<tool>-v<ver>` tag; the workflow does the rest.

## 8. Caveats

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
   they're already set. Publish scripts MUST chmod 0755 before packing.
4. **Unpublish policy.** npm restricts `unpublish` after 72 hours. Prefer
   dry-run publishes before tagging a real release.
5. **Scope squatting.** The `@<org>` scope must be claimed on npmjs.com
   before the first publish. Check availability before settling on a name.
