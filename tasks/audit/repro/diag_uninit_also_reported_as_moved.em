# Audit reproducer for DIAG-3 (tasks/audit/TASKS.md).
# Expected: one E3050
# Observed at 46f225a: E3050 plus a spurious E3040 "may have been moved out of"
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/diag_uninit_also_reported_as_moved.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    x: i32
    if false:
        x = 1
    println(x)
