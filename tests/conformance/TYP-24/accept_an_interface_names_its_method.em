#$ test: run-pass
#$ rules: TYP-24, OBJ-1
#$ stdout: 1 2 1 2 7
# `[TYP-24]` — when two interfaces supply a method of one name for one type,
# `I.m(recv, …)` names which; each is its own function, and a `dyn` of each
# interface calls its own (D-241). With one interface, `I.m(recv)` is the
# ordinary call.

interface A:
    fn m(self) -> int

interface B:
    fn m(self) -> int

interface Sized2:
    fn area(self, scale: int) -> int

struct S:
    n: int

extend S implements A:
    fn m(self) -> int:
        return 1

extend S implements B:
    fn m(self) -> int:
        return 2

extend S implements Sized2:
    fn area(self, scale: int) -> int:
        return self.n * scale

fn via_a(x: ref dyn A) -> int:
    return x.m()

fn via_b(x: ref dyn B) -> int:
    return x.m()

fn main():
    s = S(n=1)
    println(A.m(s), B.m(s), via_a(ref s), via_b(ref s), Sized2.area(s, 7))
