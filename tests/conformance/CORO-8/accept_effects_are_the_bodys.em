#$ test: run-pass
#$ rules: CORO-8, EFF-1
#$ profiles: debug, development
#$ stdout: 1
#$ 2
# `[CORO-8]` — a generator carries the effects of its body: one that prints is not `@noio`, and
# a `@noalloc` caller may run one whose body allocates nothing (`[EFF-1]`: calling it runs none
# of the body, and each `next` the body's effects).

gen fn printing() -> Generator[int]:
    println(1)
    yield 0

@noalloc
gen fn counting() -> Generator[int]:
    yield 2

@noalloc
fn second() -> int:
    for x in counting():
        return x
    return 0

fn main():
    for _ in printing():
        pass
    println(second())
