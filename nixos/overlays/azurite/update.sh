#!/usr/bin/env nix-shell
#!nix-shell -i bash -p curl jq nodejs_20 prefetch-npm-deps moreutils nix git

set -euo pipefail

OVERLAY_DIR="$(git rev-parse --show-toplevel)/nixos/overlays/azurite"
INFO="$OVERLAY_DIR/info.json"

# Latest GitHub release
LATEST="$(curl -fsSL https://api.github.com/repos/Azure/Azurite/releases/latest \
  | jq -r '.tag_name | ltrimstr("v")')"

# commit SHA for the tagged release (resolves annotated tags automatically)
COMMIT_SHA="$(curl -fsSL \
  "https://api.github.com/repos/Azure/Azurite/commits/v${LATEST}" \
  | jq -r '.sha')"

CURRENT="$(jq -r .version "$INFO")"
CURRENT_COMMIT="$(jq -r .commitHash "$INFO")"

if [[ "$LATEST" == "$CURRENT" && "$COMMIT_SHA" == "$CURRENT_COMMIT" ]]; then
  echo "azurite is already at $CURRENT ($CURRENT_COMMIT)"
  exit 0
fi

echo "Updating azurite $CURRENT -> $LATEST ($COMMIT_SHA)"

# src hash — NAR hash of unpacked tarball, matching fetchFromGitHub
SRC_URL="https://github.com/Azure/Azurite/archive/refs/tags/v${LATEST}.tar.gz"
SRC_HASH="$(nix-prefetch-url --unpack "$SRC_URL" 2>/dev/null \
  | xargs nix hash convert --hash-algo sha256 --to sri)"

# Regenerate package-lock.json from the new package.json.
# nodejs_20 (npm 10) is used because npm 11 generates lockfileVersion 3 without
# resolved/integrity fields, which breaks prefetch-npm-deps.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "https://raw.githubusercontent.com/Azure/Azurite/v${LATEST}/package.json" \
  -o "$TMP/package.json"

(cd "$TMP" && npm install --package-lock-only --ignore-scripts --audit=false 2>/dev/null)

cp "$TMP/package-lock.json" "$OVERLAY_DIR/package-lock.json"

# npmDeps hash
NPM_DEPS_HASH="$(prefetch-npm-deps "$OVERLAY_DIR/package-lock.json" 2>/dev/null)"

# Update info.json in place
jq \
  --arg version     "$LATEST" \
  --arg hash        "$SRC_HASH" \
  --arg npmDepsHash "$NPM_DEPS_HASH" \
  --arg commitHash  "$COMMIT_SHA" \
  '.version = $version | .hash = $hash | .npmDepsHash = $npmDepsHash | .commitHash = $commitHash' \
  "$INFO" | sponge "$INFO"

echo "Done — commit nixos/overlays/azurite/info.json and nixos/overlays/azurite/package-lock.json"
