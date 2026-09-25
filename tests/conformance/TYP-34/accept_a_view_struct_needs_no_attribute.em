#$ test: run-pass
#$ rules: TYP-34, TYP-15
#$ stdout: 3 2
# `[TYP-34]` (0.9.9, replacing 0.9.8's required `@view`) — a struct holding a
# reference, a `Span` or another view is a view type, which the compiler
# infers; `@view` is optional documentation. Its fields are bounded places
# (`[TYP-15]`), so the view may live in a local.

struct Holder:
    s: Span[i32]

@view
struct Cursor:
    at: ref i32
    step: i32

fn main():
    xs: [i32; 3] = [1, 2, 3]
    h = Holder(xs)
    n: i32 = 2
    c = Cursor(ref n, 1)
    println(h.s.len(), c.at)
