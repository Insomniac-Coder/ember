#$ test: run-pass
#$ rules: TYP-17, TYP-24
#$ stdout: 1 true 3
# D-433 — a generic body sees what an extension gives where its own bounds
# meet the extension's: `T: Copy` in `g` gets `dup`. And with `T`
# unbounded only `Conv[bool]`'s `conv` applies, so `w.conv()` is that one;
# `Conv[Array[T]]` needs `T: Copy`, and the call was a false `E2070`.

interface Conv[T]:
    fn conv() -> T

[T]
struct Wrap:
    x: T

extend[T: needs Copy] Wrap[T]:
    fn dup() -> int:
        return 1

extend[T: needs Copy] Wrap[T] implements Conv[Array[T]]:
    fn conv() -> Array[T]:
        return [this.x, this.x, this.x]

extend[U] Wrap[U] implements Conv[bool]:
    fn conv() -> bool:
        return true

[T: needs Copy]
fn g(w: Wrap[T]) -> int:
    return w.dup()

[T]
fn flag(w: Wrap[T]) -> bool:
    return w.conv()

fn main():
    w = Wrap(x = 5)
    a: Array[int] = Conv[Array[int]].conv(w)
    println(g(w), flag(Wrap(x = "s".to_string())), a.len())
