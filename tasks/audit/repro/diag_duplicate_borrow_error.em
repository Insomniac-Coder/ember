# Audit reproducer for DIAG-1 (tasks/audit/TASKS.md).
# Expected: one E3021
# Observed at 46f225a: the same E3021 printed twice ("2 error(s)")
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/diag_duplicate_borrow_error.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    b: Box[Array[i32]] = Box(Array())
    r = b.get()
    b = Box(Array())
    println(r.len())
