#$ test: run-pass
#$ rules: TYP-24, TYP-21, RNG-5a2
#$ stdout: 64 32 64 64 32 64
# D-428, ODR-096 — of two instances of one generic interface that fit a
# call or an operator, the one whose parameter type is the argument's own
# goes before one reached through a widening; an untyped literal counts as
# its default type, `int`. The operator took the instance registered last
# (`v + s` with `s: i32` widened into `Add[i64]`, `v + 1` went to
# `Add[i32]`), and the method form called both of these ambiguous.

struct V:
    x: int

extend V implements Add[i64]:
    type Output = int
    fn add(self, other: i64) -> int:
        return 64

extend V implements Add[i32]:
    type Output = int
    fn add(self, other: i32) -> int:
        return 32

fn main():
    v = V(x = 0)
    s: i32 = 3
    t: i64 = 4
    println(v + 1, v + s, v + t, v.add(1), v.add(s), v.add(t))
