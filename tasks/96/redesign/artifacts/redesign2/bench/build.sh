#!/usr/bin/env bash
# build.sh: every program timed for tasks/96/redesign/phase3b-measurements.md. Run from Git Bash.
# C: clang -O2. Rust: rustc -C opt-level=2 --edition 2021. Output: bin/.
set -e
R=/c/Users/ism19/Code/ember/target/redesign2
B=$R/bench; S=$R/salvage; O=$B/bin
CC="/c/Program Files/LLVM/bin/clang.exe"
H="-I /c/Users/ism19/Code/ember/target/redesign/bench"
c() { "$CC" -O2 $H "$@"; }
mkdir -p $O
# plain C baselines
c $B/w1_group.c -o $O/plain_w1_group.exe
c -DLOCAL $B/w1_group.c -o $O/plain_w1_group_local.exe
c $B/w1_arch.c  -o $O/plain_w1_arch.exe
c $B/w2.c -o $O/plain_w2.exe
c $B/w3.c -o $O/plain_w3.exe
c $B/w4.c -o $O/plain_w4.exe
# plain C in the same layout as the combined model (named by phase3b-combined.md section 7)
c -DSLAB -DARRAY24 $R/graphs-callbacks/bench/w2/w2_plain_malloc.c -o $O/plain_w2_same.exe
c -DSLAB $R/graphs-callbacks-attack2/bench/w4_plain_same.c -o $O/plain_w4_same.exe
c $S/plain/w2_emberlist.c -I $S -o $O/plain_w2_emberlist.exe
# safe Rust baselines
for w in w1 w2 w3 w4 w4_return; do rustc -C opt-level=2 --edition 2021 $B/rust_$w.rs -o $O/rust_$w.exe; done
# the combined model, and the two W3 alternatives
c $R/combined/w1.c -o $O/combined_w1.exe
c $R/combined/w2.c -o $O/combined_w2.exe
c $R/combined/w3.c -o $O/combined_w3.exe
c $R/combined/w4.c -o $O/combined_w4.exe
c $R/combined/w3_registration.c -o $O/combined_w3_registration.exe
c $R/combined-attack/w3_beside.c -o $O/combined_w3_beside.exe
# the salvaged designs (phase3b-salvage.md, all "partly saved"), default builds
for d in d1 d2 d3; do
  c $S/$d/w1_group.c -o $O/${d}_w1.exe
  for w in w2 w3 w4; do c $S/$d/$w.c -o $O/${d}_$w.exe; done
done
# W2 without the whole-program fact the salvage's d2 and d3 W2 speeds rest on
c -DCHECKED $S/d2/w2.c -o $O/d2_w2_checked.exe
c -DUNPROVEN $S/d3/w2.c -o $O/d3_w2_unproven.exe
rm -f $O/*.pdb
echo built
