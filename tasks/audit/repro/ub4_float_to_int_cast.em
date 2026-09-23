# Audit reproducer for UB-4 (tasks/audit/TASKS.md).
# Expected: 2147483647, -2147483648, 0, 255 (saturating, NaN -> 0)
# Observed at 46f225a: debug -2147483648 x3, 0; release prints values outside i32 range
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/ub4_float_to_int_cast.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn to_int(x: f64) -> i32:
    return x as i32

fn main():
    println(to_int(3000000000.0))
    println(to_int(-3000000000.0))
    nan: f64 = 0.0 / 0.0
    println(to_int(nan))
    big: f32 = 1e20
    println(big as u8)
