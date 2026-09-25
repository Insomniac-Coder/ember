#$ test: run-pass
#$ rules: TYP-21, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 7 5.0 -7 -6 3 1 4 12 5 2.5
#$ 16 12 7 0.75
# ODR-040 — each number type implements the interface of each operator it
# has, in `std.core`, so a bound on them takes numbers, and the interface's
# method is the operator: `3.add(4)` is `3 + 4`.

fn masked_shift[T: BitAnd[Output = T] + Shl[Output = T] + Copy](x: T, mask: T, by: T) -> T:
    return (x & mask) << by

fn bump[T: AddAssign + Copy](x: T, by: T) -> T:
    y = x
    y += by
    return y

fn main():
    n: i32 = 7
    println(3.add(4), (2.5).mul(2.0), n.neg(), 5.not(), 7.floordiv(2), 7.rem(2), 2.pow(2), 3.shl(2), 13.bitand(7), (5.0).div(2.0))
    a: u8 = 12
    b: u8 = 10
    c: u8 = 1
    println(masked_shift(a, b, c), masked_shift(7, 3, 2), bump(5, 2), bump(0.5, 0.25))
