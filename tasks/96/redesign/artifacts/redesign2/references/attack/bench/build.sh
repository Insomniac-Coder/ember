#!/bin/sh
# Builds every timing program of phase3b-references-attack.md into out/. Run from this directory: sh build.sh
set -e
CC="C:/Program Files/LLVM/bin/clang.exe"
H=../../../../redesign/bench          # harness.h
B=../../bench                         # the references group's sources (words_common.h, state*.c, *.rs)
O=out
"$CC" -O2 -I $H -I $B words_ember.c -o $O/words_ember.exe
"$CC" -O2 -I $H -I $B $B/words_plain.c -o $O/words_plain.exe
"$CC" -O2 -I $H $B/state.c -o $O/state.exe
"$CC" -O2 -I $H $B/state_checked.c -o $O/state_checked.exe
"$CC" -O2 -I $H state_inplace.c -o $O/state_inplace.exe
"$CC" -O2 -I $H state_inplace.c -S -o $O/state_inplace.s
"$CC" -O2 -I $H $B/state_checked.c -S -o $O/state_checked.s
for v in REUSE COPY SHRINK HALF; do
  l=$(echo $v | tr A-Z a-z)
  "$CC" -O2 -I $H -D$v keep.c -o $O/keep_${l}_short.exe
  "$CC" -O2 -I $H -D$v -DTRIM keep.c -o $O/keep_${l}_trim.exe
  "$CC" -O2 -I $H -D$v -DLONGLINES keep.c -o $O/keep_${l}_long.exe
done
python -I make_get_attack.py
"$CC" -O2 -I $H -DPLAIN get_attack.c -o $O/get_plain.exe
"$CC" -O2 -I $H -DTYPED get_attack.c -o $O/get_typed.exe
"$CC" -O2 -I $H -DTYPED -DSLOTVAR get_attack.c -o $O/get_typed_slotvar.exe
"$CC" -O2 -I $H -DTYPED -DSLOTVAR get_attack.c -S -o $O/get_typed_slotvar.s
"$CC" -O2 -I $H -DCAST get_attack.c -o $O/get_cast.exe
cd $B
rustc -C opt-level=2 --edition 2021 words_rust_vecstr.rs -o ../attack/bench/$O/words_rust_vecstr.exe
rustc -C opt-level=2 --edition 2021 state_rust.rs -o ../attack/bench/$O/state_rust.exe
rustc -C opt-level=2 --edition 2021 state_rust_inplace.rs -o ../attack/bench/$O/state_rust_inplace.exe
rustc -C opt-level=2 --edition 2021 get_rust.rs -o ../attack/bench/$O/get_rust.exe
echo built
