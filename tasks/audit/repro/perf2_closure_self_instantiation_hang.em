# Audit reproducer for PERF-2 (tasks/audit/TASKS.md).
# Expected: a diagnostic
# Observed at 46f225a: compiler never terminates
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/perf2_closure_self_instantiation_hang.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn invoke(f: fn() -> i32) -> i32:
    return f()

    counter = 0
    increment = fn() -> i32:
        counter = counter + 1
        return counter
    println(invoke(increment))

fn main():
    pass
