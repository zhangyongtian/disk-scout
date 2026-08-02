#!/usr/bin/env bash
set -euo pipefail

root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

mkdir -p "$root/keep" "$root/skip"
truncate -s 10 "$root/keep/small.bin"
truncate -s 2048 "$root/keep/big.bin"
truncate -s 4096 "$root/skip/ignored.bin"

out="$(
  cargo run --quiet -- scan "$root" \
    --top-files 10 \
    --top-dirs 10 \
    --min-size 1024 \
    --ignore 'skip/*' \
    --format text
)"

echo "$out"

echo "$out" | grep -q 'top_files:'
echo "$out" | grep -q 'top_dirs:'

echo "$out" | grep -q 'big.bin'
echo "$out" | grep -q 'KiB\|MiB\|GiB'

if echo "$out" | grep -q 'small.bin'; then
  echo "unexpected: small.bin present while --min-size 1024"
  exit 1
fi

if echo "$out" | grep -q 'ignored.bin'; then
  echo "unexpected: ignored.bin present while --ignore 'skip/*'"
  exit 1
fi
