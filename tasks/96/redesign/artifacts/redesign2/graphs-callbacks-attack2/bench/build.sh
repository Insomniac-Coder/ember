#!/bin/sh
# build.sh: builds every timed program into out/ from its source. The graphs group's models are built
# from their own files (read only) with the switches their report names. Run from this folder.
set -e
CC="/c/Program Files/LLVM/bin/clang.exe"
INC="-I ../../../redesign/bench"
G=../../graphs-callbacks/bench
mkdir -p out
# the tree workload (W2), the group's sources
"$CC" -O2 $INC -DSLAB -DARRAY24 $G/w2/w2_plain_malloc.c -o out/w2_plain_slab_array24.exe
"$CC" -O2 $INC -DSLAB $G/w2/w2_backlink.c -o out/w2_backlink_slab.exe
"$CC" -O2 $INC -DSLAB -DNOHEADER $G/w2/w2_backlink.c -o out/w2_backlink_slab_noheader.exe
"$CC" -O2 $INC $G/w2/w2_plain_keep.c -o out/w2_plain_keep.exe
rustc -C opt-level=2 --edition 2021 $G/w2/w2_rust_index.rs -o out/w2_rust_index.exe
# the tree workload with a visitor call on every step of the walk (mine)
"$CC" -O2 $INC -DSLAB -DARRAY24 w2v_plain.c -o out/w2v_plain.exe
"$CC" -O2 $INC -DSLAB w2v_backlink.c -o out/w2v_backlink.exe
"$CC" -O2 $INC -DSLAB -DRETAIN w2v_backlink.c -o out/w2v_backlink_retain.exe
"$CC" -O2 $INC -DSLAB -DHAZARD w2v_backlink.c -o out/w2v_backlink_hazard.exe
# the callback workload (W4), the group's sources, and the same-work baseline (mine)
"$CC" -O2 $INC $G/w4/w4_plain.c -o out/w4_plain.exe
"$CC" -O2 $INC -DSLAB $G/w4/w4_plain.c -o out/w4_plain_slab.exe
"$CC" -O2 $INC -DSLAB w4_plain_same.c -o out/w4_plain_same_slab.exe
"$CC" -O2 $INC $G/w4/w4_ember_point.c -o out/w4_ember_point.exe
"$CC" -O2 $INC -DBOXED $G/w4/w4_ember_point.c -o out/w4_ember_point_boxed.exe
rustc -C opt-level=2 --edition 2021 $G/w4/w4_rust.rs -o out/w4_rust.exe
# a boxed captured variable in a hot loop (mine)
for v in C_LOCAL C_ESCAPED; do "$CC" -O2 $INC -D$v boxloop.c -o out/boxloop_$(echo $v | tr A-Z a-z).exe; done
for v in EMBER_NAIVE EMBER_PROMOTED; do "$CC" -O2 $INC -D$v boxloop.c rtalloc.c -o out/boxloop_$(echo $v | tr A-Z a-z).exe; done
rustc -C opt-level=2 --edition 2021 boxloop.rs -o out/boxloop_rust.exe
# a weak observer list under churn (mine)
for v in PLAIN PLAIN_TRACKED SLOTS ARRAY; do "$CC" -O2 $INC -D$v weakarr.c -o out/weakarr_$(echo $v | tr A-Z a-z).exe; done
"$CC" -O2 $INC -DARRAY -DCACHE1 weakarr.c -o out/weakarr_array_cache1.exe
"$CC" -O2 $INC -DBUFFER weakarr.c -o out/weakarr_buffer.exe
"$CC" -O2 $INC -DBUFFER -DCACHE1 weakarr.c -o out/weakarr_buffer_cache1.exe
"$CC" -O2 $INC -DBUFFER -DDIRECT weakarr.c -o out/weakarr_buffer_direct.exe
# the filter idiom on a back-linked list (mine)
for v in PLAIN LITERAL ORDERED; do
  "$CC" -O2 $INC -D$v filter.c -o out/filter_$(echo $v | tr A-Z a-z).exe
  "$CC" -O2 $INC -D$v -DNCH=20000 filter.c -o out/filter20k_$(echo $v | tr A-Z a-z).exe
done
"$CC" -O2 $INC -DORDERED -DNOCOUNT filter.c -o out/filter_ordered_nocount.exe
"$CC" -O2 $INC -DORDERED -DNOCOUNT -DNCH=20000 filter.c -o out/filter20k_ordered_nocount.exe

rustc -C opt-level=2 --edition 2021 weakarr.rs -o out/weakarr_rust.exe
echo built
# a graph whose edges are kept in step both ways (mine)
"$CC" -O2 $INC graph.c -o out/graph_plain.exe
"$CC" -O2 $INC -DEMBER graph.c -o out/graph_ember.exe
echo built graph
