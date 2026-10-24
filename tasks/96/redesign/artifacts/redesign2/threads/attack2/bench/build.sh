#!/bin/sh
# build.sh: every program of the second attack, as timed (clang 22.1.8, -O2). Run from this folder.
# Programs from the first attack (../../attack/bench) are built from their sources unchanged.
CC="C:/Program Files/LLVM/bin/clang.exe"
mkdir -p bin asm runs
for m in 0 1 2 3 4 5 6; do "$CC" -O2 -DMODEL=$m claim3.c -o bin/claim3_$m.exe; done
for m in 0 1 2 3 4; do "$CC" -O2 -DMODEL=$m -DSWEEPS=8 claim3.c -o bin/claim3s_$m.exe; done
"$CC" -O2 -DMODEL=1 -S -masm=intel claim3.c -o asm/claim3_1.s
"$CC" -O2 -DMODEL=6 -S -masm=intel claim3.c -o asm/claim3_6.s
for m in 0 7; do "$CC" -O2 -DMODEL=$m ../../attack/bench/claim2.c -o bin/claim2_$m.exe
                 "$CC" -O2 -DMODEL=$m -DSWEEPS=8 ../../attack/bench/claim2.c -o bin/claim2s_$m.exe; done
for m in 0 8 9 10 11; do "$CC" -O2 -DMODEL=$m ffi3.c -o bin/ffi3_$m.exe; "$CC" -O2 -DMODEL=$m -S -masm=intel ffi3.c -o asm/ffi3_$m.s; done
"$CC" -O2 -DMODEL=7 ../../attack/bench/ffi2.c -o bin/ffi2_7.exe
"$CC" -O2 tagfault.c -o bin/tagfault.exe
for e in 64 65536; do for m in 0 1 2 3; do "$CC" -O2 -DMODEL=$m -DE=$e retarget.c -o bin/retarget_${m}_e$e.exe; done; done
for m in 0 1 2 3; do "$CC" -O2 -DMODEL=$m -DE=64 -DSEARCH=1 retarget.c -o bin/retarget_${m}_e64s.exe; done
for m in 0 1 2 3; do "$CC" -O2 -DMODEL=$m hist.c -o bin/hist_$m.exe; done
for m in 0 1 2 5 6 7; do "$CC" -O2 -DMODEL=$m gen2.c -o bin/gen2_$m.exe; "$CC" -O2 -DMODEL=$m -S -masm=intel gen2.c -o asm/gen2_$m.s; done
for m in 0 1 2 3 4 5; do "$CC" -O2 -DMODEL=$m ../../attack/bench/ripchain.c -o bin/ripchain_$m.exe; done
# Functional checks (pinned, mains checked, not timings):
#   claim3_5.exe and claim3_6.exe, three times each, `start /b /wait /affinity C03C03`: runs/claim3_dups.txt
#   tagfault.exe once: runs/tagfault.txt
#   diff of `tramp` in asm/ffi3_8.s against asm/ffi3_10.s, and asm/ffi3_9.s against asm/ffi3_11.s: no difference
# Timed with (each line one batch; summaries in runs/<name>_summary.txt):
#   ./run_group.sh claim3 claim3-0-plainC claim3_0 claim3_1 claim3_2 claim3_3 claim3_4 claim2_0 claim2_7
#   ./run_group.sh claim3_b2 claim3-0-plainC claim2_7 claim2_0 claim3_4 claim3_3 claim3_2 claim3_1 claim3_0
#   ./run_group.sh claim3s claim3-0-plainC claim3s_0 claim3s_1 claim3s_2 claim3s_3 claim3s_4
#   ./run_group.sh claim3s_b2 claim3-0-plainC claim3s_0 claim3s_1 claim2s_0 claim2s_7
#   ./run_group.sh ffi3 ffi3-0-plainC ffi3_0 ffi3_8 ffi3_10 ffi3_9 ffi3_11
#   ./run_group.sh ffi3_b2 ffi3-0-plainC ffi3_0 ffi2_7 ffi3_9 ffi3_11 ffi3_10
#   ./run_group.sh retarget retarget-0-plainC-e64-search retarget_0_e64s retarget_1_e64s retarget_2_e64s retarget_3_e64s retarget_0_e64 retarget_1_e64 retarget_2_e64 retarget_3_e64 retarget_0_e65536 retarget_1_e65536 retarget_2_e65536 retarget_3_e65536
#   ./run_group.sh hist hist-1-element-reduction hist_1 hist_0 hist_2 hist_3
#   ./run_group.sh gen2 gen2-0-plainC gen2_0 gen2_2 gen2_5 gen2_6 gen2_1 gen2_7
#   ./run_group.sh ripchain ripchain-0-loop-rip ripchain_0 ripchain_1 ripchain_2 ripchain_3 ripchain_4 ripchain_5
