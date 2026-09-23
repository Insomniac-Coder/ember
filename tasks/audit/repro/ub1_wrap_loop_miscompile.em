# Audit reproducer for UB-1 (tasks/audit/TASKS.md).
# Expected: 2 in release/shipping (loop exits when x + 1 wraps)
# Observed at 46f225a: -1 (compiler assumed x + 1 > x)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub1_wrap_loop_miscompile.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn count_until_wrap(start: i32) -> i32:
    steps: i32 = 0
    x: i32 = start
    while x + 1 > x:
        x = x + 1
        steps = steps + 1
        if steps > 10:
            return -1
    return steps

fn main():
    println(count_until_wrap(2147483645))
