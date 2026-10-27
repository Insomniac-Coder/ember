#$ test: compile-fail
#$ rules: CORO-6
## design rules: R15
## R15: "E2221 for a view into its frame held across `yield`" (AN-77).

gen fn numbers() -> Generator[int]:
    n = 5
    r = ref n
    yield 1    #$ error[E2221]: `r` (line 7) points into this generator's own frame, which may move while it is suspended at the `yield` on line 8; use `n` itself: remove line 7 and write `yield n` instead of line 9
    yield r

fn main():
    for x in numbers():
        println(x)
