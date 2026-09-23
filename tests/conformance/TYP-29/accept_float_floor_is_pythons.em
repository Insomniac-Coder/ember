#$ test: run-pass
#$ rules: TYP-29
#$ profiles: debug, release, shipping
#$ stdout: 9
#$ 0.09999999999999995
#$ -4
#$ 0.5
#$ -0.5
#$ -15
#$ assert-c: contains("ember_floorrem_f64")
#$ assert-c: contains("ember_floordiv_f64")
# ODR-021: float `//` and `%` are Python's. `0.1` is slightly more than a
# tenth, so ten of them exceed 1.0: the floor quotient is 9, and the
# remainder is exact, with the divisor's sign. The last line also pins D-183:
# a negated literal inside an operator tree keeps its float type.

fn main():
    println(1.0 // 0.1)
    println(1.0 % 0.1)
    println(-7.5 // 2.0)
    println(-7.5 % 2.0)
    println(7.5 % -2.0)
    println(-7.5 * 2.0)
