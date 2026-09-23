# Audit reproducer for SAFE-3 (tasks/audit/TASKS.md).
# Expected: reject: E3021/B3
# Observed at 46f225a: accepted; heap-use-after-free
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe3_mut_self_and_span_of_field.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct Bag:
    items: Array[i32]

    fn absorb(mut self, s: Span[i32]) -> i32:
        i: i32 = 0
        while i < 1000:
            self.items.push(i)
            i = i + 1
        return s[0]

fn main():
    b = Bag(Array())
    b.items.push(42)
    println(b.absorb(b.items.as_span()))
