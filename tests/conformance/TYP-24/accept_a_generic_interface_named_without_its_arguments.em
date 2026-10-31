#$ test: run-pass
#$ rules: TYP-24
#$ stdout: true 7
# D-415 — `I.m(recv)` with a generic interface written without its
# arguments names the one instance of it the receiver has that offers the
# method, chosen by the arguments where there are more. It said `W` has no
# `conv` from `Conv`.

interface Conv[T]:
    fn conv() -> T

interface Scale[T]:
    fn scale(by: T) -> int

struct W:
    x: int

extend W implements Conv[bool]:
    fn conv() -> bool:
        return true

extend W implements Scale[int]:
    fn scale(by: int) -> int:
        return this.x * by

extend W implements Scale[bool]:
    fn scale(by: bool) -> int:
        return 0

fn main():
    w = W(x = 7)
    println(Conv.conv(w), Scale.scale(w, 1))
