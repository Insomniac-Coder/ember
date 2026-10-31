#$ test: run-pass
#$ rules: LT-1, BRW-8, TYP-16
#$ stdout: 5 text
#$ stdout: 3 s
#$ stdout: 4 t
# `[LT-1]`, `[BRW-8]` (ODR-024) — a generic body is checked once, over its
# declared parameters, and each instance passes a parameter the result may
# point into as the declaration does: `Holder[T]`'s receiver and `inner`'s
# `w: Wrap[T]` are passed by address, so `return ref self.item` and
# `return ref w.item` point into the caller's value for every `T`. D-284: a
# view argument (`str`) made the receiver or `w` a view passed as a copy, and
# only that instance failed, with `E3060`.

[T]
struct Holder:
    item: T

    fn first() -> ref T:
        return ref this.item

[T]
struct Wrap:
    item: T

[T]
fn inner(w: Wrap[T]) -> ref T:
    return ref w.item

@derive(Copy)
[T]
struct Pt:
    v: T

    fn get() -> ref T:
        return ref this.v

fn main():
    h = Holder(5)
    t = Holder("text")
    println(h.first(), t.first())
    println(inner(Wrap(3)), inner(Wrap("s")))
    a = Pt(4)
    b = Pt("t")
    println(a.get(), b.get())
