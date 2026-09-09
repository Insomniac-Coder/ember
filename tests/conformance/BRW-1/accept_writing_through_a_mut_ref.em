#$ test: run-pass
#$ rules: BRW-1, TYP-14
# The other half: `ref mut` is the borrow that may write, and assigning to it
# writes through to what it points at rather than re-seating it (ADR-010,
# which is what lets regions be location-insensitive). So this sets `x`, and
# both names read 5.

fn main():
    x: i32 = 1
    y: i32 = 5
    m: ref mut i32 = ref mut x
    m = ref mut y
    println(x)
    println(y)
#$ stdout: 5
#$ 5
