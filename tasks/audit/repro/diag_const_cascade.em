# Audit reproducer for DIAG-4 (tasks/audit/TASKS.md).
# Expected: one E2130
# Observed at 46f225a: E2130 plus cascaded E1010 "cannot find `Q`" (DIA-14)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/diag_const_cascade.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

const Z: i32 = 0
const Q: i32 = 10 / Z

fn main():
    println(Q)
