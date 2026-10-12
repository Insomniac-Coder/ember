#$ test: compile-fail
#$ rules: LT-34, BRW-1
## `[LT-34]` — both fields borrow `values`, so either one being live keeps all
## of `values` borrowed, exactly as a single-region view would.

@view
struct Halves:
    front: Span[i32]
    back: Span[i32]

fn halves(all: Span[i32]) -> Halves:
    middle = all.len() // 2
    return Halves(all[0..middle], all[middle..all.len()])

fn main():
    values: Array[i32] = Array[i32]()
    parts = halves(values.as_span())
    values.push(6) #$ error[E3021]: `values` is borrowed here and mutably borrowed elsewhere
    println(parts.back.len())
