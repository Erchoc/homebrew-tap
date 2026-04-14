#!/bin/bash
set -euo pipefail

OUT="release"
BIN="vvpn"

echo "构建 x86_64..."
cargo build --release --target x86_64-apple-darwin

echo "构建 aarch64..."
cargo build --release --target aarch64-apple-darwin

mkdir -p "$OUT"

echo "合并通用二进制..."
lipo -create \
  "target/x86_64-apple-darwin/release/$BIN" \
  "target/aarch64-apple-darwin/release/$BIN" \
  -output "$OUT/$BIN"

echo "完成: $OUT/$BIN"
file "$OUT/$BIN"
