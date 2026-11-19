#$ test: run-pass
#$ rules: HEAP-10, THR-8, THR-9
#$ profiles: debug, development
#$ stdout: ['a', 'b']
#$ 2
#$ 7
#$ dropped 7
#$ assert-c: contains("EMBER_TI_SYNC")
# `[HEAP-10]`, `[THR-1]` — `thread_shared T` is `shares T` with atomic counts: its owners own one value,
# it reads as `T`, a change goes through `T`'s own lock or atomic, and the last owner's end drops the value
# once. An object may hold one whose `T` is `Send` and `ThreadShared` (`[THR-9]`).

from std.sync import Mutex, Atomic

struct Loud:
    n: int

    fn drop():
        println("dropped", this.n)

class Registry:
    names: thread_shared Mutex[Array[String]]

fn add(r: thread_shared Mutex[Array[String]], name: StrView):
    with g = r.lock():
        g.push(owns name.to_string())

fn main():
    names: thread_shared Mutex[Array[String]] = thread_shared Mutex([])
    reg = thread_shared Registry(thread_shared names)
    add(names, "a")
    add(reg.names, "b")
    with g = names.lock():
        println(g)
    hits = thread_shared Atomic(0)
    copy = thread_shared hits
    copy.fetch_add(2)
    println(hits.load())
    loud = thread_shared Loud(7)
    other = thread_shared loud
    println(other.n)
    mem.drop(loud)
    mem.drop(other)
    mem.drop(hits)
    mem.drop(copy)
