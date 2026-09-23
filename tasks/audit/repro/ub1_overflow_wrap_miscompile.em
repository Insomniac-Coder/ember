# Audit reproducer for UB-1 (tasks/audit/TASKS.md).
# Expected: false in every profile (two's-complement wrap)
# Observed at 46f225a: debug false; release/shipping true (signed overflow UB in C)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub1_overflow_wrap_miscompile.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

@overflow(wrap)
fn grows(x: i32) -> bool:
    return x + 1 > x

fn main():
    println(grows(2147483647))
