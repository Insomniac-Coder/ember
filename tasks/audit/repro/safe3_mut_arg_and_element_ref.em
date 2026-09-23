# Audit reproducer for SAFE-3 (tasks/audit/TASKS.md).
# Expected: reject: E3021/B3
# Observed at 46f225a: accepted; heap-use-after-free
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/safe3_mut_arg_and_element_ref.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn grow_then_read(mut xs: Array[i32], r: ref i32) -> i32:
    i: i32 = 0
    while i < 1000:
        xs.push(i)
        i = i + 1
    return r

fn main():
    xs: Array[i32] = Array()
    xs.push(42)
    println(grow_then_read(xs, ref xs[0]))
