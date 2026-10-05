#$ test: run-pass
#$ rules: STD-19, RNG-4
#$ stdout: 9223372036854775807
#$ stdout: 18446744073709551615
#$ stdout: 18446744073709551616
#$ stdout: 340282366920938463463374607431768211456
#$ profiles: debug, release, shipping
# G8-4 (the owner's design, 0.9.10) — every range has its length: a `..` range of 64-bit numbers
# counts in `u64`, a `..=` one in `u128`, and a `..=` range of 128-bit numbers in `i256`, the
# 256-bit count. It panicked before, "a range of more than int.MAX values has no length".

fn main():
    full = 0..i64.MAX
    println(full.iter().len())
    wide = i64.MIN..i64.MAX
    println(wide.iter().len())
    whole = i64.MIN..=i64.MAX
    println(whole.iter().len())
    println((i128.MIN..=i128.MAX).iter().len())
