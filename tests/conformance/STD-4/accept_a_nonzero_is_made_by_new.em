#$ test: run-pass
#$ rules: STD-4
#$ stdout: false true 5 -3 255
#$ stdout: true true true true true true
#$ stdout: true true true true true true
#$ stdout: 170141183460469231731687303715884105727 340282366920938463463374607431768211455
# `[STD-4]` — `NonZero[T]` is an integer that is not zero, one for each
# integer type. `NonZero.new(v)` is the only way to make one: `None` for 0,
# else `Some`; `get()` gives the integer back. `T` is found from the argument,
# where a literal takes its default type, or is written.

from std.core import NonZero

fn main():
    zero = NonZero.new(0)
    five = NonZero.new(5)
    println(zero.is_some(), five.is_some(), five.unwrap().get(), NonZero[i8].new(-3).unwrap().get(), NonZero[u8].new(255).unwrap().get())
    println(NonZero[i8].new(0).is_none(), NonZero[i16].new(0).is_none(), NonZero[i32].new(0).is_none(), NonZero[i64].new(0).is_none(), NonZero[i128].new(0).is_none(), NonZero[isize].new(0).is_none())
    println(NonZero[u8].new(0).is_none(), NonZero[u16].new(0).is_none(), NonZero[u32].new(0).is_none(), NonZero[u64].new(0).is_none(), NonZero[u128].new(0).is_none(), NonZero[usize].new(0).is_none())
    big: i128 = 170141183460469231731687303715884105727
    huge: u128 = 340282366920938463463374607431768211455
    println(NonZero.new(big).unwrap().get(), NonZero.new(huge).unwrap().get())
