#$ test: run-pass
#$ rules: HEAP-10, THR-8, THR-9, HEAP-7, WK-11
#$ profiles: debug, release
#$ stdout: ['a', 'b']
#$ 2
#$ true
#$ 7
#$ dropped 7
#$ false
#$ assert-c: contains("EMBER_TI_SYNC")
# `[HEAP-10]` — `SyncShared[T]` is `Shared[T]` with atomic counts: copies own one value, `get` lends
# `ref T`, mutation goes through `T`'s own lock or atomic, and the last owner's end drops the value
# once. A `@sync` class may hold one whose `T` is `Send` and `Sync` (`[THR-9]`), and a `Weak` of one
# upgrades while an owner lives (`[HEAP-7]`, `[WK-11]`).

from std.sync import SyncShared, Mutex, Atomic

struct Loud:
    n: int

    fn drop(mut self):
        println("dropped", self.n)

@sync
class Registry:
    names: SyncShared[Mutex[Array[String]]]

fn add(r: SyncShared[Mutex[Array[String]]], name: str):
    with g = r.get().lock():
        g.push(name.to_string())

fn main():
    names: SyncShared[Mutex[Array[String]]] = SyncShared(Mutex([]))
    reg = Registry(names)
    add(names, "a")
    add(reg.names, "b")
    with g = names.get().lock():
        println(g)
    hits = SyncShared(Atomic(0))
    copy = hits
    copy.get().fetch_add(2)
    println(hits.get().load())
    w = Weak(hits)
    println(w.upgrade().is_some())
    loud = SyncShared(Loud(7))
    other = loud
    println(other.get().n)
    mem.drop(loud)
    mem.drop(other)
    mem.drop(hits)
    mem.drop(copy)
    println(w.upgrade().is_some())
