#!/usr/bin/env bash

# change this to point to your own checkout of nixpkgs
NIXPKGS_PATH='/home/djacu/dev/nixos/nixpkgs'

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
NIX_FILE="$SCRIPT_DIR/release-packages.nix"
JSON_FILE="$SCRIPT_DIR/release-packages.json"
LIST_FILE="$SCRIPT_DIR/release-packages-list"
DRVS_PATH="$SCRIPT_DIR/release_packages_ca/"

if [[ ! -d $DRVS_PATH ]]; then
  mkdir "$DRVS_PATH"
fi
find "$DRVS_PATH" -type f -name '*.drv' -delete

# need to instantiate them into existence first
nix-instantiate \
  --strict \
  --json \
  --arg path $NIXPKGS_PATH \
  --arg caDrvs true \
  --attr pkgSet \
  --extra-experimental-features ca-derivations \
  "$NIX_FILE" 

# then we can get their paths into json
nix-instantiate \
  --eval \
  --strict \
  --json \
  --arg path $NIXPKGS_PATH \
  --arg caDrvs true \
  --attr pkgSet \
  --extra-experimental-features ca-derivations \
  "$NIX_FILE" \
  >"$JSON_FILE"

jq \
  -r \
  '[to_entries[] | .value] | unique | .[]' \
  <"$JSON_FILE" \
  >"$LIST_FILE"

while read -r line; do
  cp "$line" "$DRVS_PATH"
done <"$LIST_FILE"
