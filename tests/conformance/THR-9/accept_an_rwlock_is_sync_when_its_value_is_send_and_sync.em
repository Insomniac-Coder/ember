#$ test: run-pass
#$ rules: THR-9, THR-1, THR-3
#$ stdout: 2 a
# `[THR-9]` — `RwLock[T]` for a `T` that is `Send` and `Sync` is `Sync`: a `@sync` class may hold
# one, and write what it guards after `init`.

from std.sync import RwLock

@sync
class Names:
    names: RwLock[Array[String]] = RwLock([])

fn main():
    names = Names()
    with w = names.names.write():
        w.push("a".to_string())
        w.push("b".to_string())
    with r = names.names.read():
        println(r.len(), r[0])
