#$ test: run-pass
#$ rules: TYP-29
#$ profiles: debug, development, release
#$ stdout: 10.0
#$ 0.0
#$ -4.0
#$ 0.5
#$ -0.5
#$ -15.0
#$ assert-c: contains("ember_floorrem_f64")
#$ assert-c: contains("ember_floordiv_f64")
# `[TYP-29]` (the owner's reversal of ODR-021, 2026-10-21): float `a // b` is `floor(a / b)` and
# `a % b` is `a - b * floor(a / b)`, each step rounded as the machine rounds it. `1.0 / 0.1`
# rounds to exactly 10.0, so `1.0 // 0.1` is 10.0 and `1.0 % 0.1` is 0.0, not Python's 9.0 and
# 0.09999999999999995. The last line also pins D-183: a negated literal inside an operator tree
# keeps its float type.

fn main():
    println(1.0 // 0.1)
    println(1.0 % 0.1)
    println(-7.5 // 2.0)
    println(-7.5 % 2.0)
    println(7.5 % -2.0)
    println(-7.5 * 2.0)
