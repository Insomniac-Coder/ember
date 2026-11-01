#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` (verdict 144) — the help that says to remove `from` is for a result that may point into
# what `from` names and into another parameter too. One that points only into others is refused
# for each of them, and the help is to return a view of what `from` names.
#$ help: return a view of `a`, or write `-> Pair from b`
#$ help: return a view of `a`, or write `-> Pair from c`
#$ not-help: remove `from a`

@view
struct Pair:
    first: Span[int]
    second: Span[int]

fn pair(a: Span[int], b: Span[int], c: Span[int]) -> Pair from a:
    return Pair(b, c)    #$ error[E3062]: `pair` returns a view of `b`, but its first line says `from a`
                         #$ error[E3062]: `pair` returns a view of `c`, but its first line says `from a`

fn main():
    xs = [1, 2]
    ys = [3]
    zs = [4]
    println(pair(xs, ys, zs).first)
