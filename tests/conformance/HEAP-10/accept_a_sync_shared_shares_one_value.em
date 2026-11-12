#$ test: run-pass
#$ rules: HEAP-10, THR-8, THR-9
#$ profiles: debug, development
#$ stdout: ['a', 'b']
#$ 2
#$ 7
#$ dropped 7
#$ assert-c: contains("EMBER_TI_SYNC")
# `[HEAP-10]` — `Syncshares T` is `shares T` with atomic counts: copies own one value, `get` lends
# `ref T`, mutation goes through `T`'s own lock or atomic, and the last owner's end drops the value
# once. A `@sync` class may hold one whose `T` is `Send` and `Sync` (`[THR-9]`). A link to one waits
# for M4's slice E (`weakref thread_shared T`).

from std.sync import SyncShared, Mutex, Atomic

struct Loud:
    n: int

    fn drop():
        println("dropped", this.n)

@sync
class Registry:
    names: SyncShared[Mutex[Array[String]]]

fn add(r: SyncShared[Mutex[Array[String]]], name: StrView):
    with g = r.get().lock():
        g.push(owns name.to_string())

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
    loud = SyncShared(Loud(7))
    other = loud
    println(other.get().n)
    mem.drop(loud)
    mem.drop(other)
    mem.drop(hits)
    mem.drop(copy)
