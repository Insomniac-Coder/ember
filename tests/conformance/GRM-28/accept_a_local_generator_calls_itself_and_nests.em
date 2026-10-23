#$ test: run-pass
#$ rules: GRM-28, GRM-21, CLO-12, CORO-3, CORO-5
#$ profiles: debug, development, release
#$ stdout: 6 10
#$ 2
#$ 3 9
#$ done 4
## `[GRM-28]` (ODR-268) — a local generator may call itself (`depth`, its inner frame not held
## across a `yield`, `[CORO-5]`), may declare one of its own (`inner`, which captures its `n`), may
## capture its method's `self` (`walk`), and may return a result (`[CORO-3]`).

class Counter:
    step: int = 3

    fn steps(self, n: int) -> int:
        gen fn walk(k: int) -> Generator[int]:
            for i in 0..k:
                yield i * self.step
        last = 0
        for v in walk(n):
            last = v
        return last

fn main():
    floor = 1
    gen fn depth(n: int) -> Generator[int]:
        if n > 0:
            below = 0
            for v in depth(n - 1):
                below += v
            yield below + n
        else:
            yield floor - 1
    got = [v for v in depth(3)]
    d = depth(4)
    println(got[0], d.next().unwrap())
    gen fn outer(n: int) -> Generator[int]:
        gen fn inner(m: int) -> Generator[int]:
            yield m + n
        got = [v for v in inner(1)]
        yield got[0]
    o = outer(1)
    println(o.next().unwrap())
    c = Counter()
    println(c.steps(2), c.steps(4))
    gen fn finish(n: int) -> Generator[int, String]:
        yield n
        return f"done {n}"
    g = finish(4)
    while g.next().is_some():
        pass
    println(g.result().unwrap())
