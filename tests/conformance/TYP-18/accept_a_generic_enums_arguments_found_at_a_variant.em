#$ test: run-pass
#$ rules: TYP-18, ENM-1
#$ stdout: 3 -1 4 7
#$ stdout: left 2 right x
#$ stdout: right r
# `[TYP-18]` — a generic enum's arguments are found at a variant as a generic
# function's are at a call: from the payload's values (`Maybe.Just(3)`, a
# literal taking its default type) and from the type the context expects,
# which a variant without a payload needs (`Maybe.Nothing` returned as a
# `Maybe[int]`). D-341 (and D-270, found before it): each of these was
# "cannot find `Maybe`".

enum Maybe[T]:
    Nothing
    Just(T)

enum Either[L, R]:
    Left(L)
    Right(R)

fn show(m: Maybe[int]) -> int:
    match m:
        Maybe.Just(v):
            return v
        Maybe.Nothing:
            return -1

fn none() -> Maybe[int]:
    return Maybe.Nothing

fn deep(m: Maybe[Maybe[int]]) -> int:
    match m:
        Maybe.Just(inner):
            return show(inner)
        Maybe.Nothing:
            return 0

fn side(e: Either[int, str]) -> String:
    match e:
        Either.Left(n):
            return f"left {n}"
        Either.Right(s):
            return f"right {s}"

fn main():
    a = Maybe.Just(3)
    c: Maybe[int] = Maybe.Just(4)
    println(show(a), show(none()), show(c), deep(Maybe.Just(Maybe.Just(7))))
    println(side(Either.Left(2)), side(Either.Right("x")))
    e: Either[String, String] = Either.Right("r")
    match e:
        Either.Left(l):
            println("left", l)
        Either.Right(r):
            println("right", r)
