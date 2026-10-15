#$ test: compile-fail
#$ rules: EFF-5, THR-14
#$ error[E4002]: `@nosync` function `count` reaches a synchronising operation
# `[EFF-5]` — every `Atomic` operation synchronises, so `@nosync` refuses it.

from std.sync import Atomic

@nosync
fn count(hits: Atomic[int]) -> int:
    return hits.load()

fn main():
    hits = Atomic(0)
    println(count(hits))
