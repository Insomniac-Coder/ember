# Audit reproducer for SAFE-5 (tasks/audit/TASKS.md).
# Expected: reject: E3022 (same as the control)
# Observed at 46f225a: accepted; heap-use-after-free
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe5_struct_named_refcell_prefix.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct RefCell_Evil:
    items: Array[i32]

fn main():
    e = RefCell_Evil(Array())
    e.items.push(1)
    a: ref mut Array[i32] = ref mut e.items
    b: ref mut Array[i32] = ref mut e.items
    s: Span[i32] = a.as_span()
    i: i32 = 0
    while i < 1000:
        b.push(i)
        i = i + 1
    println(s[0])
