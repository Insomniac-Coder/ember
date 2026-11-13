#$ test: compile-fail
#$ rules: CTL-7, CORO-7
## `[CTL-7]` - a `defer` block cannot `yield` (`E2160`): it runs while its block exits.

gen fn g(n: int) -> Generator[int]:
    defer:
        yield 1    #$ error[E2160]: a `defer` block runs while its block exits, and `yield` would suspend the generator in the middle of that exit; yield before the block exits, or move the `defer` line below the `yield`
    yield n

fn main():
    for v in g(1):
        println(v)
