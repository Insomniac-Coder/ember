#$ test: run-pass
#$ rules: TYP-10, LEX-16
#$ profiles: debug, release, shipping
#$ stdout: 512 1099511627776 2 -1 8 12
# `[TYP-10]` — a shift's amount may be any integer type and does not choose
# the shifted value's: an untyped literal on the left takes its type from
# where the shift is used (`small: u8`), else is an `int`, as any literal is.
# D-395: it took the amount's type, so `1 << z` with `z: u8 = 9` panicked
# (a `u8` shifted by 9) and `1024 >> z` did not compile.

fn main():
    z: u8 = 9
    w: i16 = 40
    small: u8 = 1 << (z - 6)
    x: i8 = 3
    println(1 << z, 1 << w, 1024 >> z, -16 >> z, small, x << 2)
