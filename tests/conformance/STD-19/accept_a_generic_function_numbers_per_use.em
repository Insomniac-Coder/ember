#$ test: run-pass
#$ rules: STD-19, TYP-5, TYP-17
#$ stdout: 4 4 9223372036854775808
# `[STD-19]` (the owner's rulings of 2026-10-06) — in a function written for any type of item,
# what `enumerate` numbers in depends on the item type: a list of 1-byte items could hold more
# items than one of `int`s, so its numbers could pass `int`'s top. The function's own check leaves
# that to each use, which numbers by what it knows: a list of `int`s from 2 numbers in `int`, and
# `last` takes the numbers whether it is an `int` or a `u64` (`[TYP-5]` rule 12); from `int.MAX - 1`
# the numbers pass `int`'s top and are `u64`s.

fn labels_u64[T: Copy](xs: Array[T]) -> u64:
    last: u64 = 0
    for i, _x in enumerate(xs, start=2):
        last = i
    return last

fn labels_int[T: Copy](xs: Array[T]) -> int:
    last: int = 0
    for i, _x in enumerate(xs, start=2):
        last = i
    return last

fn labels_near_the_top[T: Copy](xs: Array[T]) -> u64:
    last: u64 = 0
    for i, _x in enumerate(xs, start=int.MAX - 1):
        last = i
    return last

fn main():
    println(labels_u64([10, 20, 30]), labels_int([10, 20, 30]), labels_near_the_top([10, 20, 30]))
