#$ test: compile-fail
#$ rules: CORO-6, LT-43
#$ profiles: debug
# `[CORO-6]` — a borrow of a local of the generator's own frame may not be live across a
# `yield` (`E2221`): moving the suspended frame would leave it dangling. Iterating a local list
# borrows it, and so does a reference kept in a local; a borrow given out by the `yield` itself
# outlives the suspension too.

gen fn over_a_local() -> Generator[int]:
    xs = [1, 2, 3]
    for x in xs:
        yield x    #$ error[E2221]: a borrow of `xs`, a local of this generator, is held across this `yield`

gen fn kept() -> Generator[int]:
    n = 5
    r = ref n
    yield 1    #$ error[E2221]: a borrow of `n`
    yield r

gen fn given_out() -> Generator[ref int]:
    n = 5
    yield ref n    #$ error[E2221]: gives out a borrow of `n`

fn main():
    for x in over_a_local():
        println(x)
    for x in kept():
        println(x)
    for x in given_out():
        println(x)
