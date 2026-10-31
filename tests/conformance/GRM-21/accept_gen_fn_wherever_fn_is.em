#$ test: run-pass
#$ rules: GRM-21, CORO-1, CORO-12
#$ profiles: debug, development
#$ stdout: 1 2
#$ 3
#$ 4 5
#$ 6
#$ 7 8
#$ 10 11
#$ 9
# `[GRM-21]` — `gen fn` is legal wherever `fn` is: an item, generic or not; a method in a
# struct's, a class's or an enum's body, or in an `extend` block, a generic one's too; and an
# `unsafe` one, `gen` coming right before `fn` (`fn_header`).

[T]
gen fn items(xs: Span[T]) -> Generator[ref T]:
    for x in xs:
        yield x

struct Pair:
    a: int
    b: int

    gen fn both() -> Generator[int]:
        yield this.a
        yield this.b

class Counter:
    n: int = 3

    gen fn once() -> Generator[int]:
        yield this.n

enum Shape:
    Square(int)
    Line(int, int)

    gen fn sides() -> Generator[int]:
        match this:
            Shape.Square(s):
                yield s
            Shape.Line(x, y):
                yield x
                yield y

[T]
struct Box2:
    first: T
    second: T

extend[T] Box2[T]:
    gen fn each() -> Generator[ref T]:
        yield this.first
        yield this.second

extend Pair:
    gen fn sum_only() -> Generator[int]:
        yield this.a + this.b

@safety("nothing the caller must uphold; this shows `unsafe gen fn` parses.")
unsafe gen fn raw_count() -> Generator[int]:
    yield 9

fn main():
    data = [1, 2]
    got = [x for x in items(data)]
    println(got[0], got[1])
    for n in Counter().once():
        println(n)
    sides = [s for s in Shape.Line(4, 5).sides()]
    println(sides[0], sides[1])
    for s in Pair(2, 4).sum_only():
        println(s)
    p = Pair(7, 8)
    both = [x for x in p.both()]
    println(both[0], both[1])
    b = Box2(10, 11)
    words = [w for w in b.each()]
    println(words[0], words[1])
    # SAFETY: `raw_count` asks nothing of its caller.
    unsafe:
        for x in raw_count():
            println(x)
