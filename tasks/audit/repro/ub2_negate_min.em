# Audit reproducer for UB-2 (tasks/audit/TASKS.md).
# Expected: debug: panic integer overflow; release: wrap without UB
# Observed at 46f225a: no panic in debug; C emits (-x) on INT32_MIN
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub2_negate_min.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    a: i32 = -2147483648
    println(-a)
