#$ test: compile-fail
#$ rules: ALC-1, TYP-45
# `[ALC-1]` — the memory manager a value's memory comes from is named by `using` after the type, never as
# a type argument: `Box[int, Global]` is `E1010` ("write `owns int using Global`") and so is `Array[int,
# Global]` ("write `Array[int] using Global`"). Without `using` the memory comes from `Global`.

from std.mem import Global

fn total(xs: Array[int, Global]) -> int:    #$ error[E1010]: `Array[int, Global]` is not a type of Ember: write `Array[int] using Global`
    return xs.len()

fn main():
    println(total([1, 2]))
