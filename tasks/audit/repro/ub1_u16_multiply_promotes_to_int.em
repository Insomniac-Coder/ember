# Audit reproducer for UB-1 (tasks/audit/TASKS.md).
# Expected: release/shipping: 1 without UB
# Observed at 46f225a: result correct by luck; C computes int 65535*65535 (UB)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub1_u16_multiply_promotes_to_int.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    a: u16 = 65535
    b: u16 = 65535
    println(a * b)
