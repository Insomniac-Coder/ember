#$ test: run-fail
#$ rules: STD-20, TYP-30
#$ panics: `wrapping_pow` with a negative exponent
# ODR-039 — only `checked_pow` answers a negative exponent (with `None`); the
# other forms panic, as `**` does, since an integer to a negative power is a
# fraction.

fn power(x: int, e: int) -> int:
    return x.wrapping_pow(e)

fn main():
    println(power(2, 3))
    println(power(2, -1))
