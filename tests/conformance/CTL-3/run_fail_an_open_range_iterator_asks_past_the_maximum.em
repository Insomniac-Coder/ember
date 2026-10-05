#$ test: run-fail
#$ rules: CTL-3, TYP-8, STD-19
#$ panics: integer overflow in `+`
#$ stdout: Some(254) Some(255)
#$ profiles: debug, release
# `[CTL-3]` (D-526) — `a..`'s iterator gives its type's maximum, as the `for`
# over it does, and panics when the value after it is asked for.

fn main():
    it = (254u8..).iter()
    first = it.next()
    second = it.next()
    println(first, second)
    println(it.next())
