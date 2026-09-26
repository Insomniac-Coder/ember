#$ test: run-pass
#$ rules: TYP-17, GRM-34
#$ stdout: 5
# `[TYP-17]` — inside a generic body a parameter meets bounds by its own:
# `Outer[T: Eq + Hash]` holds an `Inner[T, bool]`, and `Inner`'s extension
# bounded `K: Eq + Hash` gives it `make` there (D-340: the instance was made
# before any body's bounds were in scope, and never took the extension).

interface Make:
    type Out
    fn make(owned self) -> Out

struct Inner[K: Eq + Hash, V]:
    k: K
    v: V

extend[K: Eq + Hash, V] Inner[K, V] implements Make:
    type Out = K

    fn make(owned self) -> K:
        return self.k

struct Outer[T: Eq + Hash]:
    i: Inner[T, bool]

    fn get(owned self) -> T:
        return self.i.make()

fn main():
    o = Outer(Inner[i64, bool](5, true))
    println(o.get())
