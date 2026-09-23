# Audit reproducer for SAFE-6 (tasks/audit/TASKS.md).
# Expected: panic (allocation failure / capacity overflow) or reject
# Observed at 46f225a: heap-buffer-overflow ("malloc(): corrupted top size")
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe6_arena_capacity_overflow.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    n: usize = 18446744073709551615
    arena = Arena.with_capacity(n)
    i: i32 = 0
    total: i32 = 0
    while i < 64:
        r = arena.alloc(i)
        total = total + r
        i = i + 1
    println(total)
