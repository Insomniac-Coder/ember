# Audit reproducer for SAFE-1 (tasks/audit/TASKS.md).
# Expected: reject: E3011 (shape O2)
# Observed at 46f225a: accepted; double free
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe1_span_element_into_owned_call.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct R:
    items: Array[i32]

fn filled() -> R:
    a: Array[i32] = Array()
    a.push(1)
    return R(a)

fn keep(owned r: R) -> usize:
    return r.items.len()

fn main():
    xs: Array[R] = Array()
    xs.push(filled())
    s = xs.as_span()
    println(keep(s[0]))
