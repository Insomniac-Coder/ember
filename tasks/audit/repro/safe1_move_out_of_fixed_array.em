# Audit reproducer for SAFE-1 (tasks/audit/TASKS.md).
# Expected: reject: E3011 (shape O2)
# Observed at 46f225a: accepted; fixed array treated as moved; leak
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe1_move_out_of_fixed_array.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

struct R:
    items: Array[i32]

fn filled() -> R:
    a: Array[i32] = Array()
    a.push(1)
    return R(a)

fn main():
    arr = [filled(), filled()]
    r = arr[0]
    println(r.items.len())
