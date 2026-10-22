#!/usr/bin/env bash
# Mirrors wiki/ into the repository's GitHub Wiki (a separate git repo).
#
# Prerequisite: Settings > General > Features > tick "Wikis", then create the
# first page once (or just run this script — it pushes to an empty wiki repo).
#
# Usage: bash tools/sync-wiki.sh

set -euo pipefail
cd "$(dirname "$0")/.."

REPO="build-your-own-x-with-ai/RustNES_NESL_Studio"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

if ! git ls-remote "git@github.com:${REPO}.wiki.git" >/dev/null 2>&1; then
  echo "GitHub Wiki 还没有启用：Settings > General > Features > 勾选 Wikis 后重试。"
  exit 1
fi

git clone --depth 1 "git@github.com:${REPO}.wiki.git" "$WORK/wiki" 2>/dev/null
cp wiki/*.md "$WORK/wiki/"
cd "$WORK/wiki"

git add -A
if git diff --cached --quiet; then
  echo "wiki 没有变化，无需提交。"
  exit 0
fi

git -c user.name="$(git -C "$OLDPWD" config user.name)" \
    -c user.email="$(git -C "$OLDPWD" config user.email)" \
    commit -m "docs: sync wiki from wiki/ ($(date +%Y-%m-%d))"
git push origin HEAD:master 2>/dev/null || git push origin HEAD:main

echo "已同步到 https://github.com/${REPO}/wiki"
