#$ test: run-fail
#$ rules: STD-19, RNG-4
#$ panics: a range of more than int.MAX values has no length
#$ stdout: 9223372036854775807
#$ profiles: debug, release, shipping
# G8-4 — a range of signed 64-bit values longer than int.MAX has no length,
# and says so as the unsigned ones do: the gap is counted in i128, where it was
# an overflow of the subtraction (`integer overflow in -`). One exactly int.MAX
# long has its length.

fn main():
    full = 0..i64.MAX
    println(full.iter().len())
    wide = i64.MIN..i64.MAX
    println(wide.iter().len())
