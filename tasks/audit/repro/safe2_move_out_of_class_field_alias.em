# Audit reproducer for SAFE-2 (tasks/audit/TASKS.md).
# Expected: reject: E3012 (shape O2) move out of a class field
# Observed at 46f225a: accepted; heap-use-after-free through the alias handle
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe2_move_out_of_class_field_alias.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

class Holder:
    items: Array[i32]

fn main():
    a: Array[i32] = Array()
    a.push(1)
    h = Holder(a)
    other = h
    stolen = h.items
    i: i32 = 0
    while i < 1000:
        stolen.push(i)
        i = i + 1
    println(other.items[0])
    println(other.items.len())
