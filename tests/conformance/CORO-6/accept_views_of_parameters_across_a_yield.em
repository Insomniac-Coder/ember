#$ test: run-pass
#$ rules: CORO-6, LT-43
#$ profiles: debug, release
#$ stdout: 2 4
#$ ab cd
#$ 30
#$ 10 30 20
#$ 5 6
# `[CORO-6]` — references and views that came in as parameters, and views derived from them,
# may be held across a `yield`: the frame is then a view bound by their regions, as a returned
# view is (`[LT-1]`). A list iterated `owned`, or by index, holds no borrow of a local.

gen fn evens(xs: Span[int]) -> Generator[ref int]:
    for x in xs:
        if x % 2 == 0:
            yield x

gen fn words(text: str) -> Generator[str]:
    for w in text.split_whitespace():
        yield w

struct Bag:
    items: Array[int]

extend Bag:
    gen fn largest(self) -> Generator[int]:
        best = 0
        for x in self.items:
            if x > best:
                best = x
        yield best

    gen fn each(self) -> Generator[int]:
        for x in self.items:
            yield x

gen fn owned_list() -> Generator[int]:
    xs = [5, 6]
    for x in owned xs:
        yield x

fn main():
    data = [1, 2, 3, 4]
    out = [x for x in evens(data)]
    println(out[0], out[1])
    ws = [w for w in words("ab cd")]
    println(ws[0], ws[1])
    bag = Bag([10, 30, 20])
    for b in bag.largest():
        println(b)
    every = [x for x in bag.each()]
    println(every[0], every[1], every[2])
    got = [x for x in owned_list()]
    println(got[0], got[1])
