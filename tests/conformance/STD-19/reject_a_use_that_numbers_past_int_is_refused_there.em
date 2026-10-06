#$ test: compile-fail
#$ rules: STD-19, TYP-5, TYP-17
# `[STD-19]` (the owner's rulings of 2026-10-06) — a function written for any type of item is
# checked for each use where what it numbers in depends on the item type. Used with a list of
# `int`s, `last = i` takes every number; used with a list of `u8`s, whose list could hold so many
# items that numbering from 2 passes `int`'s top, it does not, and that use is refused, naming it.

fn labels[T: Copy](xs: Array[T]) -> int:
    last: int = 0
    for i, _x in enumerate(xs, start=2):
        last = i                    #$ error[E2020]: in `labels` used with `T` = `u8`
    return last

fn main():
    bs: Array[u8] = [1, 2, 3]
    println(labels([10, 20, 30]), labels(bs))
