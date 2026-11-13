#$ test: run-pass
#$ rules: BRW-1, TYP-14, BRW-13
# The other half: `ref mut` is the name that may write, and `=` with a value on it writes the place it
# names (`[TYP-14]`): `m = y` sets `x`, and both names read 5. `m = ref mut y` would give `m` another
# ref instead (`[BRW-13]`).

fn main():
    x: i32 = 1
    y: i32 = 5
    m: ref mut i32 = ref mut x
    m = y
    println(x)
    println(y)
#$ stdout: 5
#$ 5
