#$ test: run-pass
#$ rules: TYP-17, GRM-34
#$ stdout: 5
# `[TYP-17]` — inside a generic body a parameter meets bounds by its own:
# `Outer[T: Eq + Hash]` holds an `Inner[T, bool]`, and `Inner`'s extension
# bounded `K: Eq + Hash` gives it `make` there (D-340: the instance was made
# before any body's bounds were in scope, and never took the extension).

interface Make:
    type Out
    fn make() owns this -> Out

[K: needs (Eq, Hash), V]
struct Inner:
    k: K
    v: V

extend[K: needs (Eq, Hash), V] Inner[K, V] implements Make:
    type Out = K

    fn make() owns this -> K:
        return this.k

[T: needs (Eq, Hash)]
struct Outer:
    i: Inner[T, bool]

    fn get() owns this -> T:
        return this.i.make()

fn main():
    o = Outer(Inner[i64, bool](5, true))
    println(o.get())
