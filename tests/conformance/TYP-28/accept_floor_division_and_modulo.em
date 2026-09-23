#$ test: run-pass
#$ rules: TYP-28, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ -4
#$ -4
#$ 3
#$ 1
#$ -1
#$ 1
#$ 0
#$ 1
#$ 3
#$ 3
# `//` rounds the quotient toward negative infinity and `%` takes the sign of
# the divisor, as in Python. `MIN % -1` is 0 and does not overflow. Unsigned
# operands agree with C. The compound forms are the same operators.

fn main():
    println(7 // 2)
    println(-7 // 2)
    println(7 // -2)
    println(-7 // -2)
    println(-7 % 2)
    println(7 % -2)
    println(-7 % -2 + 2)
    low: i32 = -2147483647 - 1
    println(low % -1)
    n = 10
    n //= 3
    n %= 2
    println(n)
    u: u32 = 7
    println(u // 2)
    println(u % 4)
