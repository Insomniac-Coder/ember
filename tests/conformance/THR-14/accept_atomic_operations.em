#$ test: run-pass
#$ rules: THR-14, THR-9, THR-1, STA-1
#$ profiles: debug, release
#$ stdout: 3 3
#$ 7 9
#$ 250 4
#$ Some(4) None 254
#$ Some(254) None 0
#$ Err(0) Ok(0) 7
#$ false true false false
#$ -3 -7 0 1 2
#$ 2 3
#$ true true true true
# `[THR-14]` — `Atomic[T]` for the integer types, `bool` and raw pointers: `load`, `store`, `swap`,
# `compare_exchange`, the `fetch_` family, each with an `order` (by position or by name; `SeqCst`
# when left out). `wrapping_fetch_add` wraps; `checked_fetch_add` gives `None` and stores nothing
# when the result is out of range. An order held in a variable works as a constant one does. An
# `Atomic` is `Sync`: a `@sync` class holds one and changes it after `init` (`[THR-9]`), and a
# `static` is mutated through one (`[STA-1]`).

from std.sync import Atomic, MemoryOrder

static HITS: Atomic[int] = Atomic(0)

@sync
class Stats:
    words: Atomic[int] = Atomic(0)

fn pick(strong: bool) -> MemoryOrder:
    if strong:
        return MemoryOrder.SeqCst
    return MemoryOrder.Relaxed

fn main():
    stats = Stats()
    stats.words.fetch_add(1)
    stats.words.fetch_add(2, MemoryOrder.Relaxed)
    println(stats.words.load(), stats.words.load(order=MemoryOrder.Acquire))
    HITS.fetch_add(5)
    HITS.store(7, MemoryOrder.Release)
    println(HITS.swap(9), HITS.load())
    a: Atomic[u8] = Atomic(250)
    println(a.wrapping_fetch_add(10), a.load())
    println(a.checked_fetch_add(250), a.checked_fetch_add(2), a.load())
    println(a.checked_fetch_sub(254), a.checked_fetch_sub(1), a.load())
    println(a.compare_exchange(1, 7), a.compare_exchange(0, 7, MemoryOrder.AcqRel), a.load())
    f = Atomic(false)
    println(f.fetch_or(true), f.fetch_xor(true), f.fetch_and(true), f.load())
    b: Atomic[i32] = Atomic(-3)
    println(b.fetch_sub(4), b.fetch_and(6), b.fetch_or(1), b.fetch_xor(3), b.load())
    o = pick(false)
    println(b.fetch_add(1, o), b.load(pick(true)))
    unsafe:  # SAFETY: two fresh allocations, compared and freed once each
        q: *mut i64 = alloc[i64](2)
        r: *mut i64 = alloc[i64](2)
        p: Atomic[*mut i64] = Atomic(q)
        println(p.swap(r) == q, p.load() == r, p.compare_exchange(r, q).is_ok(), p.load() == q)
        free(q, 2)
        free(r, 2)
