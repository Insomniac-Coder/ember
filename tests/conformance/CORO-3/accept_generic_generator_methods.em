#$ test: run-pass
#$ rules: CORO-3, CORO-1, GRM-21
#$ profiles: debug, development
#$ stdout: 1 7
#$ 2 7
#$ 1
#$ 2
#$ 20
# `[CORO-3]` — a generator method of a generic type, generic itself, with a `where` clause: its
# frame is generic over both; and `Generator[Y, R]`'s result after the end.

struct Wrap[T]:
    items: Array[T]

extend[T] Wrap[T]:
    gen fn pairs_with[U](self, other: U) -> Generator[(ref T, U)] where U: Copy:
        for x in self.items:
            yield (x, other)

    gen fn count_up(self) -> Generator[int, int]:
        n = 0
        for _ in self.items:
            n += 1
            yield n
        return n * 10

fn main():
    w = Wrap([1, 2])
    for (a, b) in w.pairs_with(7):
        println(a, b)
    g = w.count_up()
    while true:
        match g.next():
            Some(x):
                println(x)
            None:
                break
    println(g.result().unwrap())
