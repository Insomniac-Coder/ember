#$ test: compile-fail
#$ rules: TYP-17, GRM-34
# `[TYP-17]` — without the bound, the extension is not there: `Outer[T]`'s `T`
# is not known to be `Eq`, so its `Inner[T, bool]` has no `make`.

interface Make:
    type Out
    fn make(owned self) -> Out

struct Inner[K, V]:
    k: K
    v: V

extend[K: Eq, V] Inner[K, V] implements Make:
    type Out = K

    fn make(owned self) -> K:
        return self.k

struct Outer[T]:
    i: Inner[T, bool]

    fn get(owned self) -> T:
        return self.i.make() #$ error[E1010]: `Inner[T, bool]` has no method named `make`

fn main():
    o = Outer(Inner[i64, bool](5, true))
    println(o.get())
