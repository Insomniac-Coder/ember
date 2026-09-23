#$ test: run-fail
#$ rules: TYP-28, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `//`
# `MIN // -1` overflows in every profile, and the panic names the operator
# that was written.

fn main():
    low: i32 = -2147483647 - 1
    divisor: i32 = -1
    println(low // divisor)
