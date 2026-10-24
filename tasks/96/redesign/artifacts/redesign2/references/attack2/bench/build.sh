#!/bin/sh
# Builds the timing programs of the second attack on phase3b-references.md into out/. Run from this folder: sh build.sh
set -e
CC="C:/Program Files/LLVM/bin/clang.exe"
H=../../../../redesign/bench          # the shared harness.h (it calls SetErrorMode)
B=../../bench                         # the references group's own sources, built unchanged
O=out
"$CC" -O2 -I $H -DPLAIN $B/get.c -o $O/get_plain.exe
"$CC" -O2 -I $H -DTYPED $B/get.c -o $O/get_typed.exe
"$CC" -O2 -I $H -DCAST  $B/get.c -o $O/get_cast.exe
"$CC" -O2 -I $H -DTABLE get2.c -o $O/get2_table.exe
"$CC" -O2 -I $H -DTABLE -DSLOTVAR get2.c -o $O/get2_table_slotvar.exe
"$CC" -O2 -I $H -DCAST get2.c -o $O/get2_cast.exe
"$CC" -O2 -I $H -DTABLE -DSLOTHOIST get2.c -o $O/get2_table_slothoist.exe
"$CC" -O2 -I $H -DTABLE -DSLOTHOIST get2.c -S -o $O/get2_table_slothoist.s
"$CC" -O2 -I $H -DTABLE get2.c -S -o $O/get2_table.s
"$CC" -O2 -I $H -DCAST get2.c -S -o $O/get2_cast.s
"$CC" -O2 -I $H -DTABLE -DSLOTVAR get2.c -S -o $O/get2_table_slotvar.s
for v in HALF_LEN HALF_CAP COPY; do
  l=$(echo $v | tr A-Z a-z)
  "$CC" -O2 -I $H -D$v keep2.c -o $O/keep2_${l}.exe
done
rustc -C opt-level=2 --edition 2021 $B/get_rust.rs -o $O/get_rust.exe
echo built
