#!/usr/bin/env bash
# build.sh: every program timed for tasks/96/redesign/phase3b-salvage.md, with the exact commands.
# Run from this folder (Git Bash). C: clang -O2; Rust: rustc -C opt-level=2 --edition 2021.
set -e
cd "$(dirname "$0")"
CC="/c/Program Files/LLVM/bin/clang.exe"
I="-I ../../redesign/bench"          # the shared harness.h (sh.h includes it)
c() { "$CC" -O2 $I "$@"; }
mkdir -p bin
# plain C baselines and variants
c plain/w1_group.c -o bin/plain_w1_group.exe
c plain/w1_arch.c -o bin/plain_w1_arch.exe
c ../../redesign/bench/plain/w1_sparse.c -o bin/plain_w1_sparse.exe
c plain/w2.c -o bin/plain_w2.exe
c plain/w2_emberlist.c -o bin/plain_w2_emberlist.exe
c plain/w3.c -o bin/plain_w3.exe
c plain/w4.c -o bin/plain_w4.exe
c plain/w4_env32.c -o bin/plain_w4_env32.exe
# safe Rust baselines
for w in w1_group w2 w3 w4; do rustc -C opt-level=2 --edition 2021 rust/$w.rs -o bin/rust_$w.exe; done
# "copy instead of share", salvaged
c d1/w1_group.c -o bin/d1_w1_group.exe
c -DD1_PROVEN d1/w1_group.c -o bin/d1_w1_group_proven.exe
c -DNOINLINE d1/w1_group.c -o bin/d1_w1_group_noinline.exe
c d1/w2.c -o bin/d1_w2.exe
c -DD1_PROVEN d1/w2.c -o bin/d1_w2_proven.exe
c d1/w3.c -o bin/d1_w3.exe
c -DD1_MALLOC d1/w3.c -o bin/d1_w3_malloc.exe
c -DD1_TYPEPTR d1/w3.c -o bin/d1_w3_typeptr.exe
c d1/w4.c -o bin/d1_w4.exe
c -DMALLOCBOX d1/w4.c -o bin/d1_w4_callee.exe
c -DONECLOSURE d1/w4.c -o bin/d1_w4_oneclosure.exe
c -DARRAY3 d1/w4.c -o bin/d1_w4_array3.exe
# "checked pointers", salvaged
c d2/w1_group.c -o bin/d2_w1_group.exe
c d2/w2.c -o bin/d2_w2.exe
c -DCHECKED d2/w2.c -o bin/d2_w2_checked.exe
c d2/w3.c -o bin/d2_w3.exe
c d2/w4.c -o bin/d2_w4.exe
# "today's rules, but never an error", salvaged
c d3/w1_group.c -o bin/d3_w1_group.exe
c -DNOBOUNDS d3/w1_group.c -o bin/d3_w1_group_nobounds.exe
c d3/w2.c -o bin/d3_w2.exe
c -DUNPROVEN d3/w2.c -o bin/d3_w2_unproven.exe
c d3/w3.c -o bin/d3_w3.exe
c d3/w4.c -o bin/d3_w4.exe
# phase 3's models, unchanged, rebuilt here so they run in the same batch
for m in D1 D2 D3; do for w in w1 w2 w3 w4; do c ../../redesign/bench/$m/$w.c -o bin/orig_${m}_$w.exe; done; done
c ../../redesign/break-D2/w2_listobj.c -o bin/orig_D2_w2_listobj.exe
c ../../redesign/break-D2/w3_scan.c -o bin/orig_D2_w3_scan.exe
c ../../redesign/break-D2/w4_listobj.c -o bin/orig_D2_w4_listobj.exe
echo built
