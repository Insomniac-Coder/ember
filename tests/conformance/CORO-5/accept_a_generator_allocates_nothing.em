#$ test: run-pass
#$ rules: CORO-5, CORO-8, EFF-1
#$ profiles: debug, development
#$ stdout: 6 15
#$ 3
#$ 1 3 6
# `[CORO-5]` — a generator allocates nothing: a `@noalloc` function makes one, resumes it and
# drops it. Its frame is an ordinary move-only value, kept in a field of a struct and in a list.
# `[CORO-8]` — the generator's effects are its body's.

struct Holder[G]:
    source: G

@noalloc
gen fn upto(n: int) -> Generator[int]:
    for i in 1..=n:
        yield i

@noalloc
fn total(n: int) -> int:
    t = 0
    for x in upto(n):
        t += x
    return t

fn main():
    println(total(3), total(5))
    h = Holder(upto(3))
    last = 0
    for x in h.source:
        last = x
    println(last)
    frames = [upto(1), upto(2), upto(3)]
    sums = []
    for i in 0..frames.len():
        s = 0
        while true:
            match frames[i].next():
                Some(x):
                    s += x
                None:
                    break
        sums.push(s)
    println(sums[0], sums[1], sums[2])
