#$ test: run-pass
#$ rules: TYP-36, TYP-1, CG-C-2
#$ profiles: debug, release
#$ stdout: 1 1 true
#$ stdout: 1 true false
#$ stdout: 1 true 7
#$ stdout: hashed
# `[TYP-36]` — `void`, and `Option[T]` and `Array[T]` of a hashable `T`, are
# `Hash`, and `[TYP-1]` lets a `void` value stand wherever a value can. So a
# borrow of a `void` place must give C that compiles (`[CG-C-2]`): `ref u` of a
# `void` local, the `void` inside an `Option[void]` or an `Array[void]` that a
# `Set` or a `Map` hashes, and the `void` a `match` binds (D-449).
from std.collections import DefaultHasher

fn take(x: ref void) -> int:
    return 1

fn main():
    u = ()
    r = ref u
    println(take(r), take(ref u), r == ())
    options = {Some(())}
    nothing: Option[void] = None
    println(len(options), Some(()) in options, nothing in options)
    a: Array[void] = [(), ()]
    arrays = {a.clone()}
    m: Map[Array[void], int] = {}
    m.insert(a.clone(), 7)
    println(len(arrays), a in arrays, m[a])
    h = DefaultHasher.default()
    u.hash(h)
    o: Option[void] = Some(())
    match o:
        Some(x):
            x.hash(h)
        None:
            pass
    h.finish()
    println("hashed")
