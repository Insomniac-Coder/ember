#$ test: run-pass
#$ rules: CORO-4, CT-1
#$ profiles: debug, release
#$ stdout: 55 55
# `[CORO-4]` — the state machine is ordinary code, so it runs while compiling as well
# (`[CT-1]`): a compile-time value computed by a generator equals the one computed at run time.

gen fn squares(n: int) -> Generator[int]:
    for i in 1..=n:
        yield i * i

fn total(n: int) -> int:
    t = 0
    for x in squares(n):
        t += x
    return t

const AT_COMPILE: int = comptime(total(5))

fn main():
    println(AT_COMPILE, total(5))
