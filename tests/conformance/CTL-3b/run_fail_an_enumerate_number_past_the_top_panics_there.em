#$ test: run-fail
#$ rules: CTL-3b, STD-19, TYP-8
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_ck_add_i64(") == 0
#$ stdout:
#$ 9223372036854775806
#$ 9223372036854775807
#$ panics: integer overflow in `+`
# `[CTL-3b]`, `[TYP-8]` — an item's number is `start + k`, and one past
# `int`'s top panics before the turn that would give it. The loop checks no
# number on each turn: it takes only the turns whose numbers fit, and panics
# after them.

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    for (i, _) in xs.iter().enumerate(start=9223372036854775806):
        println(i)
