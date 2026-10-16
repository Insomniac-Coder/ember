#$ test: run-pass
#$ rules: CORO-2, LNT-4
#$ profiles: debug
#$ stdout: 0
# `[CORO-2]`, `[LNT-4]` — a `gen fn` with no `yield` is accepted and produces `L2004`; it
# yields nothing.

gen fn quiet() -> Generator[int]:    #$ warning[L2004]: never yields
    pass

fn main():
    count = 0
    for _ in quiet():
        count += 1
    println(count)
