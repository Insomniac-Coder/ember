#$ test: run-pass
#$ rules: TYP-24, TYP-21
#$ stdout: 1 true 3 104 3 104
# D-402 — `[TYP-24]`'s `I.m(recv, …)` names an instance of a generic
# interface too: `Conv[bool].conv(w)`. It was `E1010`, cannot find `Conv`.
# Arguments that fit one instance choose it, by method or operator (D-313).

interface Conv[T]:
    fn conv(self) -> T

struct W:
    x: int

extend W implements Conv[Array[int]]:
    fn conv(self) -> Array[int]:
        return [self.x]

extend W implements Conv[bool]:
    fn conv(self) -> bool:
        return true

struct V:
    x: int

extend V implements Add[int]:
    type Output = int
    fn add(self, other: int) -> int:
        return self.x + other

extend V implements Add[V]:
    type Output = int
    fn add(self, other: V) -> int:
        return self.x + other.x + 100

fn main():
    w = W(x = 5)
    a: Array[int] = Conv[Array[int]].conv(w)
    b = Conv[bool].conv(w)
    v = V(x = 1)
    println(a.len(), b, v.add(2), v.add(V(x = 3)), v + 2, v + V(x = 3))
