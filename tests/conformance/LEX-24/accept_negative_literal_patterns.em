#$ test: run-pass
#$ rules: LEX-24, TYP-1
#$ stdout: least minus five zero other
#$ top other least minus one other max
# D-310 — III.6's `literal_pattern := ["-"] INT`: a pattern may be a negative
# literal, a signed type's least value included (`[LEX-24]`). A `u128` above
# `i128`'s range and a `u64` above `i64`'s match by value too.

fn describe(x: int) -> str:
    match x:
        -9223372036854775808 => return "least"
        -5 => return "minus five"
        0 => return "zero"
        _ => return "other"

fn top(b: u64) -> str:
    match b:
        18446744073709551615 => return "top"
        _ => return "other"

fn wide(x: i128) -> str:
    match x:
        -170141183460469231731687303715884105728 => return "least"
        -1 => return "minus one"
        _ => return "other"

fn huge(x: u128) -> str:
    match x:
        340282366920938463463374607431768211455 => return "max"
        _ => return "other"

fn main():
    println(describe(-9223372036854775807 - 1), describe(-5), describe(0), describe(3))
    least: i128 = -170141183460469231731687303715884105728
    println(top(18446744073709551615), top(1), wide(least), wide(-1), wide(0), huge(340282366920938463463374607431768211455))
