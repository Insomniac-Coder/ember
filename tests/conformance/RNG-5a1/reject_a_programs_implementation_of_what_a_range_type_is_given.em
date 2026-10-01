#$ test: compile-fail
#$ rules: RNG-5a1, TYP-20, TYP-19
# D-418 — a range type is given its operators, `Eq` and `Ord` from its
# representation (`[RNG-5a1]`), and `Clone`, `Hash`, `Display` and `Debug`
# where the representation has them (ODR-093). A program's own
# implementation of one is a second implementation, `E2041`: it was accepted
# and replaced the generated one, so `l + l` was the program's `add` and
# `l == l` was false.

type Level = i32 in 0 ..= 10

extend Level implements Add[Level]:    #$ error[E2041]: `Level` already implements `Add[Level]`
    type Output = Level
    fn add(self, other: Level) -> Level:
        return Level.clamped(5)

extend Level implements Eq:    #$ error[E2041]: `Level` already implements `Eq`
    fn eq(self, other: Level) -> bool:
        return false

extend Level implements Clone:    #$ error[E2041]: `Level` already implements `Clone`
    fn clone(self) -> Level:
        return Level.clamped(9)

extend i32 implements Mul[Level]:    #$ error[E2041]: `i32` already implements `Mul[Level]`
    type Output = i32
    fn mul(self, other: Level) -> i32:
        return 0

fn main():
    l: Level = 3
    println(l + l, l == l, l.clone())
