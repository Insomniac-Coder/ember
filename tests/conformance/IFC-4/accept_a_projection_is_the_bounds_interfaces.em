#$ test: run-pass
#$ rules: IFC-4, TYP-17, RNG-5a1
#$ stdout: 3 true 6 true
# D-417 — `T.Output` through a bound is what the bound's interface says it
# is. `V` states an `Output` for `Add[V]` and another for `Producer`, and
# `T.Output` took the one declared last, whatever the bound named; a range
# type always lost its generated `Add`'s to another interface's.

interface Producer:
    type Output
    fn make(self) -> Output

struct V:
    x: int

extend V implements Add[V]:
    type Output = int
    fn add(self, other: V) -> int:
        return self.x + other.x

extend V implements Producer:
    type Output = bool
    fn make(self) -> bool:
        return true

type Level = i32 in 0 ..= 10

extend Level implements Producer:
    type Output = bool
    fn make(self) -> bool:
        return true

fn plus[T: Add[T]](x: T, y: T) -> T.Output:
    return x + y

fn mk[T: Producer](x: T) -> T.Output:
    return x.make()

fn main():
    v = V(x = 1)
    i: int = plus(v, V(x = 2))
    j: bool = mk(v)
    l = Level.clamped(3)
    k: i32 = plus(l, l)
    m: bool = mk(l)
    println(i, j, k, m)
