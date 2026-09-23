# Audit reproducer for UB-3 (tasks/audit/TASKS.md).
# Expected: debug: panic (amount out of range); release: masked
# Observed at 46f225a: debug check is signed n >= 32, so a >> -1 runs (UB)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub3_negative_shift_amount.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    a: i32 = 1
    n: i32 = -1
    println(a >> n)
