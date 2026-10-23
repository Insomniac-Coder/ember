#$ test: run-pass
#$ rules: UNS-10, CG-LL-3, CELL-1, CELL-5, THR-3, THR-14
#$ profiles: debug, development, release
#$ stdout: 1 2
#$ stdout: 1 2
#$ stdout: 1 2
#$ stdout: 1 2
#$ stdout: 7 3
# `[UNS-10]` (ODR-275) — what a function changes through a shared reference is what a `Cell` or an
# `UnsafeCell` keeps: a `Cell`'s value, a `RefCell`'s value and borrow state, a lock's value and
# word, an atomic's value, here each changed by a function that is never inlined and read again by
# its caller after the call. `[CG-LL-3]`: the LLVM backend marks a shared reference `readonly` only
# where the referent holds no cell (`Pair`, read by `sum`), so a value read before such a call is
# never taken for the value after it.

from std.sync import Atomic, Mutex, MemoryOrder

struct Pair:
    a: int
    b: int

struct Stats:
    hits: Atomic[int]
    name: int

@noinline
fn poke(cell: ref Cell[int]):
    cell.set(cell.get() + 1)

@noinline
fn grow(cell: ref RefCell[int]):
    with g = cell.borrow_mut():
        g += 1

@noinline
fn tick(stats: ref Stats):
    stats.hits.fetch_add(1, MemoryOrder.Relaxed)

@noinline
fn bump(m: ref Mutex[int]):
    with g = m.lock():
        g += 1

@noinline
fn sum(p: ref Pair) -> int:
    return p.a + p.b

fn main():
    c = Cell(1)
    before = c.get()
    poke(ref c)
    println(before, c.get())

    r = RefCell(1)
    first = 0
    with g = r.borrow():
        first = g
    grow(ref r)
    with g = r.borrow():
        println(first, g)

    stats = Stats(hits=Atomic(1), name=0)
    seen = stats.hits.load(MemoryOrder.Relaxed)
    tick(ref stats)
    println(seen, stats.hits.load(MemoryOrder.Relaxed))

    m = Mutex(1)
    held = 0
    with g = m.lock():
        held = g
    bump(ref m)
    with g = m.lock():
        println(held, g)

    p = Pair(a=3, b=4)
    println(sum(ref p), p.a)
