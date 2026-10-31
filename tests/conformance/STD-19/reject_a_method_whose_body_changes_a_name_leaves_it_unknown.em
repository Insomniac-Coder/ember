#$ test: compile-fail
#$ rules: STD-19, CLS-7
#$ error[E2020]: expected `i64`, found `u64`
# `[STD-19]`, `[CLS-7]` — what a method does to its object is worked out from its body, and the
# known-numbers rule follows a name only when nothing changes it. `widen`, written after its call,
# changes the range it is called on, so nothing is known of `kept`: `-10 .. huge` may have more items
# than an `int` holds, and the count is a `u64`, which `e: int` cannot hold.

interface Widen:
    fn widen()

fn main():
    kept = -10 .. 10
    kept.widen()
    e: int = kept.iter().len()
    println(e)

extend Range[int] implements Widen:
    fn widen():
        this.end = 9223372036854775807
