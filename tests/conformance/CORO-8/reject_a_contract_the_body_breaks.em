#$ test: compile-fail
#$ rules: CORO-8, EFF-5
#$ profiles: debug
# `[CORO-8]` — a generator's effect set is the union over its whole body, and its contracts
# apply to it as to any function: a `@noalloc` generator that allocates between two `yield`s is
# `E4001`.

@noalloc
gen fn grows(n: int) -> Generator[int]:
    yield 1
    xs = [n]    #$ error[E4001]: `@noalloc` function `grows` reaches an allocation
    yield xs[0]

fn main():
    for x in grows(2):
        println(x)
