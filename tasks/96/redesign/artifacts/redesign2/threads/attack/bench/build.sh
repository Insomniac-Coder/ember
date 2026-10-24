#!/bin/sh
# build.sh: every program in this folder, as timed (clang 22.1.8, -O2). Run from this folder.
CC="C:/Program Files/LLVM/bin/clang.exe"
mkdir -p bin asm runs
for m in 0 1 3 4 5 6 7 8; do "$CC" -O2 -DMODEL=$m claim2.c -o bin/claim2_$m.exe; done
for m in 0 1 3 7 8; do "$CC" -O2 -DMODEL=$m -DSWEEPS=8 claim2.c -o bin/claim2s_$m.exe; done
for m in 0 3 4; do "$CC" -O2 -DMODEL=$m reduce2.c -o bin/reduce2_$m.exe; done
for r in 4 16 64; do c=2000000; [ $r = 16 ] && c=1000000; [ $r = 64 ] && c=250000
  for m in 0 1; do "$CC" -O2 -DMODEL=$m -DREPS=$r -DCALLS=$c "-DNAME0=\"lock2-0-accessor-reps$r\"" "-DNAME1=\"lock2-1-copyout-reps$r\"" lock2.c -o bin/lock2_${m}_r$r.exe; done; done
for m in 0 2 5 6 7 8 9; do "$CC" -O2 -DMODEL=$m ffi2.c -o bin/ffi2_$m.exe; "$CC" -O2 -DMODEL=$m -S -masm=intel ffi2.c -o asm/ffi2_$m.s; done
for f in 0 1 2; do "$CC" -O2 -DFIX=$f ffi_toctou.c -o bin/ffi_toctou_$f.exe; done
for m in 0 1 2 3 4 5; do "$CC" -O2 -DMODEL=$m ripchain.c -o bin/ripchain_$m.exe; "$CC" -O2 -DMODEL=$m -S -masm=intel ripchain.c -o asm/ripchain_$m.s; done
for m in 4 6 7; do "$CC" -O2 -DMODEL=$m -DSTEPRC=1 counts2.c -o bin/counts2_${m}s.exe; done
for m in 0 1 2; do "$CC" -O2 -DMODEL=$m walkbuild.c -o bin/walkbuild_$m.exe; done
for m in 0 4 6; do "$CC" -O2 -DMODEL=$m globals2.c -o bin/globals2_$m.exe; done
for m in 0 1 2; do "$CC" -O2 -DMODEL=$m matches.c -o bin/matches_$m.exe; "$CC" -O2 -DMODEL=$m -DHOT=1 matches.c -o bin/matches_${m}h.exe; done
for m in 0 3 7; do "$CC" -O2 -DMODEL=$m -S -masm=intel claim2.c -o asm/claim2_$m.s; done
"$CC" -O2 -DMODEL=3 -S -masm=intel reduce2.c -o asm/reduce2_3.s
# Timed with (each line one batch; summaries in runs/<name>_summary.txt):
#   ./run_group.sh claim2 claim2-0-plainC claim2_0 claim2_1 claim2_3 claim2_4      (and claim2_b2 .. claim2_b5, claim2s, claim2s_b2)
#   ./run_group.sh reduce2 reduce2-0-local-accumulator reduce2_0 reduce2_3 reduce2_4
#   ./run_group.sh ripchain ripchain-0-loop-rip ripchain_0 ripchain_1 ripchain_2 ripchain_3 ripchain_4 ripchain_5
#   ./run_group.sh lock2 lock2-1-copyout-reps4 lock2_1_r4 lock2_0_r4 lock2_1_r16 lock2_0_r16 lock2_1_r64 lock2_0_r64
#   ./run_group.sh ffi2 ffi2-0-plainC ffi2_0 ffi2_2 ffi2_5 ffi2_6 ffi2_7
#   ./run_group.sh ffi2_b2 ffi2-0-plainC ffi2_0 ffi2_2 ffi2_8 ffi2_5 ffi2_7 ffi2_9
#   ./run_group.sh counts2 counts-4s-atomic-always counts2_4s counts2_6s counts2_7s
#   ./run_group.sh walkbuild walkbuild-0-plain-noshare walkbuild_0 walkbuild_1 walkbuild_2
#   ./run_group.sh globals2 globals2-4-plain-base-pointer globals2_4 globals2_6 globals2_0
#   ./run_group.sh matches matches-0-plainC matches_0 matches_1 matches_2
#   ./run_group.sh matches_hot matches-0-plainC-hot matches_0h matches_1h matches_2h
#   ffi_toctou_2.exe five times pinned (start /affinity C03C03), mains checked each time: runs/ffi_unregister.txt
