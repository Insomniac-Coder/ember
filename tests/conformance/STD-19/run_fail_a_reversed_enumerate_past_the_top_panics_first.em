#$ test: run-fail
#$ rules: STD-19, CTL-3b, TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# ODR-091 — run backwards, `enumerate` gives its greatest number first
# (`number + len - 1`), so numbers that pass `int`'s top panic at the first
# item, before the body: nothing is printed. The counted loop checks it
# before the loop, as `Enumerate.next_back` does at its first call.

fn main():
    xs: Array[int] = [1, 2, 3]
    for i, x in xs.iter().enumerate(9223372036854775806).rev():
        println(i, x)
