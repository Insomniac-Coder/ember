#!/usr/bin/env bash
# In WSL: CI's gcc job — the workspace's tests through gcc, the conformance suite with the C gate
# ([TST-27]), on a tree of its own (make_tree.sh's is deleted by the gcc sweep). Results in ~/gcc_gate.txt.
set -u
source ~/.cargo/env
REPO=/mnt/c/Users/ism19/Code/ember
DEST=~/ember-gate-tree
rm -rf "$DEST"
mkdir -p "$DEST"
cd "$REPO"
for f in $(git ls-files -co --exclude-standard Cargo.toml Cargo.lock rust-toolchain.toml .github compiler std runtime docs tests examples tools tasks/spec-0.9.9/parts cmake bench/programs bench/probes bench/bigpkg); do
  [ -f "$f" ] || continue
  mkdir -p "$DEST/$(dirname "$f")"
  cp "$f" "$DEST/$f"
done
cd "$DEST"
export EMBER_CC=gcc
cargo test --profile ci --workspace --no-fail-fast > ~/gcc_gate.txt 2>&1
echo "exit $?" >> ~/gcc_gate.txt
grep -E "differs|panicked|test result" ~/gcc_gate.txt | head -60
