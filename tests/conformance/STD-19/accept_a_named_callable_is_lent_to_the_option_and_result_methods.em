#$ test: run-pass
#$ rules: STD-19, ERR-4, CLO-6, BRW-2
#$ profiles: debug, development, release
#$ stdout: Some(4) Some(4) Ok(6) 6 3
#$ stdout: 7 7 2
#$ stdout: 1
#$ stdout: dropped 1
#$ stdout: Some(4)
#$ stdout: Some(30)
# `[STD-19]` (ODR-240) — every std function that takes a callable borrows a named callable the
# caller passes, the `Option` and `Result` methods (`map`, `unwrap_or_else`, …) and
# `Once.call_once` too, though each calls it at most once (`once fn`, `[CLO-6]`): `inc` counts its
# calls through three `map`s, `fallback` through two `unwrap_or_else`s, and `work` is lent to two
# `call_once`s, the second of which does not call it. What each changes is the caller's to read
# once the callable's last use is past (`[BRW-2]`). A callable whose call gives a capture away
# (`taker`) can be called only once, so it is given, as a lambda written at the call is.

from std.sync import Once

struct Noisy:
    value: int

    fn drop(mut self):
        println("dropped", self.value)

fn main():
    o: Option[int] = Some(3)
    calls = 0
    inc = fn(owned x: int) -> int:
        calls += 1
        return x + 1
    a = o.map(inc)
    b = o.map(inc)
    r: Result[int, int] = Ok(5)
    c = r.map(inc)
    e: Result[int, int] = Err(2)
    d = e.unwrap_or_else(fn(owned s: int) -> int => s * 3)
    println(a, b, c, d, calls)
    n: Option[int] = None
    made = 0
    fallback = fn() -> int:
        made += 1
        return 7
    println(n.unwrap_or_else(fallback), n.unwrap_or_else(fallback), made)
    once = Once()
    ran = 0
    work = fn():
        ran += 1
    once.call_once(work)
    once.call_once(work)
    println(ran)
    k = Noisy(1)
    taker = owned fn(owned x: int) -> int:
        kept = k
        return x + kept.value
    println(o.map(taker))
    println(o.map(fn(owned x: int) -> int => x * 10))
