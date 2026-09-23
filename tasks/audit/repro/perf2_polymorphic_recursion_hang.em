# Audit reproducer for PERF-2 (tasks/audit/TASKS.md).
# Expected: a monomorphisation-depth diagnostic
# Observed at 46f225a: compiler never terminates
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/perf2_polymorphic_recursion_hang.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct W[T]:
    v: T

fn grow[T](x: T, n: i32) -> i32:
    if n == 0:
        return 0
    return grow(W(x), n - 1)

fn main():
    println(grow(1, 3))
