#$ test: run-pass
#$ rules: STD-15
#$ profiles: debug, release, shipping
#$ stdout: false true true false
#$ 1 -1
#$ 3 5 4 none none
# `is_empty`, `contains`, `index_of`, and `get`, `first`, `last`, which answer
# `None` rather than panic.

fn shown(found: Option[ref int]) -> str:
    match found:
        Some(v):
            return "some" if v > 0 else "zero"
        None:
            return "none"

fn value(found: Option[ref int]) -> int:
    match found:
        Some(v):
            return v
        None:
            return -1

fn main():
    xs = [3, 1, 4, 1, 5]
    empty: Array[int] = []
    println(xs.is_empty(), empty.is_empty(), xs.contains(4), xs.contains(7))
    println(xs.index_of(1).unwrap(), xs.index_of(9).unwrap_or(-1))
    println(value(xs.first()), value(xs.last()), value(xs.get(2)), shown(xs.get(10)), shown(empty.last()))
