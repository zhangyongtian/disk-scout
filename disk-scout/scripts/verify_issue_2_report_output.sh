#!/usr/bin/env bash
set -euo pipefail

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

mkdir -p "$root/dir_a" "$root/dir_b"
truncate -s 10 "$root/dir_a/a.bin"
truncate -s 2048 "$root/dir_a/b.bin"
truncate -s 1048576 "$root/dir_b/c.bin"

echo "=== text ==="
cargo run --quiet -- scan "$root" --top-files 5 --top-dirs 5 --format text

echo
echo "=== json ==="
cargo run --quiet -- scan "$root" --top-files 5 --top-dirs 5 --format json
