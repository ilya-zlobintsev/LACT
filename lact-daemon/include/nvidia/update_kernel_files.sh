#!/usr/bin/env bash
set -euo pipefail

UPSTREAM_ROOT="${1:-../../../../open-gpu-kernel-modules}"

find "kernel-open" "src" -type f | while IFS= read -r relpath; do
  src="$UPSTREAM_ROOT/$relpath"
  dst="./$relpath"

  if [[ ! -f "$src" ]]; then
    echo "File $src was deleted upstream"
    rm -f $dst
  fi

  mkdir -p "$(dirname "$dst")"

  if ! cmp -s "$src" "$dst"; then
    cp "$src" "$dst"
    echo "Updated $relpath"
  fi
done
