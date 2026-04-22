#!/usr/bin/env node
// Build a publishable npm package by downloading binaries from a GitHub
// Release in the caller's own repo. Designed to be called by a tool's own
// `publish-npm.yml` workflow (typically via the reusable workflow in this
// repo) right after the release is published.
//
// Unlike `build-npm-package.mjs` (which reads a Homebrew formula and
// verifies sha256 against it), this variant trusts the release it just
// downloaded — it's the decentralised flow where the tool's own repo is
// both builder and publisher.
//
// Usage:
//   node build-npm-package-from-release.mjs \
//     --tool cb \
//     --repo Erchoc/chatbot \
//     --release v0.1.0-beta \
//     --pkg-dir ./npm
//
// Assumes the GitHub CLI (`gh`) is installed and authenticated via
// GITHUB_TOKEN or similar (default on GitHub Actions runners).

import { createHash } from "node:crypto";
import { readFile, writeFile, chmod, mkdir, readdir, rm, copyFile } from "node:fs/promises";
import { createReadStream } from "node:fs";
import { spawn } from "node:child_process";
import { basename, join, resolve } from "node:path";
import { argv, exit } from "node:process";

function parseArgs(list) {
  const out = {};
  for (let i = 2; i < list.length; i++) {
    const a = list[i];
    if (a.startsWith("--")) {
      const key = a.slice(2);
      const val = list[i + 1];
      if (val && !val.startsWith("--")) { out[key] = val; i++; }
      else out[key] = true;
    }
  }
  return out;
}

function die(msg) { console.error(`build-npm-package-from-release: ${msg}`); exit(1); }

// Map a GitHub Release asset filename to the in-package binary name, per the
// embedded-binary convention. Returns null for assets that don't look like
// a platform binary (source tarballs, .sha256 sidecars, etc.).
function mapAssetToBinName(tool, filename) {
  const f = basename(filename);
  if (f.endsWith(".sha256") || f.endsWith(".txt") || f.endsWith(".md")) return null;
  if (f.endsWith(".zip") || f.endsWith(".tgz")) return null;
  // Friendly OS-arch patterns (bare binaries or zips named by convention)
  if (/macos[-_]universal/i.test(f))                             return `${tool}-darwin`;
  if (/(macos|darwin)[-_]arm64/i.test(f))                        return `${tool}-darwin-arm64`;
  if (/(macos|darwin)[-_](x86[_-]?64|x64|amd64|intel)/i.test(f)) return `${tool}-darwin-x64`;
  if (/linux[-_](x86[_-]?64|x64|amd64)/i.test(f))                return `${tool}-linux-x64`;
  if (/linux[-_](arm64|aarch64)/i.test(f))                       return `${tool}-linux-arm64`;
  // Rust cross-compilation triples (e.g. dfctl-0.1.0-aarch64-apple-darwin.tar.gz)
  if (/aarch64-apple-darwin/i.test(f))    return `${tool}-darwin-arm64`;
  if (/x86_64-apple-darwin/i.test(f))     return `${tool}-darwin-x64`;
  if (/x86_64-unknown-linux/i.test(f))    return `${tool}-linux-x64`;
  if (/aarch64-unknown-linux/i.test(f))   return `${tool}-linux-arm64`;
  // Remaining tarballs that didn't match any platform pattern
  if (f.endsWith(".tar.gz")) return null;
  return null;
}

// Extract the tool binary from a .tar.gz archive into dest.
// Expects the archive to contain a file named `tool` at root level
// (the layout produced by cargo cross-compilation release workflows).
async function extractBinaryFromTarball(archivePath, tool, dest) {
  const extractDir = `${archivePath}.extract`;
  await mkdir(extractDir, { recursive: true });
  try {
    await new Promise((resolve, reject) => {
      const p = spawn("tar", ["-xzf", archivePath, "-C", extractDir], { stdio: "inherit" });
      p.on("error", reject);
      p.on("close", code => code === 0 ? resolve() : reject(new Error(`tar exited ${code}`)));
    });
    const files = await readdir(extractDir);
    const binary = files.find(f => f === tool) ?? files.find(f => !f.includes("."));
    if (!binary) throw new Error(`'${tool}' not found in tarball. Contents: ${files.join(", ")}`);
    await copyFile(join(extractDir, binary), dest);
  } finally {
    await rm(extractDir, { recursive: true, force: true });
  }
}

function runGh(args) {
  return new Promise((resolve, reject) => {
    const p = spawn("gh", args, { stdio: ["ignore", "inherit", "pipe"] });
    let stderr = "";
    p.stderr.on("data", c => { process.stderr.write(c); stderr += c.toString(); });
    p.on("error", reject);
    p.on("close", code => code === 0 ? resolve() : reject(new Error(`gh ${args.join(" ")} exited ${code}`)));
  });
}

function sha256OfFile(path) {
  return new Promise((resolve, reject) => {
    const h = createHash("sha256");
    const rs = createReadStream(path);
    rs.on("data", c => h.update(c));
    rs.on("end", () => resolve(h.digest("hex")));
    rs.on("error", reject);
  });
}

async function main() {
  const args = parseArgs(argv);
  for (const k of ["tool", "repo", "release", "pkg-dir"]) {
    if (!args[k]) die(`missing --${k}`);
  }

  const tool    = args.tool;
  const repo    = args.repo;
  const release = args.release;
  const pkgDir  = resolve(args["pkg-dir"]);
  const binDir  = join(pkgDir, "bin");
  const tmpDir  = join(pkgDir, ".release-downloads");
  const tmplPath = join(pkgDir, "package.template.json");
  const outPkg   = join(pkgDir, "package.json");

  await mkdir(binDir, { recursive: true });
  await rm(tmpDir, { recursive: true, force: true });
  await mkdir(tmpDir, { recursive: true });

  console.log(`build-npm-package-from-release: tool=${tool} repo=${repo} release=${release}`);
  console.log(`downloading release assets into ${tmpDir}`);
  await runGh([
    "release", "download", release,
    "--repo", repo,
    "--dir", tmpDir,
    "--clobber",
    "--pattern", "*",
  ]);

  const files = await readdir(tmpDir);
  let matched = 0;
  for (const f of files.sort()) {
    const binName = mapAssetToBinName(tool, f);
    if (!binName) { console.log(`  skip ${f} (no platform match)`); continue; }
    const src = join(tmpDir, f);
    const dst = join(binDir, binName);
    if (f.endsWith(".tar.gz")) {
      console.log(`  extracting ${f} → ${binName}`);
      await extractBinaryFromTarball(src, tool, dst);
    } else {
      await copyFile(src, dst);
    }
    await chmod(dst, 0o755);
    const sha = await sha256OfFile(dst);
    console.log(`  OK ${binName}  (from ${f}, sha256 ${sha.slice(0, 16)}…)`);
    matched++;
  }

  if (matched === 0) {
    die(`no assets matched any platform pattern. inspect ${tmpDir} to debug.`);
  }

  // Version: strip any leading v from the release tag.
  const version = release.replace(/^v/, "");
  const tmpl = JSON.parse(await readFile(tmplPath, "utf8"));
  if (tmpl.version !== "__VERSION__") {
    die(`package.template.json must have "version": "__VERSION__" (got ${JSON.stringify(tmpl.version)})`);
  }
  tmpl.version = version;
  await writeFile(outPkg, JSON.stringify(tmpl, null, 2) + "\n");
  console.log(`wrote ${outPkg} (version=${version})`);

  await rm(tmpDir, { recursive: true, force: true });
  console.log(`done. matched ${matched} platform binaries.`);
}

main().catch(e => { console.error(e); exit(1); });
