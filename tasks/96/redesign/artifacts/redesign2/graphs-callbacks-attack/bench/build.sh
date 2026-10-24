#!/bin/sh
# Builds every timed program into out/. The report's originals are rebuilt from their own sources
# (graphs-callbacks/bench/...) with the same commands the report gives, so the binaries match them.
set -e
cd "$(dirname "$0")"
CC="/c/Program Files/LLVM/bin/clang.exe"
INC="-I ../../../redesign/bench -I ../../graphs-callbacks/bench"
G=../../graphs-callbacks/bench
# tree workload (W2)
"$CC" -O2 $INC $G/w2/w2_plain_keep.c -o out/w2_plain_keep.exe
"$CC" -O2 $INC -DSLAB -DARRAY24 $G/w2/w2_plain_malloc.c -o out/w2_plain_slab_array24.exe
"$CC" -O2 $INC -DSLAB $G/w2/w2_backlink.c -o out/w2_backlink_slab.exe
"$CC" -O2 $INC -DSLAB -DNOHEADER $G/w2/w2_backlink.c -o out/w2_backlink_slab_noheader.exe
"$CC" -O2 $INC -DNOHEADER -DPLAINNODES -DPLAINCHILD w2_backlink_parts.c -o out/w2_bl_nohdr_plainnodes_plainchild.exe
"$CC" -O2 $INC -DNOHEADER -DPLAINNODES w2_backlink_parts.c -o out/w2_bl_nohdr_plainnodes.exe
"$CC" -O2 $INC -DPLAINNODES w2_backlink_parts.c -o out/w2_bl_plainnodes.exe
rustc -C opt-level=2 --edition 2021 $G/w2/w2_rust_index.rs -o out/w2_rust_index.exe
# callback workload (W4) and its log variant
"$CC" -O2 $INC $G/w4/w4_plain.c -o out/w4_plain.exe
"$CC" -O2 $INC -DSLAB $G/w4/w4_plain.c -o out/w4_plain_slab.exe
"$CC" -O2 $INC $G/w4/w4_ember_point.c -o out/w4_ember_point.exe
"$CC" -O2 $INC -DBOXED $G/w4/w4_ember_point.c -o out/w4_ember_point_boxed.exe
rustc -C opt-level=2 --edition 2021 $G/w4/w4_rust.rs -o out/w4_rust.exe
"$CC" -O2 $INC w4log_plain.c -o out/w4log_plain_slab.exe
"$CC" -O2 $INC w4log_ember.c -o out/w4log_ember.exe
"$CC" -O2 $INC -DNORETAIN w4log_ember.c -o out/w4log_ember_noretain.exe
rustc -C opt-level=2 --edition 2021 w4log_rust.rs -o out/w4log_rust.exe


"$CC" -O2 $INC -DHDR4 w2_backlink_parts.c -o out/w2_bl_hdr4.exe
rm -f out/*.pdb
echo built
