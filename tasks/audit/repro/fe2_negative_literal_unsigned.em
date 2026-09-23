# Audit reproducer for FE-2 (tasks/audit/TASKS.md).
# Expected: reject: E2010 literal does not fit
# Observed at 46f225a: accepted; prints 4294967295
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/fe2_negative_literal_unsigned.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    x: u32 = -1
    println(x)
