# Audit reproducer for DIAG-2 (tasks/audit/TASKS.md).
# Expected: user-facing type names
# Observed at 46f225a: expected `fn() -> i32`, found `closure0_env`
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/diag_internal_type_names.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn make() -> fn() -> i32:
    n: i32 = 3
    r: ref i32 = ref n
    return fn() -> i32 => r

fn main():
    f = make()
    println(f())
