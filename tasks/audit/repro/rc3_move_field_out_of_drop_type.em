# Audit reproducer for RC-3 (tasks/audit/TASKS.md).
# Expected: reject: E3010 move out of a field of a type with drop
# Observed at 46f225a: accepted; destructor silently skipped (prints 1, 0)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/rc3_move_field_out_of_drop_type.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct Loud:
    items: Array[i32]
    id: i32

    fn drop(mut self):
        println(100 + self.id)

fn main():
    a: Array[i32] = Array()
    a.push(1)
    l = Loud(a, 7)
    stolen = l.items
    println(stolen.len())
    println(0)
