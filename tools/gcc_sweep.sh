#!/usr/bin/env bash
# The gcc C warning sweep in WSL (tools/push_gate.ps1's last sweep). Builds the compiler on Linux's disk
# (bench/gcc/make_tree.sh), adds the programs and the tool, sweeps, deletes the copy.
set -eu
REPO="$(cd "$(dirname "$(readlink -f "${GATE_REPO:-$0}")")/.." 2>/dev/null && pwd)"
[ -d "$REPO/tests" ] || REPO=/mnt/c/Users/ism19/Code/ember
bash <(tr -d '\r' < "$REPO/bench/gcc/make_tree.sh")
DEST=~/ember-bench-tree
cd "$REPO"
for f in $(git ls-files -co --exclude-standard tests examples bench/programs bench/probes tools); do
  [ -f "$f" ] || continue
  mkdir -p "$DEST/$(dirname "$f")"
  cp "$f" "$DEST/$f"
done
cd "$DEST"
python3 tools/c_warning_sweep.py gcc --jobs 12 || echo "sweep exit $?"
cd ~ && rm -rf "$DEST"
