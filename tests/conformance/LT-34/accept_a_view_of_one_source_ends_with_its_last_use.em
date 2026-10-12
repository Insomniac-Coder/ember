#$ test: run-pass
#$ rules: LT-34, LT-24
#$ stdout: 6
#$ stdout: 7
## `[LT-34]` — both fields of `Halves` borrow `values`: the view acts as a
## single-region view, and `values` may change once neither field is used again.

@view
struct Halves:
    front: Span[i32]
    back: Span[i32]

fn halves(all: Span[i32]) -> Halves:
    middle = all.len() // 2
    return Halves(all[0..middle], all[middle..all.len()])

fn main():
    values: Array[i32] = Array[i32]()
    for i in 0..6:
        values.push(i)
    parts = halves(values.as_span())
    println(parts.front.len() + parts.back.len())
    values.push(6)
    println(values.len())
