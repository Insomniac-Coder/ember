#$ test: run-pass
#$ rules: STD-19, RNG-4
#$ stdout: 255 4294967295
#$ stdout: 18446744073709551615 18446744073709551616
#$ stdout: 340282366920938463463374607431768211455 340282366920938463463374607431768211456
#$ stdout: 0 0 1
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — a range's `len()` is in the smallest type that holds every
# count a range of its kind can have: `int` for numbers of 8, 16 or 32 bits; `u64` for a `..`
# range of 64-bit numbers and `u128` for a `..=` one; `u128` for a `..` range of 128-bit numbers
# and `u256`, the 256-bit count, for a `..=` one. The ranges here are kept in variables, where
# their numbers are not visible, so each gets its kind's type.

fn main():
    tiny = -128 as i8 .. 127 as i8
    small = 0 as u32 .. u32.MAX
    a: int = tiny.iter().len()
    b: int = small.iter().len()
    println(a, b)
    wide = i64.MIN .. i64.MAX
    whole = i64.MIN ..= i64.MAX
    c: u64 = wide.iter().len()
    d: u128 = whole.iter().len()
    println(c, d)
    big = 0 as u128 .. u128.MAX
    every = 0 as u128 ..= u128.MAX
    e: u128 = big.iter().len()
    f: u256 = every.iter().len()
    println(e, f)
    empty = 5 .. 5
    reversed = 5 ..= 4
    one = 7 ..= 7
    println(empty.iter().len(), reversed.iter().len(), one.iter().len())
