# @__ORG__/__TOOL__ — npm package template

Starter kit for publishing `@__ORG__/__TOOL__` from your tool's own source
repo, following the
[npm embedded-binary convention](../../../docs/npm-convention.md).

## How to use

Copy this directory into your tool's source repo so it lives at `npm/` in
the repo root (or anywhere else; update `package_dir` in the caller workflow
to match).

Then:

1. Rename `bin/__TOOL__.js` → `bin/<your-tool>.js`.
2. Find-and-replace every `__TOOL__`, `__ORG__`, `__DESCRIPTION__`,
   `__HOMEPAGE__`, `__REPOSITORY_URL__`, `__ISSUES_URL__`, `__LICENSE__`,
   `__AUTHOR__` placeholder across the tree.
3. Keep `"version": "__VERSION__"` in `package.template.json` — CI
   substitutes it from the release tag at publish time.
4. Commit **only** `package.template.json`, `README.md`, `LICENSE`,
   `bin/<tool>.js`, `.gitignore`, and `.github/workflows/publish-npm.yml`.
   The native binaries under `bin/` are downloaded by CI and never committed
   (see `.gitignore`).
5. Create the `NPM_TOKEN` repo secret (an npm Automation token with
   read+write access to the `@__ORG__` scope). Consider using a GitHub
   organisation secret so all your tool repos share it.
6. Ensure your existing release workflow uploads assets whose filenames
   include one of the recognised platform patterns — see §7 of
   [docs/npm-convention.md](../../../docs/npm-convention.md) for the full
   mapping.
7. Next release: push a tag, let your existing release workflow produce
   binaries, and the `publish-npm.yml` workflow fires on the
   `release: published` event to publish `@__ORG__/__TOOL__` automatically.

## Files in this template

- `bin/__TOOL__.js` — launcher shim; only rename the file and update the
  `map` values to match your binary filenames.
- `bin/.gitkeep` — keeps `bin/` tracked even though everything in it is
  `.gitignore`d.
- `package.template.json` — npm manifest with `__VERSION__` placeholder.
- `.github/workflows/publish-npm.yml` — caller workflow; calls the shared
  reusable workflow from `Erchoc/homebrew-tap`.
- `.gitignore` — excludes downloaded binaries, generated `package.json`,
  and temp download directories from git.
- `LICENSE` — match your project's existing license.
- `README.md` — the npm package's public-facing readme (what users see on
  npmjs.com).

## See also

- Full spec: [`docs/npm-convention.md`](../../../docs/npm-convention.md) in
  the `Erchoc/homebrew-tap` repo.
- Reusable workflow:
  [`.github/workflows/publish-npm-reusable.yml`](../../../.github/workflows/publish-npm-reusable.yml)
  in the same repo.
