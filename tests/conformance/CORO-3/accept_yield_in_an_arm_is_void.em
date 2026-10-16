#$ test: run-pass
#$ rules: CORO-3, GRM-16
#$ profiles: debug, release
#$ stdout: 1 1 2 2 0
# `[CORO-3]` — `yield` as an expression has type `void`: it is the whole body of a `=>` arm
# (`[GRM-16]`), and the arms around it are `void` too.

gen fn signs(xs: Span[int]) -> Generator[int]:
    for x in xs:
        match x > 0:
            true => yield x
            false => yield -x

fn main():
    data = [1, -1, 2, -2, 0]
    out = [s for s in signs(data)]
    println(out[0], out[1], out[2], out[3], out[4])
