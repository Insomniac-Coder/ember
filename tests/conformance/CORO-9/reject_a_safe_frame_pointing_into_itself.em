#$ test: compile-fail
#$ rules: CORO-9, CORO-6
#$ profiles: debug
# `[CORO-9]` — in Safe code a frame never points into itself: a raw pointer to the generator's own
# local is as much a borrow of it across a `yield` as a reference is (`E2221`).

gen fn pointing() -> Generator[int]:
    n = 5
    p = ref_to_ptr(ref n)
    yield 1    #$ error[E2221]: a borrow of `n`
    _q = p

fn main():
    for x in pointing():
        println(x)
