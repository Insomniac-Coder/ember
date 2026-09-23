#$ test: run-pass
#$ rules: TYP-1, LEX-16, LEX-17
#$ profiles: debug, release, shipping
#$ stdout: 5000000000
#$ 5000000000
#$ 5
#$ assert-c: contains("int64_t")
# `int` is `i64` and `float` is `f64`, and they are what unannotated literals
# become. `5_000_000_000` does not fit 32 bits.

fn twice(x: int) -> int:
    return x * 2

fn main():
    big = 5_000_000_000
    println(big)
    same: i64 = twice(2_500_000_000)
    println(same)
    half: float = 2.5
    println(half * 2.0)
