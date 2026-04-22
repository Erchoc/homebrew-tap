#!/usr/bin/env bash
# Bump a Formula's version + sha256 values to match a GitHub release.
#
# What it does:
#   1. Reads Formula/<tool>.rb to discover source repo + artifact filenames
#   2. Resolves the target tag (arg or newest release incl. prereleases)
#   3. Fetches <artifact>.sha256 for each platform url in the Formula
#   4. sed-replaces the version string and each sha256 in place
#   5. Prints a colored diff and optionally commits + pushes
#
# Usage:
#   scripts/bump-formula.sh cb v0.1.0-beta.6     # pin tag
#   scripts/bump-formula.sh cb                   # auto: newest release
#   scripts/bump-formula.sh cb v0.1.0 --commit   # commit + push after edit
#
# Env:
#   GITHUB_TOKEN  — used for API calls (auto-picked up from `gh auth token`
#                   if unset). Recommended to avoid 60 req/hr rate limit.

set -euo pipefail

# Re-exec under real bash if invoked as /bin/sh (macOS POSIX mode).
if [ -z "${BASH_VERSION:-}" ] \
   || [ -n "${POSIXLY_CORRECT:-}" ] \
   || [ "${BASH##*/}" = "sh" ]; then
  if command -v bash >/dev/null 2>&1; then
    exec bash "$0" "$@"
  else
    echo "This script needs bash." >&2
    exit 1
  fi
fi

# ── Colors ──────────────────────────────────────────────────────────
C_RED=$'\033[91m'; C_GREEN=$'\033[92m'; C_DIM=$'\033[90m'
C_BOLD=$'\033[1m'; C_CYAN=$'\033[96m'; C_RESET=$'\033[0m'

usage() {
  sed -n '2,/^set -euo/p' "$0" | sed 's/^#//' | head -n -1
  exit 0
}

die() { echo "${C_RED}✗${C_RESET} $*" >&2; exit 1; }
log() { echo "${C_DIM}·${C_RESET} $*"; }

[ $# -lt 1 ] && usage
case "${1:-}" in -h|--help) usage ;; esac

TOOL="$1"
TAG="${2:-}"
COMMIT_PUSH=0
for arg in "$@"; do
  [ "$arg" = "--commit" ] && COMMIT_PUSH=1
done

# Resolve repo root from script location so this works from any cwd.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FORMULA="$REPO_ROOT/Formula/${TOOL}.rb"
[ -f "$FORMULA" ] || die "Formula not found: $FORMULA"

# ── Parse Formula for source repo + artifact names ───────────────────
# Grab the first `url "https://github.com/<owner>/<repo>/..."` line.
FIRST_URL=$(grep -m1 -E '^\s*url\s*"https://github\.com/' "$FORMULA" || true)
[ -n "$FIRST_URL" ] || die "no github.com url line found in $FORMULA"

SOURCE_REPO=$(echo "$FIRST_URL" | sed -E 's#.*github\.com/([^/]+/[^/]+)/.*#\1#')
[ -n "$SOURCE_REPO" ] || die "could not parse source repo from url"
log "source repo:  ${C_CYAN}$SOURCE_REPO${C_RESET}"

# Collect all distinct artifact filenames referenced in the Formula.
# Each url looks like:  url "https://github.com/<owner>/<repo>/releases/download/<tag>/<artifact>"
# The interesting part is the last path component before the closing quote.
ARTIFACTS=()
while IFS= read -r line; do
  # Drop everything up to and including the last "/" within the quoted URL,
  # then drop the closing quote and anything after.
  a=${line##*/}
  a=${a%%\"*}
  [ -n "$a" ] || continue
  # Dedupe
  skip=0
  for existing in "${ARTIFACTS[@]:-}"; do
    [ "$existing" = "$a" ] && skip=1 && break
  done
  [ "$skip" = 0 ] && ARTIFACTS+=("$a")
done < <(grep -E '^\s*url\s*"https://github\.com/' "$FORMULA")

[ "${#ARTIFACTS[@]}" -gt 0 ] || die "no artifacts parsed from $FORMULA"
log "artifacts:    ${ARTIFACTS[*]}"

# ── Resolve tag ──────────────────────────────────────────────────────
AUTH_ARGS=()
EFFECTIVE_TOKEN="${GITHUB_TOKEN:-}"
if [ -z "$EFFECTIVE_TOKEN" ] && command -v gh >/dev/null 2>&1; then
  EFFECTIVE_TOKEN=$(gh auth token 2>/dev/null || true)
fi
[ -n "$EFFECTIVE_TOKEN" ] && AUTH_ARGS=(-H "Authorization: Bearer $EFFECTIVE_TOKEN")

if [ -z "$TAG" ]; then
  log "fetching newest release from $SOURCE_REPO..."
  API_URL="https://api.github.com/repos/${SOURCE_REPO}/releases?per_page=1"
  api_body=$(mktemp)
  api_status=$(curl -sSL -o "$api_body" -w '%{http_code}' "${AUTH_ARGS[@]}" "$API_URL" || echo '000')
  if [ "$api_status" = "403" ] && [ -z "$EFFECTIVE_TOKEN" ]; then
    rm -f "$api_body"
    die "GitHub API 403 (rate limited). Set GITHUB_TOKEN or pass a tag explicitly."
  fi
  [ "$api_status" = "200" ] || { rm -f "$api_body"; die "API $api_status fetching releases"; }
  TAG=$(grep '"tag_name"' "$api_body" | head -1 | sed 's/.*"tag_name": "\(.*\)".*/\1/')
  rm -f "$api_body"
  [ -n "$TAG" ] || die "couldn't parse tag_name from API response"
fi
log "target tag:   ${C_CYAN}$TAG${C_RESET}"

# Strip leading v for the Formula's `version "X.Y.Z"` field.
VERSION="${TAG#v}"

# ── Current state ────────────────────────────────────────────────────
CURRENT_VERSION=$(grep -E '^[[:space:]]*version[[:space:]]+' "$FORMULA" \
  | head -1 | sed -E 's/^[[:space:]]*version[[:space:]]+"([^"]+)".*/\1/')
log "current:      ${CURRENT_VERSION:-<unset>}  →  ${C_GREEN}$VERSION${C_RESET}"
if [ "$CURRENT_VERSION" = "$VERSION" ]; then
  echo
  echo "${C_BOLD}Formula is already at $VERSION.${C_RESET} Nothing to do."
  exit 0
fi

# ── Fetch sha256 per artifact ────────────────────────────────────────
declare -A SHAS
echo
log "fetching sha256 values..."
for a in "${ARTIFACTS[@]}"; do
  # Substitute Ruby's #{version} interpolation (used in tarball artifact names)
  # so the GitHub download URL resolves correctly; keep original key for awk mapping.
  a_resolved="${a//\#\{version\}/$VERSION}"
  url="https://github.com/${SOURCE_REPO}/releases/download/${TAG}/${a_resolved}.sha256"
  body=$(curl -fsSL "$url" 2>/dev/null || true)
  [ -n "$body" ] || die "failed to fetch $url — is the release published?"
  sha=$(echo "$body" | awk '{print $1}')
  [ ${#sha} -eq 64 ] || die "unexpected sha256 length for $a_resolved: '$sha'"
  SHAS[$a]="$sha"
  printf "    %-30s %s\n" "$a_resolved" "${sha:0:16}..."
done

# ── Patch Formula (sed in place, portable) ───────────────────────────
# macOS sed needs `-i ''`; GNU sed needs `-i`. Use a portable wrapper.
sed_inplace() {
  if [[ "$(uname -s)" == "Darwin" ]]; then
    sed -i '' "$@"
  else
    sed -i "$@"
  fi
}

# 1. version
sed_inplace -E "s#(^[[:space:]]*version[[:space:]]+\")[^\"]+(\".*)#\1${VERSION}\2#" "$FORMULA"

# 2. sha256 per artifact. Locate each `url "....<artifact>"` line and
#    rewrite the next `sha256 "..."` under it. Awk is safer than sed
#    for that kind of scoped rewrite. Passing a table of (artifact, sha)
#    to awk via stdin is cleaner and more portable (BSD awk chokes on
#    newlines inside -v string values).
SHAS_FILE=$(mktemp)
for a in "${ARTIFACTS[@]}"; do
  printf '%s\t%s\n' "$a" "${SHAS[$a]}"
done > "$SHAS_FILE"

TMP=$(mktemp)
awk -v shasfile="$SHAS_FILE" '
BEGIN {
  while ((getline line < shasfile) > 0) {
    # artifact\tsha
    tab = index(line, "\t")
    if (tab > 0) shas[substr(line, 1, tab - 1)] = substr(line, tab + 1)
  }
  close(shasfile)
  pending = ""
}
{
  if (match($0, /url "https:\/\/github\.com\/[^\/]+\/[^\/]+\/releases\/download\/[^\/]+\/[^"]+"/)) {
    artifact = $0
    sub(/.*\//, "", artifact); sub(/".*/, "", artifact)
    if (artifact in shas) pending = artifact
    print
    next
  }
  if (pending != "" && match($0, /^[[:space:]]*sha256 "[^"]+"/)) {
    sub(/sha256 "[^"]+"/, "sha256 \"" shas[pending] "\"")
    pending = ""
  }
  print
}
' "$FORMULA" > "$TMP" && mv "$TMP" "$FORMULA"
rm -f "$SHAS_FILE"

# ── Show diff ────────────────────────────────────────────────────────
echo
echo "${C_BOLD}Updated ${FORMULA#$REPO_ROOT/}:${C_RESET}"
if command -v git >/dev/null 2>&1; then
  ( cd "$REPO_ROOT" && git --no-pager diff -- "Formula/${TOOL}.rb" )
else
  diff -u /dev/null "$FORMULA" | head -30
fi

# ── Commit? ──────────────────────────────────────────────────────────
echo
if [ "$COMMIT_PUSH" = 1 ]; then
  (
    cd "$REPO_ROOT"
    git add "Formula/${TOOL}.rb"
    git commit -m "chore($TOOL): bump to $TAG"
    git push origin "$(git symbolic-ref --short HEAD)"
  )
  echo "${C_GREEN}✓${C_RESET} Formula committed and pushed."
else
  echo "${C_DIM}Diff is unstaged. To commit + push:${C_RESET}"
  echo "  cd ${REPO_ROOT#$HOME/~}"
  echo "  git add Formula/${TOOL}.rb"
  echo "  git commit -m 'chore($TOOL): bump to $TAG'"
  echo "  git push"
  echo
  echo "${C_DIM}Or re-run with --commit to do that in one go.${C_RESET}"
fi
