# Audit reproducer for UB-5 (tasks/audit/TASKS.md).
# Expected: 3
# Observed at 46f225a: debug: panic integer overflow; release: infinite loop
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub5_inclusive_range_to_max.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    total: i32 = 0
    for i in 2147483645..=2147483647:
        total = total + 1
    println(total)
