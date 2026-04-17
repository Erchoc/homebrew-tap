#!/usr/bin/env node
// Launcher shim for an @<org>/<tool> embedded-binary npm package.
// See docs/npm-convention.md (section 3) for the exact contract.
//
// Replace every `__TOOL__` placeholder with your CLI's name before publishing.

const { spawnSync } = require("node:child_process");
const { join } = require("node:path");

const map = {
  "darwin-arm64": "__TOOL__-darwin",
  "darwin-x64":   "__TOOL__-darwin",
  "linux-x64":    "__TOOL__-linux-x64",
  "linux-arm64":  "__TOOL__-linux-arm64",
};

const key = `${process.platform}-${process.arch}`;
const name = map[key];
if (!name) {
  console.error(
    `__TOOL__: unsupported platform ${key}. Supported: ${Object.keys(map).join(", ")}`
  );
  process.exit(1);
}

const bin = join(__dirname, name);
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) {
  console.error(r.error.message);
  process.exit(1);
}
process.exit(r.status ?? 1);
