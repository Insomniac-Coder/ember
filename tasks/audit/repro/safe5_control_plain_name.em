# Audit reproducer for SAFE-5 (tasks/audit/TASKS.md).
# Expected: reject: E3022
# Observed at 46f225a: rejected with E3022 (correct control)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe5_control_plain_name.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct Plain:
    items: Array[i32]

fn main():
    e = Plain(Array())
    e.items.push(1)
    a: ref mut Array[i32] = ref mut e.items
    b: ref mut Array[i32] = ref mut e.items
    s: Span[i32] = a.as_span()
    i: i32 = 0
    while i < 1000:
        b.push(i)
        i = i + 1
    println(s[0])
