#!/usr/bin/env bash
# Compatibility wrapper for GitHub Pages (project site under /uni-share/).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export SITE_BASE_PATH="${SITE_BASE_PATH:-${PAGES_BASE_PATH:-/uni-share}}"
exec bash "$ROOT/scripts/build-site.sh"
