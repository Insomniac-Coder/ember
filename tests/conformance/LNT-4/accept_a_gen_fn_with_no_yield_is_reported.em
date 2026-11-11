#$ test: compile-pass
#$ rules: LNT-4, CORO-2
#$ warning[L2004]: never yields
# `[LNT-4]` — `L2004`: a `gen fn` with no `yield`.

gen fn quiet() -> Generator[int]:
    pass

fn main():
    count = 0
    for _ in quiet():
        count += 1
    println(count)
