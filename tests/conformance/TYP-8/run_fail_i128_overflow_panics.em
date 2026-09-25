#$ test: run-fail
#$ rules: TYP-8, TYP-1
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `*`
# D-272 — `i128` arithmetic overflows and panics like any other integer's,
# in every profile, with the same message on every compiler.

fn times(a: i128, b: i128) -> i128:
    return a * b

fn main():
    println(times(85070591730234615865843651857942052863, 2))
    println(times(85070591730234615865843651857942052864, 2))
