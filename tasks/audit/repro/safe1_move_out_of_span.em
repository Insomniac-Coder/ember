# Audit reproducer for SAFE-1 (tasks/audit/TASKS.md).
# Expected: reject: E3011 (shape O2) move out of a span element
# Observed at 46f225a: accepted; double free in debug/release/shipping
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe1_move_out_of_span.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct R:
    items: Array[i32]

fn main():
    xs: Array[R] = Array()
    a: Array[i32] = Array()
    a.push(1)
    xs.push(R(a))
    s: Span[R] = xs.as_span()
    r = s[0]
    println(r.items.len())
