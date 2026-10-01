#$ test: run-pass
#$ rules: TYP-17, TYP-24
#$ stdout: 1 true 3
# D-433 — a generic body sees what an extension gives where its own bounds
# meet the extension's: `T: Copy` in `g` gets `dup`. And with `T`
# unbounded only `Conv[bool]`'s `conv` applies, so `w.conv()` is that one;
# `Conv[Array[T]]` needs `T: Copy`, and the call was a false `E2070`.

interface Conv[T]:
    fn conv(self) -> T

struct Wrap[T]:
    x: T

extend[T: Copy] Wrap[T]:
    fn dup(self) -> int:
        return 1

extend[T: Copy] Wrap[T] implements Conv[Array[T]]:
    fn conv(self) -> Array[T]:
        return [self.x, self.x, self.x]

extend[U] Wrap[U] implements Conv[bool]:
    fn conv(self) -> bool:
        return true

fn g[T: Copy](w: Wrap[T]) -> int:
    return w.dup()

fn flag[T](w: Wrap[T]) -> bool:
    return w.conv()

fn main():
    w = Wrap(x = 5)
    a: Array[int] = Conv[Array[int]].conv(w)
    println(g(w), flag(Wrap(x = "s".to_string())), a.len())
