#!/usr/bin/env bash
# Build and sign static/shortcut/UNI-SHARE.shortcut from PUBLIC_BASE_URL in .env
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

PUBLIC_BASE_URL="${PUBLIC_BASE_URL:-http://127.0.0.1:8080}"
PUBLIC_BASE_URL="${PUBLIC_BASE_URL%/}"
ENDPOINT="${PUBLIC_BASE_URL}/api/v1/convert"
UNSIGNED="$ROOT/static/shortcut/UNI-SHARE-unsigned.shortcut"
SIGNED="$ROOT/static/shortcut/UNI-SHARE.shortcut"

python3 "$ROOT/scripts/gen_shortcut.py" "$ENDPOINT" "$UNSIGNED"
shortcuts sign -m anyone -i "$UNSIGNED" -o "$SIGNED"
echo "signed → $SIGNED ($(wc -c < "$SIGNED") bytes)"
echo "Réimporte ce fichier dans Raccourcis (Ajouter / Remplacer)."
