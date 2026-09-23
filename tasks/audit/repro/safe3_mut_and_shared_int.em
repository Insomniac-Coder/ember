# Audit reproducer for SAFE-3 (tasks/audit/TASKS.md).
# Expected: reject: E3021/B3
# Observed at 46f225a: accepted; shared ref observes the write (prints 1)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe3_mut_and_shared_int.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn both(mut a: i32, b: ref i32) -> i32:
    a = 1
    return b

fn main():
    n: i32 = 0
    println(both(n, ref n))
