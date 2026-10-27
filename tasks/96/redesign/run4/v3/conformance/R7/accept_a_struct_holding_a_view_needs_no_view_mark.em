#$ test: run-pass
#$ rules: R7
#$ stdout: 3 2
## R7: a struct that holds a view (`Span[T]`, `ref T`) is a view whether or not it says `@view`; `@view`
## may say so and is checked. Today's `TYP-34` example in the v3 spellings.

struct Holder:
    s: Span[int]

@view
struct Cursor:
    at: ref int
    step: int

fn main():
    xs = [1, 2, 3]
    h = Holder(s=xs.as_span())
    n = 2
    c = Cursor(at=ref n, step=1)
    println(h.s.len(), c.at)
