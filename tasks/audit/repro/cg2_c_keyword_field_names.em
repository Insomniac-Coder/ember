# Audit reproducer for CG-2 (tasks/audit/TASKS.md).
# Expected: 10
# Observed at 46f225a: "the C compiler failed": field names emitted verbatim
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/cg2_c_keyword_field_names.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct S:
    int: i32
    NULL: i32
    stdout: i32
    errno: i32

fn main():
    s = S(1, 2, 3, 4)
    println(s.int + s.NULL + s.stdout + s.errno)
