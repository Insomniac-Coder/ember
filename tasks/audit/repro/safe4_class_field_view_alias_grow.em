# Audit reproducer for SAFE-4 (tasks/audit/TASKS.md).
# Expected: reject statically or exclusivity panic
# Observed at 46f225a: accepted; heap-use-after-free
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe4_class_field_view_alias_grow.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

class Holder:
    items: Array[i32]

    fn grow(mut self):
        i: i32 = 0
        while i < 1000:
            self.items.push(i)
            i = i + 1

fn filled() -> Array[i32]:
    a: Array[i32] = Array()
    a.push(7)
    return a

fn main():
    h = Holder(filled())
    alias = h
    s: Span[i32] = h.items.as_span()
    alias.grow()
    println(s[0])
