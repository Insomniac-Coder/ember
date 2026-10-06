#!/usr/bin/env bash
# In WSL: copy the repository's compiler, std and runtime (tracked and new files) to
# ~/ember-bench-tree, on Linux's own disk (building from /mnt/c is slow), and build the release
# compiler there for run_gcc.py. The copy is made fresh each time; run_all.sh deletes it after.
set -eu
source ~/.cargo/env
REPO=/mnt/c/Users/ism19/Code/ember
DEST=~/ember-bench-tree
rm -rf "$DEST"
mkdir -p "$DEST"
cd "$REPO"
for f in $(git ls-files -co --exclude-standard Cargo.toml Cargo.lock rust-toolchain.toml compiler std runtime); do
  [ -f "$f" ] || continue
  mkdir -p "$DEST/$(dirname "$f")"
  tr -d '\r' < "$f" > "$DEST/$f"
done
cd "$DEST"
cargo build --release -q 2>&1 | grep -E "^(error|warning)" -A5 | head -20
ls -la target/release/ember
