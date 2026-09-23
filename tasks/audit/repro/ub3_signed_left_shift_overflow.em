# Audit reproducer for UB-3 (tasks/audit/TASKS.md).
# Expected: no C-level UB in any profile
# Observed at 46f225a: C emits signed << that overflows (UB)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub3_signed_left_shift_overflow.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    a: i32 = 1073741824
    n: i32 = 1
    println(a << n)
