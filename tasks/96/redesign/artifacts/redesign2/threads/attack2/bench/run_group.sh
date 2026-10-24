#!/bin/sh
# run_group.sh <out-name> <base-program-name> <exe> ...: 5 pinned rounds with the ownership group's run.ps1
# (mains checked before every round; each round runs every program once, one at a time, in the given order,
# pinned to the P-cores with `start /affinity C03C03`), then its medians.py (median of 5, ratio to the base).
# The same runner the threads group and the first attack used.
cd "$(dirname "$0")"
out="runs/$1.txt"; base="$2"; shift 2
list=""
for n in "$@"; do list="$list,$(cygpath -w "$PWD/bin/$n.exe")"; done
rm -f "$out"
powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w ../../../ownership/bench/run.ps1)" -Out "$(cygpath -w "$PWD/$out")" -Rounds 5 -Exes "${list#,}"
python ../../../ownership/bench/medians.py "$out" "$base" | tee "runs/$(basename "$out" .txt)_summary.txt"
