#$ test: compile-fail
#$ rules: CORO-2, GRM-16
#$ profiles: debug
# `[GRM-16]` — `yield e` is a jump expression of type `void`: the whole of a statement, a `=>`
# arm or a conditional's branch, never an operand or a value to bind (`E0107`).

gen fn g() -> Generator[int]:
    v = yield 1    #$ error[E0107]: `yield` must be a whole statement
    yield 2

fn main():
    for x in g():
        println(x)
