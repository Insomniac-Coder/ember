# Audit reproducer for FE-1 (tasks/audit/TASKS.md).
# Expected: 11
# Observed at 46f225a: E1010 cannot find `f` in this scope
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/fe1_capture_callable_parameter.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn compose(f: fn(i32) -> i32) -> i32:
    g = fn(x: i32) -> i32 => f(x) + 1
    return g(1)

fn main():
    println(compose(fn(x) => x * 10))
