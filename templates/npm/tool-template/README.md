# @__ORG__/__TOOL__ — npm package template

Starter kit for publishing an `@<org>/<tool>` npm package that mirrors a
Homebrew-style pre-built binary distribution, following the
[npm embedded-binary convention](../../../docs/npm-convention.md).

## How to use this template

1. Copy this directory into your project — typically as `npm/<tool>/` inside
   a distribution repo, or at any path inside the tool's own source repo.
2. Rename `bin/__TOOL__.js` → `bin/<your-tool>.js`.
3. Fill in every `__TOOL__`, `__ORG__`, `__DESCRIPTION__`, `__HOMEPAGE__`,
   `__REPOSITORY_URL__`, `__ISSUES_URL__`, `__LICENSE__`, `__AUTHOR__`
   placeholder (search-and-replace).
4. Keep `"version": "__VERSION__"` as-is — the build script substitutes it.
5. Point your build script at the authoritative version + sha256 source.
   The reference implementation in this repo reads a Homebrew formula; if
   your source of truth is different, adapt `scripts/build-npm-package.mjs`
   accordingly.
6. Set the `NPM_TOKEN` repo secret (npm Automation token).
7. Adapt the tag format. The reference workflow triggers on
   `npm-<tool>-v<version>` tags.

## Files

- `bin/__TOOL__.js` — launcher shim. Do not modify the logic; only rename
  the file and change the `map` values to match your binary filenames.
- `package.template.json` — npm manifest with `__VERSION__` placeholder.

## See also

- The full spec: `docs/npm-convention.md` in `Erchoc/homebrew-tap`.
- A fully-fleshed reference: `npm/cb/` in the same repo.
