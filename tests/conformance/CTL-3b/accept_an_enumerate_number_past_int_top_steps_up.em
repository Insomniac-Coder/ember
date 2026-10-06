#$ test: run-pass
#$ rules: CTL-3b, STD-19, TYP-8
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_ck_add_i64(") == 0
#$ assert-c: !contains("integer overflow in")
#$ stdout:
#$ 9223372036854775806
#$ 9223372036854775807
#$ 9223372036854775808
#$ 9223372036854775809
# `[CTL-3b]`, `[STD-19]` (the owner's ruling of 2026-10-06, replacing G8-4 decision F) — an item's
# number is `start + k`, in a type that holds every number: from `int.MAX - 1` over four items, a
# `u64`. The loop computes each number with no check, and has no check for numbers past a top.

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    for (i, _) in xs.iter().enumerate(start=9223372036854775806):
        println(i)
