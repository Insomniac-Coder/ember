#$ test: run-pass
#$ rules: STD-4, EFF-16
#$ stdout: -3 3 2 1 -4 -1
#$ stdout: 4
#$ assert-c-count: contains("ember_panic_div_zero(") == 1
# `[STD-4]` — dividing by a `NonZero` needs no check for zero, since none
# holds 0: the check is not there at all (`[EFF-16]`). `x // d` and `x % d`
# take a `NonZero` of `x`'s type, as operators or as the interfaces' methods.
# The one check in the program is the ordinary division's, in `plain`.

from std.core import NonZero

fn share(total: i64, parts: NonZero[i64]) -> i64:
    return total // parts

fn left(total: i64, parts: NonZero[i64]) -> i64:
    return total % parts

fn per(bytes: u32, size: NonZero[u32]) -> u32:
    return bytes // size

fn plain(total: i64, parts: i64) -> i64:
    return total // parts

fn main():
    four = NonZero.new(4).unwrap()
    minus_two = NonZero.new(-2).unwrap()
    size = NonZero[u32].new(3).unwrap()
    println(share(-9, four), left(-9, four), per(7, size), per(5, size), 7.floordiv(minus_two), 7.rem(minus_two))
    println(plain(9, 2))
