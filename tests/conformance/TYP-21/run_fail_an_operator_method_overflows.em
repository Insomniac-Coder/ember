#$ test: run-fail
#$ rules: TYP-21, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# ODR-040 — a number's operator method is its operator, so `x.add(1)` at
# `i32.MAX` overflows and panics as `x + 1` does.

fn main():
    x: i32 = i32.MAX
    println(x.add(1))
