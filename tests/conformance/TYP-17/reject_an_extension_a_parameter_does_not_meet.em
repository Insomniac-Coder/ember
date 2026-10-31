#$ test: compile-fail
#$ rules: TYP-17, GRM-34
# `[TYP-17]` — without the bound, the extension is not there: `Outer[T]`'s `T`
# is not known to be `Eq`, so its `Inner[T, bool]` has no `make`.

interface Make:
    type Out
    fn make() owns this -> Out

[K, V]
struct Inner:
    k: K
    v: V

extend[K: needs Eq, V] Inner[K, V] implements Make:
    type Out = K

    fn make() owns this -> K:
        return this.k

[T]
struct Outer:
    i: Inner[T, bool]

    fn get() owns this -> T:
        return this.i.make() #$ error[E2072]: `Inner[T, bool]` has no method named `make`

fn main():
    o = Outer(Inner[i64, bool](5, true))
    println(o.get())
