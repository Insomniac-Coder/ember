#$ test: run-pass
#$ rules: THR-14, STA-1
#$ profiles: debug, development
#$ stdout: 18446744073709551615 18446744073709551616
#$ stdout: 340282366920938463463374607431768211455 0 None
#$ stdout: Err(5) Ok(5) 6
#$ stdout: 6 14 12 13
#$ stdout: -170141183460469231731687303715884105727 -170141183460469231731687303715884105728
#$ stdout: 4000
# `[THR-14]` — `Atomic[T]` for the 128-bit integers too, every operation atomic over all sixteen
# bytes: a carry crosses the halves, `wrapping_fetch_add` wraps and `checked_fetch_sub` stores
# nothing out of range, and four threads adding to one `static` lose no update (`[STA-1]`).

from std.sync import Atomic
import std.thread

static TOTAL: Atomic[i128] = Atomic(0)

fn add_many() -> int:
    for i in range(1000):
        TOTAL.fetch_add(1)
    return 0

fn main():
    a: Atomic[u128] = Atomic(18446744073709551615)
    old = a.fetch_add(1)
    println(old, a.load())
    a.store(340282366920938463463374607431768211455)
    println(a.wrapping_fetch_add(1), a.load(), a.checked_fetch_sub(1))
    b: Atomic[i128] = Atomic(5)
    println(b.compare_exchange(4, 9), b.compare_exchange(5, 6), b.load())
    println(b.fetch_or(8), b.fetch_and(12), b.fetch_xor(1), b.load())
    c: Atomic[i128] = Atomic(-170141183460469231731687303715884105727)
    println(c.wrapping_fetch_sub(1), c.load())
    h1 = thread.spawn(add_many)
    h2 = thread.spawn(add_many)
    h3 = thread.spawn(add_many)
    h4 = thread.spawn(add_many)
    h1.join() + h2.join() + h3.join() + h4.join()
    println(TOTAL.load())
