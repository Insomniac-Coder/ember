#$ test: run-pass
#$ rules: TYP-24
#$ stdout: true 7
# D-415 — `I.m(recv)` with a generic interface written without its
# arguments names the one instance of it the receiver has that offers the
# method, chosen by the arguments where there are more. It said `W` has no
# `conv` from `Conv`.

interface Conv[T]:
    fn conv(self) -> T

interface Scale[T]:
    fn scale(self, by: T) -> int

struct W:
    x: int

extend W implements Conv[bool]:
    fn conv(self) -> bool:
        return true

extend W implements Scale[int]:
    fn scale(self, by: int) -> int:
        return self.x * by

extend W implements Scale[bool]:
    fn scale(self, by: bool) -> int:
        return 0

fn main():
    w = W(x = 7)
    println(Conv.conv(w), Scale.scale(w, 1))
