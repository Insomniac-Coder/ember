#$ test: run-pass
#$ rules: TYP-38
#$ stdout: 9
#$ ['ann', 'bob'] 2 2
#$ [0, 0, 0] [7, 7]
#$ [1, 2, 3] 300
#$ 2
#$ 14
## `[TYP-38]` - a list literal has the type its context expects (`Array[T]`, `[T; N]`, `Span[T]`); with no
## context it is an `Array` of the elements' own type, text written in the program staying a `StrView`; `[v; N]`
## is `N` copies; every element converts to the element type by `[TYP-5]`; an `owns` list is made with `owns`; a
## generator expression is never an allocating collection.

struct Circle:
    r: int

fn total(xs: Span[int]) -> int:
    t = 0
    for x in xs:
        t += x
    return t

fn views(xs: Array[StrView]) -> int:
    return xs.len()

fn strings(xs: Array[String]) -> int:
    return xs.len()

@noalloc
fn squares_total(n: int) -> int:
    t = 0
    for sq in (i * i for i in 0..n):
        t += sq
    return t

fn main():
    println(total([4, 5]))                              # a `Span[int]` context: a temporary fixed array
    names = ["ann", "bob"]                               # no context: an `Array[StrView]`
    println(names, views(names), strings(["ann", "bob"]))   # `Array[String]` position: two `String`s
    zeros = [0; 3]
    fixed: [int; 2] = [7, 7]
    println(zeros, fixed)
    wide: Array[i64] = [1, 2, 3]
    small: u8 = 200
    mixed: Array[u16] = [small, 100]                    # elements widen, `[TYP-5]` rule 1
    println(wide, mixed[0] + mixed[1])
    circles: Array[owns Circle] = [owns Circle(1), owns Circle(2)]
    println(circles.len())
    println(squares_total(4) - 0)
