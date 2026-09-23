# Audit reproducer for FE-3 (tasks/audit/TASKS.md).
# Expected: 25, or a clear "not supported in this phase" diagnostic
# Observed at 46f225a: "the C compiler failed": undeclared ember_i128
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/fe3_i128_codegen.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    y: i128 = 5
    z: i128 = y * y
    println(z)
