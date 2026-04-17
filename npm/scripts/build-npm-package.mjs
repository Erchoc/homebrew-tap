#!/usr/bin/env node
// Build a publishable npm package by reading a Homebrew formula as the source
// of truth for version + per-platform sha256, then downloading the matching
// binaries from the upstream GitHub Release into the package's bin/ dir.
//
// Usage:
//   node npm/scripts/build-npm-package.mjs \
//     --formula Formula/cb.rb \
//     --pkg-dir npm/cb
//
// Optional:
//   --skip-download   Reuse any existing bin/* files; still verifies sha256.
//   --dry             Alias for --skip-download, also skips chmod failures.
//
// The mapping from formula asset filename to the in-package binary name is
// derived from the URL, per the npm convention:
//   *macos-universal*  -> <tool>-darwin
//   *macos-arm64*      -> <tool>-darwin-arm64
//   *macos-x86_64*     -> <tool>-darwin-x64
//   *linux-x86_64*     -> <tool>-linux-x64
//   *linux-arm64*      -> <tool>-linux-arm64

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile, chmod, stat } from "node:fs/promises";
import { spawn } from "node:child_process";
import { dirname, join, basename, resolve } from "node:path";
import { argv, exit } from "node:process";

function parseArgs(list) {
  const out = {};
  for (let i = 2; i < list.length; i++) {
    const a = list[i];
    if (a === "--skip-download" || a === "--dry") { out.skipDownload = true; continue; }
    if (a.startsWith("--")) {
      const key = a.slice(2);
      const val = list[i + 1];
      if (val && !val.startsWith("--")) { out[key] = val; i++; }
      else out[key] = true;
    }
  }
  return out;
}

function die(msg) {
  console.error(`build-npm-package: ${msg}`);
  exit(1);
}

function mapAssetToBinName(tool, url) {
  const f = basename(url);
  if (/macos[-_]universal/i.test(f)) return `${tool}-darwin`;
  if (/macos[-_]arm64/i.test(f))     return `${tool}-darwin-arm64`;
  if (/macos[-_](x86[_-]?64|x64|amd64|intel)/i.test(f)) return `${tool}-darwin-x64`;
  if (/darwin[-_]arm64/i.test(f))    return `${tool}-darwin-arm64`;
  if (/darwin[-_](x86[_-]?64|x64|amd64)/i.test(f)) return `${tool}-darwin-x64`;
  if (/linux[-_](x86[_-]?64|x64|amd64)/i.test(f)) return `${tool}-linux-x64`;
  if (/linux[-_]arm64/i.test(f))     return `${tool}-linux-arm64`;
  die(`cannot classify asset filename ${f}; extend mapAssetToBinName()`);
}

function parseFormula(src) {
  // Extract `version "X"` — tolerant of single or double quotes.
  const vm = src.match(/^\s*version\s+["']([^"']+)["']/m);
  if (!vm) die("formula: no version line found");
  const version = vm[1];

  // Walk the file line by line; the url that appears immediately before each
  // sha256 is treated as that binary's source. Ruby-style `#{version}`
  // interpolation in the url string is expanded against the parsed version.
  const lines = src.split(/\r?\n/);
  const entries = [];
  let pendingUrl = null;
  const urlRe = /^\s*url\s+["']([^"']+)["']/;
  const shaRe = /^\s*sha256\s+["']([0-9a-f]{64})["']/i;
  const interpolate = (u) => u.replace(/#\{version\}/g, version);
  for (const line of lines) {
    const um = line.match(urlRe);
    if (um) { pendingUrl = interpolate(um[1]); continue; }
    const sm = line.match(shaRe);
    if (sm && pendingUrl) {
      if (/#\{[^}]+\}/.test(pendingUrl)) die(`formula: unresolved interpolation in URL ${pendingUrl}`);
      entries.push({ url: pendingUrl, sha256: sm[1].toLowerCase() });
      pendingUrl = null;
    }
  }
  if (!entries.length) die("formula: no url+sha256 pairs found");

  // Deduplicate: a universal macOS binary is usually listed twice (on_arm +
  // on_intel) with the same URL. Merge those into a single asset.
  const byUrl = new Map();
  for (const e of entries) {
    if (!byUrl.has(e.url)) byUrl.set(e.url, e);
    else if (byUrl.get(e.url).sha256 !== e.sha256)
      die(`formula: same URL ${e.url} listed with two different sha256`);
  }
  return { version, assets: [...byUrl.values()] };
}

function sha256OfStream(rs) {
  return new Promise((resolve, reject) => {
    const h = createHash("sha256");
    rs.on("data", chunk => h.update(chunk));
    rs.on("end", () => resolve(h.digest("hex")));
    rs.on("error", reject);
  });
}

function runCurl(url, dest) {
  // Shell out to curl: preinstalled on every CI runner and every developer
  // box, and handles the github.com TLS/IPv6 dance more robustly than
  // Node's built-in fetch on flaky residential links.
  return new Promise((resolve, reject) => {
    const args = [
      "--silent", "--show-error", "--fail",
      "--location", "--max-redirs", "10",
      "--connect-timeout", "30",
      "--max-time", "600",
      "--retry", "3", "--retry-delay", "2", "--retry-all-errors",
      "--output", dest,
      url,
    ];
    const p = spawn("curl", args, { stdio: ["ignore", "ignore", "pipe"] });
    let stderr = "";
    p.stderr.on("data", c => { stderr += c.toString(); });
    p.on("error", reject);
    p.on("close", code => {
      if (code === 0) resolve();
      else reject(new Error(`curl exited ${code}: ${stderr.trim() || "no stderr"}`));
    });
  });
}

async function downloadToFile(url, dest) {
  try { await runCurl(url, dest); }
  catch (e) { die(`download failed: ${e.message} (url=${url})`); }
}

async function sha256OfFile(path) {
  const { createReadStream } = await import("node:fs");
  return await sha256OfStream(createReadStream(path));
}

async function main() {
  const args = parseArgs(argv);
  if (!args.formula || !args["pkg-dir"]) {
    die("usage: build-npm-package.mjs --formula <path> --pkg-dir <path> [--skip-download]");
  }

  const formulaPath = resolve(args.formula);
  const pkgDir      = resolve(args["pkg-dir"]);
  const tool        = basename(pkgDir); // e.g. "cb"
  const templatePath = join(pkgDir, "package.template.json");
  const outPkgJson   = join(pkgDir, "package.json");
  const binDir       = join(pkgDir, "bin");

  const formulaSrc = await readFile(formulaPath, "utf8");
  const { version, assets } = parseFormula(formulaSrc);
  console.log(`build-npm-package: tool=${tool} version=${version} assets=${assets.length}`);

  await mkdir(binDir, { recursive: true });

  for (const { url, sha256 } of assets) {
    const binName = mapAssetToBinName(tool, url);
    const target = join(binDir, binName);
    if (!args.skipDownload) {
      console.log(`  downloading ${url}`);
      await downloadToFile(url, target);
    } else {
      try { await stat(target); }
      catch { die(`--skip-download set but ${target} is missing`); }
    }
    const actual = await sha256OfFile(target);
    if (actual !== sha256) {
      die(`sha256 mismatch for ${binName}:\n  expected ${sha256}\n  got      ${actual}`);
    }
    try { await chmod(target, 0o755); }
    catch (e) {
      if (!args.skipDownload) throw e;
      console.warn(`  chmod failed on ${target}: ${e.message} (continuing, --dry set)`);
    }
    console.log(`  OK ${binName}  (sha256 verified)`);
  }

  const tmpl = JSON.parse(await readFile(templatePath, "utf8"));
  if (!tmpl.version || tmpl.version !== "__VERSION__") {
    die(`package.template.json must have "version": "__VERSION__" (got ${JSON.stringify(tmpl.version)})`);
  }
  tmpl.version = version;
  await writeFile(outPkgJson, JSON.stringify(tmpl, null, 2) + "\n");
  console.log(`  wrote ${outPkgJson}`);

  console.log("done. next: cd", pkgDir, "&& npm pack --dry-run");
}

main().catch(e => { console.error(e); exit(1); });
